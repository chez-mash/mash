# Install the latest Windows x64 release. MASH_VERSION pins a release;
# MASH_INSTALL_DIR overrides the default $HOME\.local\bin directory.
param([switch] $Setup, [switch] $Dashboard)

$ErrorActionPreference = 'Stop'

function Start-MashSetup {
    param([string] $Executable, [switch] $Dashboard)

    $command = if ($Dashboard) { 'dash' } else { 'init' }
    Write-Host 'Preparing the local search service (Qdrant).'
    & $Executable qdrant install > $null
    if ($LASTEXITCODE -ne 0) {
        $quotedExecutable = $Executable.Replace("'", "''")
        Write-Host 'Mash is installed.'
        Write-Host 'See https://getmash.io/troubleshooting/#qdrant-connection-failed to start the search service.'
        Write-Host 'Then resume setup with:'
        Write-Host "  & '$quotedExecutable' $command"
        throw "Mash could not start the local search service (exit $LASTEXITCODE)."
    }
    if ($Dashboard) {
        Write-Host 'Open the address below in your browser to choose which chats to index.'
        Write-Host 'Keep this terminal open while you use the dashboard.'
        & $Executable dash
        if ($LASTEXITCODE -ne 0) { throw "Mash dashboard exited with an error (exit $LASTEXITCODE)." }
    } else {
        & $Executable init
        if ($LASTEXITCODE -ne 0) { throw "Mash setup did not finish (exit $LASTEXITCODE)." }
    }
}

function Install-Mash {
    param(
        [string] $Directory = $(if ($env:MASH_INSTALL_DIR) { $env:MASH_INSTALL_DIR } else { Join-Path $HOME '.local\bin' }),
        [string] $Version = $env:MASH_VERSION,
        [switch] $Setup,
        [switch] $Dashboard
    )

    if ($Setup -and $Dashboard) { throw 'Choose -Dashboard for browser setup or -Setup for terminal setup.' }
    if ($Setup -and ([Console]::IsInputRedirected -or [Console]::IsOutputRedirected -or [Console]::IsErrorRedirected)) {
        throw '-Setup needs an interactive terminal. Run without -Setup to install the binaries only.'
    }
    if ($env:OS -ne 'Windows_NT') { throw 'This installer supports Windows x64 and ARM64. Use install.sh on macOS or Linux.' }
    $architecture = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    $target = switch ($architecture) {
        'AMD64' { 'x86_64-pc-windows-msvc' }
        'ARM64' { 'aarch64-pc-windows-msvc' }
        default { throw "No prebuilt Mash binary for Windows $architecture; build from source." }
    }
    if ($Version -and $Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?$') {
        throw 'MASH_VERSION must be a release tag such as v0.8.0.'
    }

    $base = 'https://github.com/mashhq/mash/releases/latest/download'
    if ($Version) { $base = "https://github.com/mashhq/mash/releases/download/$Version" }
    $archive = "mash-$target.zip"
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
        # The Visual C++ runtime the programs need, which Windows does not
        # include: the archive carries it, and Windows looks beside a program
        # first. A copy a running program holds open is already in place.
        foreach ($library in Get-ChildItem -LiteralPath (Join-Path $scratch 'unpacked') -Filter '*.dll') {
            try { Copy-Item -LiteralPath $library.FullName -Destination (Join-Path $Directory $library.Name) -Force }
            catch { if (-not (Test-Path -LiteralPath (Join-Path $Directory $library.Name))) { throw } }
        }
    } finally {
        [Net.ServicePointManager]::SecurityProtocol = $securityProtocol
        if ($incoming -and (Test-Path -LiteralPath $incoming)) { Remove-Item -LiteralPath $incoming -Force }
        if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force }
    }
    if ($Directory -notin ($env:PATH -split ';')) { Write-Host "Add $Directory to your user PATH, then open a new terminal." }
    $mash = Join-Path $Directory 'mash.exe'
    $quotedMash = $mash.Replace("'", "''")
    if ($Dashboard) {
        Start-MashSetup -Executable $mash -Dashboard
        return
    }
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
if (-not $env:MASH_INSTALL_DEFINE_ONLY) { Install-Mash -Setup:$Setup -Dashboard:$Dashboard }
