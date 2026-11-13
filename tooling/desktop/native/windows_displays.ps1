param([string]$Output = "desktop-shell-results/windows")

$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Virtual display setup requires a disposable GitHub Actions machine' }
New-Item -ItemType Directory -Force $Output | Out-Null
Start-Transcript -Path (Join-Path $Output 'display-setup.log') -Force | Out-Null
$directory = Join-Path $env:RUNNER_TEMP 'anyq-virtual-displays'
New-Item -ItemType Directory -Force $directory | Out-Null
$archive = Join-Path $directory 'driver.zip'
$url = 'https://github.com/VirtualDrivers/Virtual-Display-Driver/releases/download/25.7.23/VirtualDisplayDriver-x86.Driver.Only.zip'
Invoke-WebRequest -Uri $url -OutFile $archive
$digest = (Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($digest -ne 'e24210692b442b39af763536330ce78b423f19342b7a7792c26de3944e418b3a') { throw 'Virtual display archive digest mismatch' }
Expand-Archive $archive -DestinationPath $directory -Force
$driver = Join-Path $directory 'VirtualDisplayDriver'
$catalog = Join-Path $driver 'mttvdd.cat'
$signature = Get-AuthenticodeSignature $catalog
if (!$signature.SignerCertificate) { throw 'Virtual display catalog has no signer' }
$certificate = Join-Path $directory 'driver.cer'
Export-Certificate -Cert $signature.SignerCertificate -FilePath $certificate | Out-Null
Import-Certificate -FilePath $certificate -CertStoreLocation Cert:\LocalMachine\Root | Out-Null
Import-Certificate -FilePath $certificate -CertStoreLocation Cert:\LocalMachine\TrustedPublisher | Out-Null
if ((Get-AuthenticodeSignature $catalog).Status -ne 'Valid') { throw 'Virtual display catalog signature is invalid' }

New-Item -ItemType Directory -Force 'C:\VirtualDisplayDriver' | Out-Null
@'
<?xml version="1.0" encoding="utf-8"?>
<vdd_settings>
  <monitors><count>2</count></monitors>
  <gpu><friendlyname>default</friendlyname></gpu>
  <global><g_refresh_rate>60</g_refresh_rate></global>
  <resolutions>
    <resolution><width>1280</width><height>960</height><refresh_rate>60</refresh_rate></resolution>
    <resolution><width>1920</width><height>1080</height><refresh_rate>60</refresh_rate></resolution>
    <resolution><width>2560</width><height>1440</height><refresh_rate>60</refresh_rate></resolution>
  </resolutions>
  <options><CustomEdid>false</CustomEdid><HardwareCursor>true</HardwareCursor><logging>false</logging></options>
</vdd_settings>
'@ | Set-Content -Path 'C:\VirtualDisplayDriver\vdd_settings.xml' -Encoding utf8

Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;

public static class AnyqDisplayDevice {
    [StructLayout(LayoutKind.Sequential)]
    public struct DeviceInfo {
        public uint Size;
        public Guid Class;
        public uint Instance;
        public IntPtr Reserved;
    }

    [DllImport("setupapi.dll", SetLastError=true)]
    static extern IntPtr SetupDiCreateDeviceInfoList(ref Guid guid, IntPtr parent);
    [DllImport("setupapi.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool SetupDiCreateDeviceInfoW(IntPtr set, string name, ref Guid guid, string description, IntPtr parent, uint flags, ref DeviceInfo info);
    [DllImport("setupapi.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool SetupDiSetDeviceRegistryPropertyW(IntPtr set, ref DeviceInfo info, uint property, byte[] value, uint size);
    [DllImport("setupapi.dll", SetLastError=true)]
    static extern bool SetupDiCallClassInstaller(uint function, IntPtr set, ref DeviceInfo info);
    [DllImport("setupapi.dll", SetLastError=true)]
    static extern bool SetupDiDestroyDeviceInfoList(IntPtr set);
    [DllImport("newdev.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool UpdateDriverForPlugAndPlayDevicesW(IntPtr parent, string hardware, string inf, uint flags, out bool reboot);

    static void Check(bool value) {
        if (!value) throw new Win32Exception(Marshal.GetLastWin32Error());
    }

    public static void Install(string inf) {
        var guid = new Guid("4d36e968-e325-11ce-bfc1-08002be10318");
        var set = SetupDiCreateDeviceInfoList(ref guid, IntPtr.Zero);
        if (set == new IntPtr(-1)) throw new Win32Exception(Marshal.GetLastWin32Error());
        try {
            var info = new DeviceInfo {Size=(uint)Marshal.SizeOf(typeof(DeviceInfo))};
            Check(SetupDiCreateDeviceInfoW(set, "Display", ref guid, "anyq CI virtual monitors", IntPtr.Zero, 1, ref info));
            var hardware = Encoding.Unicode.GetBytes("Root\\MttVDD\0\0");
            Check(SetupDiSetDeviceRegistryPropertyW(set, ref info, 1, hardware, (uint)hardware.Length));
            Check(SetupDiCallClassInstaller(0x19, set, ref info));
            bool reboot;
            Check(UpdateDriverForPlugAndPlayDevicesW(IntPtr.Zero, "Root\\MttVDD", inf, 1, out reboot));
            if (reboot) throw new InvalidOperationException("Virtual display setup unexpectedly requires reboot");
        } finally {
            SetupDiDestroyDeviceInfoList(set);
        }
    }
}
'@
[AnyqDisplayDevice]::Install((Join-Path $driver 'MttVDD.inf'))
Start-Process -FilePath "$env:SystemRoot\System32\DisplaySwitch.exe" -ArgumentList '/extend' -Wait
$deadline = [DateTime]::UtcNow.AddSeconds(30)
do {
    $devices = @(Get-PnpDevice -PresentOnly | Where-Object {$_.FriendlyName -eq 'Virtual Display Driver'})
    if ($devices.Count -gt 0 -and @($devices | Where-Object {$_.Status -ne 'OK'}).Count -eq 0) { break }
    if ([DateTime]::UtcNow -ge $deadline) { throw "Virtual display driver not ready: $($devices | ConvertTo-Json -Compress)" }
    Start-Sleep -Milliseconds 100
} while ($true)
@{url=$url;sha256=$digest;signer=$signature.SignerCertificate.Thumbprint;devices=$devices} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Output 'display-driver.json')
Stop-Transcript | Out-Null
