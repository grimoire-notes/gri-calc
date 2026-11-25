# gri-calc

A calculator widget for [Grimoire](https://github.com/grimoire-notes/grimoire-scribe).

## Install

In Grimoire:

1. Open **Settings → Packages → Widgets**.
2. Tap **Install from repository...**.
3. Enter `grimoire-notes/gri-calc`.
4. Read the trust warning and continue.

Then open **Widgets** on the editor toolbar and choose **Calculator**.

## What it does

- **Simple**: types the whole sum as you go (`5+5`, then `=`), with parentheses, percent, sign change,
  decimal point, backspace and clear. The smaller line above the answer previews the result while you
  type; after `=` the sum moves up to that line and the answer takes its place.
- **Scientific**: an expression reader with parentheses, `sin`, `cos` and `tan` in degrees, `ln`, `log`,
  square root, powers, π and e. It previews the result above the answer the same way.
- **Programming**: decimal, hexadecimal, binary and octal, with bitwise AND/OR/XOR/NOT, shifts, MOD,
  power, and a converter for length, mass, data and temperature.
- **Finance**: mortgage repayment, compound interest, simple interest and a savings goal. Tap a field
  (loan/principal, rate, years) and type with the keypad; the results update as you type.
- **Graphing**: type a formula for `y` using `x`, powers, π, e, `sin`, `cos`, `tan`, `ln`, `log` and
  square root, and the curve is plotted against axes with range labels. It redraws on every key.
- A mode bar at the top of the frame switches between the five calculators. Each mode keeps its own screen.
- Dividing by zero shows `Error`; AC starts again, and backspace repairs a mistyped expression.
- The frame can be moved, resized, minimized and closed like any widget.

## Settings

The package's settings page chooses what a new calculator window opens with and which modes the
mode bar offers:

- **Default calculator**: Simple, Scientific, Programming, Finance or Graphing.
- **Simple calculator**, **Scientific calculator**, **Programming calculator**, **Finance calculator**,
  **Graphing calculator**: turn individual modes on or off.

A disabled mode is left out of the mode bar. If the default mode is disabled, the window opens on
the first enabled one; if every mode is disabled, Simple stays on. A mode turned off while a frame
is showing it falls back to the first enabled mode.

The widget is a WebAssembly module (`spec.runtime: wasm`) and draws through Grimoire's generic
widget ABI. Nothing about the calculator is built into Grimoire: it installs and runs from this
repository like any other widget package.

## Build

```sh
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/gri_calc.wasm calculator.bin
```

Then update `artifact.sha256` and `artifact.size` in `package.yaml` (for example with Grimoire's
`tools/widget-pack.sh`), and bump `version` before publishing.

## Files

- `package.yaml` — the manifest Grimoire installs.
- `calculator.bin` — the built package artifact named by the manifest.
- `src/lib.rs` — the calculator's source.
- `Cargo.toml`, `Cargo.lock` — the Rust build.
- `LICENSE` — MIT.
