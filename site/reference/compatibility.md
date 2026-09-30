# Compatibility and exact limits

## Implemented opcode groups

| Opcode range | Implemented group |
|---|---|
| 00–11 | NOP, move variants/results/exceptions, returns |
| 12–1c | Numeric/wide/string/class constants |
| 1d–27 | Single-thread monitors, casts/type tests, arrays/instances, explicit throw |
| 28–3d | Goto, packed/sparse switches, numeric comparisons and conditions |
| 44–6d | Array and instance/static field reads/writes |
| 6e–72, 74–78 | Virtual/super/direct/static/interface calls and range forms |
| 7b–8f | Negation/complement and numeric conversions |
| 90–cf | Int/long/float/double arithmetic and two-address forms |
| d0–e2 | Integer literal operations |

Decoder coverage is not proof of every valid/malformed edge case. Tests cover
arithmetic boundaries and compiled loops, fields, class initialization, wide
values, arrays, dispatch and explicit/implicit throw/catch/finally. Complete verifier/debug/
annotation coverage remains pending. Reserved/newer opcodes fail with method/PC.

## Framework families reached by real execution/tests

Activity constructors/lifecycle/content/title/findViewById/window metrics;
Context/Resources strings/resources; View ID/visibility/enabled/background/
uniform padding/click/key listeners; ViewGroup addView; LinearLayout orientation;
TextView text/append/size/color/gravity; EditText text/null key listener/selection;
KeyEvent action/keycode; String valueOf/toString/length/equals/startsWith/contains/
substring/concat/charAt; StringBuilder constructors/append/toString; Integer
parseInt/toString; Double parseDouble/valueOf(D)/doubleValue/toString/isNaN(D); Long toString(J); Math sqrt/cbrt/sin/cos/tan/log/
exp/abs/pow; Log d/i/w/e. The source's exact signature table is authoritative;
other overloads remain unsupported.
Throwable constructors/getMessage/getCause/toString and common runtime exception types
are implemented for the fault paths covered by conformance tests.
Current source adds explicit same-APK Intent constructors/setClass/setClassName,
startActivity, getIntent, finish/isFinishing/onBackPressed, Bundle typed extras and
back-stack lifecycle. This is authored-fixture evidence, not a third-party notes
app claim; the v0.1.0 release predates navigation, persistence, collections and the new calculator demo.
SharedPreferences adds String/int/long/float/boolean reads and staged editors,
commit/apply/remove/clear, MODE_PRIVATE stores, Activity.getPreferences and a
minimal application-context singleton. Native authored-note save/restart/clear
and isolated persistence are verified. Apply is synchronous; preference listeners,
String sets and general file APIs remain unsupported. [Storage limits](storage.md).
HashSet/HashMap add bounded basic operations using guest equals; Set iterators
support removal and fail-fast next/remove. Collections.unmodifiableSet stays live
and rejects mutation. Canonical Class literals work as Map keys, with basic package
metadata. [Collections methods, evidence and ceilings](collections.md).

APK-local Class.forName(String), getName/getClass and no-argument construction
execute guest code. Field opcodes canonicalize inherited declaring owners.
[Reflection scope and evidence](reflection.md).

## Known ceilings

- One foreground Activity with a bounded preserved back stack; single-threaded VM
  and approximate layout/style/configuration. No saved-state recreation/tasks/
  launch modes/activity results/implicit or external Intents.
- Explicit and common implicit Java exceptions are catchable; unsupported APIs,
  malformed instructions and host resource ceilings remain terminal diagnostics.
- UTF-16 lengths/substrings are honored; isolated surrogates are rejected by Rust.
- Java float string scientific-notation edge cases differ from Rust formatting.
- Failed class initialization is sticky and retains causes; concurrent initialization
  is unsupported. Instruction/field/method checks are not a complete Java type verifier.
- No bulk collections/Map views/custom class loaders/method or field reflection/general file I/O/Handler/Looper/SQLite/images/networking/JNI/JIT,
  APK signature verification, installation registry or Linux native renderer.
- AndroidX, modern Kotlin patterns, Compose, multimedia and games are unsupported.

The catalog calls both public calculators **interactive subsets**, not fully
working. Menus, gestures, complete visual fidelity and all possible numeric
behavior have not been tested. TableLayout/TableRow use the basic linear model;
stretchColumns, gradients and Android themes remain incomplete.
Custom fixtures are labeled separately from independent APK evidence.
