#!/usr/bin/env python3
"""Replay real APK clicks through guest callbacks; no Android tools involved."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sqlite3
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument("--binary", type=Path, default=root / "target/release/droidless")
args = p.parse_args()
runtime_sha = hashlib.sha256(args.binary.read_bytes()).hexdigest()
runtime_report = root / "artifacts/public-replay-runtime.json"
runtime_report.parent.mkdir(parents=True, exist_ok=True)
runtime_evidence = {"binary": str(args.binary), "cli_sha256": runtime_sha, "verified": False}
runtime_report.write_text(json.dumps(runtime_evidence, indent=2) + "\n")
apk = root / "artifacts/apks/KasCalc.apk"
expected = "6010d2f142cd8d0114ab627a44b50b5dc223b4d94ae5a235f836afbae211f505"

def flatten(node):
    yield node
    for child in node["children"]:
        yield from flatten(child)

def visible_nodes(node, alpha=1):
    if node["view"]["visible"] != 0:
        return
    alpha *= max(0, min(1, node["view"]["alpha"]))
    yield node, alpha
    for child in node["children"]:
        yield from visible_nodes(child, alpha)

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
document_replay = args.binary.parent / "examples/document-replay"
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
process = subprocess.run([
    str(document_replay), str(swpie), str(root / "examples/images/assets"),
    "--click-first-image", "--gestures",
], text=True, capture_output=True, check=True, timeout=120)
assert "confirmed taps hide and restore controls" in process.stderr
assert "Decoded image views: 1" in process.stderr
print("PASS Public SwpieView: bounded next/previous swipes and confirmed taps hide/restore controls")
for option, expected in [
    ("--slideshow", "UP cancels it; no task or image change"),
    ("--slideshow-hold", "UI access rejected with clean frames"),
]:
    process = subprocess.run([
        str(document_replay), str(swpie), str(root / "examples/images/assets"),
        "--click-first-image", option,
    ], text=True, capture_output=True, check=True, timeout=120)
    assert expected in process.stderr
    assert "Decoded image views: 1" in process.stderr
print("PASS Public SwpieView slideshow diagnosis: tap starts/cancels; held task rejects worker UI access")

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
# Frozen headless time makes creation/reference Dates equal. PrettyTime's
# documented zero-duration format is "moments from now" (ocpsoft.org/prettytime/).
if " " not in toolbar_labels or "Created moments from now" not in labels:
    raise SystemExit("Notepad editor screen labels were not rendered")
report = {
    "screens": ["notes-list", "note-editor"],
    "viewport": [390, 844],
    "editable_fields": len(editable),
    "first_field_text": editable[0]["view"]["text"],
    "visible_labels": [label for label in labels if label],
}
probe_titles = ["Hello, desktop", "Native desktop test"]
survivor_body = "Retained body from the original APK."
expected_saved = sorted([(probe_titles[0], ""), (probe_titles[1], survivor_body)])
with tempfile.TemporaryDirectory(prefix="droidless-notepad-") as app_data:
    saved_process = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844",
        "--data-dir", app_data,
        "--click", "＋", "--input", probe_titles[0], "--back",
        "--click", "＋", "--input", probe_titles[1], "--input-at", "1", survivor_body,
        "--back", str(notepad),
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
    if saved != expected_saved:
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
    if retained != expected_saved:
        raise SystemExit("Notepad did not retain both note rows after a fresh process")

    with sqlite3.connect(database) as connection:
        original_id = connection.execute("SELECT id FROM Note WHERE title = ?", [probe_titles[0]]).fetchone()[0]
    opened = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", app_data,
        "--click", probe_titles[0], str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    fields = [node["view"]["text"] for node in flatten(json.loads(opened.stdout)) if node["view"]["kind"] == "EditText"]
    if fields != [probe_titles[0], ""]:
        raise SystemExit("Notepad did not reopen the existing note through its row callback")
    revised_title = "Hello, revised desktop"
    body = "Saved body from the original APK.\nSecond line survives restart."
    edited = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", app_data,
        "--click", probe_titles[0], "--input", revised_title, "--input-at", "1", body,
        "--back", str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    labels = [node["view"]["text"] for node in flatten(json.loads(edited.stdout))]
    if revised_title not in labels or probe_titles[1] not in labels or probe_titles[0] in labels:
        raise SystemExit("Notepad did not refresh its list after editing the existing note")
    with sqlite3.connect(database) as connection:
        rows = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
    if len(rows) != 2 or (original_id, revised_title, body) not in rows:
        raise SystemExit("Notepad edit did not update the same row with the title and body")
    reopened = subprocess.run([
        str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", app_data,
        "--click", revised_title, str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    fields = [node["view"]["text"] for node in flatten(json.loads(reopened.stdout)) if node["view"]["kind"] == "EditText"]
    if fields != [revised_title, body]:
        raise SystemExit("Notepad title/body did not survive restart and reopen in the editor")

    for phase, milliseconds, actions in [
        ("frame", "100", []),
        ("open", "1000", []),
        ("closed", "1000", ["--back", "--advance-ms", "1000"]),
    ]:
        with tempfile.TemporaryDirectory(prefix="droidless-notepad-drawer-") as drawer_root:
            drawer_data = Path(drawer_root) / "apps"
            shutil.copytree(app_data, drawer_data)
            process = subprocess.run([
                str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", str(drawer_data),
                "--tap", "24", "22", "--advance-ms", milliseconds, *actions, str(notepad),
            ], text=True, capture_output=True, check=True, timeout=120)
            tree = json.loads(process.stdout)
            if tree is None:
                raise SystemExit("Notepad drawer replay finished the Activity instead of retaining Notes")
            frames = [(node, alpha) for node, alpha in visible_nodes(tree)
                if node["view"]["text"] == "Create or edit folders"]
            if phase != "closed":
                if len(frames) != 1 or any(alpha != 1 or not 0 <= node["rect"]["y"] < 844
                    or node["rect"]["x"] + node["rect"]["width"] <= 0
                    or node["rect"]["x"] >= 390 for node, alpha in frames):
                    raise SystemExit(f"Notepad original drawer {phase} did not reveal its folders menu entry")
            elif frames or not {"Notes", revised_title, probe_titles[1]} <= {
                node["view"]["text"] for node, _ in visible_nodes(tree)
            }:
                raise SystemExit("Notepad Back did not close its drawer and retain the Notes screen")
            with sqlite3.connect(drawer_data / "ir.cafebazaar.notepad/databases/AppDatabase.db") as connection:
                retained = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
            if retained != rows:
                raise SystemExit(f"Notepad drawer {phase} changed an existing note row")

    for phase, actions in [("open", []), ("back", ["--back"])]:
        with tempfile.TemporaryDirectory(prefix="droidless-notepad-folders-") as folder_root:
            folder_data = Path(folder_root) / "apps"
            shutil.copytree(app_data, folder_data)
            process = subprocess.run([
                str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", str(folder_data),
                "--tap", "24", "22", "--advance-ms", "1000", "--click", "Create or edit folders",
                *actions, str(notepad),
            ], text=True, capture_output=True, check=True, timeout=120)
            nodes = list(flatten(json.loads(process.stdout)))
            labels = [node["view"]["text"] for node in nodes]
            if phase == "open":
                editors = [node["view"] for node in nodes if node["view"]["kind"] == "EditText"]
                left = [node["view"] for node in nodes if node["view"]["id"] == 2131493021]
                if "Edit Folders" not in labels or len(editors) != 1 or editors[0]["id"] != 2131493023:
                    raise SystemExit("Notepad folder Activity did not bind its original editor")
                if len(left) != 1 or left[0]["listener"] is None:
                    raise SystemExit("Notepad original folder button was not bound to its guest listener")
            elif "Notes" not in labels or not {revised_title, probe_titles[1]} <= set(labels):
                raise SystemExit("Notepad folder Back did not return to both retained notes")
            with sqlite3.connect(folder_data / "ir.cafebazaar.notepad/databases/AppDatabase.db") as connection:
                retained = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
            if retained != rows:
                raise SystemExit(f"Notepad folder {phase} changed an existing note row")

    # Create and reopen a real folder without touching the saved seed.
    with tempfile.TemporaryDirectory(prefix="droidless-notepad-folder-create-") as folder_root:
        folder_data = Path(folder_root) / "apps"
        shutil.copytree(app_data, folder_data)
        open_folders = ["--tap", "24", "22", "--advance-ms", "1000", "--click", "Create or edit folders"]
        for phase, actions in [
            ("create", open_folders + ["--tap", "180", "72", "--input", "Runtime folder", "--tap", "362", "72"]),
            ("restart-open", open_folders),
            ("restart-back", open_folders + ["--back"]),
            ("focus-input", open_folders + ["--tap", "150", "136", "--input-at", "1", "Pending folder name"]),
            ("restart-discard", open_folders),
            ("host-focus-input", open_folders + ["--focus-at", "1", "--input-at", "1", "Host pending folder name"]),
            ("restart-host-discard", open_folders),
            ("rename-confirm", open_folders + ["--focus-at", "1", "--input-at", "1", "Confirmed folder name", "--tap", "362", "136"]),
            ("restart-renamed", open_folders),
            ("renamed-back", open_folders + ["--back"]),
        ]:
            process = subprocess.run([
                str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", str(folder_data),
                *actions, str(notepad),
            ], text=True, capture_output=True, check=True, timeout=120)
            tree = json.loads(process.stdout)
            nodes = list(flatten(tree))
            labels = {node["view"]["text"] for node in nodes}
            renamed_phase = phase in ("rename-confirm", "restart-renamed", "renamed-back")
            saved_name = "Confirmed folder name" if renamed_phase else "Runtime folder"
            if phase in ("restart-back", "renamed-back"):
                if not {"Notes", revised_title, probe_titles[1]} <= labels:
                    raise SystemExit("Notepad saved-folder Back did not restore both note titles")
            else:
                visible_name = {"focus-input": "Pending folder name", "host-focus-input": "Host pending folder name"}.get(phase, saved_name)
                visible = [node for node, alpha in visible_nodes(tree) if node["view"]["text"] == visible_name
                           and alpha > 0 and node["rect"]["width"] > 0 and node["rect"]["height"] > 0
                           and node["rect"]["y"] >= 0 and node["rect"]["y"] + node["rect"]["height"] <= 844]
                if "Edit Folders" not in labels or len(visible) != 1:
                    raise SystemExit(f"Notepad folder {phase} did not display exactly one saved row")
            with sqlite3.connect(folder_data / "ir.cafebazaar.notepad/databases/AppDatabase.db") as connection:
                retained = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
                folders = connection.execute("SELECT id,name FROM Folder ORDER BY id").fetchall()
            with sqlite3.connect(database) as connection:
                original = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
            if retained != rows or original != rows or folders != [(1, saved_name)]:
                raise SystemExit(f"Notepad folder {phase} changed the saved folder or seed/copy notes")

        # Verify the APK's real backup and restore callbacks on a disposable copy.
        with tempfile.TemporaryDirectory(prefix="droidless-notepad-backup-restore-") as backup_root:
            backup_data = Path(backup_root) / "apps"
            shutil.copytree(folder_data, backup_data)
            copied_app = backup_data / "ir.cafebazaar.notepad"
            copied_db = copied_app / "databases/AppDatabase.db"
            backup_file = copied_app / "external/notepad_backup.nbu"
            with sqlite3.connect(copied_db) as connection:
                before = {
                    "notes": connection.execute("SELECT id,title,body FROM Note ORDER BY id").fetchall(),
                    "folders": connection.execute("SELECT id,name FROM Folder ORDER BY id").fetchall(),
                }
            process = subprocess.run([
                str(args.binary), "run", "--headless", "--trace-methods", "--trace-framework",
                "--size", "390x844", "--data-dir", str(backup_data),
                "--tap", "24", "22", "--advance-ms", "1000", "--click", "Backup data", str(notepad),
            ], text=True, capture_output=True, timeout=120)
            if process.returncode != 0 or not backup_file.is_file():
                raise SystemExit("Notepad Backup data did not create its real .nbu file")
            if "framework: Ljava/io/FileInputStream;->getChannel()Ljava/nio/channels/FileChannel;" not in process.stderr:
                raise SystemExit("Notepad backup did not read the private database through its real file channel")
            if backup_file.read_bytes() != copied_db.read_bytes():
                raise SystemExit("Notepad backup bytes differ from the exact saved database")
            with sqlite3.connect(copied_db) as connection:
                if connection.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
                    raise SystemExit("Notepad database was corrupt after backup")
                backed_up = {
                    "notes": connection.execute("SELECT id,title,body FROM Note ORDER BY id").fetchall(),
                    "folders": connection.execute("SELECT id,name FROM Folder ORDER BY id").fetchall(),
                }
            if backed_up != before:
                raise SystemExit("Notepad backup changed saved notes or folders")

            with sqlite3.connect(copied_db) as connection:
                connection.execute(
                    "UPDATE Note SET title='TAMPERED',body='discard this copy' "
                    "WHERE id=(SELECT MIN(id) FROM Note)"
                )
            process = subprocess.run([
                str(args.binary), "run", "--headless", "--trace-methods", "--trace-framework",
                "--size", "390x844", "--data-dir", str(backup_data),
                "--tap", "24", "22", "--advance-ms", "1000", "--click", "Restore data",
                "--click", "notepad_backup.nbu", "--advance-ms", "1000", "--click", "Restore",
                "--advance-ms", "1000", str(notepad),
            ], text=True, capture_output=True, timeout=120)
            exit_boundary = "unsupported method Ljava/lang/System;->exit(I)V"
            if process.returncode != 1 or exit_boundary not in process.stderr:
                raise SystemExit("Notepad restore did not reach its known post-restore System.exit boundary")
            if copied_db.read_bytes() != backup_file.read_bytes():
                raise SystemExit("Notepad Restore did not recover the exact backup database bytes")
            with sqlite3.connect(copied_db) as connection:
                if connection.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
                    raise SystemExit("Notepad restore produced a corrupt database")
                restored = {
                    "notes": connection.execute("SELECT id,title,body FROM Note ORDER BY id").fetchall(),
                    "folders": connection.execute("SELECT id,name FROM Folder ORDER BY id").fetchall(),
                }
            if restored != before:
                raise SystemExit("Notepad Restore did not recover every exact note and folder row")

            process = subprocess.run([
                str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", str(backup_data),
                str(notepad),
            ], text=True, capture_output=True, check=True, timeout=120)
            labels = {node["view"]["text"] for node in flatten(json.loads(process.stdout))
                      if node["view"]["text"]}
            if not {row[1] for row in before["notes"]} <= labels or "Confirmed folder name" not in labels:
                raise SystemExit("Notepad fresh launch did not render restored notes and folder data")

            with sqlite3.connect(folder_data / "ir.cafebazaar.notepad/databases/AppDatabase.db") as connection:
                untouched = {
                    "notes": connection.execute("SELECT id,title,body FROM Note ORDER BY id").fetchall(),
                    "folders": connection.execute("SELECT id,name FROM Folder ORDER BY id").fetchall(),
                }
            if untouched != before:
                raise SystemExit("Notepad backup/restore modified the test seed copy")
        backup_restore_boundary = exit_boundary

        # Replay the original modal buttons and persist the result, using only this copy.
        show_delete = open_folders + ["--focus-at", "1", "--tap", "24", "128"]
        for phase, actions, deleted in [
            ("shown", show_delete, False),
            ("cancel", show_delete + ["--click", "Cancel", "--advance-ms", "250"], False),
            ("cancel-restart", open_folders, False),
            ("cancel-back", open_folders + ["--back"], False),
            ("confirmed", show_delete + ["--click", "Delete Folder", "--advance-ms", "250"], True),
            ("delete-restart", open_folders, True),
            ("delete-back", open_folders + ["--back"], True),
        ]:
            tracing = ["--trace-methods", "--trace-framework"] if phase in ("shown", "confirmed") else []
            process = subprocess.run([
                str(args.binary), "run", "--headless", *tracing, "--size", "390x844", "--data-dir", str(folder_data),
                *actions, str(notepad),
            ], text=True, capture_output=True, check=True, timeout=120)
            tree = json.loads(process.stdout)
            visible = [node for node, alpha in visible_nodes(tree)
                       if alpha > 0 and node["rect"]["width"] > 0 and node["rect"]["height"] > 0]
            labels = [node["view"]["text"] for node in visible]
            if phase == "shown":
                title = "Delete folder?"
                message = "Folder 'Confirmed folder name' will be deleted however notes in this folder will remain safe"
                if not {title, message, "Cancel", "Delete Folder"} <= set(labels):
                    raise SystemExit("Notepad did not display its original complete folder-delete dialog")
                for label in ["Cancel", "Delete Folder"]:
                    buttons = [node for node in visible if node["view"]["text"] == label
                               and node["view"]["kind"] == "Button" and node["view"]["listener"] is not None]
                    if len(buttons) != 1:
                        raise SystemExit("Notepad dialog did not bind its original " + label + " button")
                if "EditFolderViewHolder;->clickLeftButton" not in process.stderr:
                    raise SystemExit("Notepad did not execute its original saved-folder delete listener")
                for stage in ["Landroid/view/ViewGroup;->setClipToPadding(Z)V",
                              "Landroid/app/Dialog;->onStart()V", "Landroid/app/Dialog;->onAttachedToWindow()V",
                              "Landroid/text/Layout;->getEllipsisCount(I)I"]:
                    if "framework: " + stage not in process.stderr:
                        raise SystemExit("Notepad dialog did not complete " + stage)
                queries = [line for line in process.stderr.splitlines()
                           if line.startswith("framework: Landroid/view/View;->canScrollVertically(I)Z ")]
                if not all(any(line.endswith(", Bits(" + direction + ")]") for line in queries)
                           for direction in ["4294967295", "1"]):
                    raise SystemExit("Notepad dialog did not query both original scroll-indicator directions")
            elif phase.endswith("back"):
                if not {"Notes", revised_title, probe_titles[1]} <= set(labels):
                    raise SystemExit("Notepad folder-delete Back did not restore both note titles")
            elif "Edit Folders" not in labels or labels.count("Confirmed folder name") != (0 if deleted else 1):
                raise SystemExit("Notepad folder-delete " + phase + " displayed an incorrect saved-folder list")
            if phase != "shown" and {"Delete folder?", "Cancel", "Delete Folder"} & set(labels):
                raise SystemExit("Notepad folder-delete modal remained after " + phase)
            if phase == "confirmed" and "framework: Landroid/app/Dialog;->dismiss()V" not in process.stderr:
                raise SystemExit("Notepad confirmation did not dismiss its original Dialog")
            with sqlite3.connect(folder_data / "ir.cafebazaar.notepad/databases/AppDatabase.db") as connection:
                retained = connection.execute("SELECT id,title,body FROM Note ORDER BY id").fetchall()
                folders = connection.execute("SELECT id,name FROM Folder ORDER BY id").fetchall()
            with sqlite3.connect(database) as connection:
                original = connection.execute("SELECT id,title,body FROM Note ORDER BY id").fetchall()
            expected_folders = [] if deleted else [(1, "Confirmed folder name")]
            if retained != rows or original != rows or folders != expected_folders:
                raise SystemExit("Notepad folder-delete " + phase + " changed exact notes, the seed or the wrong folder state")
        dialog_blocker = None

    survivor = next(row for row in rows if row[0] != original_id)
    # Original Undo calls note.save(); the APK's INSERT omits its auto-increment ID.
    restored_id = max(row[0] for row in rows) + 1
    expected_undo_rows = sorted([survivor, (restored_id, revised_title, body)])
    command = [str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", app_data]
    feedback_text = "Deleted Note " + revised_title
    for label, actions in [
        ("shown", ["--advance-ms", "250"]),
        ("dismissed", ["--advance-ms", "250", "--advance-ms", "3000", "--advance-ms", "250"]),
        ("undo", ["--advance-ms", "250", "--click", "UNDO", "--advance-ms", "250", "--advance-ms", "3000"]),
    ]:
        with tempfile.TemporaryDirectory(prefix="droidless-notepad-feedback-") as feedback_root:
            feedback_data = Path(feedback_root) / "apps"
            shutil.copytree(app_data, feedback_data)
            process = subprocess.run([
                str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", str(feedback_data),
                "--click", revised_title, "--menu-item", "Delete", *actions, str(notepad),
            ], text=True, capture_output=True, check=True, timeout=120)
            tree = json.loads(process.stdout)
            nodes = list(flatten(tree))
            feedback = [node for node in nodes if node["view"]["text"] in [feedback_text, "UNDO"]]
            if label == "shown":
                frames = [(node, alpha) for node, alpha in visible_nodes(tree) if node["view"]["text"] in [feedback_text, "UNDO"]]
                if len(frames) != 2 or any(alpha != 1 or not 0 <= node["rect"]["y"] < 844 for node, alpha in frames):
                    raise SystemExit("Notepad timed feedback did not show its original message and UNDO action")
            elif feedback:
                raise SystemExit("Notepad timed dismissal did not remove its original Snackbar")
            with sqlite3.connect(feedback_data / "ir.cafebazaar.notepad/databases/AppDatabase.db") as connection:
                remaining = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
            expected_rows = expected_undo_rows if label == "undo" else [survivor]
            if remaining != expected_rows:
                raise SystemExit(f"Notepad feedback {label} did not retain the exact surviving row")
            if label == "undo":
                for phase, replay in [("restart", []), ("reopen", ["--click", revised_title])]:
                    restarted = subprocess.run([
                        str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", str(feedback_data),
                        *replay, str(notepad),
                    ], text=True, capture_output=True, check=True, timeout=120)
                    nodes = list(flatten(json.loads(restarted.stdout)))
                    if phase == "reopen":
                        fields = [node["view"]["text"] for node in nodes if node["view"]["kind"] == "EditText"]
                        if fields != [revised_title, body]:
                            raise SystemExit("Notepad Undo did not reopen both restored fields")
                    elif not {"Notes", revised_title, probe_titles[1]} <= {node["view"]["text"] for node in nodes}:
                        raise SystemExit("Notepad Undo did not restore both list rows after restart")
                    with sqlite3.connect(feedback_data / "ir.cafebazaar.notepad/databases/AppDatabase.db") as connection:
                        remaining = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
                    if remaining != expected_undo_rows:
                        raise SystemExit(f"Notepad Undo {phase} changed a restored row or its survivor")
    for label, actions in [
        ("delete", ["--click", revised_title, "--menu-item", "Delete"]),
        ("restart", []),
        ("reopen survivor", ["--click", probe_titles[1]]),
    ]:
        process = subprocess.run(command + actions + [str(notepad)],
            text=True, capture_output=True, check=True, timeout=120)
        nodes = list(flatten(json.loads(process.stdout)))
        labels = [node["view"]["text"] for node in nodes]
        with sqlite3.connect(database) as connection:
            remaining = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
        if remaining != [survivor] or revised_title in labels:
            raise SystemExit(f"Notepad {label} did not preserve only the original survivor row")
        if label == "reopen survivor":
            fields = [node["view"]["text"] for node in nodes if node["view"]["kind"] == "EditText"]
            if fields != [probe_titles[1], survivor_body]:
                raise SystemExit("Notepad did not reopen the surviving title/body after deletion")
        elif "Notes" not in labels or probe_titles[1] not in labels:
            raise SystemExit(f"Notepad {label} did not render the surviving note in its Notes list")

report["note_row_survives_fresh_process"] = True
report["note_title_visible_after_save"] = True
report["note_title_visible_in_reopened_list"] = True
report["saved_notes"] = probe_titles
report["existing_note_title_and_body_edit_verified"] = True
report["existing_note_id_retained"] = True
report["edited_note_fields_survive_restart"] = True
report["revised_title"] = revised_title
report["reopened_body"] = body
report["headless_note_delete_workflow_verified"] = True
report["delete_survivor_id_title_body_retained"] = True
report["delete_return_restart_reopen_verified"] = True
report["timed_delete_feedback_shown"] = True
report["timed_delete_feedback_dismissed"] = True
report["delete_feedback_clock_steps_ms"] = [250, 3000, 250]
report["headless_delete_undo_verified"] = True
report["undo_restored_id"] = restored_id
report["undo_restored_title_body_restart_reopen_verified"] = True
report["undo_survivor_id_title_body_retained"] = True
report["undo_old_timeout_harmless"] = True
report["headless_drawer_navigation_tap_verified"] = True
report["headless_drawer_animation_frame_verified"] = True
report["headless_drawer_open_back_close_verified"] = True
report["drawer_existing_note_rows_retained"] = True
report["drawer_frame_steps_ms"] = [100]
report["drawer_open_close_steps_ms"] = [1000, 1000]
report["headless_folder_open_back_verified"] = True
report["headless_folder_existing_note_rows_retained"] = True
report["native_folder_input_verified"] = False
report["headless_folder_creation_verified"] = True
report["headless_saved_folder_display_restart_back_verified"] = True
report["folder_creation_blocker"] = None
report["folder_creation_existing_note_rows_retained"] = True
report["headless_folder_editing_verified"] = True
report["headless_saved_folder_focus_input_verified"] = True
report["headless_saved_folder_host_focus_input_verified"] = True
report["headless_saved_folder_unconfirmed_input_discarded_verified"] = True
report["folder_rename_confirmation_blocker"] = None
report["headless_folder_rename_verified"] = True
report["headless_folder_rename_restart_back_verified"] = True
report["folder_rename_id_and_existing_notes_retained"] = True
report["headless_folder_delete_dialog_boundary_verified"] = True
report["headless_folder_delete_dialog_constructor_verified"] = True
report["headless_folder_delete_dialog_builder_verified"] = True
report["headless_folder_delete_dialog_oncreate_entered"] = True
report["headless_folder_delete_appcompat_foreground_setup_verified"] = True
report["headless_folder_delete_dialog_layout_inflation_verified"] = True
report["headless_folder_delete_dialog_clipping_setup_verified"] = True
report["headless_folder_delete_dialog_start_attachment_verified"] = True
report["headless_folder_delete_dialog_scroll_queries_verified"] = True
report["headless_folder_delete_dialog_title_measurement_verified"] = True
report["headless_folder_delete_dialog_presentation_verified"] = True
report["headless_folder_delete_cancel_restart_back_verified"] = True
report["headless_folder_deletion_verified"] = True
report["headless_folder_delete_restart_back_verified"] = True
report["folder_delete_retains_exact_notes"] = True
report["native_folder_delete_dialog_input_verified"] = False
report["folder_deletion_first_blocker"] = dialog_blocker
report["backup_restore_boundary_diagnosed"] = True
report["backup_private_input_channel_reached"] = True
report["backup_restore_exact_notes_folders_and_seed_retained"] = True
report["headless_backup_verified"] = True
report["headless_restore_verified"] = True
report["backup_restore_completes_before_guest_system_exit"] = True
report["backup_restore_exit_boundary"] = backup_restore_boundary
(root / "artifacts/notepad-compatibility.json").write_text(json.dumps(report, indent=2) + "\n")
print("PASS Notepad: Notes screen → note editor → typed title visible")
print("PASS Notepad: two saved titles appear immediately and survive restart")
print("PASS Notepad: existing row reopened, title/body edited, list refreshed and both fields retained after restart")
print("PASS Notepad: original Delete menu returns to Notes; survivor ID/title/body survive restart and reopen")
print("PASS Notepad: original Snackbar message/UNDO show at 250ms and are removed after timed dismissal; exact survivor retained")
print("PASS Notepad: original UNDO restores title/body with a fresh ID; exact survivor, harmless old timeout, restart and reopen verified")
print("PASS Notepad: original navigation tap reveals an on-screen drawer animation frame at 100ms; both exact note rows retained")
print("PASS Notepad: original drawer settles at 1000ms; Back closes it, retains Notes and preserves both exact rows")

print("PASS Notepad: original Edit Folders binds its editor/listener; Back retains both exact note rows")
print("PASS Notepad: original editor/Done creates one visible folder; saved row reopens after restart and Back restores both exact notes")
print("PASS Notepad: saved-row focus and unconfirmed input complete; restart discards that input and preserves exact notes/folder")
print("PASS Notepad: host editor focus runs guest callbacks; pending input is discarded on restart with exact notes/folder retained")
print("PASS Notepad: original rename confirmation completes; same folder ID/name survive restart and Back with both exact notes")
print("PASS Notepad: original folder-delete modal displays title/message/buttons; Cancel retains folder, confirmation deletes it, restart/Back preserve both exact notes")
print("PASS Notepad: APK creates a byte-exact SQLite backup; Restore recovers tampered notes/folder and fresh launch renders them")
print("NOTE Notepad: guest System.exit(0) remains unsupported after the exact database restore")

# The original APK stores XML metacharacters unescaped. Its own catch path must
# log the actual exception and show !ERROR!, without rewriting the stored body.
with tempfile.TemporaryDirectory(prefix="droidless-notepad-malformed-") as app_data:
    title = "Malformed XML diagnostic"
    raw_body = "Plain & <broken text"
    command = [str(args.binary), "run", "--headless", "--size", "390x844", "--data-dir", app_data]
    saved = subprocess.run(command + [
        "--click", "＋", "--input", title, "--input-at", "1", raw_body,
        "--back", str(notepad),
    ], text=True, capture_output=True, check=True, timeout=120)
    saved_labels = [node["view"]["text"] for node in flatten(json.loads(saved.stdout))]
    if title not in saved_labels or "Notes" not in saved_labels:
        raise SystemExit("Notepad did not return to its list after logging the malformed body")
    database = Path(app_data) / "ir.cafebazaar.notepad/databases/AppDatabase.db"
    with sqlite3.connect(database) as connection:
        before = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
    if before != [(1, title, raw_body)]:
        raise SystemExit("Notepad did not retain the original malformed body after save")
    reopened = subprocess.run(command + ["--click", title, str(notepad)],
        text=True, capture_output=True, check=True, timeout=120)
    fields = [node["view"]["text"] for node in flatten(json.loads(reopened.stdout))
        if node["view"]["kind"] == "EditText"]
    if fields != [title, "!ERROR!"]:
        raise SystemExit("Notepad did not show its own malformed XML fallback in the editor")
    for process in (saved, reopened):
        if "org.xml.sax.SAXException:" not in process.stderr or \
            "\tat Lir/cafebazaar/notepad/d/l;->c()Landroid/text/Spannable; [classes.dex, PC 0x0033]" not in process.stderr:
            raise SystemExit("Notepad did not log its retained XML-fault DEX location")
    with sqlite3.connect(database) as connection:
        after = connection.execute("SELECT id, title, body FROM Note ORDER BY id").fetchall()
    if before != after:
        raise SystemExit("Opening Notepad's malformed body changed the stored row")
report["malformed_body_guest_error_fallback_verified"] = True
report["malformed_body_open_preserves_database_row"] = True
(root / "artifacts/notepad-compatibility.json").write_text(json.dumps(report, indent=2) + "\n")
print("PASS Notepad: malformed XML logs original DEX fault, shows guest !ERROR! and preserves the stored row")
if hashlib.sha256(args.binary.read_bytes()).hexdigest() != runtime_sha:
    raise SystemExit("Runtime binary changed during public replay; rebuild and rerun before using these results")
runtime_evidence["verified"] = True
runtime_report.write_text(json.dumps(runtime_evidence, indent=2) + "\n")
print("PASS runtime identity: the full public replay used unchanged CLI SHA-256 " + runtime_sha)
