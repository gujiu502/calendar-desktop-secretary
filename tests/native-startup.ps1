param([string]$Executable = (Join-Path $PSScriptRoot '..\src-tauri\target\x86_64-pc-windows-msvc\release\calendar-desktop-secretary.exe'))
$ErrorActionPreference = 'Stop'
$Executable = [IO.Path]::GetFullPath($Executable)
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CalendarWindowProbe {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  public delegate bool EnumProc(IntPtr hwnd, IntPtr data);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr data);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
}
'@
$previousDpi = [CalendarWindowProbe]::SetThreadDpiAwarenessContext([IntPtr](-4))
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9223'
$probeProcess = Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru
$probeClock = [Diagnostics.Stopwatch]::StartNew()
$script:firstWindow = $null
$script:probePid = $probeProcess.Id
try {
  while ($probeClock.Elapsed.TotalSeconds -lt 20 -and $null -eq $script:firstWindow) {
    $probeProcess.Refresh()
    if ($probeProcess.HasExited) { throw "Application exited before showing a window: $($probeProcess.ExitCode)" }
    [CalendarWindowProbe]::EnumWindows({ param($hwnd, $data)
      [uint32]$owner = 0
      [void][CalendarWindowProbe]::GetWindowThreadProcessId($hwnd, [ref]$owner)
      if ($owner -eq $script:probePid -and [CalendarWindowProbe]::IsWindowVisible($hwnd)) {
        $rect = New-Object CalendarWindowProbe+Rect
        [void][CalendarWindowProbe]::GetWindowRect($hwnd, [ref]$rect)
        if (($rect.Right-$rect.Left) -gt 200 -and ($rect.Bottom-$rect.Top) -gt 200) {
          $script:firstWindow = @{ x=$rect.Left; y=$rect.Top; width=$rect.Right-$rect.Left; height=$rect.Bottom-$rect.Top }
          return $false
        }
      }
      return $true
    }, [IntPtr]::Zero) | Out-Null
    if ($null -eq $script:firstWindow) { Start-Sleep -Milliseconds 25 }
  }
  if ($null -eq $script:firstWindow) { throw 'No visible app window within 20 seconds' }
  $startup=@{pid=$probeProcess.Id; firstVisible=$script:firstWindow; firstVisibleMs=$probeClock.ElapsedMilliseconds}
  $startup | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $PSScriptRoot '..\work\native-startup.json') -Encoding utf8
  Write-Output ($startup | ConvertTo-Json -Compress -Depth 5)
  node (Join-Path $PSScriptRoot 'native.mjs') $Executable
  if ($LASTEXITCODE -ne 0) { throw 'Native integration checks failed' }
} finally {
  if (-not $probeProcess.HasExited) { Stop-Process -Id $probeProcess.Id }
  [void][CalendarWindowProbe]::SetThreadDpiAwarenessContext($previousDpi)
  Remove-Item Env:\WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
}
