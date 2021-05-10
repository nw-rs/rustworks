#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate panic_halt;

use rtic::app;

use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, pac, prelude::*};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use st7789::{Orientation, ST7789};

const HCLK_MHZ: u32 = 192;

#[app(device = stm32f7xx_hal::pac, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        // Cortex-M peripherals
        let cp: cortex_m::Peripherals = cx.core;

        // Device specific peripherals
        let dp: pac::Peripherals = cx.device;

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

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.hclk(HCLK_MHZ.mhz()).freeze();
        let mut delay = Delay::new(cp.SYST, clocks);

        let gpioc = dp.GPIOC.split();
        let gpiob = dp.GPIOB.split();
        let gpiod = dp.GPIOD.split();
        let gpioe = dp.GPIOE.split();

        let mut backlight_control = PWMPin::new(gpioe.pe0.into_push_pull_output());

        backlight_control.send_pulses(1, &mut delay);

        let red_pin = PWMPin::new(gpiob.pb4.into_push_pull_output());
        let green_pin = PWMPin::new(gpiob.pb5.into_push_pull_output());
        let blue_pin = PWMPin::new(gpiob.pb0.into_push_pull_output());

        let mut led = Led {
            red_pin,
            green_pin,
            blue_pin,
        };

        led.green(&mut delay);

        let lcd_bus = display_interface_parallel_gpio::Generic16BitBus::new((
            gpiod.pd14.into_push_pull_output(),
            gpiod.pd15.into_push_pull_output(),
            gpiod.pd0.into_push_pull_output(),
            gpiod.pd1.into_push_pull_output(),
            gpioe.pe7.into_push_pull_output(),
            gpioe.pe8.into_push_pull_output(),
            gpioe.pe9.into_push_pull_output(),
            gpioe.pe10.into_push_pull_output(),
            gpioe.pe11.into_push_pull_output(),
            gpioe.pe12.into_push_pull_output(),
            gpioe.pe13.into_push_pull_output(),
            gpioe.pe14.into_push_pull_output(),
            gpioe.pe15.into_push_pull_output(),
            gpiod.pd8.into_push_pull_output(),
            gpiod.pd9.into_push_pull_output(),
            gpiod.pd10.into_push_pull_output(),
        ))
        .unwrap();

        let lcd_interface = display_interface_parallel_gpio::PGPIO16BitInterface::new(
            lcd_bus,
            gpiod.pd11.into_push_pull_output(),
            gpiod.pd5.into_push_pull_output(),
        );

        let mut lcd_power = gpioc.pc8.into_push_pull_output();

        let lcd_reset = gpioe.pe1.into_push_pull_output();

        lcd_power.set_high().unwrap();

        let mut display = ST7789::new(lcd_interface, lcd_reset, 320, 240);

        display.init(&mut delay).unwrap();

        display.set_orientation(Orientation::Landscape).unwrap();

        display.clear(Rgb565::BLUE).unwrap();

        loop {
            led.red(&mut delay);
            delay.delay_ms(500u32);
            led.off();
            delay.delay_ms(500u32);
        }
    }
};

/// Simple PWM pin interface
struct PWMPin<P> {
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

    fn off(&mut self) {
        let _ = self.pin.set_low();
    }
}

struct Led<RP: OutputPin, GP: OutputPin, BP: OutputPin> {
    red_pin: PWMPin<RP>,
    green_pin: PWMPin<GP>,
    blue_pin: PWMPin<BP>,
}

impl<RP: OutputPin, GP: OutputPin, BP: OutputPin> Led<RP, GP, BP> {
    fn set_rgb(&mut self, delay: &mut Delay, red: bool, green: bool, blue: bool) {
        if red {
            self.red_pin.send_pulses(1, delay);
        } else {
            self.red_pin.off();
        }
        if green {
            self.green_pin.send_pulses(1, delay);
        } else {
            self.green_pin.off();
        }
        if blue {
            self.blue_pin.send_pulses(1, delay);
        } else {
            self.blue_pin.off();
        }
    }

    fn red(&mut self, delay: &mut Delay) {
        self.set_rgb(delay, true, false, false)
    }

    fn green(&mut self, delay: &mut Delay) {
        self.set_rgb(delay, false, true, false)
    }

    fn blue(&mut self, delay: &mut Delay) {
        self.set_rgb(delay, false, false, true)
    }

    fn off(&mut self) {
        self.red_pin.off();
        self.green_pin.off();
        self.blue_pin.off();
    }
}
