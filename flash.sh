#!/bin/bash
cargo objcopy --bin numworks_custom_os --release -- -O binary target/thumbv7em-none-eabihf/numworks_custom_os.bin
dfu-util -d 0483:df11 -a 0 -s 0x08000000:leave -D target/thumbv7em-none-eabihf/numworks_custom_os.bin
