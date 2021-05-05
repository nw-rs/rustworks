#![no_std]
#![no_main]
use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, pac::Peripherals, prelude::*};

extern crate panic_halt;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let mut delay = Delay::new(cp.SYST, clocks);

    let gpioe = peripherals.GPIOE.split();
    let mut backlight = gpioe.pe0.into_push_pull_output();

    for _ in 0..(16 + 0xF - 8) {
        backlight.set_low().expect("GPIO cannot fail.");
        delay.delay_us(20u8);
        backlight.set_high().expect("GPIO cannot fail.");
        delay.delay_us(20u8);
    }

    let data_str = "This is a test message writing to flash.";
    let data: &[u8] = data_str.as_bytes();

    let mut flash = Flash::new(peripherals.FLASH);

    // The flash needs to be unlocked before any erase or program operations.
    flash.unlock();

    // Erase flash sector 3, which is located at address 0x0800C000
    flash.blocking_erase_sector(3).unwrap();

    // Program the DATA slice into the flash memory starting at offset 0xC00 from the
    // beginning of the flash memory.
    flash.blocking_program(0xC000, data).unwrap();

    // Lock the flash memory to prevent any accidental modification of the flash content.
    flash.lock();

    // Create a slice that can be used to read the written data.
    #[allow(unsafe_code)]
    let flash_data = unsafe { core::slice::from_raw_parts(0x0800C000 as *const u8, data.len()) };

    // Compare the written data with the expected value.
    if flash_data == data {
        hprintln!("Flash programming successful").unwrap();
    }

    loop {
        asm::nop()
    }
}
