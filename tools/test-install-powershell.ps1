# Offline installer checks: fake release assets, real checksums and file replacement.
$ErrorActionPreference = 'Stop'
$env:MASH_INSTALL_DEFINE_ONLY = '1'
. (Join-Path $PSScriptRoot '../install.ps1')
Remove-Item Env:MASH_INSTALL_DEFINE_ONLY

$root = Join-Path ([IO.Path]::GetTempPath()) ('mash-installer-test-' + [Guid]::NewGuid().ToString('N'))
$originalOs = $env:OS
$originalArchitecture = $env:PROCESSOR_ARCHITECTURE
$originalWowArchitecture = $env:PROCESSOR_ARCHITEW6432
$script:mode = 'valid'
$script:requests = @()

function Invoke-WebRequest {
    param([switch] $UseBasicParsing, [string] $Uri, [string] $OutFile)
    $script:requests += $Uri
    if ($script:mode -eq 'network-failure') { throw 'fixture download failed' }
    $asset = if ($Uri.EndsWith('/SHA256SUMS')) { 'SHA256SUMS' } else { 'release.zip' }
    Copy-Item -LiteralPath (Join-Path $root $asset) -Destination $OutFile
}

function Assert-Equal($Actual, $Expected, [string] $Label) {
    if ($Actual -ne $Expected) { throw "$Label : expected '$Expected', got '$Actual'" }
}

function Assert-Fails([scriptblock] $Action, [string] $Message) {
    try { & $Action } catch {
        if ($_.Exception.Message -notlike "*$Message*") { throw }
        return
    }
    throw "Expected failure containing: $Message"
}

try {
    $env:OS = 'Windows_NT'
    $env:PROCESSOR_ARCHITECTURE = 'AMD64'
    $env:PROCESSOR_ARCHITEW6432 = ''
    New-Item -ItemType Directory -Path $root | Out-Null
    $fixture = Join-Path $root 'mash.exe'
    $archive = Join-Path $root 'release.zip'
    $checksums = Join-Path $root 'SHA256SUMS'
    $destination = Join-Path $root "install with 'quotes'"
    $installed = Join-Path $destination 'mash.exe'
    Set-Content -LiteralPath $fixture -Value 'new binary' -NoNewline
    $serverFixture = Join-Path $root 'mash-server.exe'
    Set-Content -LiteralPath $serverFixture -Value 'new server' -NoNewline
    Compress-Archive -LiteralPath @($fixture, $serverFixture) -DestinationPath $archive
    $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    $validChecksum = "$hash  mash-x86_64-pc-windows-msvc.zip"
    Set-Content -LiteralPath $checksums -Value $validChecksum

    $output = Install-Mash -Directory $destination -Version 'v0.8.0' 6>&1 |
        ForEach-Object { $_.ToString() }
    $commandPath = Join-Path (Join-Path $root "install with ''quotes''") 'mash.exe'
    Assert-Equal ($output -contains "  & '$commandPath' qdrant install") $true 'search service command'
    Assert-Equal ($output -contains "  & '$commandPath' init") $true 'setup command'
    Assert-Equal ($output -contains "  & '$commandPath' dash") $true 'browser dashboard command'
    Assert-Equal ($output -contains 'Then choose either the terminal wizard or the browser dashboard.') $true 'choice of setup'
    Assert-Equal ($output -contains 'To set up in your browser, start the dashboard and open the address it prints.') $true 'browser setup guidance'
    Assert-Equal ($output -contains 'The setup wizard lets you choose which coding agents can search your chats.') $true 'optional agent connections'
    Assert-Equal (Get-Content -LiteralPath $installed -Raw) 'new binary' 'fresh install'
    Assert-Equal (Get-Content -LiteralPath (Join-Path $destination 'mash-server.exe') -Raw) 'new server' 'server install'
    Assert-Equal $script:requests[0] 'https://github.com/mashium/mash/releases/download/v0.8.0/mash-x86_64-pc-windows-msvc.zip' 'pinned release URL'

    Set-Content -LiteralPath $installed -Value 'previous binary' -NoNewline
    Set-Content -LiteralPath $checksums -Value (('0' * 64) + '  mash-x86_64-pc-windows-msvc.zip')
    Assert-Fails { Install-Mash -Directory $destination } 'checksum mismatch'
    Assert-Equal (Get-Content -LiteralPath $installed -Raw) 'previous binary' 'bad checksum preserves existing binary'

    Set-Content -LiteralPath $checksums -Value ($validChecksum + "`n" + $validChecksum)
    Assert-Fails { Install-Mash -Directory $destination } 'exactly one entry'
    Set-Content -LiteralPath $checksums -Value ($validChecksum.Replace('windows-msvc', 'other-target'))
    Assert-Fails { Install-Mash -Directory $destination } 'exactly one entry'

    Set-Content -LiteralPath $checksums -Value $validChecksum
    $script:mode = 'network-failure'
    Assert-Fails { Install-Mash -Directory $destination } 'fixture download failed'
    Assert-Equal (Get-Content -LiteralPath $installed -Raw) 'previous binary' 'download failure preserves existing binary'
    $script:mode = 'valid'

    $script:requests = @()
    Install-Mash -Directory $destination -Version ''
    Assert-Equal (Get-Content -LiteralPath $installed -Raw) 'new binary' 'replacement install'
    Assert-Equal $script:requests[0] 'https://github.com/mashium/mash/releases/latest/download/mash-x86_64-pc-windows-msvc.zip' 'latest release URL'
    Assert-Equal @(Get-ChildItem -LiteralPath $destination -Force -Filter '.mash-incoming-*').Count 0 'staging cleanup'

    $setupSteps = Join-Path $root 'setup-steps.txt'
    $env:MASH_INSTALL_TEST_STEPS = $setupSteps
    if ($originalOs -eq 'Windows_NT') {
        $setupFixture = Join-Path $destination 'setup-fixture.cmd'
        @'
@echo off
echo %*>>"%MASH_INSTALL_TEST_STEPS%"
if "%MASH_INSTALL_TEST_QDRANT_FAIL%"=="1" if "%1"=="qdrant" exit /b 7
if "%MASH_INSTALL_TEST_DASH_FAIL%"=="1" if "%1"=="dash" exit /b 11
echo ran %*
'@ | Set-Content -LiteralPath $setupFixture -Encoding Ascii
    } else {
        $setupFixture = Join-Path $destination 'setup-fixture'
        @'
#!/bin/sh
printf '%s\n' "$*" >> "$MASH_INSTALL_TEST_STEPS"
if [ "${MASH_INSTALL_TEST_QDRANT_FAIL:-0}" = 1 ] && [ "$1" = qdrant ]; then exit 7; fi
if [ "${MASH_INSTALL_TEST_DASH_FAIL:-0}" = 1 ] && [ "$1" = dash ]; then exit 11; fi
printf 'ran %s\n' "$*"
'@ | Set-Content -LiteralPath $setupFixture -Encoding Ascii
        chmod +x $setupFixture
    }
    $output = @(Start-MashSetup -Executable $setupFixture 6>$null)
    Assert-Equal ($output -join ',') 'ran init' 'only the wizard writes to output'
    Assert-Equal ((Get-Content -LiteralPath $setupSteps) -join ',') 'qdrant install,init' 'setup order'
    Remove-Item -LiteralPath $setupSteps
    $env:MASH_INSTALL_TEST_QDRANT_FAIL = '1'
    Assert-Fails { Start-MashSetup -Executable $setupFixture 6>$null } 'could not start the local search service'
    Assert-Equal ((Get-Content -LiteralPath $setupSteps) -join ',') 'qdrant install' 'failed search service stops setup'
    Remove-Item Env:MASH_INSTALL_TEST_QDRANT_FAIL
    Remove-Item -LiteralPath $setupSteps
    Start-MashSetup -Executable $setupFixture 6>$null | Out-Null
    Assert-Equal ((Get-Content -LiteralPath $setupSteps) -join ',') 'qdrant install,init' 'setup succeeds on retry'

    Remove-Item -LiteralPath $setupSteps
    $output = @(Start-MashSetup -Executable $setupFixture -Dashboard 6>$null)
    Assert-Equal ($output -join ',') 'ran dash' 'only the dashboard writes to output'
    Assert-Equal ((Get-Content -LiteralPath $setupSteps) -join ',') 'qdrant install,dash' 'browser setup skips the terminal wizard'
    Remove-Item -LiteralPath $setupSteps
    $env:MASH_INSTALL_TEST_QDRANT_FAIL = '1'
    Assert-Fails { Start-MashSetup -Executable $setupFixture -Dashboard 6>$null } 'could not start the local search service'
    Assert-Equal ((Get-Content -LiteralPath $setupSteps) -join ',') 'qdrant install' 'failed search service stops browser setup'
    Remove-Item Env:MASH_INSTALL_TEST_QDRANT_FAIL
    $env:MASH_INSTALL_TEST_DASH_FAIL = '1'
    Assert-Fails { Start-MashSetup -Executable $setupFixture -Dashboard 6>$null } 'dashboard exited with an error'
    Remove-Item Env:MASH_INSTALL_TEST_DASH_FAIL
    Assert-Fails { Install-Mash -Directory $destination -Setup -Dashboard } 'Choose -Dashboard'
    Remove-Item -LiteralPath $setupSteps
    Start-MashSetup -Executable $setupFixture -Dashboard 6>$null | Out-Null
    Assert-Equal ((Get-Content -LiteralPath $setupSteps) -join ',') 'qdrant install,dash' 'browser setup succeeds on retry'

    if ([Console]::IsInputRedirected -or [Console]::IsOutputRedirected -or [Console]::IsErrorRedirected) {
        $downloadsBefore = $script:requests.Count
        $installer = [scriptblock]::Create((Get-Content -LiteralPath (Join-Path $PSScriptRoot '../install.ps1') -Raw))
        Assert-Fails { & $installer -Setup } 'needs an interactive terminal'
        Assert-Equal $script:requests.Count $downloadsBefore 'noninteractive setup does not download'
    }

    Remove-Item -LiteralPath $archive
    $wrongFile = Join-Path $root 'README.txt'
    Set-Content -LiteralPath $wrongFile -Value 'not an executable'
    Compress-Archive -LiteralPath $wrongFile -DestinationPath $archive
    $wrongHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash
    Set-Content -LiteralPath $checksums -Value "$wrongHash  mash-x86_64-pc-windows-msvc.zip"
    Assert-Fails { Install-Mash -Directory $destination } 'contains no mash.exe'
    Assert-Equal (Get-Content -LiteralPath $installed -Raw) 'new binary' 'incomplete archive preserves existing binary'

    Assert-Fails { Install-Mash -Directory $destination -Version '../invalid' } 'release tag'
    $env:PROCESSOR_ARCHITECTURE = 'ARM64'
    Assert-Fails { Install-Mash -Directory $destination } 'No prebuilt Mash binary'
    $env:OS = 'Unix'
    Assert-Fails { Install-Mash -Directory $destination } 'supports Windows x64'
    Write-Host 'PowerShell installer checks passed.'
} finally {
    $env:OS = $originalOs
    $env:PROCESSOR_ARCHITECTURE = $originalArchitecture
    $env:PROCESSOR_ARCHITEW6432 = $originalWowArchitecture
    Remove-Item Env:MASH_INSTALL_TEST_STEPS -ErrorAction SilentlyContinue
    Remove-Item Env:MASH_INSTALL_TEST_QDRANT_FAIL -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $root -Recurse -Force
}
