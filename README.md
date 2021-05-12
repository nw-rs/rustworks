# RustWorks

An OS (eventually) for the Numworks calculator (model n0110).

## Setup

First install Rust by following [these](https://www.rust-lang.org/tools/install) instuctions then:

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

# If you have an STLink debugger and can connect it to the calculator install `probe-run` for faster flashing and easy debugging.
cargo install probe-run
```

## DFU Flash

Complete setup, plug in your calculator and put it into dfu mode (press 6 and reset at the same time), then run the following:

```zsh
cargo make dfu
```

## STLink

If you have an STLink debugger (I am using the STLink V3SET) you can flash faster by using one of the following:

### Flash
```zsh
cargo flash
```

### Debug
```zsh
cargo run
```

