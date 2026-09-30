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
values, arrays, dispatch and explicit throw/catch. Complete verifier/debug/
annotation coverage remains pending. Reserved/newer opcodes fail with method/PC.

## Framework families reached by real execution/tests

Activity constructors/lifecycle/content/title/findViewById/window metrics;
Context/Resources strings/resources; View ID/visibility/enabled/background/
uniform padding/click/key listeners; ViewGroup addView; LinearLayout orientation;
TextView text/append/size/color/gravity; EditText text/null key listener/selection;
KeyEvent action/keycode; String valueOf/toString/length/equals/startsWith/contains/
substring/concat/charAt; StringBuilder constructors/append/toString; Integer
parseInt/toString; Double parseDouble/toString; Math sqrt/cbrt/sin/cos/tan/log/
exp/abs/pow; Log d/i/w/e. The source's exact signature table is authoritative;
other overloads remain unsupported.

## Known ceilings

- One Activity, single-threaded VM and approximate layout/style/configuration.
- Explicit throw/catch works; many runtime-created Java faults remain terminal.
- UTF-16 lengths/substrings are honored; isolated surrogates are rejected by Rust.
- Java float string scientific-notation edge cases differ from Rust formatting.
- Transitive interface assignability, array covariance, class-init failure semantics
  and verification of all instruction boundaries are incomplete.
- No collections/I/O/Intent/Handler/Looper/SQLite/images/networking/JNI/JIT,
  APK signature verification, installation registry or Linux native renderer.
- AndroidX, modern Kotlin patterns, Compose, multimedia and games are unsupported.

The catalog calls KasCalc an **interactive subset**, not fully working. Its menus,
gestures, visual fidelity and all possible numeric behavior have not been tested.
Custom fixtures are labeled separately from independent APK evidence.
