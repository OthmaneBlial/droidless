#!/usr/bin/env python3
"""Package only the locally built macOS ARM64 CLI and intentional runtime examples."""
import hashlib
import io
import json
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import tempfile

root = Path(__file__).resolve().parents[1]
if sys.platform != "darwin" or platform.machine() not in ("arm64", "aarch64"):
    raise SystemExit("This package target requires a native macOS ARM64 host")
subprocess.run(["cargo", "build", "--release", "--locked", "-p", "droidless"], cwd=root, check=True)
binary = root / "target/release/droidless"
version = subprocess.check_output([binary, "--version"], text=True).strip().split()[1]
name = f"droidless-{version}-macos-arm64"
output = root / "artifacts/releases" / f"{name}.tar.gz"
output.parent.mkdir(parents=True, exist_ok=True)
readme = f"""DROIDLESS {version} — experimental macOS ARM64 preview

From this directory:
  ./droidless fixtures/counter.apk
  ./droidless run --headless fixtures/counter.apk --click Increment

Fetch the original KasCalc calculator (curl and Python 3 needed):
  sh tools/fetch-kascalc.sh
  ./droidless artifacts/apks/KasCalc.apk

This is a limited Java/Android compatibility subset, not an audited sandbox.
Use trusted APKs. This binary has no Apple developer signature or notarization.
No Android runtime, emulator or SDK is needed to execute the included fixture.
KasCalc is fetched from its original GPL-3.0 release and is not bundled here.

Scope, evidence and source: https://othmaneblial.github.io/droidless/
Docs: https://othmaneblial.github.io/droidless/docs.html
Release: https://github.com/OthmaneBlial/droidless/releases/tag/v{version}
""".encode()
with tarfile.open(output, "w:gz") as archive:
    for source, target in [
        (binary, "droidless"),
        (root / "LICENSE", "LICENSE"),
        (root / "fixtures/generated/counter.apk", "fixtures/counter.apk"),
        (root / "tools/fetch-kascalc.sh", "tools/fetch-kascalc.sh"),
    ]:
        archive.add(source, arcname=f"{name}/{target}")
    entry = tarfile.TarInfo(f"{name}/README.txt")
    entry.size = len(readme)
    entry.mode = 0o644
    archive.addfile(entry, io.BytesIO(readme))
with tempfile.TemporaryDirectory(prefix="droidless-consumer-") as consumer:
    with tarfile.open(output) as archive:
        archive.extractall(consumer, filter="data")
    installed = Path(consumer) / name
    tree = json.loads(subprocess.check_output([
        installed / "droidless", "run", "--headless",
        installed / "fixtures/counter.apk", "--click", "Increment",
    ], text=True))
    def texts(node):
        return [node["view"]["text"]] + [text for child in node["children"] for text in texts(child)]
    if "1" not in texts(tree):
        raise SystemExit("Packaged consumer DEX callback did not increment the counter")
print("Clean extracted consumer: APK launch and Increment callback passed")
digest = hashlib.sha256(output.read_bytes()).hexdigest()
output.with_name("SHA256SUMS").write_text(f"{digest}  {output.name}\n")
print(output.relative_to(root))
print(f"SHA-256: {digest}")
