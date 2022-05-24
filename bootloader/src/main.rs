#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate cortex_m_rt as rt;

use rt::entry;

use rtt_target::{rprintln, rtt_init_print};

use core::{panic::PanicInfo, slice};

use nw_board_support::*;

use nw_board_support::hal::{self};

use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_dfu::DFUClass;

use dfu::QspiDfu;

mod dfu;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {}
}

// #[interrupt]
// fn QUADSPI() {
//     get_led().blue();

//     rprintln!("QSPI interrupt.");

//     unsafe {
//         let qspi = &(*hal::pac::QUADSPI::ptr());
//         qspi.cr.modify(|_, w| w.ftie().clear_bit());
//     }
// }

// #[entry]
// fn main() -> ! {
//     rtt_init_print!(NoBlockTrim, 4096);

//     get_led().off();

//     // unsafe {
//     //     cortex_m::peripheral::NVIC::unmask(interrupt::QUADSPI);
//     // }

//     static mut EP_MEMORY: [u32; 1024] = [0; 1024];

//     init_mpu();
//     let mut external_flash = get_external_flash();

//     rprintln!("first word: {:08x}", external_flash.read_u32(0));

//     rprintln!("first word: {:08x}", external_flash.read_u32(0));

//     // external_flash.erase_all();

//     external_flash.into_memory_mapped();

//     let test = unsafe { slice::from_raw_parts(0x90000000u32 as *const u32, 32) };

//     rprintln!("{:08x?}\n", test);

//     // unsafe { cortex_m::asm::bootload(0x90000000u32 as *const u32) }

//     for i in 0..10 {
//         rprintln!(
//             "0x{:08x} read: 0x{:08x}",
//             0x90000000u32 + (i * 4 * 1024),
//             unsafe { ((0x90000000u32 + (i * 4 * 1024)) as *const u32).read_volatile() }
//         );
//     }

//     rprintln!("");

//     external_flash::reset();

//     let _external_flash = get_external_flash().into_memory_mapped();

//     let qspi = unsafe { &(*QUADSPI::ptr()) };

//     for i in 0..10 {
//         rprintln!(
//             "0x{:08x} read: 0x{:08x}",
//             0x90000000u32 + (i * 4 * 1024),
//             unsafe { ((0x90000000u32 + (i * 4 * 1024)) as *const u32).read_volatile() }
//         );

//     }

//     external_flash::reset();

//     loop {}
// }

#[entry]
fn main() -> ! {
    // Initialize RTT printing (for debugging).
    rtt_init_print!(NoBlockTrim, 4096);

    // let cp = cortex_m::Peripherals::take().unwrap();
    let dp = unsafe { hal::pac::Peripherals::steal() };

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    init_mpu();

    let clocks = init_clocks(dp.RCC);

    let usb_bus = get_usb_bus_allocator(&clocks, unsafe { &mut EP_MEMORY });

    let mut display = get_display(&clocks);

    external_flash::init();

    let test = unsafe { slice::from_raw_parts(0x90000000u32 as *const u32, 32) };

    rprintln!("{:08x?}\n", test);

    let dfu_mem = QspiDfu::new();

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

// #[entry]
// fn main() -> ! {
//     rtt_init_print!(NoBlockTrim, 4096);

//     init_mpu();

//     let mut external_flash = get_external_flash().init();

//     let dp = unsafe { hal::pac::Peripherals::steal() };

//     let _clocks = init_clocks(dp.RCC);

//     rprintln!("indirect +0 first: {:08x}", external_flash.read_byte(0x00000000u32));
//     rprintln!("indirect +4 first: {:08x}", external_flash.read_byte(0x00000004u32));

//     external_flash.into_memory_mapped();

//     rprintln!("+0 first: {:08x}", unsafe {
//         (0x90000000u32 as *const u32).read_volatile()
//     });

//     rprintln!("+4 first: {:08x}", unsafe {
//         (0x90000004u32 as *const u32).read_volatile()
//     });

//     for i in 0..1000 {
//         unsafe {
//             ((0x90000000u32 + (i * 4321 % 0x800000u32)) as *const u8).read_volatile();
//         }
//     }

//     rprintln!("+0 first: {:08x}", unsafe {
//         (0x90000000u32 as *const u32).read_volatile()
//     });

//     rprintln!("+4 first: {:08x}", unsafe {
//         (0x90000004u32 as *const u32).read_volatile()
//     });

//     cortex_m::asm::delay(216000);

//     // unsafe { cortex_m::asm::bootload(0x90000000u32 as *const u32) }

//     loop {
//     }

//     // let clocks = init_clocks(dp.RCC, &mut dp.PWR, &mut dp.FLASH);
//     // let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

//     // static mut EP_MEMORY: [u32; 1024] = [0; 1024];

//     // let usb_bus = get_usb_bus_allocator(clocks.clone(), unsafe { &mut EP_MEMORY });

//     // let dfu_mem = QspiDfu::new(external_flash);

//     // let mut dfu = DFUClass::new(&usb_bus, dfu_mem);

//     // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0483, 0xdf11))
//     //     .manufacturer("Numworks")
//     //     .product("RustWorks Bootloader")
//     //     .serial_number("TEST")
//     //     .device_class(0x02)
//     //     .build();

//     // let mut display = get_display(&clocks, &mut delay);

//     // display.write_top("DFU interface enabled, write to 0x900000000 for external flash.");
//     // display.draw_all();

//     // loop {
//     //     usb_dev.poll(&mut [&mut dfu]);
//     // }
// }
