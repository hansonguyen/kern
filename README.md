# kern

[![CI](https://github.com/hansonguyen/kern/actions/workflows/ci.yml/badge.svg)](https://github.com/hansonguyen/kern/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/kern.svg)](https://crates.io/crates/kern)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A terminal-native typing test inspired by Monkeytype — fast, minimal, and offline-first.

## Features

- Timed tests: 15s, 30s, or 60s (cycle with `Tab`)
- Live WPM, raw WPM, accuracy, and character breakdown
- Persistent stats saved to `~/.config/kern/stats.json`
- Zero config, zero network — runs entirely offline

## Install

### From crates.io

```bash
cargo install kern
```

### From source

```bash
git clone https://github.com/hansonguyen/kern
cd kern
cargo install --path .
```

## Usage

```bash
kern
```

kern starts a 15-second timed test immediately. Press `Tab` to cycle through duration options (15s → 30s → 60s) before typing begins.

## Keybindings

| Key               | Action                                    |
|-------------------|-------------------------------------------|
| `Tab`             | Cycle duration (waiting) / Restart test   |
| `Space` / `Enter` | Commit current word                       |
| `Backspace`       | Delete last character                     |
| `Esc`             | Quit                                      |

## Results

After each test, kern shows:

- **WPM** — words per minute (correctly typed words only)
- **Raw WPM** — all keystrokes, including errors
- **Accuracy** — percentage of correct keystrokes
- **Breakdown** — correct / incorrect / extra / missed characters

Stats are saved automatically to `~/.config/kern/stats.json`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
