<div align="center">
  
# RustWorks

  <i>Creating a truly free operating system for calculators.</i>  
</div>

<div align="center">
  
[![GitHub issues](https://img.shields.io/github/issues/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/issues)
![GitHub pull requests](https://img.shields.io/github/issues-pr/nw-rs/rustworks?style=flat-square)
[![GitHub forks](https://img.shields.io/github/forks/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/network)
[![GitHub stars](https://img.shields.io/github/stars/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/stargazers)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/nw-rs/rustworks?style=flat-square)
![GitHub contributors](https://img.shields.io/github/contributors/nw-rs/rustworks?style=flat-square)
![Maintenance](https://img.shields.io/maintenance/yes/2022?style=flat-square)
[![GitHub license](https://img.shields.io/github/license/nw-rs/rustworks?style=flat-square)](https://github.com/nw-rs/rustworks/blob/master/LICENSE)  
  
</div>

An operating system and bootloader for the NumWorks calculator (model n0110).

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

## ❤️ Contributing ❤️

Contributions are extremely valued, specially now, since the lead developer (@willemml) has little time to work on this project.

If you would like to contribute, please, fork the repo and open a Pull Request (PR).

**Thank you!**

## ⚙️ Components ⚙️

RustWorks is composed of several components which are listed here:
- [rcas](https://github.com/nw-rs/rcas) Open source computer algebra system
- [board-support](https://github.com/nw-rs/board-support) Files for compatibility with NumWorks n0110 hardware
- [bootloader](https://github.com/nw-rs/bootloader) It's job is to bring the OS up and running


## ⚖️ Licensing ⚖️

The code in this project is licensed under the MIT license.
