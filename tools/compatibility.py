#!/usr/bin/env python3
"""Replay real APK clicks through guest callbacks; no Android tools involved."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile

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

notepad = root / "artifacts/apks/notepad-v1.0.0.apk"
notepad_digest = "2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f"
if not notepad.exists():
    raise SystemExit(f"Missing {notepad}; run tools/fetch-notepad.sh first")
if hashlib.sha256(notepad.read_bytes()).hexdigest() != notepad_digest:
    raise SystemExit("Notepad v1.0.0 checksum mismatch")
home_process = subprocess.run([
    str(args.binary), "run", "--headless", "--ephemeral", "--size", "390x844", str(notepad),
], text=True, capture_output=True, check=True, timeout=120)
home = json.loads(home_process.stdout)
home_labels = [node["view"]["text"] for node in flatten(home)]
if "Notes" not in home_labels or "You have no notes!" not in home_labels or "＋" not in home_labels:
    raise SystemExit("Notepad Notes list screen was not rendered")
print("PASS Notepad: public Notes list and empty state rendered")
process = subprocess.run([
    str(args.binary), "run", "--headless", "--ephemeral", "--size", "390x844",
    "--click", "＋", "--input", "Hello, desktop", str(notepad),
], text=True, capture_output=True, check=True, timeout=120)
tree = json.loads(process.stdout)
nodes = list(flatten(tree))
editable = [node for node in nodes if node["view"]["kind"] == "EditText"]
labels = [node["view"]["text"] for node in nodes]
if len(editable) != 2 or editable[0]["view"]["text"] != "Hello, desktop":
    raise SystemExit("Notepad did not expose both editor fields and the entered title")
if "Notepad" not in labels or "Created moments ago" not in labels:
    raise SystemExit("Notepad editor screen labels were not rendered")
report = {
    "screens": ["notes-list", "note-editor"],
    "viewport": [390, 844],
    "editable_fields": len(editable),
    "first_field_text": editable[0]["view"]["text"],
    "visible_labels": [label for label in labels if label],
}
probe_title = "Droidless persistence probe"
with tempfile.TemporaryDirectory(prefix="droidless-notepad-") as app_data:
    saved_process = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844",
        "--data-dir", app_data, "--click", "＋", "--input", probe_title,
        "--back", str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    saved_tree = json.loads(saved_process.stdout)
    saved_labels = [node["view"]["text"] for node in flatten(saved_tree)]
    if probe_title not in saved_labels:
        raise SystemExit("Notepad did not render its saved title after returning to Notes")
    database = Path(app_data) / "ir.cafebazaar.notepad/databases/AppDatabase.db"
    with sqlite3.connect(database) as connection:
        saved = connection.execute(
            "SELECT id, title, body FROM Note WHERE title = ?", (probe_title,)
        ).fetchall()
    if len(saved) != 1 or saved[0][2] != "":
        raise SystemExit("Notepad did not persist the edited title before restart")

    restarted = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844",
        "--data-dir", app_data, str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    restarted_tree = json.loads(restarted.stdout)
    restarted_labels = [node["view"]["text"] for node in flatten(restarted_tree)]
    if "Notes" not in restarted_labels:
        raise SystemExit("Notepad did not return to its Notes screen after restart")
    if probe_title not in restarted_labels:
        raise SystemExit("Notepad did not render its saved title in the reopened Notes list")
    with sqlite3.connect(database) as connection:
        retained = connection.execute(
            "SELECT title, body FROM Note WHERE id = ?", (saved[0][0],)
        ).fetchone()
    if retained != (probe_title, ""):
        raise SystemExit("Notepad did not retain the note row after a fresh process")

report["note_row_survives_fresh_process"] = True
report["note_title_visible_after_save"] = True
report["note_title_visible_in_reopened_list"] = True
(root / "artifacts/notepad-compatibility.json").write_text(json.dumps(report, indent=2) + "\n")
print("PASS Notepad: Notes screen → note editor → typed title visible")
print("PASS Notepad: saved title appears immediately, survives restart and returns to Notes")
