# Compatibility-driven next milestones

The first third-party interactive calculator milestone is proven on macOS ARM64.
The following are objectives, not completed capability claims.

1. Correct VM faults/types: catchable runtime Java exceptions, full array/reference
   validation, inherited interfaces, failed class initialization and instruction
   boundaries. Gate: compiled conformance/malformed-input regressions.
2. Improve input/resources: native text/key evidence, full focus/IME, qualifier/
   style resolution, images/drawables and weighted measurement. Gate: native
   interaction and comparisons with expected Android behavior.
3. Multi-screen/persistence: explicit Intents/Bundle/back stack and isolated
   SharedPreferences/files. Gate: notes/todo APK and restart/persistence tests.
4. Async Java: deterministic Handler/Looper/Runnable scheduling before threads.
   Gate: timer callbacks update UI under the documented main-thread model.
5. Lists/images/SQLite/network: follow first failures in substantial third-party
   apps, with host capabilities explicit and tested.
6. Linux native backend: common VM/View model, real Linux builds/render/input.
7. Kotlin/AndroidX, then fragments/Compose/JNI/foreign libraries: major subsequent
   compatibility projects. No emulator fallback.
8. Profile before JIT/AOT, Canvas/GLES, audio or games; interpreter stays the
   correctness reference.

Every stable increment runs local CI, updates evidence/docs, commits and pushes
main. GitHub Actions stay disabled. Releases/demos name exact artifacts and scope.
