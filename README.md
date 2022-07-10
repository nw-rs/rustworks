# RustWorks

An OS (eventually) for the Numworks calculator (model n0110).

## Setup

First install Rust using rustup by following [these instructions](https://www.rust-lang.org/tools/install) then
open a terminal and execute the following commands:

```zsh
rustup target add thumbv7em-none-eabihf

# Ubuntu
sudo apt-get install gcc-arm-none-eabi binutils-arm-none-eabi 
sudo apt-get install dfu-util
# macOS
brew tap osx-cross/arm
brew install arm-gcc-bin
brew install dfu-util

rustup component add llvm-tools-preview
cargo install cargo-binutils
cargo install cargo-make

# If you have an STLink debugger and can connect it to the calculator install `probe-run` or
# `cargo-embed` for easy debugging and `cargo-flash` if you just want faster flashing speeds.
cargo install probe-run
cargo install cargo-embed
cargo install cargo-flash

# Clone the repo (recursively to get submodules)
git clone --recursive https://github.com/nw-rs/rustworks.git

# If you cloned the repo before reading this and did not do so recursively do the following in the repo dir
git submodule update --init --recursive
```

## DFU Flash

Complete setup, plug in your calculator and put it into DFU mode (press 6 and reset at the same
time), then run the following command:

```zsh
cargo make dfu
```

## STLink

If you have an STLink debugger (I am using the STLink V3SET) you can flash faster by using one of
the following:

### Flash
```zsh
cargo flash --chip=stm32f730V8Tx
# Or you can use this which does the same thing but might be easier to remember:
cargo make flash
```

### Debug

Using `cargo-embed` (recommended):
```zsh
cargo embed
```

Using `probe-rs`:
```zsh
cargo run
```

