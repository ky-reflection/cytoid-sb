#requires -Version 5.1
param(
    [string]$Source = "$env:USERPROFILE\AppData\LocalLow\TigerHix\Cytoid Lab",
    [string]$Destination = (Join-Path $PSScriptRoot "..\examples\levels")
)

$ErrorActionPreference = "Stop"
$Destination = [System.IO.Path]::GetFullPath($Destination)

if (-not (Test-Path $Source)) {
    Write-Error "Lab cache not found: $Source"
}

New-Item -ItemType Directory -Force -Path $Destination | Out-Null

$dirs = Get-ChildItem $Source -Directory | Where-Object { $_.Name -ne "Unity" }
foreach ($d in $dirs) {
    $target = Join-Path $Destination $d.Name
    if (Test-Path $target) {
        Remove-Item $target -Recurse -Force
    }
    Copy-Item -Path $d.FullName -Destination $target -Recurse -Force
    Write-Host "synced $($d.Name)"
}

$total = (Get-ChildItem $Destination -Recurse -File | Measure-Object -Property Length -Sum).Sum
Write-Host "done: $($dirs.Count) levels, $([math]::Round($total / 1MB, 1)) MB -> $Destination"
