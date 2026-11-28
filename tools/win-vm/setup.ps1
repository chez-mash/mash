# First-logon setup of the Windows golden image, run once by autounattend.xml
# as the auto-logged-on test user `mash`, from the unattend ISO. Installs the
# virtio drivers the VM needs, turns on OpenSSH Server with the Mac's key,
# and quiets the desktop so screenshots show only the app under test. Writes
# C:\mash-vm\ready.txt last; mac.sh build-image waits for that file.
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force C:\mash-vm | Out-Null
Start-Transcript -Path C:\mash-vm\setup.log -Append
$media = Split-Path -Parent $MyInvocation.MyCommand.Path

function Set-Reg($path, $name, $value, $type = 'DWord') {
  if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
  New-ItemProperty -Path $path -Name $name -Value $value -PropertyType $type -Force | Out-Null
}

# Drivers from virtio-win.iso: the network card first, because OpenSSH Server
# comes from Windows Update, and the display, which lets Windows use the
# 1440x900 mode QEMU offers. The disk is NVMe and needs nothing.
$virtio = Get-PSDrive -PSProvider FileSystem | Where-Object { Test-Path "$($_.Root)NetKVM\w11\ARM64" } | Select-Object -First 1
if ($virtio) {
  foreach ($driver in 'NetKVM', 'viogpudo', 'vioinput', 'Balloon') {
    pnputil /add-driver "$($virtio.Root)$driver\w11\ARM64\*.inf" /install
  }
} else {
  Write-Output 'virtio-win.iso not found; no network or display driver'
}

# Check an actual HTTPS request with a deadline. Test-NetConnection stalled
# in this guest even while curl reached the same host successfully.
$networkReady = $false
for ($i = 0; $i -lt 12; $i++) {
  curl.exe --silent --fail --head --connect-timeout 5 --max-time 10 https://www.microsoft.com *> $null
  if ($LASTEXITCODE -eq 0) { $networkReady = $true; break }
  Start-Sleep -Seconds 5
}
if (-not $networkReady) { throw 'HTTPS is unavailable; see C:\mash-vm\setup.log before retrying setup' }

# OpenSSH Server. Windows 11 ships it as an optional capability that Windows
# Update provides, so this has to happen before updates are turned off.
if (-not (Get-Service sshd -ErrorAction SilentlyContinue)) {
  Add-WindowsCapability -Online -Name 'OpenSSH.Server~~~~0.0.1.0'
}
Set-Service sshd -StartupType Automatic
Start-Service sshd
if (-not (Get-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -ErrorAction SilentlyContinue)) {
  New-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -DisplayName 'OpenSSH Server (sshd)' -Enabled True -Profile Any -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22 | Out-Null
} else {
  # The capability's existing rule can be Private-only, while QEMU's network
  # is Public. Host forwarding is bound to loopback by mac.sh.
  Set-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -Enabled True -Profile Any -Action Allow
}
# mash is an administrator, and OpenSSH reads administrators' keys from one
# file that only SYSTEM and Administrators may open; any other ACL makes sshd
# ignore it.
$keys = 'C:\ProgramData\ssh\administrators_authorized_keys'
Copy-Item "$media\authorized_keys" $keys -Force
icacls $keys /inheritance:r /grant 'Administrators:F' /grant 'SYSTEM:F' | Out-Null

# Keep the automatic logon for good: the answer file's count runs down.
$winlogon = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon'
Set-Reg $winlogon AutoAdminLogon '1' String
Set-Reg $winlogon DefaultUserName 'mash' String
Set-Reg $winlogon DefaultPassword 'mash-vm-test' String
Remove-ItemProperty $winlogon -Name AutoLogonCount -ErrorAction SilentlyContinue
# No consent prompts for the test administrator, so installs run unattended.
Set-Reg 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System' ConsentPromptBehaviorAdmin 0

# Windows Update stays off: a run must not reboot or change under a test.
Set-Reg 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU' NoAutoUpdate 1
Set-Reg 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate' SetDisableUXWUAccess 1
Set-Reg 'HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings' PauseUpdatesExpiryTime '2099-12-31T00:00:00Z' String
Stop-Service wuauserv -Force -ErrorAction SilentlyContinue
Set-Service wuauserv -StartupType Disabled
# Nor may the store update apps behind a test.
Set-Reg 'HKLM:\SOFTWARE\Policies\Microsoft\WindowsStore' AutoDownload 2

# Never sleep, blank or lock: the screen must be there whenever VNC looks.
powercfg /change monitor-timeout-ac 0
powercfg /change standby-timeout-ac 0
powercfg /hibernate off
Set-Reg 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Personalization' NoLockScreen 1
Set-Reg 'HKCU:\Control Panel\Desktop' ScreenSaveActive '0' String

# Dark mode and a plain dark wallpaper.
$personalize = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
Set-Reg $personalize AppsUseLightTheme 0
Set-Reg $personalize SystemUsesLightTheme 0
Set-Reg $personalize EnableTransparency 0
Set-Reg 'HKCU:\Control Panel\Desktop' Wallpaper '' String
Set-Reg 'HKCU:\Control Panel\Colors' Background '32 34 38' String
Set-Reg 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Wallpapers' BackgroundType 1

# Quiet: no toasts, tips, suggestions or "finish setting up" screens.
Set-Reg 'HKCU:\Software\Microsoft\Windows\CurrentVersion\PushNotifications' ToastEnabled 0
Set-Reg 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings' NOC_GLOBAL_SETTING_TOASTS_ENABLED 0
Set-Reg 'HKCU:\Software\Microsoft\Windows\CurrentVersion\UserProfileEngagement' ScoobeSystemSettingEnabled 0
$cdm = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager'
foreach ($name in 'SubscribedContent-310093Enabled', 'SubscribedContent-338388Enabled', 'SubscribedContent-338389Enabled',
  'SubscribedContent-338393Enabled', 'SubscribedContent-353694Enabled', 'SubscribedContent-353696Enabled',
  'SystemPaneSuggestionsEnabled', 'SoftLandingEnabled', 'SilentInstalledAppsEnabled') {
  Set-Reg $cdm $name 0
}
Set-Reg 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' DisableWindowsConsumerFeatures 1
Set-Reg 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' HideFirstRunExperience 1

# The display mode: QEMU's virtio GPU offers 1440x900 as preferred, but say
# so explicitly in case Windows picked another mode before the driver came.
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Display {
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)]
  public struct DEVMODE {
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmDeviceName;
    public short dmSpecVersion, dmDriverVersion, dmSize, dmDriverExtra;
    public int dmFields, dmPositionX, dmPositionY, dmDisplayOrientation, dmDisplayFixedOutput;
    public short dmColor, dmDuplex, dmYResolution, dmTTOption, dmCollate;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmFormName;
    public short dmLogPixels;
    public int dmBitsPerPel, dmPelsWidth, dmPelsHeight, dmDisplayFlags, dmDisplayFrequency;
    public int dmICMMethod, dmICMIntent, dmMediaType, dmDitherType, dmReserved1, dmReserved2, dmPanningWidth, dmPanningHeight;
  }
  [DllImport("user32.dll")] public static extern bool EnumDisplaySettings(string name, int mode, ref DEVMODE dm);
  [DllImport("user32.dll")] public static extern int ChangeDisplaySettings(ref DEVMODE dm, int flags);
  public static int Set(int w, int h) {
    DEVMODE dm = new DEVMODE();
    dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
    EnumDisplaySettings(null, -1, ref dm);
    dm.dmPelsWidth = w; dm.dmPelsHeight = h; dm.dmFields = 0x80000 | 0x100000;
    return ChangeDisplaySettings(ref dm, 1);
  }
}
'@
Write-Output "display mode change: $([Display]::Set(1440, 900))"

Set-Content -Path C:\mash-vm\ready.txt -Value (Get-Date -Format o)
Stop-Transcript
