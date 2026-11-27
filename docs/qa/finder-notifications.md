# Finder and Island notifications (F-12/I-13)

## Purpose
Verify notification access, deduplication, expiry, menu rendering, and Island handoff.

## Prerequisites
- Windows 11; unpackaged Nexora build; generate notifications from two applications.

## Test Steps
1. Enable notification integration and grant any required Windows consent.
2. Generate, update, and dismiss notifications; inspect Finder and Island views.
3. Restart Nexora and verify expired entries are not restored.

## Expected Behavior
Only live notifications appear, duplicates update in place, and expiry follows the configured duration.

## Actual Behavior
Not run. The unpackaged-app API feasibility spike remains open.

## Recovery
Disable integration and confirm Windows notifications continue to appear normally.
