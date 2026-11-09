# Requires Windows PowerShell 5.1 or newer. No administrator access is needed.
[CmdletBinding()]
param(
    [string]$Version = 'latest',
    [string]$BinDir = (Join-Path $env:LOCALAPPDATA 'anyq\bin'),
    [string]$ReleaseBaseUrl = 'https://github.com/itzephir/anyq/releases',
    [switch]$NoPathUpdate
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$tempDir = $null
$staged = $null

try {
    if ($env:OS -ne 'Windows_NT') { throw 'Use install.sh on macOS and Linux' }
    $arch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    if ($arch -ne 'AMD64') { throw 'This installer supports Windows x64 (AMD64)' }
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
    if ($Version -cnotmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$') { throw 'Expected a release version such as v0.2.0' }

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
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
    if ($actual -cne $expected) { throw 'SHA-256 mismatch; the installed binary was preserved' }
    $unpacked = Join-Path $tempDir 'unpacked'
    Expand-Archive -LiteralPath $archive -DestinationPath $unpacked
    $binary = Join-Path $unpacked 'anyq.exe'
    if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw 'Archive has no anyq.exe' }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    $BinDir = (Resolve-Path -LiteralPath $BinDir).Path
    $staged = Join-Path $BinDir ('.anyq-' + [Guid]::NewGuid().ToString('N') + '.exe')
    Copy-Item -LiteralPath $binary -Destination $staged
    $reported = & $staged --version
    if ($LASTEXITCODE -ne 0 -or $reported -cne "anyq $($Version.Substring(1))") { throw 'Binary cannot run or its version does not match the release' }
    $destination = Join-Path $BinDir 'anyq.exe'
    if (Test-Path -LiteralPath $destination) {
        [IO.File]::Replace($staged, $destination, $null)
    } else {
        [IO.File]::Move($staged, $destination)
    }
    $staged = $null

    if (-not $NoPathUpdate) {
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        $entries = @($userPath -split ';' | Where-Object { $_ })
        if (-not ($entries | Where-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') -ieq $BinDir.TrimEnd('\') })) {
            [Environment]::SetEnvironmentVariable('Path', (($entries + $BinDir) -join ';'), 'User')
        }
        if (-not ($env:Path -split ';' | Where-Object { $_.TrimEnd('\') -ieq $BinDir.TrimEnd('\') })) { $env:Path += ";$BinDir" }
    }
    Write-Host "Installed $reported at $destination"
    Write-Host 'Open a new terminal and run: anyq --help'
} catch {
    [Console]::Error.WriteLine("anyq install: $($_.Exception.Message)")
    exit 1
} finally {
    if ($staged -and (Test-Path -LiteralPath $staged)) { Remove-Item -LiteralPath $staged -Force }
    if ($tempDir -and (Test-Path -LiteralPath $tempDir)) { Remove-Item -LiteralPath $tempDir -Recurse -Force }
}
