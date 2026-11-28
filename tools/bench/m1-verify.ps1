# M1 release baseline. Uses the agent control channel for an orderly shutdown.
param(
    [string]$OutputFile = "docs/bench.md",
    [int]$SettleSeconds = 30
)

$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
Push-Location $repo
$env:NACRE_AGENT = "1"
$env:NACRE_CONFIG_DIR = "target/cfg-bench"
# The control pipe is intentionally gated by debug_assertions; keep release optimization for the measurement.
$env:CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS = "true"
$process = $null
$ctl = Join-Path $repo "target/release/nexora-ctl.exe"

try {
    cargo build --release --features agent-debug --bin nexora --bin nexora-ctl
    if ($LASTEXITCODE -ne 0) { throw "Release build failed ($LASTEXITCODE)" }

    $app = Join-Path $repo "target/release/nexora.exe"
    $process = Start-Process -FilePath $app -ArgumentList @("--keep-taskbar", "--exit-after", "120") -PassThru -WindowStyle Hidden

    $deadline = (Get-Date).AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 500
        $process.Refresh()
        if ($process.HasExited) { throw "Nexora exited during startup ($($process.ExitCode))" }
        & $ctl --pid $process.Id state *> $null
        $ready = $LASTEXITCODE -eq 0
    } while (-not $ready -and (Get-Date) -lt $deadline)
    if (-not $ready) { throw "nexora-ctl could not read runtime state" }

    & $ctl --pid $process.Id wait_idle
    if ($LASTEXITCODE -ne 0) { throw "Nexora did not become idle" }
    Start-Sleep -Seconds $SettleSeconds
    $process.Refresh()
    if ($process.HasExited) { throw "Nexora exited before measurement" }

    $memory = Get-Process -Id $process.Id
    $workingSet = [math]::Round($memory.WorkingSet64 / 1MB, 2)
    $privateMemory = [math]::Round($memory.PrivateMemorySize64 / 1MB, 2)
    $virtualMemory = [math]::Round($memory.VirtualMemorySize64 / 1MB, 2)
}
finally {
    if ($null -ne $process) {
        $process.Refresh()
        if (-not $process.HasExited) {
            & $ctl --pid $process.Id quit *> $null
            if ($LASTEXITCODE -ne 0) { Write-Warning "nexora-ctl quit failed; Nexora has --exit-after 120 as a fallback" }
            try { Wait-Process -Id $process.Id -Timeout 10 -ErrorAction Stop } catch { }
        }
    }
    Pop-Location
}

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss zzz"
$entry = @"
## M1 Baseline - $timestamp

- Working Set: $workingSet MB
- Private Memory: $privateMemory MB
- Virtual Memory: $virtualMemory MB
- Runtime: optimized release with debug assertions for agent control, idle after ${SettleSeconds}s, primary display
- Shutdown: nexora-ctl quit; isolated configuration at target/cfg-bench

"@
$outputPath = Join-Path $repo $OutputFile
if (Test-Path $outputPath) {
    Add-Content -Path $outputPath -Value $entry
} else {
    Set-Content -Path $outputPath -Value "# Nexora Desktop Memory Benchmarks`n`n$entry"
}
Write-Host "Recorded release memory baseline in $outputPath"
