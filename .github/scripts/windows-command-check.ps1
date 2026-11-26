# Check that the Windows app's installer brings the `mash` command with it and
# that its uninstaller takes it away again (docs/internal/desktop-app.md, "The
# mash command"). Run one step at a time, as the user the app is installed for:
#
#   install    install silently; the copy and its PATH entry are there
#   probe      `mash` runs from cmd, from PowerShell, and when started with no
#              shell at all, which is how an agent starts `mash mcp`
#   uninstall  uninstall with the registered quiet uninstall command, as
#              `winget uninstall --silent` does; the copy, its folder and the
#              entry are gone
#   gone       `mash` is no longer found
#
# probe and gone use this process's PATH, so run them in a new session; with
# -FreshPath they use the PATH a new session would get, read from the
# registry, which is what CI can do. Nothing here reads or writes Mash data:
# `mash mcp` answers `initialize` before it opens an index.
param(
    [Parameter(Mandatory)] [ValidateSet('install', 'probe', 'uninstall', 'gone')] [string] $Step,
    [string] $Installer,
    [string] $Product = 'Mash',
    [string] $Node = 'node',
    [switch] $FreshPath
)
$ErrorActionPreference = 'Stop'
$uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$Product"

function Fail([string] $why) { Write-Output "FAIL: $why"; exit 1 }
function Pass([string] $what) { Write-Output "ok: $what" }

function App-Folder {
    $location = (Get-ItemProperty -LiteralPath $uninstallKey -ErrorAction SilentlyContinue).InstallLocation
    if ($location) { return $location.Trim('"') }
    return Join-Path $env:LOCALAPPDATA $Product
}

function User-Path {
    $key = Get-Item -LiteralPath 'HKCU:\Environment'
    # The value as written, with its %VARIABLES% unexpanded.
    return [string] $key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
}

function Has-Folder([string] $path, [string] $folder) {
    $wanted = $folder.TrimEnd('\').ToLowerInvariant()
    return [bool] ($path -split ';' | Where-Object { $_.Trim().Trim('"').TrimEnd('\').ToLowerInvariant() -eq $wanted })
}

if ($FreshPath) {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = "$machine;$user"
}

switch ($Step) {
    'install' {
        if (-not $Installer) { Fail 'name the installer with -Installer' }
        $process = Start-Process -FilePath $Installer -ArgumentList '/S' -Wait -PassThru
        if ($process.ExitCode -ne 0) { Fail "the installer exited with $($process.ExitCode)" }
        $app = App-Folder
        $engine = Join-Path $app 'mash.exe'
        $command = Join-Path $app 'bin\mash.exe'
        if (-not (Test-Path -LiteralPath $engine)) { Fail "the app's mash is not at $engine" }
        if (-not (Test-Path -LiteralPath $command)) { Fail "the installer left no command at $command" }
        if ((Get-FileHash $engine).Hash -ne (Get-FileHash $command).Hash) { Fail 'the command is not a copy of the app''s mash' }
        Pass "$command is a copy of $engine"
        $listed = Get-ChildItem -LiteralPath (Join-Path $app 'bin') -Filter '*.exe' | ForEach-Object Name
        if ("$listed" -ne 'mash.exe') { Fail "the command's folder holds more programs than mash.exe: $listed" }
        Pass 'the folder on PATH holds no program but mash.exe'
        $path = User-Path
        if (-not (Has-Folder $path (Join-Path $app 'bin'))) { Fail "the user's PATH does not name $app\bin: $path" }
        Pass "the user's PATH ends with $(($path -split ';' | Where-Object { $_ })[-1])"
        $quiet = (Get-ItemProperty -LiteralPath $uninstallKey).QuietUninstallString
        if ($quiet -ne "`"$app\uninstall.exe`" /S") { Fail "the quiet uninstall command is '$quiet'" }
        Pass "the quiet uninstall command is $quiet"
    }
    'probe' {
        Write-Output "PATH: $env:Path"
        $expected = Join-Path (App-Folder) 'bin\mash.exe'
        $found = @((& cmd.exe /d /c 'where mash 2>nul') -split "`r?`n" | Where-Object { $_ })
        Write-Output "where mash: $($found -join ', ')"
        if ($found.Count -eq 0) { Fail 'cmd does not find mash' }
        if ($found[0] -ne $expected) { Write-Output "note: a terminal runs $($found[0]) first, not the app's command" }

        $cmd = & cmd.exe /d /c 'mash --version' 2>&1
        if ($LASTEXITCODE -ne 0 -or "$cmd" -notmatch '^mash \S+') { Fail "cmd: mash --version said: $cmd" }
        Pass "cmd: mash --version: $cmd"

        $shell = @(& powershell.exe -NoProfile -NonInteractive -Command '(Get-Command mash).Source; mash --version' 2>&1)
        if ($LASTEXITCODE -ne 0 -or "$($shell[-1])" -notmatch '^mash \S+') { Fail "PowerShell: mash --version said: $shell" }
        Pass "PowerShell: $($shell -join ': ')"

        # No shell: Node's spawn, as Claude Code starts an MCP server, and
        # CreateProcess itself (Python's subprocess passes the bare name to
        # it). Neither runs a .cmd, which the control below shows.
        $script = @'
const { spawn } = require('child_process');
const name = process.argv[2];
const child = spawn(name, ['mcp'], { shell: false, stdio: ['pipe', 'pipe', 'ignore'] });
let output = '';
child.on('error', (error) => { console.log(`spawn ${name}: ${error.code}`); process.exit(2); });
child.stdout.on('data', (chunk) => {
  output += chunk;
  const line = output.split('\n')[0];
  if (output.includes('\n')) {
    const answer = JSON.parse(line);
    console.log(`initialize: ${answer.result ? answer.result.serverInfo.name + ' ' + answer.result.serverInfo.version + ', protocol ' + answer.result.protocolVersion : JSON.stringify(answer.error)}`);
    child.kill();
    process.exit(answer.result ? 0 : 3);
  }
});
child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'windows-command-check', version: '1' } } }) + '\n');
setTimeout(() => { console.log('no answer within 60 seconds'); child.kill(); process.exit(4); }, 60000);
'@
        $file = Join-Path ([IO.Path]::GetTempPath()) "mash-mcp-check-$PID.js"
        Set-Content -LiteralPath $file -Value $script -Encoding ascii
        $answer = & $Node $file mash 2>&1
        if ($LASTEXITCODE -ne 0) { Fail "no shell: mash mcp: $answer" }
        Pass "no shell (Node spawn): mash mcp $answer"

        $python = Get-Command python, py -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($python) {
            $direct = & $python.Source -c "import subprocess; r = subprocess.run(['mash', '--version'], capture_output=True, text=True); print(r.stdout.strip())" 2>&1
            if ($LASTEXITCODE -ne 0 -or "$direct" -notmatch '^mash \S+') { Fail "no shell: CreateProcess: $direct" }
            Pass "no shell (CreateProcess): $direct"
        }

        # The control: a .cmd on PATH is found by cmd but not without a shell.
        $folder = Join-Path ([IO.Path]::GetTempPath()) "mash-cmd-control-$PID"
        New-Item -ItemType Directory -Path $folder -Force | Out-Null
        Set-Content -LiteralPath (Join-Path $folder 'mash-cmd-control.cmd') -Value '@echo found' -Encoding ascii
        $env:Path = "$folder;$env:Path"
        $viaCmd = & cmd.exe /d /c 'mash-cmd-control' 2>&1
        $viaNode = & $Node $file mash-cmd-control 2>&1
        Write-Output "control: a .cmd from cmd says '$viaCmd'; without a shell: $viaNode"
        Remove-Item -LiteralPath $folder, $file -Recurse -Force
    }
    'uninstall' {
        $app = App-Folder
        # What winget uninstall --silent runs: the quiet uninstall command.
        $quiet = (Get-ItemProperty -LiteralPath $uninstallKey).QuietUninstallString
        if (-not $quiet) { Fail 'no quiet uninstall command is registered' }
        # The uninstaller copies itself away and returns at once, so wait for
        # what it removes.
        if ($quiet -notmatch '^"([^"]+)"\s+(.+)$') { Fail "the quiet uninstall command is not a quoted program and its arguments: $quiet" }
        Start-Process -FilePath $Matches[1] -ArgumentList $Matches[2] -Wait
        $deadline = (Get-Date).AddSeconds(120)
        while ((Test-Path -LiteralPath $uninstallKey) -and (Get-Date) -lt $deadline) { Start-Sleep -Seconds 1 }
        if (Test-Path -LiteralPath $uninstallKey) { Fail 'the uninstaller did not finish within two minutes' }
        if (Test-Path -LiteralPath (Join-Path $app 'bin')) { Fail "$app\bin is still there: $(Get-ChildItem (Join-Path $app 'bin') | ForEach-Object Name)" }
        Pass "$app\bin is gone"
        if (Test-Path -LiteralPath $app) { Fail "$app is still there: $(Get-ChildItem $app | ForEach-Object Name)" }
        Pass "$app is gone"
        $path = User-Path
        if (Has-Folder $path (Join-Path $app 'bin')) { Fail "the user's PATH still names $app\bin: $path" }
        Pass "the user's PATH no longer names $app\bin"
    }
    'gone' {
        $found = @((& cmd.exe /d /c 'where mash 2>nul') -split "`r?`n" | Where-Object { $_ })
        $ours = Join-Path (App-Folder) 'bin\mash.exe'
        if ($found -contains $ours) { Fail "a new session still finds $ours" }
        Pass "a new session does not find $ours$(if ($found) { " (it finds $($found -join ', '))" })"
    }
}
exit 0
