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
binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest()
replay = json.loads((root / "artifacts/public-replay-runtime.json").read_text())
if not replay["verified"] or replay["cli_sha256"] != binary_sha:
    raise SystemExit("Run tools/compatibility.py on this exact binary before packaging")
provenance = {
    "version": version, "platform": "macos-arm64", "cli_sha256": binary_sha,
    "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
    "full_public_replay_verified": True,
}
name = f"droidless-{version}-macos-arm64"
output = root / "artifacts/releases" / f"{name}.tar.gz"
output.parent.mkdir(parents=True, exist_ok=True)
readme = f"""DROIDLESS {version} — experimental macOS ARM64 preview

From this directory:
  ./droidless run fixtures/counter.apk --ephemeral
  ./droidless run --headless --ephemeral fixtures/counter.apk --click Increment
  ./droidless run --headless --ephemeral fixtures/intents.apk --click 'Open detail' --back

Fetch the original public notes app (curl and Python 3 needed):
  sh tools/fetch-notepad.sh
  ./droidless run --headless --size 390x844 --data-dir ./test-apps \\
    --click '＋' --input 'Hello, desktop' --back artifacts/apks/notepad-v1.0.0.apk
  ./droidless run --headless --size 390x844 --data-dir ./test-apps artifacts/apks/notepad-v1.0.0.apk

Neutral calculator: sh tools/fetch-simple-calculator.sh
Image viewer: sh tools/fetch-swpieview.sh
Public APKs are fetched unchanged with SHA-256 verification, never bundled here.
Included authored fixtures: counter, intents, preferences, scheduling and images.
The original Notepad backup/restore recovers exact database bytes on an isolated
copy; its post-restore System.exit call remains outside the API profile.
FileOutputStream and channel transfers stay inside the package data root and use
bounded staged writes.
Broad Android API compatibility remains unfinished.

This is a limited Java/Android compatibility subset, not an audited sandbox.
Use trusted APKs. This binary has no Apple developer signature or notarization.
No Android runtime, emulator or SDK is needed to execute the included fixture.
CLI SHA-256: {binary_sha}
RELEASE.json records source provenance and the matching full public replay.

Scope, evidence and source: https://othmaneblial.github.io/droidless/
Docs: https://othmaneblial.github.io/droidless/docs.html
Release: https://github.com/OthmaneBlial/droidless/releases/tag/v{version}
""".encode()
with tarfile.open(output, "w:gz") as archive:
    for source, target in [
        (binary, "droidless"),
        (root / "LICENSE", "LICENSE"),
    ] + [
        (root / f"fixtures/generated/{fixture}.apk", f"fixtures/{fixture}.apk")
        for fixture in ["counter", "intents", "preferences", "scheduling", "images"]
    ] + [
        (root / f"tools/{helper}", f"tools/{helper}")
        for helper in ["fetch-notepad.sh", "fetch-simple-calculator.sh", "fetch-swpieview.sh"]
    ]:
        archive.add(source, arcname=f"{name}/{target}")
    for filename, contents in [("README.txt", readme), ("RELEASE.json", (json.dumps(provenance, indent=2) + "\n").encode())]:
        entry = tarfile.TarInfo(f"{name}/{filename}")
        entry.size = len(contents)
        entry.mode = 0o644
        archive.addfile(entry, io.BytesIO(contents))
with tempfile.TemporaryDirectory(prefix="droidless-consumer-") as consumer:
    with tarfile.open(output) as archive:
        archive.extractall(consumer, filter="data")
    installed = Path(consumer) / name
    def texts(node):
        return [node["view"]["text"]] + [text for child in node["children"] for text in texts(child)]
    def run(fixture, actions, data=None):
        isolation = ["--data-dir", str(data)] if data else ["--ephemeral"]
        process = subprocess.run([
            installed / "droidless", "run", "--headless", "--size", "390x844", *isolation,
            *actions, fixture,
        ], text=True, capture_output=True, timeout=120)
        if process.returncode:
            raise SystemExit("Packaged consumer failed:\n" + process.stderr)
        return texts(json.loads(process.stdout))
    if hashlib.sha256((installed / "droidless").read_bytes()).hexdigest() != binary_sha:
        raise SystemExit("Extracted release executable does not match public replay")
    if "1" not in run(installed / "fixtures/counter.apk", ["--click", "Increment"]):
        raise SystemExit("Packaged consumer DEX callback did not increment the counter")
    if "Original extras" not in run(installed / "fixtures/intents.apk", ["--click", "Open detail"]):
        raise SystemExit("Packaged consumer did not open the detail screen")
    if "Home resume 2" not in run(installed / "fixtures/intents.apk", ["--click", "Open detail", "--back"]):
        raise SystemExit("Packaged consumer did not return through Back")
    preferences = installed / "fixtures/preferences.apk"
    data = installed / "test-apps"
    saved = "Release check · café"
    if saved not in run(preferences, ["--click", "Edit note", "--input", saved, "--click", "Save note"], data):
        raise SystemExit("Packaged consumer did not save its UTF-8 preference")
    if saved not in run(preferences, [], data):
        raise SystemExit("Packaged consumer did not restore its preference after restart")
    # Test the unmodified public APK outside the archive; never redistribute it.
    notepad = root / "artifacts/apks/notepad-v1.0.0.apk"
    if hashlib.sha256(notepad.read_bytes()).hexdigest() != "2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f":
        raise SystemExit("Public consumer APK checksum mismatch")
    copied_apk = installed / "consumer-notepad.apk"
    copied_apk.write_bytes(notepad.read_bytes())
    if saved not in run(copied_apk, ["--click", "＋", "--input", saved, "--back"], data):
        raise SystemExit("Packaged public Notepad did not save and return to Notes")
    if saved not in run(copied_apk, [], data):
        raise SystemExit("Packaged public Notepad did not restore its note after restart")
print("Clean extracted consumer: Counter, navigation/Back, UTF-8 preferences and original Notepad save/restart passed")
digest = hashlib.sha256(output.read_bytes()).hexdigest()
output.with_name("SHA256SUMS").write_text(f"{digest}  {output.name}\n")
print(output.relative_to(root))
print(f"SHA-256: {digest}")
