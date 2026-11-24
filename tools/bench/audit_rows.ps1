# Audit script for settings catalog rows
# Analyzes Co/UI/Fx status markers in docs/settings_catalog.md

$catalogPath = "docs/settings_catalog.md"
$outputPath = "docs/row_audit.md"

if (-not (Test-Path $catalogPath)) {
    Write-Error "Settings catalog not found at $catalogPath"
    exit 1
}

$content = Get-Content $catalogPath -Raw

# Count status markers by counting lines with each marker
# Looking at the catalog format, the Rust field column is column 8 (index 7)
# We'll count occurrences based on the pattern

$lines = $content -split "`n"
$checkCount = 0
$dotCount = 0
$tildeCount = 0
$xCount = 0
$emptyCount = 0

foreach ($line in $lines) {
    # Look for table rows with setting IDs (XXX-## pattern)
    if ($line -match '\s*\|\s*[A-Z]{3}-\d{2}\s*\|') {
        # Count occurrences of each marker in the line
        if ($line -match '✔') { $checkCount++ }
        if ($line -match '◐') { $dotCount++ }
        if ($line -match '~') { $tildeCount++ }
        if ($line -match '✗') { $xCount++ }
        if ($line -match '⊘') { $emptyCount++ }
    }
}

$total = $checkCount + $dotCount + $tildeCount + $xCount + $emptyCount

# Generate report
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$report = "# Settings Catalog Row Audit`n`n"
$report += "Generated: $timestamp`n`n"
$report += "## Summary`n`n"
$report += "| Status | Count | Percentage |`n"
$report += "|--------|-------|------------|`n"

$checkPct = if ($total -gt 0) { [math]::Round($checkCount / $total * 100, 1) } else { 0 }
$dotPct = if ($total -gt 0) { [math]::Round($dotCount / $total * 100, 1) } else { 0 }
$tildePct = if ($total -gt 0) { [math]::Round($tildeCount / $total * 100, 1) } else { 0 }
$xPct = if ($total -gt 0) { [math]::Round($xCount / $total * 100, 1) } else { 0 }
$emptyPct = if ($total -gt 0) { [math]::Round($emptyCount / $total * 100, 1) } else { 0 }

$report += "| check (exists) | $checkCount | $checkPct% |`n"
$report += "| dot (uncompiled) | $dotCount | $dotPct% |`n"
$report += "| tilde (different model) | $tildeCount | $tildePct% |`n"
$report += "| x (does not exist) | $xCount | $xPct% |`n"
$report += "| empty (not in spec) | $emptyCount | $emptyPct% |`n"
$report += "| **Total** | **$total** | **100%** |`n"

$report += "`n`n## Notes`n`n"
$report += "This is a basic count of status markers in the settings catalog.`n"
$report += "For detailed row-by-row analysis, manual review of the catalog is recommended.`n"

Set-Content -Path $outputPath -Value $report
Write-Host "Audit report saved to $outputPath" -ForegroundColor Green
Write-Host "Total markers found: $total" -ForegroundColor Cyan
Write-Host "  check exists: $checkCount" -ForegroundColor Green
Write-Host "  dot uncompiled: $dotCount" -ForegroundColor Yellow
Write-Host "  tilde different model: $tildeCount" -ForegroundColor Cyan
Write-Host "  x does not exist: $xCount" -ForegroundColor Red
Write-Host "  empty not in spec: $emptyCount" -ForegroundColor Gray
