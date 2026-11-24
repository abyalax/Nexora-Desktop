# Milestone 1 Memory Baseline Verification Script
# Measures idle memory usage of Nexora Desktop application

param(
    [string]$OutputFile = "docs/bench.md"
)

$ErrorActionPreference = "Stop"

Write-Host "=== Nexora Desktop M1 Memory Baseline ===" -ForegroundColor Cyan
Write-Host "Starting memory measurement..." -ForegroundColor Yellow

# Build the project in release mode for accurate measurements
Write-Host "Building release version..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit 1
}

# Start the application
Write-Host "Starting Nexora Desktop..." -ForegroundColor Yellow
$process = Start-Process -FilePath "target/release/nexora.exe" -PassThru

# Wait for application to stabilize (30 seconds)
Write-Host "Waiting 30 seconds for application to stabilize..." -ForegroundColor Yellow
Start-Sleep -Seconds 30

# Get memory usage
$memoryInfo = Get-Process -Id $process.Id | Select-Object Id, ProcessName, @{
    Name = "WorkingSetMB"
    Expression = { [math]::Round($_.WorkingSet64 / 1MB, 2) }
}, @{
    Name = "PrivateMemoryMB"
    Expression = { [math]::Round($_.PrivateMemorySize64 / 1MB, 2) }
}, @{
    Name = "VirtualMemoryMB"
    Expression = { [math]::Round($_.VirtualMemorySize64 / 1MB, 2) }
}

Write-Host "`nMemory Usage:" -ForegroundColor Green
Write-Host "  Working Set: $($memoryInfo.WorkingSetMB) MB"
Write-Host "  Private Memory: $($memoryInfo.PrivateMemoryMB) MB"
Write-Host "  Virtual Memory: $($memoryInfo.VirtualMemoryMB) MB"

# Stop the application
Write-Host "`nStopping application..." -ForegroundColor Yellow
Stop-Process -Id $process.Id -Force

# Record results in markdown
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$entry = @"
## M1 Baseline - $timestamp

- Working Set: $($memoryInfo.WorkingSetMB) MB
- Private Memory: $($memoryInfo.PrivateMemoryMB) MB
- Virtual Memory: $($memoryInfo.VirtualMemoryMB) MB

"@

# Append to docs/bench.md
if (Test-Path $OutputFile) {
    Add-Content -Path $OutputFile -Value $entry
} else {
    Set-Content -Path $OutputFile -Value "# Nexora Desktop Memory Benchmarks`n`n$entry"
}

Write-Host "`nResults saved to $OutputFile" -ForegroundColor Green
Write-Host "=== Verification Complete ===" -ForegroundColor Cyan
