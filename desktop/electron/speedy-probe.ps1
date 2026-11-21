param([int]$TargetId = 0, [string]$Started = '', [switch]$Disable)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
trap { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class SpeedyProcessArchitecture {
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWow64Process(IntPtr handle, out bool wow64);
}
'@
if ($TargetId) {
    $games = @(Get-Process -Id $TargetId -ErrorAction SilentlyContinue)
} else {
    $games = @(Get-Process | Where-Object { $_.MainWindowTitle.StartsWith('懒人修仙传 v') })
}
$result = @($games | ForEach-Object {
    $game = $_
    $identity = $game.StartTime.ToUniversalTime().Ticks.ToString()
    if ($Started -and $identity -ne $Started) { return }
    $wow64 = $false
    if (![SpeedyProcessArchitecture]::IsWow64Process($game.Handle, [ref]$wow64)) { throw '无法读取游戏进程位数' }
    $architecture = if ([Environment]::Is64BitOperatingSystem -and !$wow64) { '64' } else { '32' }
    # .NET Process.Modules must enumerate WOW64 modules from a 32-bit host.
    if ($architecture -eq '32' -and [Environment]::Is64BitProcess) {
        $arguments = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $PSCommandPath,
            '-TargetId', $game.Id, '-Started', $identity)
        if ($Disable) { $arguments += '-Disable' }
        $json = & "$env:WINDIR\SysWOW64\WindowsPowerShell\v1.0\powershell.exe" @arguments
        if ($LASTEXITCODE -ne 0) { throw '读取 32 位游戏状态失败' }
        return ($json | ConvertFrom-Json)
    }
    $enabled = $false
    $ready = $false
    try {
        $map = [IO.MemoryMappedFiles.MemoryMappedFile]::OpenExisting("Local\Xiuxian.Speed.v1.$($game.Id)")
        try {
            $view = $map.CreateViewAccessor()
            try {
                if ($Disable) { $view.Write(0, [int]0); $view.Flush() }
                $enabled = $view.ReadInt32(0) -eq 1
                $ready = $view.ReadInt32(4) -eq 1
            } finally { $view.Dispose() }
        } finally { $map.Dispose() }
    } catch [IO.FileNotFoundException] { }
    [pscustomobject]@{
        id = $game.Id; started = $identity; exe = $game.Path; enabled = $enabled; ready = $ready; arch = $architecture
        modules = @($game.Modules | Where-Object { $_.ModuleName -match '^(xiuxian-speed|speedpatch)(32|64)\.dll$' } | ForEach-Object { $_.FileName })
    }
})
ConvertTo-Json -InputObject $result -Compress -Depth 4
