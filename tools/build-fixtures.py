#!/usr/bin/env python3
"""Build purpose-made APK fixtures. SDK tools are build-only, never runtime dependencies."""
import argparse
import os
from pathlib import Path
import subprocess
import zipfile

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--app", choices=sorted(app.name for app in (root / "examples").iterdir() if (app / "AndroidManifest.xml").exists()))
parser.add_argument("--sdk", default=os.environ.get("ANDROID_SDK_ROOT", os.environ.get("ANDROID_HOME", "/opt/homebrew/share/android-commandlinetools")))
args = parser.parse_args()
sdk = Path(args.sdk)
tools = sorted((sdk / "build-tools").iterdir(), key=lambda p: tuple(int(n) for n in p.name.split(".")))[-1]
platform = sorted((sdk / "platforms").glob("android-*/android.jar"), key=lambda p: int(p.parent.name[8:]))[-1]

def run(*cmd):
    subprocess.run([str(c) for c in cmd], check=True)

for app in sorted((root / "examples").iterdir()):
    if args.app and app.name != args.app:
        continue
    if not (app / "AndroidManifest.xml").exists():
        continue
    build = app / "build"
    for part in ["gen", "classes", "dex"]:
        (build / part).mkdir(parents=True, exist_ok=True)
    output = root / "fixtures" / "generated" / f"{app.name}.apk"
    output.parent.mkdir(exist_ok=True)
    package = [tools / "aapt", "package", "-f", "-M", app / "AndroidManifest.xml", "-S", app / "res", "-I", platform, "-J", build / "gen", "-F", output]
    if (app / "assets").is_dir():
        package.extend(["-A", app / "assets"])
    run(*package)
    sources = sorted(app.glob("*.java")) + sorted((build / "gen").rglob("*.java"))
    run("javac", "-source", "8", "-target", "8", "-Xlint:-options", "-classpath", platform, "-d", build / "classes", *sources)
    run(tools / "d8", "--min-api", "21", "--lib", platform, "--output", build / "dex", *sorted((build / "classes").rglob("*.class")))
    with zipfile.ZipFile(output, "a", compression=zipfile.ZIP_DEFLATED) as apk:
        apk.write(build / "dex" / "classes.dex", "classes.dex")
    print(output.relative_to(root))
