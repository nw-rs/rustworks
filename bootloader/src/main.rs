#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate cortex_m_rt as rt;

use nw_board_support::hal;

use rt::entry;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_dfu::DFUClass;

use rtt_target::{rprintln, rtt_init_print};

use core::panic::PanicInfo;

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
    // Initialize RTT printing (for debugging).
    rtt_init_print!(NoBlockTrim, 4096);

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    let mut cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = hal::pac::Peripherals::take().unwrap();

    init_mpu(&mut cp.MPU);

    let clocks = init_clocks(dp.RCC, &mut dp.PWR, &mut dp.FLASH);
    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    let external_flash = get_external_flash();

    let usb_bus = get_usb_bus_allocator(clocks.clone(), unsafe { &mut EP_MEMORY });

    let mut display = get_display(&clocks, &mut delay);

    let mut external_flash = external_flash.init(&mut delay);

    let (manufacturer, device) = external_flash.read_ids();
    assert_eq!(manufacturer, 0x1F);
    assert_eq!(device, 0x16);

    let dfu_mem = QspiDfu::new(external_flash);

    let mut dfu = DFUClass::new(&usb_bus, dfu_mem);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0483, 0xdf11))
        .manufacturer("Numworks")
        .product("RustWorks Bootloader")
        .serial_number("TEST")
        .device_class(0x02)
        .build();

    display.write_top("DFU interface enabled, write to 0x900000000 for external flash.");
    display.draw_all();

    loop {
        usb_dev.poll(&mut [&mut dfu]);
    }
}
