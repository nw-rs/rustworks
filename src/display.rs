use crate::HCLK;

use core::fmt::Write;

use alloc::format;
use alloc::string::String;

use embedded_graphics::style::PrimitiveStyleBuilder;
use st7789::Orientation;
use st7789::ST7789;

use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use stm32f7xx_hal::delay::Delay;
use stm32f7xx_hal::fmc_lcd::{AccessMode, ChipSelect1, FmcLcd, Lcd, SubBank1, Timing};
use stm32f7xx_hal::gpio::gpiob::PB11;
use stm32f7xx_hal::gpio::gpioc::PC8;
use stm32f7xx_hal::gpio::gpiod::{PD0, PD1, PD10, PD11, PD14, PD15, PD4, PD5, PD6, PD7, PD8, PD9};
use stm32f7xx_hal::gpio::gpioe::{PE0, PE1, PE10, PE11, PE12, PE13, PE14, PE15, PE7, PE8, PE9};
use stm32f7xx_hal::gpio::{Alternate, Floating, Input, Output, PushPull, AF12};
use stm32f7xx_hal::pac::FMC;
use stm32f7xx_hal::prelude::*;

use embedded_graphics::fonts::Font6x8;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_text::prelude::*;

pub const DISPLAY_WIDTH: u16 = 320;
pub const DISPLAY_HEIGHT: u16 = 240;

pub const BG_COLOUR: Rgb565 = Rgb565::BLACK;
pub const TEXT_COLOUR: Rgb565 = Rgb565::GREEN;

const TOP_STRING_SIZE: usize = 2048;
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

pub type DrawError = <LcdST7789 as DrawTarget<Color>>::Error;

pub struct Display {
    pub display: LcdST7789,
    pub top: String,
    pub bottom: String,
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
    ) -> Self {
        let ns_to_cycles = |ns: u32| (HCLK / 1_000_000) * ns;

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
            .access_mode(AccessMode::ModeA);

        let twdatast = twrl + tedge;

        let write_data_cycles = ns_to_cycles(twdatast);

        let write_addrsetup_cycles = ns_to_cycles(twc - twdatast) - 1;

        let write_timing = Timing::default()
            .data(write_data_cycles as u8)
            .address_setup(write_addrsetup_cycles as u8)
            .access_mode(AccessMode::ModeA);

        power_pin.set_high().unwrap();

        backlight_pin.set_high().unwrap();

        extd_cmd_pin.set_high().unwrap();

        let (fmc, lcd) = FmcLcd::new(fmc, HCLK.hz(), lcd_pins, &read_timing, &write_timing);

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
            top: String::with_capacity(TOP_STRING_SIZE),
            bottom: String::with_capacity(128),
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

    pub fn write_bottom_to_top(&mut self) {
        self.write_top(&format!("\n{}", self.bottom));
        self.bottom.clear();
    }

    pub fn write_top(&mut self, text: &str) {
        if self.top.len() + text.len() > TOP_STRING_SIZE {
            self.top = self
                .top
                .chars()
                .skip(text.len() + self.top.len() - TOP_STRING_SIZE)
                .collect();
        }
        self.top.write_str(text).unwrap();
        let lines = self.top.lines().count();
        if lines > TOP_LINES {
            self.top = self
                .top
                .lines()
                .skip(lines - TOP_LINES)
                .collect::<alloc::vec::Vec<&str>>()
                .join("\n");
        }
    }

    pub fn write_bottom(&mut self, text: &str, redraw: bool) -> bool {
        if !(self.bottom.len() + text.len() > BOTTOM_STRING_SIZE) {
            self.bottom.write_str(text).unwrap();
            let lines = self.bottom.lines().count();
            if lines > BOTTOM_LINES {
                self.bottom = self
                    .bottom
                    .lines()
                    .skip(lines - BOTTOM_LINES)
                    .collect::<alloc::vec::Vec<&str>>()
                    .join("\n");
            }
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
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(Rgb565::GREEN)
            .background_color(Rgb565::BLACK)
            .vertical_alignment(Scrolling)
            .build();
        let bottom_bounds = Rectangle::new(
            Point::new(3, DISPLAY_HEIGHT as i32 - 16),
            Point::new(DISPLAY_WIDTH as i32 - 3, DISPLAY_HEIGHT as i32),
        );
        let bottom_text_box = TextBox::new(&self.bottom, bottom_bounds).into_styled(textbox_style);
        if clear {
            bottom_bounds
                .into_styled(PrimitiveStyleBuilder::new().fill_color(BG_COLOUR).build())
                .draw(&mut self.display)
                .unwrap();
        }
        bottom_text_box.draw(&mut self.display).unwrap();
    }

    pub fn draw_top(&mut self, clear: bool) {
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(Rgb565::GREEN)
            .background_color(Rgb565::BLACK)
            .vertical_alignment(Scrolling)
            .build();
        let top_bounds = Rectangle::new(
            Point::new(3, 5),
            Point::new(DISPLAY_WIDTH as i32 - 3, DISPLAY_HEIGHT as i32 - 20),
        );
        let top_text_box = TextBox::new(&self.top, top_bounds).into_styled(textbox_style);
        if clear {
            top_bounds
                .into_styled(PrimitiveStyleBuilder::new().fill_color(BG_COLOUR).build())
                .draw(&mut self.display)
                .unwrap();
        }
        top_text_box.draw(&mut self.display).unwrap();
    }

    pub fn draw_all(&mut self) {
        self.clear(BG_COLOUR);
        self.draw_bottom(false);
        self.draw_top(false);
    }
}
