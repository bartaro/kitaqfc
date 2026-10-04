#!/bin/sh
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
case "$(uname -s):$(uname -m)" in
  Linux:x86_64) destination="bin/Linux-X64" ;;
  Darwin:arm64) destination="bin/macOS-ARM64" ;;
  Darwin:x86_64) destination="bin/macOS-X64" ;;
  *) echo "Unsupported host: $(uname -s) $(uname -m)" >&2; exit 1 ;;
esac
cargo test --locked --tests
cargo build --locked --release
mkdir -p "$destination"
for program in kitaqfc kitaqfc-zx0 kitaqfc-asset-pack kitaqfc-png-index-build kitaqfc-asset-pipeline kitaqfc-rights-name-guard; do
  cp "target/release/$program" "$destination/$program"
  chmod +x "$destination/$program"
done
echo "Executables: $destination"
if [ "${KEEP_INTERMEDIATES:-0}" != 1 ]; then cargo clean; fi
