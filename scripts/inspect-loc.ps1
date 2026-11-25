
param(
    [string]$Path = (Join-Path $PSScriptRoot ".."),
    [int]$Threshold = 500,
    [switch]$IncludeHidden
)

$ErrorActionPreference = "Stop"

# Source code extensions
$Extensions = @(
    ".cs", ".java", ".kt", ".kts",
    ".js", ".jsx", ".mjs", ".cjs",
    ".ts", ".tsx", ".vue", ".svelte",
    ".py", ".go", ".rs", ".php",
    ".rb", ".swift", ".dart",
    ".c", ".h", ".cpp", ".hpp",
    ".css", ".scss", ".sass", ".less",
    ".html", ".htm",
    ".sql", ".sh", ".ps1",
    ".json", ".yaml", ".yml",
    ".xml", ".graphql", ".gql"
)

# Directories to ignore
$ExcludedDirectories = @(
    "node_modules",
    "dist",
    "build",
    "out",
    ".git",
    ".next",
    ".nuxt",
    ".output",
    ".turbo",
    ".cache",
    "coverage",
    "vendor",
    "bin",
    "obj",
    ".venv",
    "venv",
    "__pycache__",
    ".idea",
    ".vscode"
)

$ResolvedPath = Resolve-Path $Path

Write-Host "Source Code LOC Inspector" -ForegroundColor Cyan
Write-Host "Root      : $ResolvedPath"
Write-Host "Threshold : $Threshold lines"

$Files = Get-ChildItem `
    -LiteralPath $ResolvedPath `
    -File `
    -Recurse `
    -Force:$IncludeHidden `
    -ErrorAction SilentlyContinue |
Where-Object {
    $File = $_

    $IsSourceFile = $Extensions -contains $File.Extension.ToLower()

    $IsExcluded = $false

    $RelativePath = $File.FullName.Substring(
        $ResolvedPath.Path.TrimEnd('\').Length
    ).TrimStart('\')

    $DirectoryParts = $RelativePath -split '[\\/]'

    foreach ($Directory in $DirectoryParts) {
        if ($ExcludedDirectories -contains $Directory) {
            $IsExcluded = $true
            break
        }
    }

    $IsSourceFile -and -not $IsExcluded
}

$Results = [System.Collections.Generic.List[object]]::new()

$Scanned = 0

foreach ($File in $Files) {
    try {
        $LineCount = 0

        $Reader = [System.IO.File]::OpenText($File.FullName)

        try {
            while ($null -ne $Reader.ReadLine()) {
                $LineCount++
            }
        }
        finally {
            $Reader.Dispose()
        }

        $Scanned++

        if ($LineCount -gt $Threshold) {
            $Results.Add([PSCustomObject]@{
                    LOC       = $LineCount
                    Extension = $File.Extension
                    File      = $File.Name
                    Path      = $File.FullName
                })
        }
    }
    catch {
        Write-Verbose "Cannot read: $($File.FullName)"
    }
}

$Results = $Results | Sort-Object LOC -Descending

Write-Host "Inspection Result " -ForegroundColor Cyan

if ($Results.Count -eq 0) {
    Write-Host "No files exceeded $Threshold LOC." -ForegroundColor Green
}
else {
    $Results |
    Format-Table `
    @{Label = "LOC"; Expression = { $_.LOC }; Align = "Right" },
    Extension,
    File,
    Path `
        -AutoSize |
    Out-Host

    Write-Host ""
    Write-Host "Files Over Threeshold " -ForegroundColor Yellow

    Write-Host "Total flagged : $($Results.Count)"
    Write-Host "Maximum LOC   : $(($Results | Measure-Object LOC -Maximum).Maximum)"
    Write-Host "Average LOC   : $([math]::Round(($Results | Measure-Object LOC -Average).Average, 2))"
}

Write-Host ""
Write-Host "Scan Summary " -ForegroundColor Cyan
Write-Host "Files scanned : $Scanned"
Write-Host "Files flagged : $($Results.Count)"
Write-Host "Threshold     : $Threshold LOC"
Write-Host ""
Write-Host "Inspection completed." -ForegroundColor Green
