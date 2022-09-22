# RustWorks

An operating system and bootloader for the Numworks calculator (model n0110).

## Setup

First follow the instructions in [`SETUP.md`](./SETUP.md) then clone the
repository **recursively**:

```zsh
git clone --recursive https://github.com/nw-rs/rustworks.git
```

Currently the bootloader and external flash drivers are not finished so
everything must be flashed individually, please read the readme of the
submodule you are interested in for more information on flashing or
using it.

## Roadmap

The following listings are ordered by priority (the first has highest priority and the last has least priority).

- [ ] Drivers for the External flash chip (read, write, XiP)
- [ ] Finish CAS (rCAS)
- [ ] REPL on the calculator to demonstrate the CAS
- [ ] Multiple programs/apps with UI to choose one on boot
- [ ] Support for 3rd party apps and binaries
