#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate cortex_m_rt as rt;

use nw_board_support::hal;

use rt::entry;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_dfu::DFUClass;

use rtt_target::{rprintln, rtt_init_print};

use core::{arch::asm, panic::PanicInfo, ptr};

use nw_board_support::*;

mod dfu;

use dfu::QspiDfu;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {}
}

#[entry]
fn main() -> ! {
    let external_flash = get_external_flash().init();
    external_flash.into_memory_mapped();
    
    unsafe { cortex_m::asm::bootload(0x90000000u32 as *const u32) }

    // let clocks = init_clocks(dp.RCC, &mut dp.PWR, &mut dp.FLASH);
    // let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    // static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    // let usb_bus = get_usb_bus_allocator(clocks.clone(), unsafe { &mut EP_MEMORY });

    // let dfu_mem = QspiDfu::new(external_flash);

    // let mut dfu = DFUClass::new(&usb_bus, dfu_mem);

    // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0483, 0xdf11))
    //     .manufacturer("Numworks")
    //     .product("RustWorks Bootloader")
    //     .serial_number("TEST")
    //     .device_class(0x02)
    //     .build();

    // let mut display = get_display(&clocks, &mut delay);

    // display.write_top("DFU interface enabled, write to 0x900000000 for external flash.");
    // display.draw_all();

    // loop {
    //     usb_dev.poll(&mut [&mut dfu]);
    // }
}
