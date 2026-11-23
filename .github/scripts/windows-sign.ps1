# Signs Windows files with Azure Trusted Signing, when the repository has the
# Azure secrets. Without them it says so and does nothing, so forks and
# unsigned builds keep working.
#
# Reads AZURE_TENANT_ID, AZURE_CLIENT_ID and AZURE_CLIENT_SECRET (the app that
# may sign) and AZURE_SIGNING_ENDPOINT, AZURE_SIGNING_ACCOUNT and
# AZURE_SIGNING_PROFILE (where its certificate lives). The release calls it for
# the command-line programs, and Tauri calls it for each file of the desktop
# app and for its installer.
param([Parameter(Mandatory, ValueFromRemainingArguments)][string[]] $Files)
$ErrorActionPreference = 'Stop'

$needed = 'AZURE_TENANT_ID', 'AZURE_CLIENT_ID', 'AZURE_CLIENT_SECRET',
  'AZURE_SIGNING_ENDPOINT', 'AZURE_SIGNING_ACCOUNT', 'AZURE_SIGNING_PROFILE'
$missing = @($needed | Where-Object { -not [Environment]::GetEnvironmentVariable($_) })
if ($missing.Count -eq $needed.Count) {
  Write-Host 'Azure signing secrets are not set; leaving the files unsigned.'
  exit 0
}
if ($missing.Count) { throw "Azure signing is half configured; missing: $($missing -join ', ')" }

if (-not (Get-Module -ListAvailable -Name TrustedSigning)) {
  Install-Module -Name TrustedSigning -Scope CurrentUser -Force -Repository PSGallery
}
$paths = @($Files | ForEach-Object { (Resolve-Path -LiteralPath $_).Path })
Invoke-TrustedSigning -Endpoint $env:AZURE_SIGNING_ENDPOINT `
  -CodeSigningAccountName $env:AZURE_SIGNING_ACCOUNT `
  -CertificateProfileName $env:AZURE_SIGNING_PROFILE `
  -Files ($paths -join ',') -FileDigest SHA256 `
  -TimestampRfc3161 'http://timestamp.acs.microsoft.com' -TimestampDigest SHA256

# A signing service can answer without having signed; check every file.
foreach ($path in $paths) {
  $signature = Get-AuthenticodeSignature -LiteralPath $path
  if ($signature.Status -ne 'Valid') { throw "$path is not validly signed: $($signature.StatusMessage)" }
  Write-Host "Signed $path by $($signature.SignerCertificate.Subject)"
}
