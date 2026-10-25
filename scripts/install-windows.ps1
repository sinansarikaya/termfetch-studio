# Installs TermFetch for the current user in one step:
#   Windows Terminal + fastfetch (winget, if missing), the exe on PATH,
#   a Start Menu shortcut, and the first configuration (`termfetch-studio setup`).
# Safe to run again. The -Skip switches exist for testing.
param(
    [string]$Source,
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\TermFetch'),
    [switch]$SkipTerminal,
    [switch]$SkipFastfetch,
    [switch]$SkipPath,
    [switch]$SkipShortcut,
    [switch]$SkipSetup,
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'

# Undo what the installer added: PATH entry, shortcuts and the install folder.
# The app's own changes (shell blocks, cmd prompt, Windows Terminal) are removed by `termfetch-studio uninstall`.
if ($Uninstall) {
    Step 'Removing TermFetch'
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $kept = @($userPath -split ';' | Where-Object { $_ -and ($_ -ine $InstallDir) })
    [Environment]::SetEnvironmentVariable('Path', ($kept -join ';'), 'User')
    Done 'removed from PATH'
    $programs = [Environment]::GetFolderPath('Programs')
    Get-ChildItem -LiteralPath $programs -Filter 'TermFetch*.lnk' -ErrorAction SilentlyContinue | Remove-Item -Force
    Done 'shortcuts removed'
    # The running exe cannot delete its own folder: a detached process removes it after we exit.
    $defaultDir = Join-Path $env:LOCALAPPDATA 'Programs\TermFetch'
    if ($InstallDir -ieq $defaultDir -and (Test-Path (Join-Path $InstallDir 'termfetch-studio.exe'))) {
        $cleanup = "Start-Sleep -Seconds 3; Remove-Item -LiteralPath '$InstallDir' -Recurse -Force -ErrorAction SilentlyContinue"
        Start-Process powershell -WindowStyle Hidden -ArgumentList '-NoProfile', '-Command', $cleanup
        Done "folder $InstallDir will be removed"
    }
    Write-Host "`nTermFetch is removed. Open a new terminal for the PATH change." -ForegroundColor Green
    exit 0
}

function Step($text) { Write-Host "`n==> $text" -ForegroundColor Cyan }
function Done($text) { Write-Host "    ok: $text" -ForegroundColor Green }

function Install-WingetPackage($id, $label) {
    if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
        throw "winget is not available. Install '$label' manually, then run this script again."
    }
    winget install --id $id --exact --accept-source-agreements --accept-package-agreements --silent
    if ($LASTEXITCODE -ne 0) { throw "winget could not install $label (exit $LASTEXITCODE)." }
}

# 1. Source exe: next to this script (release zip) or the local release build.
Step 'Locating termfetch-studio.exe'
if (-not $Source) {
    $candidates = @(
        (Join-Path $PSScriptRoot 'termfetch-studio.exe'),
        (Join-Path $PSScriptRoot '..\target\release\termfetch-studio.exe')
    )
    $Source = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $Source -or -not (Test-Path $Source)) { throw 'termfetch-studio.exe not found. Pass -Source <path>.' }
$Source = (Resolve-Path $Source).Path
Done $Source

# 2. Windows Terminal: the app's logos and icons need it (the classic console cannot show them).
if (-not $SkipTerminal) {
    Step 'Windows Terminal'
    if (Test-Path (Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps\wt.exe')) {
        Done 'already installed'
    } else {
        Install-WingetPackage 'Microsoft.WindowsTerminal' 'Windows Terminal'
        Done 'installed'
    }
}

# 3. fastfetch: the app runs it for the live preview and the real system values.
if (-not $SkipFastfetch) {
    Step 'fastfetch'
    $packages = Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Packages'
    $present = (Get-Command fastfetch -ErrorAction SilentlyContinue) -or
        (Get-ChildItem $packages -Directory -Filter 'Fastfetch-cli.Fastfetch*' -ErrorAction SilentlyContinue |
            Where-Object { Test-Path (Join-Path $_.FullName 'fastfetch.exe') })
    if ($present) {
        Done 'already installed'
    } else {
        Install-WingetPackage 'Fastfetch-cli.Fastfetch' 'fastfetch'
        Done 'installed'
    }
}

# 4. Copy the exe into a stable folder.
Step "Copying to $InstallDir"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$target = Join-Path $InstallDir 'termfetch-studio.exe'
if ((Resolve-Path -LiteralPath $Source).Path -ieq $target) {
    Done 'already in place'
} else {
    Copy-Item -LiteralPath $Source -Destination $target -Force
    Done $target
}

# 5. PATH (user scope, no admin rights needed).
if (-not $SkipPath) {
    Step 'PATH'
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $entries = @($userPath -split ';' | Where-Object { $_ })
    if ($entries -notcontains $InstallDir) {
        [Environment]::SetEnvironmentVariable('Path', (($entries + $InstallDir) -join ';'), 'User')
        Done 'added (open a new terminal to use `termfetch-studio`)'
    } else {
        Done 'already on PATH'
    }
}

# 6. Short command: `termfetch` works in cmd and PowerShell (the folder is on PATH).
$shim = Join-Path $InstallDir 'termfetch.cmd'
Set-Content -LiteralPath $shim -Value '@"%~dp0termfetch-studio.exe" %*' -Encoding ASCII

# 7. Start Menu shortcuts: the app, and its Settings screen.
if (-not $SkipShortcut) {
    Step 'Start Menu shortcuts'
    $programs = [Environment]::GetFolderPath('Programs')
    $shell = New-Object -ComObject WScript.Shell
    # Shortcuts from earlier versions carried the old name; remove them.
    Get-ChildItem -LiteralPath $programs -Filter 'TermFetch Studio*.lnk' -ErrorAction SilentlyContinue |
        Remove-Item -Force
    $shortcuts = @(
        @{ Name = 'TermFetch.lnk'; Args = ''; Desc = 'TermFetch' },
        @{ Name = 'TermFetch Settings.lnk'; Args = 'settings'; Desc = 'TermFetch settings' }
    )
    foreach ($item in $shortcuts) {
        $lnk = Join-Path $programs $item.Name
        $shortcut = $shell.CreateShortcut($lnk)
        $shortcut.TargetPath = $target
        $shortcut.Arguments = $item.Args
        $shortcut.WorkingDirectory = $env:USERPROFILE
        $shortcut.Description = $item.Desc
        $shortcut.Save()
        Done $lnk
    }
}

# 8. First configuration: fastfetch configs and the shell startup block.
if (-not $SkipSetup) {
    Step 'Configuration'
    & $target setup
    if ($LASTEXITCODE -ne 0) { throw "termfetch-studio setup failed (exit $LASTEXITCODE)." }
}

Write-Host "`nTermFetch is installed." -ForegroundColor Green
Write-Host 'Open it from the Start Menu, or run `termfetch` in a new terminal.'
