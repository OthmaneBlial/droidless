# Unmodified third-party fixtures

`smallest.apk` is the original `apk/k-release.apk` from
[krossovochkin/SmallestAPK](https://github.com/krossovochkin/SmallestAPK).
It was downloaded without modification. The adjacent Java source and license
come from commit `c6e1063d8b6c3b3ad852db534a8ccd63b877e92e` in that repository.
Java whitespace was normalized; the APK bytes are original. Its SHA-256 is
`4cd031dca892b0434274af0dee6f07cacc9163a5b685ba0f363f5642e0374318`.
This is a third-party Hello World sample, not evidence
of ordinary calculator or broad Android compatibility.

Fixtures authored for DROIDLESS live in `examples/` and are labeled separately.


The two calculator APKs are fetched into ignored `artifacts/apks/`, with SHA-256
checks; they are not redistributed as repository fixtures:

```sh
sh tools/fetch-kascalc.sh
sh tools/fetch-simple-calculator.sh
python3 tools/compatibility.py
```

Simple Calculator's original APK is pinned to upstream commit
`3ba860b281eba34f144e4e75115f0c0a06bced31`. The catalog records exact URLs, digests,
headless cases, native observations and limitations separately from authored tests.
