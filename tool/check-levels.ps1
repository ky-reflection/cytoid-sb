#requires -Version 5.1
param(
    [string]$LevelsRoot = (Join-Path $PSScriptRoot "..\examples\levels"),
    [string]$CytoidSb = "cargo run -q -p cytoid-sb-cli --"
)

$ErrorActionPreference = "Stop"
$LevelsRoot = [System.IO.Path]::GetFullPath($LevelsRoot)
Push-Location (Join-Path $PSScriptRoot "..")

function Find-StoryboardJson {
    param([string]$LevelDir)
    Get-ChildItem $LevelDir -Filter "*storyboard*.json" -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -notlike "*.generated.json" } |
        Sort-Object Length -Descending
}

$passed = 0
$failed = 0
$skipped = 0

Get-ChildItem $LevelsRoot -Directory | ForEach-Object {
    $storyboards = Find-StoryboardJson $_.FullName
    if (-not $storyboards) {
        Write-Host "skip $($_.Name) (no storyboard json)"
        $skipped++
        return
    }
    foreach ($sb in $storyboards) {
        $rel = $sb.FullName.Substring((Get-Location).Path.Length + 1)
        Write-Host "check $rel ..."
        Invoke-Expression "$CytoidSb check `"$rel`""
        if ($LASTEXITCODE -eq 0) {
            $passed++
        } else {
            $failed++
        }
    }
}

Write-Host "---"
Write-Host "passed=$passed failed=$failed skipped_dirs=$skipped"
Pop-Location
if ($failed -gt 0) { exit 1 }
