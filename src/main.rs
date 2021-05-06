#![no_std]
#![no_main]

extern crate panic_halt;

use rtic::app;

use stm32f7xx_hal::fmc_lcd::{ChipSelect1, FsmcLcd, LcdPins, Timing};
use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, pac, prelude::*};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::style::*;

use st7789::{Orientation, ST7789};

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
        let clocks = rcc.cfgr.hclk(100.mhz()).freeze();
        let mut delay = Delay::new(cp.SYST, clocks);

        let gpioc = dp.GPIOC.split();
        let gpiob = dp.GPIOB.split();
        let gpiod = dp.GPIOD.split();
        let gpioe = dp.GPIOE.split();

        let mut red = PWMPin::new(gpiob.pb4.into_push_pull_output());
        let mut green = PWMPin::new(gpiob.pb5.into_push_pull_output());
        let mut blue = PWMPin::new(gpiob.pb0.into_push_pull_output());

        red.send_pulses(20, &mut delay);
        green.send_pulses(20, &mut delay);
        blue.send_pulses(20, &mut delay);

        let lcd_pins = LcdPins {
            data: (
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
            ),
            address: gpiod.pd11.into_alternate_af12(),
            read_enable: gpiod.pd4.into_alternate_af12(),
            write_enable: gpiod.pd5.into_alternate_af12(),
            chip_select: ChipSelect1(gpiod.pd7.into_alternate_af12()),
        };

        let mut lcd_power = gpioc.pc8.into_push_pull_output();
        let mut lcd_reset = gpioe.pe1.into_push_pull_output();
        let mut lcd_extd_command = gpiod.pd6.into_push_pull_output();
        let mut lcd_tearing_effect = gpiob.pb11.into_push_pull_output();
        let mut backlight_control = PWMPin::new(gpioe.pe0.into_push_pull_output());

        lcd_power.set_high().unwrap();
        lcd_reset.set_high().unwrap();
        lcd_tearing_effect.set_high().unwrap();
        lcd_extd_command.set_high().unwrap();

        let read_timing = Timing::default().data(8).address_setup(8).bus_turnaround(0);
        let write_timing = Timing::default().data(3).address_setup(3).bus_turnaround(0);

        let (_fsmc, lcd) = FsmcLcd::new(dp.FMC, lcd_pins, &clocks, &read_timing, &write_timing);

        backlight_control.send_pulses(10, &mut delay);

        let mut display = ST7789::new(lcd, lcd_reset, 320, 240);

        display.init(&mut delay).unwrap();

        display.set_orientation(Orientation::Landscape).unwrap();

        let circle1 = Circle::new(Point::new(128, 64), 64)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
        let circle2 = Circle::new(Point::new(64, 64), 64)
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1));

        let blue_with_red_outline = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLUE)
            .stroke_color(Rgb565::RED)
            .stroke_width(1) // > 1 is not currently supported in embedded-graphics on triangles
            .build();
        let triangle = Triangle::new(
            Point::new(40, 120),
            Point::new(40, 220),
            Point::new(140, 120),
        )
        .into_styled(blue_with_red_outline);

        let line = Line::new(Point::new(180, 160), Point::new(239, 239))
            .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));

        // draw two circles on black background
        display.clear(Rgb565::BLACK).unwrap();
        circle1.draw(&mut display).unwrap();
        circle2.draw(&mut display).unwrap();
        triangle.draw(&mut display).unwrap();
        line.draw(&mut display).unwrap();

        loop {
            continue;
        }
    }
};

/// Simple PWM pin interface
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
