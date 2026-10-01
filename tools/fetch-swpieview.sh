#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p artifacts/apks
curl -fL 'https://f-droid.org/repo/org.voidptr.swpieview_15.apk' -o artifacts/apks/swpieview-1.3.2.apk
# Retain the F-Droid APK byte-for-byte; its bundled support code runs in guest DEX.
python3 - <<'PY'
from pathlib import Path
import hashlib
apk = Path('artifacts/apks/swpieview-1.3.2.apk')
digest = hashlib.sha256(apk.read_bytes()).hexdigest()
if digest != '7c7a17ddf254e6f7adb53786ab3928937a4de499785475278df2fe5a034f50f3':
    raise SystemExit('SwpieView 1.3.2 upstream checksum changed; refusing to use it')
print('Verified unmodified SwpieView 1.3.2:', digest)
PY
