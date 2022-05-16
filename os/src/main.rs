#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate cortex_m_rt as rt;

use nw_board_support::hal::{self, timer::SysTimerExt};

use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use rt::entry;

use core::panic::PanicInfo;

use nw_board_support::*;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //rprintln!("{}", info);
    loop {}
}

#[entry]
fn main() -> ! {
    let mut led = get_led();
    led.blue();

    let mut dp = hal::pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    init_mpu(&mut cp.MPU);

    let clocks = init_clocks(dp.RCC, &mut dp.PWR, &mut dp.FLASH);

    let mut display = get_display(&clocks);

    display.write_top("Booted OS.");
    display.draw_all();
    
    let mut delay = cp.SYST.delay(&clocks);

    let mut led = get_led();

    loop {
        led.green();
        delay.delay_ms(250u32);
        led.off();
        delay.delay_ms(250u32);
    }
}
