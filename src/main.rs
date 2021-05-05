#![no_std]
#![no_main]

extern crate panic_halt;
use rtic::app;
use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, pac, prelude::*};

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

        let gpioe = dp.GPIOE.split();
        let mut backlight = PWMPin::new(gpioe.pe0.into_push_pull_output());

        backlight.send_pulses(10, &mut delay);

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
    }
};
