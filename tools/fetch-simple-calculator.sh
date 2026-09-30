#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p artifacts/apks
curl -fL 'https://raw.githubusercontent.com/swiftugandan/Simple-Android-Calculator/3ba860b281eba34f144e4e75115f0c0a06bced31/bin/verysimplecalc.apk' -o artifacts/apks/SimpleCalculator.apk
# Verify the independently published artifact. Never patch/repackage its DEX.
python3 - <<'PY'
from pathlib import Path
import hashlib
p=Path('artifacts/apks/SimpleCalculator.apk')
expected='7c1adc93607c8511a3abd379f74765747d2ae72fb70c4ff5c471f13e94b98921'
actual=hashlib.sha256(p.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit('Simple Calculator upstream artifact checksum changed; refusing to use it')
print('Verified unmodified Simple Calculator 1.0:', actual)
PY
