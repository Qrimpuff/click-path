# click-path

A Windows auto-clicker tool built in Rust that records and replays mouse clicks with timing.

## Features

- Record sequences of mouse clicks and timing
- Save click patterns to a JSON file
- Replay recorded patterns once or in a loop
- Global hotkeys for all operations
- Windows-only support

## Installation

Clone and build from source:

```sh
git clone https://github.com/Qrimpuff/click-path.git
cd click-path
```

For personal use, the recommended way is to run directly with:

```sh
cargo run --release
```

If you want to install it system-wide, use:

```sh
cargo install --path .
```

This will install the executable to your Cargo bin directory (usually `%USERPROFILE%\.cargo\bin`).

To uninstall, run:

```sh
cargo uninstall click-path
```

## Usage

The program uses the following hotkeys (all with Windows key modifier):

- `Win + 1`: Start recording clicks
- `Win + 2`: Register current mouse position as a click
- `Win + 3`: Stop recording and save pattern
- `Win + 4`: Play recorded pattern once
- `Win + 5`: Play recorded pattern in a loop
- `Win + Esc`: Exit the program

Click patterns are saved to `click_path.json` in the current directory.

## Building

```sh
cargo build --release
```

The executable will be located in `target/release/click-path.exe`

## License

MIT License - See [LICENSE](LICENSE) file for details.
