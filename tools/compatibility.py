#!/usr/bin/env python3
"""Replay real APK clicks through guest callbacks; no Android tools involved."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument("--binary", type=Path, default=root / "target/release/droidless")
args = p.parse_args()
apk = root / "artifacts/apks/KasCalc.apk"
expected = "6010d2f142cd8d0114ab627a44b50b5dc223b4d94ae5a235f836afbae211f505"

def flatten(node):
    yield node
    for child in node["children"]:
        yield from flatten(child)

cases = [
    (["7", "+", "5", "="], "12.0"),
    (["8", "x", "8", "="], "64.0"),
    (["8", "÷", "2", "="], "4.0"),
    (["9", "-", "5", "="], "4.0"),
    (["1", ".", "5", "+", "2", ".", "2", "5", "="], "3.75"),
    (["9", "√"], "3.0"),
    (["8", "³√"], "2.0"),
    (["1", "2", "←"], "1"),
    (["7", "+/-"], "-7"),
    (["9", "AC"], "0"),
]
apps = [
    ("KasCalc", apk, expected, "EditText", cases, []),
    ("SimpleCalculator", root / "artifacts/apks/SimpleCalculator.apk",
     "7c1adc93607c8511a3abd379f74765747d2ae72fb70c4ff5c471f13e94b98921", "TextView", [
         (["7", "+", "5", "="], "12"),
         (["8", "x", "8", "="], "64"),
         (["8", "/", "2", "="], "4"),
         (["9", "-", "5", "="], "4"),
         (["1", ".", "5", "+", "2", ".", "2", "5", "="], "3.75"),
         (["1", "2", "+", "3", "0", "="], "42"),
         (["9", "c"], ""),
     ], ["--size", "192x400"]),
]
for name, app_apk, digest, kind, scenarios, options in apps:
    if not app_apk.exists():
        raise SystemExit(f"Missing {app_apk}; run its fetch helper first")
    if hashlib.sha256(app_apk.read_bytes()).hexdigest() != digest:
        raise SystemExit(f"{name} checksum mismatch")
    results = []
    for clicks, expected_text in scenarios:
        cmd = [str(args.binary), "run", "--headless", "--ephemeral", "--stats", str(app_apk), *options]
        for click in clicks:
            cmd.extend(["--click", click])
        process = subprocess.run(cmd, text=True, capture_output=True, check=True)
        tree = json.loads(process.stdout)
        display = next(n for n in flatten(tree) if n["view"]["kind"] == kind)
        actual = display["view"]["text"]
        if actual != expected_text:
            raise SystemExit(f"{name} {clicks}: expected {expected_text!r}, got {actual!r}")
        if options:
            assert tree["rect"]["width"] == 192 and tree["rect"]["height"] == 400
        print("PASS", name, " ".join(clicks), "=>", actual)
        results.append({"clicks": clicks, "text": actual, "stats": process.stderr.strip()})
    filename = "kascalc" if name == "KasCalc" else "simple-calculator"
    (root / f"artifacts/{filename}-compatibility.json").write_text(json.dumps(results, indent=2) + "\n")
    print(f"{len(results)} {name} scenarios passed; native window is verified separately.")
