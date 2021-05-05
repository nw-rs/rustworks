# Numworks Rust

An OS (eventually) for the Numworks calculator (model n0110).

## Setup

```zsh
rustup target add thumbv7em-none-eabihf

# Ubuntu
apt-get install gcc-arm-none-eabi binutils-arm-none-eabi 
apt-get install dfu-util
# macOS
brew tap osx-cross/arm
brew install arm-gcc-bin
brew install dfu-util

rustup component add llvm-tools-preview
cargo install cargo-binutils
cargo install cargo-make
```

## Flash

Complete setup, then run the following:

```zsh
cargo make dfu
```
