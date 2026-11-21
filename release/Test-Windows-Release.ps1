param(
    [string]$GameFolder = (Split-Path -Parent $PSScriptRoot)
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path -LiteralPath $GameFolder).Path
$Game = Join-Path $Root 'bin/windows/run-carter.exe'
if (-not (Test-Path -LiteralPath $Game -PathType Leaf)) {
    throw "The Windows executable is missing. Extract the complete game folder first."
}

function Invoke-GameCheck {
    param([string]$Name, [string[]]$Arguments, [string]$Expected)
    $Output = New-TemporaryFile
    $Errors = New-TemporaryFile
    try {
        $Options = @{
            FilePath = $Game
            ArgumentList = $Arguments
            WorkingDirectory = $Output.DirectoryName
            PassThru = $true
            RedirectStandardOutput = $Output.FullName
            RedirectStandardError = $Errors.FullName
        }
        $Process = Start-Process @Options
        if (-not $Process.WaitForExit(60000)) {
            Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
            throw "$Name timed out."
        }
        $Log = Get-Content -Raw -LiteralPath $Output.FullName
        if ($Process.ExitCode -ne 0 -or $Log -notmatch $Expected) {
            $Log | Write-Host
            Get-Content -LiteralPath $Errors.FullName | Write-Host
            throw "$Name failed (exit code $($Process.ExitCode))."
        }
        Write-Host "$Name passed."
    } finally {
        Remove-Item -LiteralPath $Output.FullName, $Errors.FullName -ErrorAction SilentlyContinue
    }
}

Invoke-GameCheck -Name 'Office rendering' -Arguments @('--smoke-frames=60') -Expected 'RUN_CARTER_SMOKE_OK frames=60 screen=office'
Invoke-GameCheck -Name 'CarterOS rendering' -Arguments @('--smoke-frames=60','--smoke-desktop') -Expected 'RUN_CARTER_SMOKE_OK frames=60 screen=desktop'
Invoke-GameCheck -Name 'Toy prop assets' -Arguments @('--audit-weapon-animations') -Expected 'ANIM_AUDIT_RESULT failures=0 models=16'
