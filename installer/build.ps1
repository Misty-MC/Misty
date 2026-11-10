<#
.SYNOPSIS
    Prépare une release de Misty (version Rust) : exe, zip (portable + mise à jour automatique), installateur.

.DESCRIPTION
    Produit dans publish\ :
      - Misty-<version>\            l'application (un seul fichier : Misty.exe)
      - Misty-<version>.zip         version portable, à joindre aussi à la release GitHub
                                    (c'est ce zip que télécharge la mise à jour automatique)
      - Misty-Setup-<version>.exe   l'installateur (nécessite Inno Setup 6.3+)

    La version est lue dans misty-rust\src\services\app_info.rs (const VERSION).

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File installer\build.ps1
#>
param(
    # Chemin de ISCC.exe si Inno Setup n'est pas installé à un emplacement standard
    [string]$Iscc = "",
    # Ne génère que le dossier et le zip
    [switch]$SansInstallateur
)

$ErrorActionPreference = "Stop"
$racine = Split-Path -Parent $PSScriptRoot
$projet = Join-Path $racine "misty-rust"

# ── Version ──
$appInfo = Get-Content (Join-Path $projet "src\services\app_info.rs") -Raw
if ($appInfo -notmatch 'pub const VERSION: &str = "([^"]+)"') { throw "Version introuvable dans app_info.rs" }
$version = $Matches[1]
Write-Host "Misty $version" -ForegroundColor Cyan

$publish = Join-Path $racine "publish"
$dossier = Join-Path $publish "Misty-$version"
$zip = Join-Path $publish "Misty-$version.zip"

# ── Misty Client (le mod en jeu, intégré à l'exe par misty-rust\build.rs) : nécessite un JDK 25 ──
Push-Location (Join-Path $racine "misty-client")
try {
    .\gradlew.bat build
    if ($LASTEXITCODE -ne 0) { throw "gradlew build (Misty Client) a échoué" }
} finally {
    Pop-Location
}

# ── Compilation (release : optimisée, sans console) ──
Push-Location $projet
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build a échoué" }
} finally {
    Pop-Location
}

# ── Signature : certificat de signature de code « CN=ENDERR.WIN » du magasin de l'utilisateur ──
# (auto-signé : Windows affiche l'éditeur sur les PC qui font confiance au certificat ; ailleurs,
# SmartScreen reste prudent tant que l'exe n'est pas signé par une autorité reconnue)
$certificat = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert -ErrorAction SilentlyContinue |
    Where-Object { $_.Subject -eq "CN=ENDERR.WIN" -and $_.NotAfter -gt (Get-Date) } |
    Sort-Object NotAfter -Descending | Select-Object -First 1
function Signer([string]$Fichier) {
    if (-not $certificat) { Write-Host "Pas de certificat CN=ENDERR.WIN : $Fichier n'est pas signé" -ForegroundColor Yellow; return }
    $null = Set-AuthenticodeSignature -FilePath $Fichier -Certificate $certificat -HashAlgorithm SHA256 -TimestampServer "http://timestamp.digicert.com"
    # un certificat auto-signé donne le statut « UnknownError » (racine non approuvée) : on vérifie
    # seulement que la signature est bien posée par ce certificat
    $v = Get-AuthenticodeSignature -FilePath $Fichier
    if ($v.SignerCertificate.Thumbprint -ne $certificat.Thumbprint) { throw "Signature de $Fichier impossible : $($v.StatusMessage)" }
    Write-Host "Signé : $Fichier (horodaté : $([bool]$v.TimeStamperCertificate))" -ForegroundColor Green
}
Signer (Join-Path $projet "target\release\Misty.exe")

# ── Dossier de l'application : tout est intégré à l'exe (polices, traductions, images) ──
if (Test-Path $dossier) { Remove-Item $dossier -Recurse -Force }
New-Item -ItemType Directory -Force $dossier | Out-Null
Copy-Item (Join-Path $projet "target\release\Misty.exe") $dossier
Copy-Item (Join-Path $racine "LICENSE.txt") $dossier

# ── Zip (le dossier Misty-<version> doit être à la racine du zip, voir services\update.rs) ──
if (Test-Path $zip) { Remove-Item $zip -Force }
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($dossier, $zip, [System.IO.Compression.CompressionLevel]::Optimal, $true)
Write-Host "Zip : $zip" -ForegroundColor Green

if ($SansInstallateur) { return }

# ── Installateur ──
if (-not $Iscc) {
    $candidats = @(
        (Get-Command ISCC.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source),
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )
    $Iscc = $candidats | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
}
if (-not $Iscc) {
    throw "Inno Setup introuvable. Installe-le (winget install JRSoftware.InnoSetup) ou passe -Iscc <chemin de ISCC.exe>."
}

& $Iscc "/DVersion=$version" "/DNumVersion=$($version -replace '-.*$', '')" "/DSource=$dossier" "/DSortie=$publish" (Join-Path $PSScriptRoot "Misty.iss")
if ($LASTEXITCODE -ne 0) { throw "ISCC a échoué" }

Signer (Join-Path $publish "Misty-Setup-$version.exe")
Write-Host "Installateur : $(Join-Path $publish "Misty-Setup-$version.exe")" -ForegroundColor Green
