#![no_std]
#![no_main]

extern crate panic_halt;
use rtic::app;
use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, gpio::Speed, pac, prelude::*};

macro_rules! fmc_pins {
    ($($pin:expr),*) => {
        (
            $(
                $pin.into_push_pull_output()
                    .set_speed(Speed::Medium)
                    .into_alternate_af12()
            ),*
        )
    };
}

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

#[app(device = stm32f7xx_hal::pac, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        // Cortex-M peripherals
        let cp: cortex_m::Peripherals = cx.core;

        // Device specific peripherals
        let dp: pac::Peripherals = cx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();
        let mut delay = Delay::new(cp.SYST, clocks);

        let gpioc = dp.GPIOC.split();
        let gpiob = dp.GPIOB.split();
        let gpiod = dp.GPIOD.split();
        let gpioe = dp.GPIOE.split();

        // Display Backlight
        {
            let mut backlight = PWMPin::new(gpioe.pe0.into_push_pull_output());

            backlight.send_pulses(10, &mut delay);
        }

        // LED
        {
            let mut red = PWMPin::new(gpiob.pb4.into_push_pull_output());
            let mut green = PWMPin::new(gpiob.pb5.into_push_pull_output());
            let mut blue = PWMPin::new(gpiob.pb0.into_push_pull_output());

            red.send_pulses(20, &mut delay);
            green.send_pulses(20, &mut delay);
            blue.send_pulses(20, &mut delay);
        }

        // LCD
        {
            let _fmc_io = fmc_pins! {
                gpiod.pd0,
                gpiod.pd1,
                gpiod.pd4,
                gpiod.pd5,
                gpiod.pd7,
                gpiod.pd8,
                gpiod.pd9,
                gpiod.pd10,
                gpiod.pd11,
                gpiod.pd14,
                gpiod.pd15,
                gpioe.pe7,
                gpioe.pe8,
                gpioe.pe9,
                gpioe.pe10,
                gpioe.pe11,
                gpioe.pe12,
                gpioe.pe13,
                gpioe.pe14,
                gpioe.pe15
            };

            let mut display_power = gpioc.pc8.into_push_pull_output();
            let mut display_reset = gpioe.pe1.into_push_pull_output();
            let mut display_extd_command = gpiod.pd6.into_push_pull_output();
            let mut display_tearing_effect = gpiob.pb11.into_push_pull_output();

            let _ = display_power.set_high();
            let _ = display_reset.set_high();
            let _ = display_extd_command.set_high();
            let _ = display_tearing_effect.set_high();

            delay.delay_ms(120u16);
        }

        // Flash
        {
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
        }
    }
};
