# Requires Windows PowerShell 5.1 or newer. System installation is the default.
[CmdletBinding()]
param(
    [string]$Version = 'latest',
    [string]$BinDir,
    [string]$ReleaseBaseUrl = 'https://github.com/itzephir/anyq/releases',
    [switch]$NoPathUpdate,
    [ValidateSet('System', 'User')]
    [string]$Scope = 'System'
)

function Get-AnyqSha256 {
    param([string]$Path)

    $algorithm = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($Path)
    try {
        return ([BitConverter]::ToString($algorithm.ComputeHash($stream))).Replace('-', '').ToLowerInvariant()
    } finally {
        $stream.Dispose()
        $algorithm.Dispose()
    }
}

function Publish-Anyq {
    param(
        [pscustomobject]$Install,
        [scriptblock]$WritePath = { param($Value, $Target) [Environment]::SetEnvironmentVariable('Path', $Value, $Target) }
    )

    if ($Install.Scope -notin @('System', 'User') -or $Install.Hash -cnotmatch '^[0-9a-f]{64}$') {
        throw 'Invalid placement request'
    }
    New-Item -ItemType Directory -Force -Path $Install.BinDir | Out-Null
    $destination = Join-Path $Install.BinDir 'anyq.exe'
    if (Test-Path -LiteralPath $destination) {
        $item = Get-Item -LiteralPath $destination -Force
        if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw 'Destination anyq.exe must be a regular file'
        }
    }
    $staged = Join-Path $Install.BinDir ('.anyq-' + [Guid]::NewGuid().ToString('N') + '.exe')
    $backup = Join-Path $Install.BinDir ('.anyq-backup-' + [Guid]::NewGuid().ToString('N') + '.exe')
    $target = if ($Install.Scope -eq 'System') { 'Machine' } else { 'User' }
    $oldPath = [Environment]::GetEnvironmentVariable('Path', $target)
    $published = $false
    $pathChanged = $false
    $completed = $false

    try {
        Copy-Item -LiteralPath $Install.Source -Destination $staged
        if ((Get-AnyqSha256 $staged) -cne $Install.Hash) {
            throw 'Staged binary changed; installed binary preserved'
        }
        if (Test-Path -LiteralPath $destination) {
            [IO.File]::Replace($staged, $destination, $backup)
        } else {
            [IO.File]::Move($staged, $destination)
        }
        $published = $true

        if (-not $Install.NoPathUpdate) {
            $entries = @($oldPath -split ';' | Where-Object { $_ })
            if (-not ($entries | Where-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') -ieq $Install.BinDir.TrimEnd('\') })) {
                & $WritePath (($entries + $Install.BinDir) -join ';') $target
                $pathChanged = $true
            }
        }
        $completed = $true
    } catch {
        if ($published) {
            if (Test-Path -LiteralPath $backup) {
                [IO.File]::Replace($backup, $destination, [NullString]::Value)
            } else {
                Remove-Item -LiteralPath $destination -Force
            }
        }
        if ($pathChanged) { & $WritePath $oldPath $target }
        throw
    } finally {
        if (Test-Path -LiteralPath $staged) { Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue }
        if ($completed -and (Test-Path -LiteralPath $backup)) {
            Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue
        }
    }
}

function New-AnyqPlacementCommand {
    param([pscustomobject]$Install)

    $payload = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes(($Install | ConvertTo-Json -Compress)))
    $worker = @'
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
function Get-AnyqSha256 {
__HASH__
}
try {
    $request = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('__PAYLOAD__')) | ConvertFrom-Json
    & {
__PLACEMENT__
    } $request
    exit 0
} catch {
    [Console]::Error.WriteLine('anyq install: ' + $_.Exception.Message)
    exit 1
}
'@
    $worker = $worker.Replace('__PAYLOAD__', $payload).Replace('__HASH__', ${function:Get-AnyqSha256}.ToString()).Replace('__PLACEMENT__', ${function:Publish-Anyq}.ToString())
    return [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($worker))
}

function Invoke-AnyqPlacement {
    param(
        [pscustomobject]$Install,
        [bool]$Administrator = ([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator),
        [scriptblock]$Elevate = {
            param($Command)
            $executable = Join-Path $PSHOME 'powershell.exe'
            if (-not (Test-Path -LiteralPath $executable)) { $executable = Join-Path $PSHOME 'pwsh.exe' }
            $process = Start-Process -FilePath $executable -Verb RunAs -Wait -PassThru -ArgumentList @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', $Command)
            return $process.ExitCode
        }
    )

    if ($Install.Scope -eq 'System' -and -not $Administrator) {
        Write-Host "Administrator access is required to install in $($Install.BinDir)"
        $code = & $Elevate (New-AnyqPlacementCommand $Install)
        if ($code -ne 0) { throw 'System placement failed; the previous binary was preserved' }
    } else {
        Publish-Anyq $Install
    }
}

# Dot-sourcing exposes the same placement worker for transport/rollback checks.
if ($MyInvocation.InvocationName -eq '.') { return }

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$tempDir = $null

try {
    if ($env:OS -ne 'Windows_NT') { throw 'Use install.sh on macOS and Linux' }
    $arch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    if ($arch -ne 'AMD64') { throw 'This installer supports Windows x64 (AMD64)' }
    if (-not $BinDir) {
        if ($Scope -eq 'User') {
            $BinDir = Join-Path $env:LOCALAPPDATA 'Programs\anyq'
        } else {
            $programs = if ($env:ProgramW6432) { $env:ProgramW6432 } else { [Environment]::GetFolderPath('ProgramFiles') }
            $BinDir = Join-Path $programs 'anyq'
        }
    }
    if (-not [IO.Path]::IsPathRooted($BinDir)) { $BinDir = Join-Path (Get-Location).ProviderPath $BinDir }
    $BinDir = [IO.Path]::GetFullPath($BinDir)

    $base = [Uri]$ReleaseBaseUrl
    if ($base.Scheme -ne 'https' -and -not ($base.Scheme -eq 'http' -and $base.IsLoopback)) {
        throw 'Release URL must use HTTPS (HTTP is allowed only for loopback tests)'
    }
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $ReleaseBaseUrl = $ReleaseBaseUrl.TrimEnd('/')
    if ($Version -eq 'latest') {
        $response = Invoke-WebRequest -UseBasicParsing -Uri "$ReleaseBaseUrl/latest"
        $finalUri = if ($PSVersionTable.PSVersion.Major -lt 6) { $response.BaseResponse.ResponseUri } else { $response.BaseResponse.RequestMessage.RequestUri }
        $Version = $finalUri.AbsolutePath.Split('/')[-1]
    }
    if ($Version -cnotmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$') { throw 'Expected a release version such as v0.1.0' }

    $asset = 'anyq-x86_64-pc-windows-msvc.zip'
    $tempDir = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tempDir | Out-Null
    $archive = Join-Path $tempDir $asset
    $checksums = Join-Path $tempDir 'SHA256SUMS'
    Write-Host "Downloading anyq $Version for Windows x64"
    Invoke-WebRequest -UseBasicParsing -Uri "$ReleaseBaseUrl/download/$Version/$asset" -OutFile $archive
    Invoke-WebRequest -UseBasicParsing -Uri "$ReleaseBaseUrl/download/$Version/SHA256SUMS" -OutFile $checksums
    $lines = @(Get-Content -LiteralPath $checksums | Where-Object { $_ -cmatch ('^[0-9a-f]{64}  ' + [regex]::Escape($asset) + '$') })
    if ($lines.Count -ne 1) { throw 'Missing or ambiguous SHA-256 checksum' }
    $expected = $lines[0].Substring(0, 64)
    if ((Get-AnyqSha256 $archive) -cne $expected) {
        throw 'SHA-256 mismatch; the installed binary was preserved'
    }
    $unpacked = Join-Path $tempDir 'unpacked'
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [IO.Compression.ZipFile]::ExtractToDirectory($archive, $unpacked)
    $binary = Join-Path $unpacked 'anyq.exe'
    if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw 'Archive has no anyq.exe' }
    $reported = & $binary --version
    if ($LASTEXITCODE -ne 0 -or $reported -cne "anyq $($Version.Substring(1))") { throw 'Binary cannot run or its version does not match the release' }
    $install = [pscustomobject]@{
        Source = $binary
        BinDir = $BinDir
        Hash = Get-AnyqSha256 $binary
        Scope = $Scope
        NoPathUpdate = [bool]$NoPathUpdate
    }

    Invoke-AnyqPlacement $install
    if ((Get-AnyqSha256 (Join-Path $BinDir 'anyq.exe')) -cne $install.Hash) {
        throw 'Installed binary does not match the verified download'
    }
    if (-not $NoPathUpdate -and -not ($env:Path -split ';' | Where-Object { $_.TrimEnd('\') -ieq $BinDir.TrimEnd('\') })) {
        $env:Path += ";$BinDir"
    }
    Write-Host "Installed $reported at $(Join-Path $BinDir 'anyq.exe') ($Scope)"
    Write-Host 'Open a new terminal and run: anyq --help'
} catch {
    [Console]::Error.WriteLine("anyq install: $($_.Exception.Message)")
    exit 1
} finally {
    if ($tempDir -and (Test-Path -LiteralPath $tempDir)) { Remove-Item -LiteralPath $tempDir -Recurse -Force }
}
