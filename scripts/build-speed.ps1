$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$installation = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$installation) { throw 'Visual Studio C++ x86/x64 build tools required' }
$vcvars = Join-Path $installation 'VC\Auxiliary\Build\vcvarsall.bat'
$source = Join-Path $root 'native\speed'
$vendor = Join-Path $source 'vendor\minhook-1.3.4'
$output = Join-Path $root 'desktop\resources\native-speed'
New-Item -ItemType Directory -Force $output | Out-Null
foreach ($arch in @('32', '64')) {
    $platform = if ($arch -eq '32') { 'x86' } else { 'amd64' }
    $objects = Join-Path $root "native\target\speed$arch"
    New-Item -ItemType Directory -Force $objects | Out-Null
    $inputs = Get-ChildItem -LiteralPath $source -Recurse -File | Where-Object { $_.Extension -in '.cpp', '.c', '.h' }
    $latestInput = ($inputs | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1).LastWriteTimeUtc
    $artifacts = @((Join-Path $output "xiuxian-speed$arch.dll"),
        (Join-Path $output "xiuxian-speed-controller$arch.exe"), (Join-Path $objects 'clock-test.exe'))
    $needsBuild = @($artifacts | Where-Object { !(Test-Path -LiteralPath $_) -or (Get-Item -LiteralPath $_).LastWriteTimeUtc -lt $latestInput })
    if (!$needsBuild.Count) { Write-Output "Native speed $arch is up to date"; continue }
    $decoder = if ($arch -eq '32') { 'hde32.c' } else { 'hde64.c' }
    $commands = @(
        "call `"$vcvars`" $platform >nul",
        "cl /nologo /O2 /MT /EHsc /std:c++17 /LD /I`"$vendor\include`" `"$source\patch.cpp`" `"$vendor\src\buffer.c`" `"$vendor\src\hook.c`" `"$vendor\src\trampoline.c`" `"$vendor\src\hde\$decoder`" /link winmm.lib /OUT:`"$output\xiuxian-speed$arch.dll`"",
        "cl /nologo /O2 /MT /EHsc /std:c++17 `"$source\controller.cpp`" /link /OUT:`"$output\xiuxian-speed-controller$arch.exe`"",
        "cl /nologo /O2 /MT /EHsc /std:c++17 `"$source\clock-test.cpp`" /link winmm.lib /OUT:`"$objects\clock-test.exe`""
    )
    # Keep compiler invocation in one cmd process so vcvars environment applies.
    $batch = Join-Path $objects 'build.cmd'
    ($commands -join "`r`nif errorlevel 1 exit /b 1`r`n") + "`r`nexit /b %errorlevel%" | Set-Content -LiteralPath $batch -Encoding ascii
    Push-Location $objects
    try { & cmd.exe /d /c $batch; if ($LASTEXITCODE -ne 0) { throw "Native speed build failed: $arch" } }
    finally { Pop-Location }
}
Copy-Item -LiteralPath (Join-Path $vendor 'LICENSE.txt') -Destination (Join-Path $output 'MinHook-LICENSE.txt')
