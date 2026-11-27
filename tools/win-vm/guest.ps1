# Runs in the Windows guest for tools/win-vm/mac.sh, over SSH as the test
# user `mash`. An SSH session has no desktop of its own, so anything that must
# appear on screen, or look at windows there, goes through a scheduled task
# that runs in mash's interactive (auto-logged-on) session.
#
# Usage: guest.ps1 install INSTALLER   install a Mash NSIS installer silently
#        guest.ps1 launch              start Mash on the desktop, wait for it
#        guest.ps1 window              print Mash's window as "x y width height"
#        guest.ps1 probe               (in the desktop session) write window.txt
param([Parameter(Mandatory)][string]$Action, [string]$Path)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path

function App-Exe {
  # Tauri's installer records the main executable as the uninstall entry's icon.
  $key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Mash'
  $icon = (Get-ItemProperty $key -ErrorAction SilentlyContinue).DisplayIcon
  if ($icon) { return $icon.Trim('"') }
  $exe = Get-ChildItem "$env:LOCALAPPDATA\Mash" -Filter *.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notin 'uninstall.exe', 'mash.exe' } | Select-Object -First 1
  if (-not $exe) { throw 'Mash is not installed' }
  return $exe.FullName
}

function In-Desktop([string]$Name, [string]$Exe, [string]$Arguments) {
  # A one-off task for this same user with an interactive logon runs in the
  # session that is already on screen; it needs no stored password.
  $action = if ($Arguments) { New-ScheduledTaskAction -Execute $Exe -Argument $Arguments } else { New-ScheduledTaskAction -Execute $Exe }
  $principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Limited
  $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -ExecutionTimeLimit (New-TimeSpan -Hours 12)
  Register-ScheduledTask -TaskName $Name -Action $action -Principal $principal -Settings $settings -Force | Out-Null
  Start-ScheduledTask -TaskName $Name
}

switch ($Action) {
  'install' {
    $p = Start-Process -FilePath $Path -ArgumentList '/S' -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw "installer exited with $($p.ExitCode)" }
    Write-Output "installed $(App-Exe)"
  }
  'launch' {
    $exe = App-Exe
    In-Desktop 'MashLaunch' $exe
    $name = [IO.Path]::GetFileNameWithoutExtension($exe)
    for ($i = 0; $i -lt 60; $i++) {
      if (Get-Process -Name $name -ErrorAction SilentlyContinue) { Write-Output "started $name"; return }
      Start-Sleep -Seconds 1
    }
    throw "$name did not start"
  }
  'window' {
    # Ask the desktop session, where the window lives, and wait for its answer.
    $out = Join-Path $here 'window.txt'
    Remove-Item $out -ErrorAction SilentlyContinue
    $ps = "$env:WINDIR\System32\WindowsPowerShell\v1.0\powershell.exe"
    In-Desktop 'MashProbe' $ps "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$here\guest.ps1`" probe"
    for ($i = 0; $i -lt 60; $i++) {
      if (Test-Path $out) { Get-Content $out; return }
      Start-Sleep -Seconds 1
    }
    throw 'the window probe never answered'
  }
  'probe' {
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
'@
    $exe = App-Exe
    $name = [IO.Path]::GetFileNameWithoutExtension($exe)
    $line = 'none'
    for ($i = 0; $i -lt 60; $i++) {
      $p = Get-Process -Name $name -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
      if ($p) {
        [Win]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
        $r = New-Object Win+RECT
        [Win]::GetWindowRect($p.MainWindowHandle, [ref]$r) | Out-Null
        $line = "$($r.Left) $($r.Top) $($r.Right - $r.Left) $($r.Bottom - $r.Top) $($p.MainWindowTitle)"
        break
      }
      Start-Sleep -Seconds 1
    }
    Set-Content -Path (Join-Path $here 'window.txt') -Value $line
  }
  default { throw "unknown action $Action" }
}
