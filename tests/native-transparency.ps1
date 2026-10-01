param([Parameter(Mandatory)][int]$AppProcessId)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TransparencyProbe {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint from, uint to, bool attach);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
}
'@
$appWindow = (Get-Process -Id $AppProcessId).MainWindowHandle
if ($appWindow -eq [IntPtr]::Zero) { throw 'Visible app window required' }
$previousDpi = [TransparencyProbe]::SetThreadDpiAwarenessContext([IntPtr](-4))
$previousFocus = [TransparencyProbe]::GetForegroundWindow()
$rect = New-Object TransparencyProbe+Rect
[void][TransparencyProbe]::GetWindowRect($appWindow,[ref]$rect)
$backdrop = New-Object System.Windows.Forms.Form
$focusTarget = New-Object System.Windows.Forms.Form
try {
  $backdrop.FormBorderStyle = 'None'; $backdrop.ShowInTaskbar = $false
  $backdrop.StartPosition = 'Manual'
  $backdrop.Bounds = [Drawing.Rectangle]::new(($rect.Left-30),($rect.Top-30),($rect.Right-$rect.Left+60),($rect.Bottom-$rect.Top+60))
  $backdrop.Show()
  $focusTarget.FormBorderStyle = 'None'; $focusTarget.ShowInTaskbar = $false
  $focusTarget.StartPosition = 'Manual'
  $focusTarget.Bounds = [Drawing.Rectangle]::new(50,50,100,60)
  $focusTarget.Show()
  [void][TransparencyProbe]::SetWindowPos($backdrop.Handle,$appWindow,0,0,0,0,0x13)
  $samples = @{}
  foreach ($state in @('focused','inactive')) {
    $target = if ($state -eq 'focused') { $appWindow } else { $focusTarget.Handle }
    [uint32]$focusPid = 0
    $foregroundThread = [TransparencyProbe]::GetWindowThreadProcessId([TransparencyProbe]::GetForegroundWindow(),[ref]$focusPid)
    $probeThread = [TransparencyProbe]::GetCurrentThreadId()
    $attached = $foregroundThread -ne $probeThread -and [TransparencyProbe]::AttachThreadInput($probeThread,$foregroundThread,$true)
    try { [void][TransparencyProbe]::SetForegroundWindow($target) }
    finally { if ($attached) { [void][TransparencyProbe]::AttachThreadInput($probeThread,$foregroundThread,$false) } }
    foreach ($color in @('Red','Blue')) {
      $backdrop.BackColor = if ($color -eq 'Red') { [Drawing.Color]::FromArgb(220,20,20) } else { [Drawing.Color]::FromArgb(20,20,220) }
      $backdrop.Refresh(); [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 300
      if ([TransparencyProbe]::GetForegroundWindow() -ne $target) { throw "Could not verify $state focus state" }
      $bitmap = [Drawing.Bitmap]::new(1,1)
      $graphics = [Drawing.Graphics]::FromImage($bitmap)
      try {
        $graphics.CopyFromScreen($rect.Left+25,$rect.Top+150,0,0,$bitmap.Size)
        $pixel = $bitmap.GetPixel(0,0)
        $samples["$state-$color"] = @([int]$pixel.R,[int]$pixel.G,[int]$pixel.B)
      } finally { $graphics.Dispose(); $bitmap.Dispose() }
      if ($color -eq 'Blue') {
        $capture = [Drawing.Bitmap]::new(($rect.Right-$rect.Left),($rect.Bottom-$rect.Top))
        $drawing = [Drawing.Graphics]::FromImage($capture)
        try {
          $drawing.CopyFromScreen($rect.Left,$rect.Top,0,0,$capture.Size)
          $capture.Save((Join-Path $PSScriptRoot "..\work\screenshots\native-$state.png"),[Drawing.Imaging.ImageFormat]::Png)
        } finally { $drawing.Dispose(); $capture.Dispose() }
      }
    }
    $red = $samples["$state-Red"]; $blue = $samples["$state-Blue"]
    if ($red[0]-$blue[0] -lt 20 -or $blue[2]-$red[2] -lt 20) { throw "Backdrop no longer visible in $state state : $red / $blue" }
  }
  foreach ($color in @('Red','Blue')) {
    for ($i=0;$i -lt 3;$i++) {
      if ([Math]::Abs($samples["focused-$color"][$i]-$samples["inactive-$color"][$i]) -gt 10) { throw 'Transparency changed on focus loss' }
    }
  }
  Write-Output ('PASS: native focused/inactive backdrop pixels ' + ($samples | ConvertTo-Json -Compress))
} finally {
  $backdrop.Dispose(); $focusTarget.Dispose()
  [void][TransparencyProbe]::SetForegroundWindow($previousFocus)
  [void][TransparencyProbe]::SetThreadDpiAwarenessContext($previousDpi)
}
