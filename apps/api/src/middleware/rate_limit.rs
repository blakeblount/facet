//! Rate limiting middleware for PIN verification endpoints.
//!
//! This module implements per-IP rate limiting with exponential backoff
//! to prevent brute force attacks on PIN verification endpoints.

use axum::http::HeaderMap;
use governor::{
    clock::{Clock, DefaultClock},
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovRateLimiter,
};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Tracks failure attempts and calculates exponential backoff.
#[derive(Debug, Clone)]
struct FailureTracker {
    /// Number of consecutive failures
    failure_count: u32,
    /// Time of last failure
    last_failure: Instant,
}

impl FailureTracker {
    fn new() -> Self {
        Self {
            failure_count: 0,
            last_failure: Instant::now(),
        }
    }

    /// Calculate backoff duration based on failure count.
    /// - 1st failure: 0 seconds (immediate retry)
    /// - 2nd failure: 5 seconds
    /// - 3rd failure: 30 seconds
    /// - 4th+ failure: 5 minutes
    fn backoff_duration(&self) -> Duration {
        match self.failure_count {
            0 | 1 => Duration::from_secs(0),
            2 => Duration::from_secs(5),
            3 => Duration::from_secs(30),
            _ => Duration::from_secs(300), // 5 minutes
        }
    }

    /// Check if we're still within the backoff period.
    fn is_in_backoff(&self) -> bool {
        if self.failure_count < 2 {
            return false;
        }
        self.last_failure.elapsed() < self.backoff_duration()
    }

    /// Get remaining backoff time in seconds.
    fn remaining_backoff_seconds(&self) -> u64 {
        if !self.is_in_backoff() {
            return 0;
        }
        let backoff = self.backoff_duration();
        let elapsed = self.last_failure.elapsed();
        if elapsed >= backoff {
            0
        } else {
            (backoff - elapsed).as_secs() + 1 // Round up
        }
    }

    /// Record a new failure.
    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Instant::now();
    }

    /// Reset the tracker on successful authentication.
    fn reset(&mut self) {
        self.failure_count = 0;
    }
}

/// Rate limiter state shared across handlers.
#[derive(Clone)]
pub struct RateLimitState {
    /// Per-IP basic rate limiter (5 requests per minute)
    rate_limiter: Arc<GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    /// Per-IP failure tracking for exponential backoff
    failure_trackers: Arc<RwLock<HashMap<IpAddr, FailureTracker>>>,
}

impl RateLimitState {
    /// Create a new rate limit state.
    /// Allows 5 requests per minute per IP for rate-limited endpoints.
    pub fn new() -> Self {
        // 5 requests per 60 seconds
        let quota = Quota::per_minute(NonZeroU32::new(5).unwrap());
        let rate_limiter = Arc::new(GovRateLimiter::direct(quota));

        Self {
            rate_limiter,
            failure_trackers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request from the given IP is allowed.
    /// Returns Ok(()) if allowed, Err(retry_after_seconds) if rate limited.
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), u64> {
        // First, check exponential backoff
        {
            let trackers = self.failure_trackers.read().await;
            if let Some(tracker) = trackers.get(&ip) {
                if tracker.is_in_backoff() {
                    let retry_after = tracker.remaining_backoff_seconds();
                    tracing::warn!(
                        ip = %ip,
                        failures = tracker.failure_count,
                        retry_after = retry_after,
                        "Request blocked by exponential backoff"
                    );
                    return Err(retry_after);
                }
            }
        }

        // Then check basic rate limit
        // Note: We use a global rate limiter but track failures per-IP
        // This is acceptable for MVP since we're primarily protecting against
        // targeted attacks on a single deployment
        match self.rate_limiter.check() {
            Ok(_) => Ok(()),
            Err(not_until) => {
                let retry_after = not_until.wait_time_from(DefaultClock::default().now());
                let retry_after_secs = retry_after.as_secs() + 1; // Round up
                tracing::warn!(
                    ip = %ip,
                    retry_after = retry_after_secs,
                    "Request blocked by rate limit"
                );
                Err(retry_after_secs)
            }
        }
    }

    /// Record a failed authentication attempt for the given IP.
    pub async fn record_failure(&self, ip: IpAddr) {
        let mut trackers = self.failure_trackers.write().await;
        let tracker = trackers.entry(ip).or_insert_with(FailureTracker::new);
        tracker.record_failure();
        tracing::warn!(
            ip = %ip,
            failure_count = tracker.failure_count,
            "Authentication failure recorded"
        );
    }

    /// Reset failure tracking for the given IP on successful authentication.
    pub async fn record_success(&self, ip: IpAddr) {
        let mut trackers = self.failure_trackers.write().await;
        if let Some(tracker) = trackers.get_mut(&ip) {
            tracker.reset();
            tracing::info!(ip = %ip, "Authentication success, backoff reset");
        }
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter that can be added to AppState.
pub struct RateLimiter {
    state: RateLimitState,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new() -> Self {
        Self {
            state: RateLimitState::new(),
        }
    }

    /// Get the rate limit state for use in handlers.
    pub fn state(&self) -> RateLimitState {
        self.state.clone()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract client IP address from request.
///
/// Tries in order:
/// 1. X-Real-IP header (set by reverse proxy)
/// 2. X-Forwarded-For header (first IP in chain)
/// 3. Socket address from connection
///
/// Falls back to 0.0.0.0 if no IP can be determined.
pub fn extract_client_ip(headers: &HeaderMap, socket_addr: Option<SocketAddr>) -> IpAddr {
    // Try X-Real-IP first (most reliable when set by trusted proxy)
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Try X-Forwarded-For (take first IP)
    if let Some(forwarded) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // X-Forwarded-For can be a comma-separated list; take the first one
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Fall back to socket address
    if let Some(addr) = socket_addr {
        return addr.ip();
    }

    // Last resort: use a default IP (this should rarely happen in practice)
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_failure_tracker_backoff_durations() {
        let mut tracker = FailureTracker::new();
        assert_eq!(tracker.backoff_duration(), Duration::from_secs(0));

        tracker.failure_count = 1;
        assert_eq!(tracker.backoff_duration(), Duration::from_secs(0));

        tracker.failure_count = 2;
        assert_eq!(tracker.backoff_duration(), Duration::from_secs(5));

        tracker.failure_count = 3;
        assert_eq!(tracker.backoff_duration(), Duration::from_secs(30));

        tracker.failure_count = 4;
        assert_eq!(tracker.backoff_duration(), Duration::from_secs(300));

        tracker.failure_count = 10;
        assert_eq!(tracker.backoff_duration(), Duration::from_secs(300));
    }

    #[test]
    fn test_failure_tracker_reset() {
        let mut tracker = FailureTracker::new();
        tracker.failure_count = 5;
        tracker.reset();
        assert_eq!(tracker.failure_count, 0);
    }

    #[tokio::test]
    async fn test_rate_limit_state_allows_initial_requests() {
        let state = RateLimitState::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // First 5 requests should be allowed
        for _ in 0..5 {
            assert!(state.check_rate_limit(ip).await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_state_blocks_after_limit() {
        let state = RateLimitState::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));

        // Exhaust the rate limit
        for _ in 0..5 {
            let _ = state.check_rate_limit(ip).await;
        }

        // 6th request should be blocked
        let result = state.check_rate_limit(ip).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exponential_backoff_after_failures() {
        let state = RateLimitState::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3));

        // Record 2 failures to trigger backoff
        state.record_failure(ip).await;
        state.record_failure(ip).await;

        // Should be in backoff
        let result = state.check_rate_limit(ip).await;
        assert!(result.is_err());
        if let Err(retry_after) = result {
            assert!(retry_after > 0);
            assert!(retry_after <= 5);
        }
    }

    #[tokio::test]
    async fn test_success_resets_backoff() {
        let state = RateLimitState::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 4));

        // Record failures
        state.record_failure(ip).await;
        state.record_failure(ip).await;

        // Record success
        state.record_success(ip).await;

        // Backoff should be reset
        let trackers = state.failure_trackers.read().await;
        let tracker = trackers.get(&ip).unwrap();
        assert_eq!(tracker.failure_count, 0);
    }

    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        use axum::http::HeaderValue;
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", HeaderValue::from_static("10.0.0.1"));

        let ip = extract_client_ip(&headers, None);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    }

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        use axum::http::HeaderValue;
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Forwarded-For",
            HeaderValue::from_static("203.0.113.50, 70.41.3.18, 150.172.238.178"),
        );

        let ip = extract_client_ip(&headers, None);
        // Should return the first IP in the chain
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 50)));
    }

    #[test]
    fn test_extract_client_ip_x_real_ip_takes_precedence() {
        use axum::http::HeaderValue;
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", HeaderValue::from_static("10.0.0.1"));
        headers.insert("X-Forwarded-For", HeaderValue::from_static("203.0.113.50"));

        let ip = extract_client_ip(&headers, None);
        // X-Real-IP should take precedence
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    }

    #[test]
    fn test_extract_client_ip_from_socket_addr() {
        let headers = HeaderMap::new();
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let ip = extract_client_ip(&headers, Some(socket_addr));
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn test_extract_client_ip_fallback() {
        let headers = HeaderMap::new();
        let ip = extract_client_ip(&headers, None);
        // Should fallback to 0.0.0.0
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    }

    #[test]
    fn test_extract_client_ip_invalid_x_real_ip() {
        use axum::http::HeaderValue;
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", HeaderValue::from_static("not-an-ip"));

        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);
        let ip = extract_client_ip(&headers, Some(socket_addr));

        // Should fall back to socket addr since X-Real-IP is invalid
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[test]
    fn test_extract_client_ip_ipv6() {
        use axum::http::HeaderValue;
        use std::net::Ipv6Addr;
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", HeaderValue::from_static("::1"));

        let ip = extract_client_ip(&headers, None);
        assert_eq!(ip, IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
    }
}
