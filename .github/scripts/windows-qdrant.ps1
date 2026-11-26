# The disposable native Qdrant the Windows test jobs search against.
#
#   windows-qdrant.ps1 start   download the pinned release, start it, wait until it answers
#   windows-qdrant.ps1 show    after a failure: whether it still runs, and the end of its logs
#   windows-qdrant.ps1 stop    end it; does nothing when it was never started
param([Parameter(Mandatory)][ValidateSet('start', 'show', 'stop')][string]$Command)

$root = Join-Path $env:RUNNER_TEMP 'qdrant'
$pidFile = Join-Path $root 'pid'

switch ($Command) {
  'start' {
    New-Item -ItemType Directory -Path $root -Force | Out-Null
    $archive = Join-Path $root 'qdrant.zip'
    Invoke-WebRequest 'https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-pc-windows-msvc.zip' -OutFile $archive
    if ((Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne '6d13553584fd0411f00637519e293331ff0aa1fb79bd9e30db45839bb75de2be') {
      throw 'Qdrant archive checksum mismatch'
    }
    Expand-Archive $archive -DestinationPath $root
    # A process started from a step ends with that step on Windows
    # runners. Create it through WMI so it lives outside the step's
    # process tree and serves the later steps. WMI does not pass this
    # shell's environment, so cmd sets the bind address and redirects
    # the logs.
    $launch = 'cmd.exe /d /c "set QDRANT__SERVICE__HOST=127.0.0.1&& qdrant.exe > stdout.log 2> stderr.log"'
    $created = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = $launch; CurrentDirectory = $root }
    if ($created.ReturnValue -ne 0) { throw "Could not start Qdrant: Win32_Process.Create returned $($created.ReturnValue)" }
    $created.ProcessId | Set-Content $pidFile
    for ($attempt = 0; $attempt -lt 60; $attempt++) {
      if (-not (Get-Process -Id $created.ProcessId -ErrorAction SilentlyContinue)) { throw 'Qdrant exited before readiness' }
      try {
        Invoke-RestMethod 'http://127.0.0.1:6333/readyz' -TimeoutSec 2 | Out-Null
        exit 0
      } catch { Start-Sleep -Seconds 1 }
    }
    throw 'Qdrant did not become ready'
  }
  'show' {
    if (Test-Path $pidFile) {
      $id = [int](Get-Content $pidFile)
      $process = Get-Process -Id $id -ErrorAction SilentlyContinue
      if ($process) { "Launcher process $id is still running." } else { "Launcher process $id has exited." }
      $qdrant = Get-Process -Name qdrant -ErrorAction SilentlyContinue
      if ($qdrant) { "qdrant.exe is running as $($qdrant.Id -join ', ')." } else { 'No qdrant.exe process is running.' }
    } else { 'No Qdrant PID file.' }
    try {
      Invoke-RestMethod 'http://127.0.0.1:6333/readyz' -TimeoutSec 5 | Out-Null
      'Qdrant answers on 127.0.0.1:6333.'
    } catch { "Qdrant does not answer on 127.0.0.1:6333: $($_.Exception.Message)" }
    foreach ($log in 'stdout.log', 'stderr.log') {
      $path = Join-Path $root $log
      "===== $log (last 80 lines) ====="
      if (Test-Path $path) { Get-Content $path -Tail 80 } else { 'missing' }
    }
  }
  'stop' {
    if (Test-Path $pidFile) {
      # The recorded process is cmd; end Qdrant beneath it too.
      taskkill /PID ([int](Get-Content $pidFile)) /T /F 2>&1 | Out-Null
    }
  }
}
