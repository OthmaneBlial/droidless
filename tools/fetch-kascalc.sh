#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p artifacts/apks
curl -fL 'https://github.com/KasRoudra/simplecalculator/releases/download/v1.0/KasCalc.apk' -o artifacts/apks/KasCalc.apk
# Verify the independently published artifact. Never patch/repackage its DEX.
python3 - <<'PY'
from pathlib import Path
import hashlib
p=Path('artifacts/apks/KasCalc.apk')
expected='6010d2f142cd8d0114ab627a44b50b5dc223b4d94ae5a235f836afbae211f505'
actual=hashlib.sha256(p.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit('KasCalc upstream artifact checksum changed; refusing to use it')
print('Verified unmodified KasCalc v1.0:', actual)
PY
