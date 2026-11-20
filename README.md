# gri-calc

A working calculator widget for [Grimoire](https://github.com/nebuff/grimoire-scribe).

## Install

In Grimoire:

1. Open **Settings → Packages → Widgets**.
2. Tap **Install from repository...**.
3. Enter `nebuff/gri-calc`.
4. Read the trust warning and continue.

Then open **Widgets** on the editor toolbar and choose **Calculator**.

## What it does

- Four functions: add, subtract, multiply and divide.
- Percent, sign change, decimal point, backspace and clear.
- Dividing by zero shows `Error`; AC or a digit starts again.
- The frame can be moved, resized, minimized and closed like any widget.

The package uses Grimoire's built-in native calculator runtime (`spec.runtime: native`,
`spec.entry: calculator`). The artifact is still size- and SHA-256-checked at install time.

## Files

- `package.yaml` — the manifest Grimoire installs.
- `calculator.bin` — the package artifact named by the manifest.
- `LICENSE` — MIT.
