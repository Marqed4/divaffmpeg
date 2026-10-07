# Builds dist\DivaFFMPEG-0.1.0.msix (+ a self-signed .cer for local installs).
# Prereqs: cargo build --release, ffmpeg\bin\{ffmpeg,ffprobe}.exe, Windows SDK.
$ErrorActionPreference = 'Stop'
$root  = Resolve-Path "$PSScriptRoot\..\.."
$stage = Join-Path $root 'dist\msix-stage'
$out   = Join-Path $root 'dist\DivaFFMPEG-0.1.0.msix'
$sdk   = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64" -Directory |
         Sort-Object { [version]$_.Parent.Name } -Descending | Select-Object -First 1
$makeappx = Join-Path $sdk.FullName 'makeappx.exe'
$signtool = Join-Path $sdk.FullName 'signtool.exe'

Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory "$stage\Assets", "$stage\ffmpeg\bin" | Out-Null

# Files
Copy-Item "$root\target\release\DivaFFMPEG.exe" $stage
Copy-Item "$root\ffmpeg\bin\ffmpeg.exe", "$root\ffmpeg\bin\ffprobe.exe" "$stage\ffmpeg\bin"
Copy-Item "$root\ffmpeg\LICENSE" "$stage\ffmpeg"
Copy-Item "$root\LICENSE", "$root\THIRD_PARTY_NOTICES.md", "$root\README.md" $stage
Copy-Item "$PSScriptRoot\AppxManifest.xml" $stage

# Logos from the repo icon, fitted onto a transparent square
Add-Type -AssemblyName System.Drawing
$src = [System.Drawing.Image]::FromFile("$root\examples\diva_icon.gif")
foreach ($l in @(@('Square44x44Logo',44), @('Square150x150Logo',150), @('StoreLogo',50))) {
    $s = $l[1]
    $bmp = New-Object System.Drawing.Bitmap $s, $s
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = 'HighQualityBicubic'
    $scale = [Math]::Min($s / $src.Width, $s / $src.Height)
    $w = $src.Width * $scale; $h = $src.Height * $scale
    $g.DrawImage($src, ($s - $w) / 2, ($s - $h) / 2, $w, $h)
    $bmp.Save("$stage\Assets\$($l[0]).png", [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
}
$src.Dispose()

# Pack
New-Item -ItemType Directory (Split-Path $out) -Force | Out-Null
Remove-Item $out -ErrorAction SilentlyContinue
& $makeappx pack /d $stage /p $out /o
if ($LASTEXITCODE) { throw 'makeappx failed' }

# Sign with a self-signed cert (Subject must match the manifest Publisher).
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object Subject -eq 'CN=Marqed' | Select-Object -First 1
if (-not $cert) {
    $cert = New-SelfSignedCertificate -Type Custom -Subject 'CN=Marqed' -KeyUsage DigitalSignature `
        -FriendlyName 'DivaFFMPEG dev' -CertStoreLocation Cert:\CurrentUser\My `
        -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
}
Export-Certificate -Cert $cert -FilePath "$root\dist\DivaFFMPEG.cer" | Out-Null
& $signtool sign /fd SHA256 /sha1 $cert.Thumbprint $out
if ($LASTEXITCODE) { throw 'signtool failed' }
Write-Host "Built $out"
