#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p artifacts/apks
curl -fL 'https://github.com/MohMah/android-notepad/releases/download/v1.0.0/notepad-v1.0.0.apk' -o artifacts/apks/notepad-v1.0.0.apk
# Keep the independent release byte-for-byte; never patch or repackage its DEX.
python3 - <<'PY'
from pathlib import Path
import hashlib
p=Path('artifacts/apks/notepad-v1.0.0.apk')
expected='2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f'
actual=hashlib.sha256(p.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit('Notepad v1.0.0 upstream artifact checksum changed; refusing to use it')
print('Verified unmodified Notepad v1.0.0:', actual)
PY
