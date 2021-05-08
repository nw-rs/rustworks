#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate panic_halt;

use rtic::app;

use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, pac, prelude::*};
/*use stm32f7xx_hal::{
    fmc_lcd::{AccessMode, ChipSelect1, FsmcLcd, LcdPins, Timing},
    gpio::Speed::VeryHigh,
};*/

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use st7789::{Orientation, ST7789};

mod fsmc;

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

        let lcd = fsmc::FSMC16BitInterface::new(
            HCLK_MHZ,
            gpiod.pd14.into_alternate_af12(),
            gpiod.pd15.into_alternate_af12(),
            gpiod.pd0.into_alternate_af12(),
            gpiod.pd1.into_alternate_af12(),
            gpioe.pe7.into_alternate_af12(),
            gpioe.pe8.into_alternate_af12(),
            gpioe.pe9.into_alternate_af12(),
            gpioe.pe10.into_alternate_af12(),
            gpioe.pe11.into_alternate_af12(),
            gpioe.pe12.into_alternate_af12(),
            gpioe.pe13.into_alternate_af12(),
            gpioe.pe14.into_alternate_af12(),
            gpioe.pe15.into_alternate_af12(),
            gpiod.pd8.into_alternate_af12(),
            gpiod.pd9.into_alternate_af12(),
            gpiod.pd10.into_alternate_af12(),
            gpiod.pd11.into_alternate_af12(),
            gpiod.pd5.into_alternate_af12(),
            gpiod.pd4.into_alternate_af12(),
            gpiod.pd7.into_alternate_af12(),
        );

        /*let lcd_pins = LcdPins {
            data: (
                gpiod.pd14.into_alternate_af12().set_speed(VeryHigh),
                gpiod.pd15.into_alternate_af12().set_speed(VeryHigh),
                gpiod.pd0.into_alternate_af12().set_speed(VeryHigh),
                gpiod.pd1.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe7.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe8.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe9.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe10.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe11.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe12.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe13.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe14.into_alternate_af12().set_speed(VeryHigh),
                gpioe.pe15.into_alternate_af12().set_speed(VeryHigh),
                gpiod.pd8.into_alternate_af12().set_speed(VeryHigh),
                gpiod.pd9.into_alternate_af12().set_speed(VeryHigh),
                gpiod.pd10.into_alternate_af12().set_speed(VeryHigh),
            ),
            address: gpiod.pd11.into_alternate_af12().set_speed(VeryHigh),
            read_enable: gpiod.pd4.into_alternate_af12(),
            write_enable: gpiod.pd5.into_alternate_af12(),
            chip_select: ChipSelect1(gpiod.pd7.into_alternate_af12()),
        };*/

        let mut lcd_power = gpioc.pc8.into_push_pull_output();

        let lcd_reset = gpioe.pe1.into_push_pull_output();

        lcd_power.set_high().unwrap();

        /*let ns_to_cycles = |ns: u32| ns * HCLK_MHZ / 1000 + 1;

        let tedge: u32 = 15;
        let twc: u32 = 66;
        let trcfm: u32 = 450;
        let twrl: u32 = 15;
        let trdlfm: u32 = 355;

        let trdatast = trdlfm + tedge;
        let twdatast = twrl + tedge;

        let read_data_cycles = ns_to_cycles(trdatast);

        let read_addrsetup_cycles = ns_to_cycles(trcfm - trdatast);

        let write_data_cycles = ns_to_cycles(twdatast);

        let write_addrsetup_cycles = ns_to_cycles(twc - twdatast) - 1;

        let read_timing = Timing::default()
            .data(read_data_cycles as u8)
            .address_hold(0)
            .address_setup(read_addrsetup_cycles as u8)
            .bus_turnaround(0)
            .access_mode(AccessMode::ModeA);

        let write_timing = Timing::default()
            .data(write_data_cycles as u8)
            .address_hold(0)
            .address_setup(write_addrsetup_cycles as u8)
            .bus_turnaround(0)
            .access_mode(AccessMode::ModeA);

        let (_fsmc, lcd) = FsmcLcd::new(dp.FMC, lcd_pins, &clocks, &read_timing, &write_timing);*/

        let mut display = ST7789::new(lcd, lcd_reset, 320, 240);

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
