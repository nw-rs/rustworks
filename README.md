<div align="center">
  
# RustWorks
  
</div>

<div align="center">
  
[![GitHub issues](https://img.shields.io/github/issues/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/issues)
[![GitHub forks](https://img.shields.io/github/forks/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/network)
[![GitHub stars](https://img.shields.io/github/stars/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/stargazers)
[![GitHub license](https://img.shields.io/github/license/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/blob/master/LICENSE)  
  
</div>

An operating system and bootloader for the Numworks calculator (model n0110).

## Setup

First follow the instructions in [`SETUP.md`](./SETUP.md) then clone the
repository **recursively**:

```zsh
git clone --recursive https://github.com/nw-rs/rustworks.git
```

Currently the bootloader and external flash drivers are not finished so
everything must be flashed individually, please read the README of the
submodule if you are interested in more information on flashing or
using it.

## 🚧 Roadmap 🚧

- [ ] Drivers for the External flash chip (read, write, XiP) **(highest priority)**
- [ ] Finish CAS (rCAS)
- [ ] REPL on the calculator to demonstrate the CAS
- [ ] Multiple programs/apps with UI to choose one on boot
- [ ] Support for 3rd party apps and binaries
