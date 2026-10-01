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

images = root / "fixtures/generated/images.apk"
if not images.exists():
    raise SystemExit(f"Missing {images}; rebuild with tools/build-fixtures.py")
image_process = subprocess.run([
    str(args.binary), "run", "--headless", "--ephemeral", str(images),
], text=True, capture_output=True, check=True, timeout=120)
image_tree = json.loads(image_process.stdout)
image_nodes = list(flatten(image_tree))
image_labels = [node["view"]["text"] for node in image_nodes]
if "PNG 96x64 | JPEG 96x64 | WebP 96x64 | XML pull OK" not in image_labels:
    raise SystemExit("BitmapFactory did not decode all three fixture image formats")
if sum(node["view"]["kind"] == "ImageView" for node in image_nodes) != 4:
    raise SystemExit("Image fixture did not produce all four ImageViews")
image_status = next(node["view"] for node in image_nodes if "XML pull OK" in node["view"]["text"])
if image_status["text_size"] != 20 or image_status["text_color"] != 0xff202a36:
    raise SystemExit("Text appearance did not apply inherited size and explicit color")
(root / "artifacts/images-compatibility.json").write_text(json.dumps({
    "formats": ["PNG", "JPEG", "WebP"],
    "decode_paths": ["resource", "stream", "byte-array"],
    "xml_resource_parser": "start tags, depth, attributes, and close",
    "typed_xml_attributes": "Resources, Theme and Context: string, dimension and float values",
    "text_appearance": "inherited 20sp size and explicit #202a36 color",
    "image_views": 4,
    "native_visual_check": "artifacts/images-native.png",
}, indent=2) + "\n")
print("PASS Images fixture: typed XML attributes, PNG/JPEG/WebP decoding; four ImageViews")

grids = root / "fixtures/generated/grids.apk"
for clicks, count, label in [
    (["Photo 3"], 7, "Selected photo 3 · id 3000000002"),
    (["Refresh photos"], 4, "4 photos · refreshed from adapter"),
]:
    cmd = [str(args.binary), "run", "--headless", "--ephemeral", str(grids)]
    for click in clicks:
        cmd.extend(["--click", click])
    process = subprocess.run(cmd, text=True, capture_output=True, check=True, timeout=120)
    nodes = list(flatten(json.loads(process.stdout)))
    if sum(node["view"]["kind"] == "ImageView" for node in nodes) != count:
        raise SystemExit("Grid fixture did not bind the expected image cells")
    if label not in [node["view"]["text"] for node in nodes]:
        raise SystemExit("Grid fixture callback or refresh did not run in guest DEX")
print("PASS Grid fixture: seven image cells, accessible item click, long row ID and four-cell refresh")

results_apk = root / "fixtures/generated/results.apk"
for options, label in [
    (["--click", "Open child", "--click", "Return result"], "Result 7:-1:at finish:image/png"),
    (["--click", "Open child", "--back"], "Result 7:0:none"),
]:
    process = subprocess.run([
        str(args.binary), "run", "--headless", "--ephemeral", str(results_apk), *options,
    ], text=True, capture_output=True, check=True, timeout=120)
    labels = [node["view"]["text"] for node in flatten(json.loads(process.stdout))]
    if label not in labels:
        raise SystemExit("Activity result callback did not receive the expected guest payload")
print("PASS Results fixture: child return snapshot and Back cancellation")

swpie = root / "artifacts/apks/swpieview-1.3.2.apk"
if not swpie.exists():
    raise SystemExit(f"Missing {swpie}; run tools/fetch-swpieview.sh first")
if hashlib.sha256(swpie.read_bytes()).hexdigest() != "7c7a17ddf254e6f7adb53786ab3928937a4de499785475278df2fe5a034f50f3":
    raise SystemExit("SwpieView 1.3.2 checksum mismatch")
document_replay = root / "target/release/examples/document-replay"
for app, expected_label in [
    (root / "fixtures/generated/documents.apk", "3 images · document streams"),
    (swpie, None),
]:
    process = subprocess.run([
        str(document_replay), str(app), str(root / "examples/images/assets"),
    ], text=True, capture_output=True, check=True, timeout=120)
    nodes = list(flatten(json.loads(process.stdout)))
    images = [node for node in nodes if node["view"]["kind"] == "ImageView"]
    if len(images) != 3 or "Decoded image views: 3" not in process.stderr:
        raise SystemExit("Document replay did not decode all three selected-folder images")
    if expected_label and expected_label not in [node["view"]["text"] for node in nodes]:
        raise SystemExit("Document fixture did not complete its guest query/decode callback")
    if not expected_label and any(image["view"]["image_scale"] != 6 for image in images):
        raise SystemExit("SwpieView did not request CENTER_CROP for its thumbnails")
print("PASS Documents and public SwpieView: selected-folder query, guest sort, three decoded thumbnails and clean close")
for back in [False, True]:
    process = subprocess.run([
        str(document_replay), str(swpie), str(root / "examples/images/assets"),
        "--click-first-image", *(["--back"] if back else []),
    ], text=True, capture_output=True, check=True, timeout=120)
    nodes = list(flatten(json.loads(process.stdout)))
    images = [node for node in nodes if node["view"]["kind"] == "ImageView"]
    expected = 3 if back else 1
    if len(images) != expected or f"Decoded image views: {expected}" not in process.stderr:
        raise SystemExit("SwpieView full-screen image/Back replay failed")
    if not back and (images[0]["rect"]["width"] < 400 or images[0]["rect"]["height"] < 600):
        raise SystemExit("SwpieView image did not fill its viewer")
print("PASS Public SwpieView: Parcelable image stack → full-screen decoded image → Back to thumbnails")

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
# activity_note.xml explicitly gives its Toolbar a space as app:title.
toolbar = next(node for node in nodes if node["view"]["id"] == 0x7f0c0072)
toolbar_labels = [node["view"]["text"] for node in flatten(toolbar)]
if " " not in toolbar_labels or "Created moments ago" not in labels:
    raise SystemExit("Notepad editor screen labels were not rendered")
report = {
    "screens": ["notes-list", "note-editor"],
    "viewport": [390, 844],
    "editable_fields": len(editable),
    "first_field_text": editable[0]["view"]["text"],
    "visible_labels": [label for label in labels if label],
}
probe_titles = ["Hello, desktop", "Native desktop test"]
with tempfile.TemporaryDirectory(prefix="droidless-notepad-") as app_data:
    saved_process = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844",
        "--data-dir", app_data,
        "--click", "＋", "--input", probe_titles[0], "--back",
        "--click", "＋", "--input", probe_titles[1], "--back", str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    saved_tree = json.loads(saved_process.stdout)
    saved_labels = [node["view"]["text"] for node in flatten(saved_tree)]
    if not all(title in saved_labels for title in probe_titles):
        raise SystemExit("Notepad did not render both saved titles after returning to Notes")
    database = Path(app_data) / "ir.cafebazaar.notepad/databases/AppDatabase.db"
    with sqlite3.connect(database) as connection:
        saved = connection.execute(
            "SELECT title, body FROM Note WHERE title IN (?, ?) ORDER BY title",
            probe_titles,
        ).fetchall()
    if saved != sorted((title, "") for title in probe_titles):
        raise SystemExit("Notepad did not persist both edited titles before restart")

    restarted = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844",
        "--data-dir", app_data, str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    restarted_tree = json.loads(restarted.stdout)
    restarted_labels = [node["view"]["text"] for node in flatten(restarted_tree)]
    if "Notes" not in restarted_labels:
        raise SystemExit("Notepad did not return to its Notes screen after restart")
    if not all(title in restarted_labels for title in probe_titles):
        raise SystemExit("Notepad did not render both saved titles in the reopened Notes list")
    with sqlite3.connect(database) as connection:
        retained = connection.execute(
            "SELECT title, body FROM Note WHERE title IN (?, ?) ORDER BY title",
            probe_titles,
        ).fetchall()
    if retained != sorted((title, "") for title in probe_titles):
        raise SystemExit("Notepad did not retain both note rows after a fresh process")

report["note_row_survives_fresh_process"] = True
report["note_title_visible_after_save"] = True
report["note_title_visible_in_reopened_list"] = True
report["saved_notes"] = probe_titles
(root / "artifacts/notepad-compatibility.json").write_text(json.dumps(report, indent=2) + "\n")
print("PASS Notepad: Notes screen → note editor → typed title visible")
print("PASS Notepad: two saved titles appear immediately and survive restart")
