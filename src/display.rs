use crate::HCLK;

use alloc::string::String;
use alloc::vec::Vec;

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

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

pub const DISPLAY_WIDTH: u16 = 320;
pub const DISPLAY_HEIGHT: u16 = 240;

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
    lines: Vec<String>,
    last_line: u8,
    power_pin: LcdPowerPin,
    extd_cmd_pin: LcdExtdCmdPin,
    _tearing_effect_pin: LcdTearingEffectPin,
    backlight_pin: LcdBacklightPin,
    backlight_state: u8,
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
            power_pin,
            backlight_pin,
            _tearing_effect_pin,
            extd_cmd_pin,
            backlight_state: 1,
            _fmc: fmc,
            lines: Vec::with_capacity(40),
            last_line: 0,
        }
    }

    pub fn clear(&mut self, color: Color) -> Result<(), DrawError> {
        self.display.clear(color)
    }

    pub fn set_backlight(&mut self, target: u8) {
        if target == 0 {
            self.backlight_pin.set_low().unwrap();
        } else {
            self.backlight_pin.set_high().unwrap();
        }
        self.backlight_state = target;
    }
}
