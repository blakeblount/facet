//! File validation utilities for checking file types by magic bytes.
//!
//! This module provides functions to detect image formats by inspecting
//! the file's magic bytes (file signature) rather than trusting client-provided
//! Content-Type headers.

/// Magic bytes for JPEG images (SOI marker).
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];

/// Magic bytes for PNG images.
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Magic bytes for WebP images - RIFF header.
const WEBP_MAGIC_RIFF: &[u8] = &[0x52, 0x49, 0x46, 0x46]; // "RIFF"

/// Magic bytes for WebP images - WEBP identifier at offset 8.
const WEBP_MAGIC_WEBP: &[u8] = &[0x57, 0x45, 0x42, 0x50]; // "WEBP"

/// Minimum bytes needed to detect any supported image format.
const MIN_BYTES_FOR_DETECTION: usize = 12;

/// Detected image format based on magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG image (image/jpeg)
    Jpeg,
    /// PNG image (image/png)
    Png,
    /// WebP image (image/webp)
    WebP,
}

impl ImageFormat {
    /// Returns the MIME type for this image format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::WebP => "image/webp",
        }
    }
}

/// Detects the image format by inspecting magic bytes.
///
/// Returns `Some(ImageFormat)` if the data starts with recognized magic bytes
/// for JPEG, PNG, or WebP. Returns `None` for unrecognized or invalid formats.
///
/// # Arguments
/// * `data` - The raw file bytes to inspect.
///
/// # Examples
/// ```
/// use api::utils::file_validation::detect_image_format;
///
/// // Valid JPEG
/// let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01];
/// assert!(detect_image_format(&jpeg_data).is_some());
/// ```
pub fn detect_image_format(data: &[u8]) -> Option<ImageFormat> {
    if data.len() < MIN_BYTES_FOR_DETECTION {
        return None;
    }

    // Check JPEG magic bytes (FFD8FF)
    if data.starts_with(JPEG_MAGIC) {
        return Some(ImageFormat::Jpeg);
    }

    // Check PNG magic bytes
    if data.starts_with(PNG_MAGIC) {
        return Some(ImageFormat::Png);
    }

    // Check WebP: RIFF at start and WEBP at offset 8
    if data.starts_with(WEBP_MAGIC_RIFF) && data.len() >= 12 && &data[8..12] == WEBP_MAGIC_WEBP {
        return Some(ImageFormat::WebP);
    }

    None
}

/// Validates that the file content matches the declared Content-Type.
///
/// This function detects the actual image format from magic bytes and compares
/// it against the provided Content-Type header. Returns `true` only if:
/// - The file contains valid magic bytes for a supported format
/// - The detected format matches the declared Content-Type
///
/// # Arguments
/// * `data` - The raw file bytes to validate.
/// * `content_type` - The Content-Type header value from the request.
///
/// # Returns
/// `true` if the detected format matches the content type, `false` otherwise.
///
/// # Examples
/// ```
/// use api::utils::file_validation::validate_image_content_type;
///
/// // JPEG data with correct Content-Type
/// let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01];
/// assert!(validate_image_content_type(&jpeg_data, "image/jpeg"));
///
/// // JPEG data with wrong Content-Type
/// assert!(!validate_image_content_type(&jpeg_data, "image/png"));
/// ```
pub fn validate_image_content_type(data: &[u8], content_type: &str) -> bool {
    match detect_image_format(data) {
        Some(format) => format.mime_type() == content_type,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid JPEG magic bytes with JFIF marker
    const VALID_JPEG: [u8; 12] = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    ];

    // Valid PNG magic bytes
    const VALID_PNG: [u8; 12] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    ];

    // Valid WebP magic bytes (RIFF....WEBP)
    const VALID_WEBP: [u8; 12] = [
        0x52, 0x49, 0x46, 0x46, // RIFF
        0x00, 0x00, 0x00, 0x00, // file size (placeholder)
        0x57, 0x45, 0x42, 0x50, // WEBP
    ];

    // HTML content that might be disguised as an image
    const FAKE_HTML: &[u8] = b"<html><body>malicious</body></html>";

    // Executable-like content
    const FAKE_EXE: &[u8] = &[0x4D, 0x5A, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00];

    #[test]
    fn test_detect_jpeg() {
        assert_eq!(detect_image_format(&VALID_JPEG), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn test_detect_png() {
        assert_eq!(detect_image_format(&VALID_PNG), Some(ImageFormat::Png));
    }

    #[test]
    fn test_detect_webp() {
        assert_eq!(detect_image_format(&VALID_WEBP), Some(ImageFormat::WebP));
    }

    #[test]
    fn test_detect_unknown_format() {
        assert_eq!(detect_image_format(FAKE_HTML), None);
        assert_eq!(detect_image_format(FAKE_EXE), None);
    }

    #[test]
    fn test_detect_too_short() {
        // Less than 12 bytes should return None
        let short_data = [0xFF, 0xD8, 0xFF];
        assert_eq!(detect_image_format(&short_data), None);

        let empty_data: [u8; 0] = [];
        assert_eq!(detect_image_format(&empty_data), None);
    }

    #[test]
    fn test_validate_jpeg_correct_type() {
        assert!(validate_image_content_type(&VALID_JPEG, "image/jpeg"));
    }

    #[test]
    fn test_validate_png_correct_type() {
        assert!(validate_image_content_type(&VALID_PNG, "image/png"));
    }

    #[test]
    fn test_validate_webp_correct_type() {
        assert!(validate_image_content_type(&VALID_WEBP, "image/webp"));
    }

    #[test]
    fn test_validate_jpeg_wrong_type() {
        // JPEG data but claims to be PNG
        assert!(!validate_image_content_type(&VALID_JPEG, "image/png"));
        assert!(!validate_image_content_type(&VALID_JPEG, "image/webp"));
    }

    #[test]
    fn test_validate_png_wrong_type() {
        assert!(!validate_image_content_type(&VALID_PNG, "image/jpeg"));
    }

    #[test]
    fn test_validate_fake_image() {
        // HTML file claiming to be JPEG
        assert!(!validate_image_content_type(FAKE_HTML, "image/jpeg"));
        assert!(!validate_image_content_type(FAKE_HTML, "image/png"));
    }

    #[test]
    fn test_validate_too_short() {
        let short_data = [0xFF, 0xD8, 0xFF];
        assert!(!validate_image_content_type(&short_data, "image/jpeg"));
    }

    #[test]
    fn test_image_format_mime_type() {
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::WebP.mime_type(), "image/webp");
    }
}
