# Install the latest Windows x64 release. MASH_VERSION pins a release;
# MASH_INSTALL_DIR overrides the default $HOME\.local\bin directory.
param([switch] $Setup)

$ErrorActionPreference = 'Stop'

function Start-MashSetup {
    param([string] $Executable)

    Write-Host 'Preparing the local search service (Qdrant). The setup wizard opens next.'
    & $Executable qdrant install > $null
    if ($LASTEXITCODE -ne 0) { throw "Mash could not start the local search service (exit $LASTEXITCODE)." }
    & $Executable init
    if ($LASTEXITCODE -ne 0) { throw "Mash setup did not finish (exit $LASTEXITCODE)." }
}

function Install-Mash {
    param(
        [string] $Directory = $(if ($env:MASH_INSTALL_DIR) { $env:MASH_INSTALL_DIR } else { Join-Path $HOME '.local\bin' }),
        [string] $Version = $env:MASH_VERSION,
        [switch] $Setup
    )

    if ($Setup -and ([Console]::IsInputRedirected -or [Console]::IsOutputRedirected -or [Console]::IsErrorRedirected)) {
        throw '-Setup needs an interactive terminal. Run without -Setup to install the binaries only.'
    }
    if ($env:OS -ne 'Windows_NT') { throw 'This installer supports Windows x64. Use install.sh on macOS or Linux.' }
    $architecture = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    if ($architecture -ne 'AMD64') { throw "No prebuilt Mash binary for Windows $architecture; build from source." }
    if ($Version -and $Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?$') {
        throw 'MASH_VERSION must be a release tag such as v0.8.0.'
    }

    $base = 'https://github.com/mashium/mash/releases/latest/download'
    if ($Version) { $base = "https://github.com/mashium/mash/releases/download/$Version" }
    $archive = 'mash-x86_64-pc-windows-msvc.zip'
    $scratch = Join-Path ([IO.Path]::GetTempPath()) ('mash-install-' + [Guid]::NewGuid().ToString('N'))
    $incoming = $null
    $securityProtocol = [Net.ServicePointManager]::SecurityProtocol
    try {
        [Net.ServicePointManager]::SecurityProtocol = $securityProtocol -bor [Net.SecurityProtocolType]::Tls12
        New-Item -ItemType Directory -Path $scratch | Out-Null
        $download = Join-Path $scratch $archive
        $checksums = Join-Path $scratch 'SHA256SUMS'
        Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile $download
        Invoke-WebRequest -UseBasicParsing -Uri "$base/SHA256SUMS" -OutFile $checksums
        $pattern = '^([a-fA-F0-9]{64})\s+\*?' + [Regex]::Escape($archive) + '$'
        $matchesForArchive = @(Get-Content -LiteralPath $checksums | Where-Object { $_ -match $pattern })
        if ($matchesForArchive.Count -ne 1) { throw 'Release checksums must contain exactly one entry for the Windows archive.' }
        $expected = [Regex]::Match($matchesForArchive[0], $pattern).Groups[1].Value
        if ((Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash -ne $expected) {
            throw 'Mash archive checksum mismatch; existing installation was not changed.'
        }

        Expand-Archive -LiteralPath $download -DestinationPath (Join-Path $scratch 'unpacked')
        foreach ($name in @('mash.exe', 'mash-server.exe')) {
            $binary = Join-Path $scratch "unpacked\$name"
            if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw "The release archive contains no $name." }
        }
        $Directory = [IO.Path]::GetFullPath($Directory)
        New-Item -ItemType Directory -Path $Directory -Force | Out-Null
        foreach ($name in @('mash.exe', 'mash-server.exe')) {
            $binary = Join-Path $scratch "unpacked\$name"
            $destination = Join-Path $Directory $name
            $incoming = Join-Path $Directory ('.mash-incoming-' + [Guid]::NewGuid().ToString('N') + '.exe')
            Copy-Item -LiteralPath $binary -Destination $incoming
            # Stage beside the destination so replacement stays on one filesystem.
            if (Test-Path -LiteralPath $destination) {
                [IO.File]::Replace($incoming, $destination, [NullString]::Value)
            } else {
                [IO.File]::Move($incoming, $destination)
            }
            Write-Host "Installed $destination"
        }
    } finally {
        [Net.ServicePointManager]::SecurityProtocol = $securityProtocol
        if ($incoming -and (Test-Path -LiteralPath $incoming)) { Remove-Item -LiteralPath $incoming -Force }
        if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force }
    }
    if ($Directory -notin ($env:PATH -split ';')) { Write-Host "Add $Directory to your user PATH, then open a new terminal." }
    $mash = Join-Path $Directory 'mash.exe'
    $quotedMash = $mash.Replace("'", "''")
    Write-Host 'The setup wizard lets you choose which coding agents can search your chats.'
    if ($Setup) {
        Start-MashSetup -Executable $mash
        Write-Host 'After setup, start the browser dashboard and open the address it prints.'
    } else {
        Write-Host 'After an upgrade, reconnect your Mash MCP sessions.'
        Write-Host 'For a new local setup, start the search service first.'
        Write-Host "  & '$quotedMash' qdrant install"
        Write-Host 'Then choose either the terminal wizard or the browser dashboard.'
        Write-Host 'To set up in the terminal, run the wizard.'
        Write-Host "  & '$quotedMash' init"
        Write-Host 'To set up in your browser, start the dashboard and open the address it prints.'
    }
    Write-Host "  & '$quotedMash' dash"
}

# The offline fixture tests load the function without installing by setting
# this variable. The way the script was invoked cannot be trusted for that:
# piped into iex from a script that was itself dot-sourced, as CI runners do,
# it looks dot-sourced too and would install nothing, silently.
if (-not $env:MASH_INSTALL_DEFINE_ONLY) { Install-Mash -Setup:$Setup }
