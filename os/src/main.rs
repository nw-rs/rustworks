#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate cortex_m_rt as rt;

use nw_board_support::hal;

use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use rt::entry;

use rtt_target::{rprintln, rtt_init_print};

use core::panic::PanicInfo;

use nw_board_support::*;

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

    let mut cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = hal::pac::Peripherals::take().unwrap();

    init_mpu(&mut cp.MPU);

    let clocks = init_clocks(dp.RCC, &mut dp.PWR, &mut dp.FLASH);
    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    let mut display = get_display(&clocks, &mut delay);

    display.write_top("Booted OS.");
    display.draw_all();

    let mut led = get_led();

    loop {
        led.green();
        delay.delay_ms(250u32);
        led.off();
        delay.delay_ms(250u32);
    }
}
