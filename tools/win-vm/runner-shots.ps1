# Runs on a GitHub-hosted Windows runner for .github/workflows/windows-shots.yml:
# photographs the Mash desktop app on real Windows without a VM. The runner's
# job runs in a logged-on desktop session, so the app's window appears on its
# screen and can be captured and right-clicked directly. The steps are the
# Windows VM's (guest.ps1): install silently, save the made-up demo chats
# offline, start the app, then take the window and a chat's right-click menu.
#
# Usage: runner-shots.ps1 -Installer PATH -Out FOLDER
param([Parameter(Mandatory)][string]$Installer, [Parameter(Mandatory)][string]$Out)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
New-Item -ItemType Directory -Force -Path $Out | Out-Null

Add-Type -AssemblyName System.Drawing, System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
  public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int x, int y, uint data, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint flags);
}
'@

# The window by itself, as the window draws it: PrintWindow with
# PW_RENDERFULLCONTENT takes WebView2's GPU-composed page, which a copy of the
# screen can leave black on a runner's display.
function WindowShot([IntPtr]$Handle, [string]$Name) {
  $r = New-Object Win+RECT
  [Win]::GetWindowRect($Handle, [ref]$r) | Out-Null
  $bitmap = New-Object System.Drawing.Bitmap ($r.Right - $r.Left), ($r.Bottom - $r.Top)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $dc = $graphics.GetHdc()
  [Win]::PrintWindow($Handle, $dc, 2) | Out-Null
  $graphics.ReleaseHdc($dc)
  $bitmap.Save((Join-Path $Out "$Name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
  $graphics.Dispose(); $bitmap.Dispose()
  Write-Output "took $Name.png (the window alone)"
}

function Shot([string]$Name) {
  $screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $bitmap = New-Object System.Drawing.Bitmap $screen.Width, $screen.Height
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)
  $bitmap.Save((Join-Path $Out "$Name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
  $graphics.Dispose(); $bitmap.Dispose()
  Write-Output "took $Name.png ($($screen.Width)x$($screen.Height))"
}

& "$here\guest.ps1" install $Installer
python "$here\..\mac-vm\demo_history.py" $env:USERPROFILE
& "$here\guest.ps1" index

# Started from this job, the app runs in the runner's own desktop session.
$key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Mash'
$exe = (Get-ItemProperty $key).DisplayIcon.Trim('"')
Start-Process -FilePath $exe
$name = [IO.Path]::GetFileNameWithoutExtension($exe)
$window = $null
for ($i = 0; $i -lt 90 -and -not $window; $i++) {
  Start-Sleep -Seconds 1
  $window = Get-Process -Name $name -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
}
if (-not $window) { Shot '0-no-window'; throw "$name opened no window" }
# The dashboard starts behind the loading page; give it time to draw.
Start-Sleep -Seconds 20
[Win]::SetForegroundWindow($window.MainWindowHandle) | Out-Null
Start-Sleep -Seconds 1
$r = New-Object Win+RECT
[Win]::GetWindowRect($window.MainWindowHandle, [ref]$r) | Out-Null
Write-Output "window: $($r.Left) $($r.Top) $($r.Right - $r.Left) $($r.Bottom - $r.Top) '$($window.MainWindowTitle)'"
Shot '1-window'
WindowShot $window.MainWindowHandle '1-window-alone'
# Whether the dashboard the window shows is up: the engine and its answer.
Get-Process -Name mash -ErrorAction SilentlyContinue | ForEach-Object { Write-Output "engine: $($_.Path)" }
try {
  $page = Invoke-WebRequest -UseBasicParsing -TimeoutSec 10 'http://127.0.0.1:47290/'
  Write-Output "dashboard: $($page.StatusCode), $($page.Content.Length) bytes"
} catch { Write-Output "dashboard: no answer ($($_.Exception.Message))" }

# The first Recent chats row on the overview, from the window's top left
# (WIN_VM_ROW_X/Y as in the VM tool); then the right-click menu it opens.
$x = $r.Left + [int]($env:WIN_VM_ROW_X ?? 420); $y = $r.Top + [int]($env:WIN_VM_ROW_Y ?? 520)
[Win]::SetCursorPos($x, $y) | Out-Null
Start-Sleep -Milliseconds 400
[Win]::mouse_event(0x0008, 0, 0, 0, [UIntPtr]::Zero); [Win]::mouse_event(0x0010, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Seconds 2
Shot '2-right-click'
WindowShot $window.MainWindowHandle '2-right-click-alone'
# The offline `mash index` leaves a failing exit code behind; the pictures are what count.
exit 0
