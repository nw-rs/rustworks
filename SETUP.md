# Setup

This guide explains how to setup an embedded Rust development environment
for the Numworks n0110 calculator.

## Install Rust

To develop software in Rust you need the Rust toolchain which can be installed
using rustup using the instructions at 
[rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

## Install the ARM toolchain

To build for STM32 we need the GNU Arm Embedded Toolchain, you can either 
install it using a package manager or download the binaries directly from the
[developer.arm.com](https://developer.arm.com/downloads/-/gnu-rm) and then add
it to your path.

Use the following instructions to use your operating system's package manager
or if your OS isn't listed use the built in search function to find it.

### Debian based

```zsh
sudo apt-get install gcc-arm-none-eabi
```

### macOS

Install [Homebrew](https://brew.sh/) then run the following:

```zsh
brew tap osx-cross/arm
brew install arm-gcc-bin
```

## Install dfu-util

If you don't have an STLink probe you will need to flash the calculator using
DFU (you'll still need this to test your changes even if you have an STLink).
You can get this by following the instructions on
[the dfu-util homepage](http://dfu-util.sourceforge.net/).

## Install cargo tools

RustWorks and some related projects use
[`cargo-make`](https://sagiegurari.github.io/cargo-make/) to make flashing
easier and
[`cargo-binutils`](https://github.com/rust-embedded/cargo-binutils) to convert
the output of `cargo build` into raw binaries. Install them using the following
commands:

```zsh
cargo install cargo-binutils
cargo install cargo-make
```

We also need to add the `thumbv7em-none-eabi` target to Rust and install the
Rust LLVM tools preview:

```zsh
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
```

If you have an STLink probe you can also install `probe-run`, `cargo-embed` and
`cargo-flash` to make flashing and debugging easier:

```zsh
cargo install probe-run
cargo install cargo-embed
cargo install cargo-flash
```



