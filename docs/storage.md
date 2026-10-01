# Persistent preferences

Current source implements bounded `SharedPreferences` and SQLite subsets. The
v0.1.0 release archive predates these capabilities. General Java file APIs,
filesDir, cacheDir, preference listeners and String sets remain unsupported.

## Host-selected data root

The CLI persists preferences by default below:

- macOS: `~/Library/Application Support/DROIDLESS/apps/<package>/`
- Other hosts: `$XDG_DATA_HOME/droidless/apps/<package>/`, falling back to
  `~/.local/share/droidless/apps/<package>/`. Linux execution is unverified.

`--data-dir APPS_ROOT` selects another host-approved root. `--ephemeral` disables
disk access and keeps values in this runtime only; the options are mutually
exclusive. The Rust API `Runtime::new` remains ephemeral; `with_data_dir` explicitly
grants the chosen directory capability. Manifest permissions grant no host access.

## API behavior

Context.getSharedPreferences and Activity.getPreferences accept MODE_PRIVATE only.
Stores are cached per name and kept alive across Activity transitions/GC. String,
int, long, float (including NaN bit patterns) and boolean values support typed
getters/defaults, contains, edit, remove and clear. A wrong getter type throws a
catchable ClassCastException. putString(key, null) removes the key.

Editor changes stay staged until commit/apply. Clear runs before every staged put,
regardless of call order, following the
[Android Editor contract](https://developer.android.com/reference/android/content/SharedPreferences.Editor).
Commit updates memory and returns whether disk persistence succeeded; a failed
write retains in-memory values and reports a diagnostic. Apply currently performs
the same synchronous write and discards the boolean result. Asynchronous disk
writes require the future scheduling subsystem. Concurrent processes are not
coordinated; last successful write wins.

## Storage boundary

Only a validated ASCII package identifier reaches directory lookup. A private
`.droidless-package` identity file rejects case-colliding package names instead
of sharing data on case-insensitive hosts. Preference names are 1–120 ASCII
letters/digits/`._-$`, excluding dot/dot-dot; hex encoding keeps `case` and `CASE`
distinct. Files use DROIDLESS JSON schema 1, not Android XML.

Opened `cap-std` directory handles keep operations under the selected package.
Package/prefs directories and data/identity files reject symlinks; regular-file
checks reject special files and Unix hard links. Reads are capped at 1 MiB; damaged
or unsupported files fail instead of being reset. Writes use an exclusively
created temporary file, sync, atomic rename and directory sync on Unix. Only the
writer's own temporary file is eligible for cleanup. New directories/files use
0700/0600 on Unix. Limits: 128 open stores, 16,384 entries per store, 1 MiB per file.

These are tested I/O boundaries, not an audited OS sandbox. Host access to the
same directory, package impersonation by another APK and simultaneous processes
are outside the current trust model. Use trusted APKs and a private apps root.

## Reproduce the authored fixture

```sh
target/release/droidless run --headless --data-dir artifacts/preferences-demo \
  fixtures/generated/preferences.apk --click "Edit note" \
  --input "This note survives restart" --click "Save note"
target/release/droidless run --headless --data-dir artifacts/preferences-demo \
  fixtures/generated/preferences.apk
```

`examples/preferences` is DROIDLESS-authored Java/XML conformance, not independent
notes-app compatibility. Restart, typed values, staged edits, failed saves,
corruption, case aliases and path/link isolation have executable regressions.

## Limited SQLite support

The runtime stores SQLite databases under the same per-package app-data root.
Its current bridge covers `SQLiteOpenHelper`, `execSQL`, transactions,
`compileStatement` binding/execution, `ContentValues` updates, `rawQuery`, and
typed cursor reads. This is a method subset, not general Android database
compatibility. The unchanged Notepad APK saves an edited title, and its `Note`
row survives a fresh process. The Notes screen renders the saved title both
after Back and after a fresh process restart; native-window interaction remains
unverified.
`tools/compatibility.py` checks this flow against the pinned upstream APK.
