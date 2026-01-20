# Project Vision

## What We're Building

A desktop-first web application for jewelry stores to intake repair jobs, capture photos, print customer receipts with matching physical tags, and manage a clear work queue. The system provides accountability through employee attribution and photo evidence, with offline capability for internet outages.

## Core Features

### MVP
- Repair ticket intake with required photos (minimum 1)
- Printable receipt (PDF) + physical label/tag with matching Ticket ID
- Status workflow: Intake → In Progress → Waiting on Parts → Ready for Pickup → Closed
- FIFO queue with Rush flag override
- Quote amount + actual charged at close
- Storage location tracking (bin, safe drawer, etc.)
- Employee attribution for intake / work / close / key edits
- Audit trail (status changes, pricing edits, photo uploads)
- Offline-capable intake (PWA)

### Phase 2
- Drag-and-drop lane ordering (explicit rank)
- Customer notifications (email/SMS)
- Better reporting (throughput, overdue, estimate vs actual)

### Phase 3
- Multi-tenant productization (multiple stores)
- Customer portal ("track my repair")
- POS integrations

## Non-Goals

These are explicitly out of scope for MVP:
- POS integration, payments, inventory for retail items
- Customer notification (email/SMS) and customer portal
- Advanced technician time tracking, payroll, or job costing
- Multi-location support / multi-tenant productization

## Success Criteria

- Intake completeness rate: required fields present, photos present
- Average intake time under 2 minutes
- Reduction in "lost" items / unclear status
- Reduced disputes due to photo evidence + audit trail
- Staff can answer "where is it and what's happening?" quickly
