$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$sources = @(
    @{ crate = 'gl_generator-0.14.0'; base = 'https://raw.githubusercontent.com/brendanzab/gl-rs/ea503e8d5fb6d73c6030e6191ce738cd3bf3433e'; files = @('LICENSE') },
    @{ crate = 'gpu-descriptor-0.3.2'; base = 'https://raw.githubusercontent.com/zakarumych/gpu-descriptor/bb28b4e55b742e0ea3e7a0968e7ee6287ba0e06a'; files = @('COPYING','license/APACHE','license/MIT') },
    @{ crate = 'gpu-descriptor-types-0.2.0'; base = 'https://raw.githubusercontent.com/zakarumych/gpu-descriptor/9f2fb9546591322e7b77ee1e79385e01effd85e4'; files = @('COPYING','license/APACHE','license/MIT') },
    @{ crate = 'khronos_api-3.1.0'; base = 'https://raw.githubusercontent.com/brendanzab/gl-rs/f150967b1c44ae888e6676f93f639ebc82771bdc'; files = @('LICENSE') },
    @{ crate = 'profiling-1.0.18'; base = 'https://raw.githubusercontent.com/aclysma/profiling/8271551172eb6fa4cba47369aedd93790c623df9'; files = @('LICENSE-APACHE','LICENSE-MIT') },
    @{ crate = 'spirv-0.3.0+sdk-1.3.268.0'; base = 'https://raw.githubusercontent.com/gfx-rs/rspirv/e91a5debc10c395b36db7721dfff6326d697a1e2'; files = @('LICENSE') },
    @{ crate = 'svg_fmt-0.4.5'; base = 'https://raw.githubusercontent.com/nical/rust_debug/93f414cd572b5c2781df93dbdc068ed13443d05a'; files = @('LICENSE') },
    @{ crate = 'hexf-parse-0.2.1'; base = 'https://creativecommons.org/publicdomain/zero/1.0'; files = @('legalcode.txt') }
)
$records = foreach ($source in $sources) {
    foreach ($file in $source.files) {
        $relative = "license-overrides/$($source.crate)/$file"
        $destination = Join-Path $root $relative
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
        $uri = "$($source.base)/$file"
        Invoke-WebRequest -Uri $uri -OutFile $destination
        [pscustomobject]@{ path = $relative; url = $uri; sha256 = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant() }
    }
}
$records | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'license-overrides/SOURCES.json') -Encoding utf8
Write-Output "Downloaded $($records.Count) unmodified upstream license texts"
