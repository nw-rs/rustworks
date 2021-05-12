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
```

## DFU Flash

Complete setup, plug in your calculator and put it into dfu mode (press 6 and reset at the same time), then run the following:

```zsh
cargo make dfu
```

## STLink Flash

If you have an STLink debugger (I am using the STLink V3SET) you can flash faster by using one of the following:

If you have [OpenOCD](http://openocd.org) installed you can use:
```zsh
cargo make flash
```

If you have the [STLink Tools](https://github.com/stlink-org/stlink) installed you can use:
```zsh
cargo make stflash
```

## OpenOCD

#### Installing OpenOCD

For macOS:
```zsh
brew install openocd
```

For Ubuntu:
```zsh
sudo apt-get install openocd
```

#### Using OpenOCD

If you have an STLink debugger and [OpenOCD](http://openocd.org) installed you can easily run and debug code.
To start, launch an OpenOCD session by entering the repository's directory and running the following:
```zsh
openocd
```

In another terminal window go to the repository's directory and execute `cargo run` which will launch a GDB session connected to the previously launched OpenOCD session and automatically upload the new binary.
