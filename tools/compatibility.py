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
if not apk.exists():
    raise SystemExit("Run sh tools/fetch-kascalc.sh first")
if hashlib.sha256(apk.read_bytes()).hexdigest() != expected:
    raise SystemExit("KasCalc checksum mismatch")

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
results = []
for clicks, expected_text in cases:
    cmd = [str(args.binary), "run", "--headless", "--ephemeral", "--stats", str(apk)]
    for click in clicks:
        cmd.extend(["--click", click])
    process = subprocess.run(cmd, text=True, capture_output=True, check=True)
    tree = json.loads(process.stdout)
    display = next(n for n in flatten(tree) if n["view"]["kind"] == "EditText")
    actual = display["view"]["text"]
    if actual != expected_text:
        raise SystemExit(f"{clicks}: expected {expected_text!r}, got {actual!r}")
    print("PASS", " ".join(clicks), "=>", actual)
    results.append({"clicks": clicks, "text": actual, "stats": process.stderr.strip()})
(root / "artifacts/kascalc-compatibility.json").write_text(json.dumps(results, indent=2) + "\n")
print(f"{len(results)} real APK scenarios passed; native window is verified separately.")
