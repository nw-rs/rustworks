#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, pac::Peripherals, prelude::*};

extern crate panic_halt;

struct PWMPin<P: OutputPin> {
    pin: P,
}

impl<P: OutputPin> PWMPin<P> {
    fn new(pin: P) -> Self {
        Self { pin }
    }

    fn send_pulses(&mut self, pulses: u32, delay: &mut Delay) {
        for _ in 0..pulses {
            let _ = self.pin.set_low();
            delay.delay_us(20u8);
            let _ = self.pin.set_high();
            delay.delay_us(20u8);
        }
    }
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let mut delay = Delay::new(cp.SYST, clocks);

    let gpioe = dp.GPIOE.split();
    let mut backlight = PWMPin::new(gpioe.pe0.into_push_pull_output());

    let gpiob = dp.GPIOB.split();
    let mut red = PWMPin::new(gpiob.pb4.into_push_pull_output());
    let mut green = PWMPin::new(gpiob.pb5.into_push_pull_output());
    let mut blue = PWMPin::new(gpiob.pb0.into_push_pull_output());

    red.send_pulses(20, &mut delay);
    green.send_pulses(20, &mut delay);
    blue.send_pulses(20, &mut delay);

    let data_str = "This is a message to test if writing to flash works.";
    let data: &[u8] = data_str.as_bytes();

    let mut flash = Flash::new(dp.FLASH);

    // The flash needs to be unlocked before any erase or program operations.
    flash.unlock();

    // Erase flash sector 3, which is located at address 0x0800C000
    flash.blocking_erase_sector(3).unwrap();

    // Program the DATA slice into the flash memory starting at offset 0xC00 from the
    // beginning of the flash memory.
    flash.blocking_program(0xC000, data).unwrap();

    // Lock the flash memory to prevent any accidental modification of the flash content.
    flash.lock();

    let mut backlight_level = 0;

    let mut red_level = 0u32;

    let mut green_level = 0u32;

    let mut blue_level = 0u32;

    loop {
        if backlight_level >= 16 {
            backlight_level = 0;
        } else {
            backlight_level = backlight_level + 1;
        }
            backlight.send_pulses(backlight_level, &mut delay);

        if red_level < u8::MAX.into() {
            red.send_pulses(red_level, &mut delay);
            red_level = red_level + 1;
        } else {
            if green_level < u8::MAX.into() {
                green.send_pulses(green_level, &mut delay);
                green_level = green_level + 1;
            } else {
                if blue_level < u8::MAX.into() {
                    blue.send_pulses(blue_level, &mut delay);
                    blue_level = blue_level + 1;
                } else {
                    red_level = 0;
                    green_level = 0;
                    blue_level = 0;
                }
            }
        }

        delay.delay_ms(100u16);
    }
}
