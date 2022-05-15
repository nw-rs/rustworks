#![allow(dead_code)]

use core::fmt::Arguments;
use core::fmt::Write;

use heapless::String;
use st7789::Orientation;
use st7789::ST7789;

use stm32f7xx_hal::delay::Delay;
use stm32f7xx_hal::fmc_lcd::{AccessMode, ChipSelect1, FmcLcd, Lcd, SubBank1, Timing};
use stm32f7xx_hal::gpio::gpiob::PB11;
use stm32f7xx_hal::gpio::gpioc::PC8;
use stm32f7xx_hal::gpio::gpiod::{PD0, PD1, PD10, PD11, PD14, PD15, PD4, PD5, PD6, PD7, PD8, PD9};
use stm32f7xx_hal::gpio::gpioe::{PE0, PE1, PE10, PE11, PE12, PE13, PE14, PE15, PE7, PE8, PE9};
use stm32f7xx_hal::gpio::{Alternate, Floating, Input, Output, PushPull, AF12};
use stm32f7xx_hal::pac::FMC;
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::rcc::Clocks;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use embedded_text::{alignment::VerticalAlignment, TextBox};

pub const DISPLAY_WIDTH: u16 = 320;
pub const DISPLAY_HEIGHT: u16 = 240;

pub const BG_COLOUR: Rgb565 = Rgb565::BLACK;
pub const TEXT_COLOUR: Rgb565 = Rgb565::GREEN;

const TOP_STRING_SIZE: usize = 1479;
const TOP_LINES: usize = 38;
const BOTTOM_STRING_SIZE: usize = 104;
const BOTTOM_LINES: usize = 3;

type LcdPins = stm32f7xx_hal::fmc_lcd::LcdPins<
    (
        PD14<Alternate<AF12>>,
        PD15<Alternate<AF12>>,
        PD0<Alternate<AF12>>,
        PD1<Alternate<AF12>>,
        PE7<Alternate<AF12>>,
        PE8<Alternate<AF12>>,
        PE9<Alternate<AF12>>,
        PE10<Alternate<AF12>>,
        PE11<Alternate<AF12>>,
        PE12<Alternate<AF12>>,
        PE13<Alternate<AF12>>,
        PE14<Alternate<AF12>>,
        PE15<Alternate<AF12>>,
        PD8<Alternate<AF12>>,
        PD9<Alternate<AF12>>,
        PD10<Alternate<AF12>>,
    ),
    PD11<Alternate<AF12>>,
    PD4<Alternate<AF12>>,
    PD5<Alternate<AF12>>,
    ChipSelect1<PD7<Alternate<AF12>>>,
>;

pub type LcdResetPin = PE1<Output<PushPull>>;
pub type LcdPowerPin = PC8<Output<PushPull>>;
pub type LcdExtdCmdPin = PD6<Output<PushPull>>;
pub type LcdBacklightPin = PE0<Output<PushPull>>;
pub type LcdTearingEffectPin = PB11<Input<Floating>>;

pub type LcdST7789 = ST7789<Lcd<SubBank1>, LcdResetPin>;

pub type Color = Rgb565;

pub type DrawError = <LcdST7789 as DrawTarget>::Error;

pub struct Display {
    pub display: LcdST7789,
    pub top: String<TOP_STRING_SIZE>,
    pub bottom: String<BOTTOM_STRING_SIZE>,
    power_pin: LcdPowerPin,
    extd_cmd_pin: LcdExtdCmdPin,
    backlight_pin: LcdBacklightPin,
    backlight_state: u8,
    _tearing_effect_pin: LcdTearingEffectPin,
    _fmc: FmcLcd<LcdPins>,
}

impl Display {
    pub fn new(
        lcd_pins: LcdPins,
        fmc: FMC,
        mut reset_pin: LcdResetPin,
        mut power_pin: LcdPowerPin,
        mut backlight_pin: LcdBacklightPin,
        _tearing_effect_pin: LcdTearingEffectPin,
        mut extd_cmd_pin: LcdExtdCmdPin,
        delay: &mut Delay,
        clocks: &Clocks,
    ) -> Self {
        let ns_to_cycles = |ns: u32| (clocks.hclk().0 / 1_000_000) * ns / 1000;

        let tedge: u32 = 15;
        let twc: u32 = 66;
        let trcfm: u32 = 450;
        let twrl: u32 = 15;
        let trdlfm: u32 = 355;

        let trdatast = trdlfm + tedge;

        let read_data_cycles = ns_to_cycles(trdatast);

        let read_addrsetup_cycles = ns_to_cycles(trcfm - trdatast);

        let read_timing = Timing::default()
            .data(read_data_cycles as u8)
            .address_setup(read_addrsetup_cycles as u8)
            .address_hold(0)
            .bus_turnaround(0)
            .access_mode(AccessMode::ModeA);

        let twdatast = twrl + tedge;

        let write_data_cycles = ns_to_cycles(twdatast);

        let write_addrsetup_cycles = ns_to_cycles(twc - twdatast) - 1;

        let write_timing = Timing::default()
            .data(write_data_cycles as u8)
            .address_setup(write_addrsetup_cycles as u8)
            .address_hold(0)
            .bus_turnaround(0)
            .access_mode(AccessMode::ModeA);

        power_pin.set_high().unwrap();

        backlight_pin.set_high().unwrap();

        extd_cmd_pin.set_high().unwrap();

        let (fmc, lcd) = FmcLcd::new(fmc, clocks, lcd_pins, &read_timing, &write_timing);

        reset_pin.set_low().unwrap();
        delay.delay_ms(5u16);
        reset_pin.set_high().unwrap();
        delay.delay_ms(10u16);
        reset_pin.set_low().unwrap();
        delay.delay_ms(20u16);
        reset_pin.set_high().unwrap();
        delay.delay_ms(10u16);

        let mut display = ST7789::new(lcd, reset_pin, DISPLAY_WIDTH, DISPLAY_HEIGHT);

        display.init(delay).unwrap();

        display
            .set_orientation(Orientation::LandscapeSwapped)
            .unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        Self {
            display,
            top: String::new(),
            bottom: String::new(),
            power_pin,
            backlight_pin,
            extd_cmd_pin,
            backlight_state: 1,
            _tearing_effect_pin,
            _fmc: fmc,
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.display.clear(color).unwrap();
    }

    pub fn set_backlight(&mut self, target: u8) {
        if target == 0 {
            self.backlight_pin.set_low().unwrap();
        } else {
            self.backlight_pin.set_high().unwrap();
        }
        self.backlight_state = target;
    }

    pub fn write_bottom_to_top(mut self) -> Self {
        let bottom_content = self.bottom;
        self.bottom = String::new();
        self.write_top_fmt(format_args!("\n{}", &bottom_content));
        self
    }

    pub fn write_top(&mut self, text: &str) {
        if text.len() > TOP_STRING_SIZE {
            self.top.clear();
            self.top
                .push_str(unsafe {
                    text.get_unchecked((text.len() - TOP_STRING_SIZE)..(text.len() - 1))
                })
                .unwrap();
        } else {
            if self.top.len() + text.len() > TOP_STRING_SIZE {
                let old_top = self.top.clone();
                self.top.clear();
                self.top
                    .push_str(unsafe {
                        let t = &old_top.as_str().get_unchecked(
                            (old_top.len() + text.len() - TOP_STRING_SIZE)..(old_top.len() - 1),
                        );
                        t
                    })
                    .unwrap();
            }
            self.top.push_str(text).unwrap();
        }
    }

    pub fn write_top_fmt(&mut self, args: Arguments<'_>) {
        self.write_fmt(args).unwrap()
    }

    pub fn write_bottom(&mut self, text: &str, redraw: bool) -> bool {
        if !(self.bottom.len() + text.len() > BOTTOM_STRING_SIZE) {
            self.bottom.write_str(text).unwrap();
            if redraw {
                self.draw_bottom(true);
            }
            true
        } else {
            false
        }
    }

    pub fn clear_bottom(&mut self, redraw: bool) {
        self.bottom.clear();
        if redraw {
            self.draw_bottom(true);
        }
    }

    pub fn pop_bottom(&mut self, redraw: bool) {
        self.bottom.pop();
        if redraw {
            self.draw_bottom(true);
        }
    }

    pub fn draw_bottom(&mut self, clear: bool) {
        let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN);

        let bottom_bounds = Rectangle::new(
            Point::new(3, DISPLAY_HEIGHT as i32 - 16),
            self.display.size() - Size::new(6, 0),
        );

        if clear {
            bottom_bounds
                .into_styled(PrimitiveStyleBuilder::new().fill_color(BG_COLOUR).build())
                .draw(&mut self.display)
                .unwrap();
        }

        TextBox::with_vertical_alignment(
            &self.bottom,
            bottom_bounds,
            character_style,
            VerticalAlignment::Scrolling,
        )
        .draw(&mut self.display)
        .unwrap();
    }

    pub fn draw_top(&mut self, clear: bool) {
        let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN);

        let top_bounds = Rectangle::new(Point::new(3, 5), self.display.size() - Size::new(6, 15));

        if clear {
            top_bounds
                .into_styled(PrimitiveStyleBuilder::new().fill_color(BG_COLOUR).build())
                .draw(&mut self.display)
                .unwrap();
        }

        TextBox::with_vertical_alignment(
            &self.top,
            top_bounds,
            character_style,
            VerticalAlignment::Scrolling,
        )
        .draw(&mut self.display)
        .unwrap();
    }

    pub fn draw_all(&mut self) {
        self.clear(BG_COLOUR);
        self.draw_bottom(false);
        self.draw_top(false);
    }
}

impl Write for Display {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_top(s);
        Ok(())
    }
}
