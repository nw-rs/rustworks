#![no_std]

use clocks::init_clocks;
use display::Display;
use external_flash::{ExternalFlash, Uninitialized};
use hal::{
    delay::Delay,
    fmc_lcd::{ChipSelect1, LcdPins},
    gpio::{
        gpiob::{PB0, PB4, PB5},
        GpioExt, Output, PushPull, Speed,
    },
    pac,
    rcc::Clocks,
};
use keypad::{KeyMatrix, KeyPad};
use led::Led;
use mpu::{init_mpu, init_mpu_bootloader};
pub use stm32f7xx_hal as hal;

pub mod clocks;
pub mod dfu;
pub mod display;
pub mod external_flash;
pub mod keypad;
pub mod led;
pub mod mpu;

pub const HCLK: u32 = 192_000_000;

use hal::otg_fs::USB;

pub fn get_devices(
    bootloader: bool,
) -> (
    ExternalFlash<Uninitialized>,
    Display,
    KeyPad,
    Led<PB4<Output<PushPull>>, PB5<Output<PushPull>>, PB0<Output<PushPull>>>,
    USB,
    Delay,
    Clocks,
) {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = pac::Peripherals::take().unwrap();

    if bootloader {
        init_mpu_bootloader(&mut cp.MPU);
    } else {
        init_mpu(&mut cp.MPU);
    }

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();
    let gpiob = dp.GPIOB.split();
    let gpiod = dp.GPIOD.split();
    let gpioe = dp.GPIOE.split();

    // Take ownership of the QSPI pins (to prevent them from being messed with later) and set
    // them to the correct modes.
    let qspi_pins = (
        gpiob.pb2.into_alternate_af9().set_speed(Speed::VeryHigh),
        gpiob.pb6.into_alternate_af10().set_speed(Speed::VeryHigh),
        gpioc.pc9.into_alternate_af9().set_speed(Speed::VeryHigh),
        gpiod.pd12.into_alternate_af9().set_speed(Speed::VeryHigh),
        gpiod.pd13.into_alternate_af9().set_speed(Speed::VeryHigh),
        gpioe.pe2.into_alternate_af9().set_speed(Speed::VeryHigh),
    );

    // Setup external flash over QSPI.
    let external_flash = ExternalFlash::new(&mut dp.RCC, dp.QUADSPI, qspi_pins);

    // Take onwership of the LCD pins and set them to the correct modes.
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

    let clocks = init_clocks(dp.RCC, &mut dp.PWR, &mut dp.FLASH);
    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    let usb = USB::new(
        dp.OTG_FS_GLOBAL,
        dp.OTG_FS_DEVICE,
        dp.OTG_FS_PWRCLK,
        (
            gpioa.pa11.into_alternate_af10(),
            gpioa.pa12.into_alternate_af10(),
        ),
        clocks.clone(),
    );

    // Setup the display.
    let display = Display::new(
        lcd_pins,
        dp.FMC,
        gpioe.pe1.into_push_pull_output(),
        gpioc.pc8.into_push_pull_output(),
        gpioe.pe0.into_push_pull_output(),
        gpiob.pb11.into_floating_input(),
        gpiod.pd6.into_push_pull_output(),
        &mut delay,
        &clocks,
    );

    // Setup the keypad for reading.
    let keymatrix = KeyMatrix::new(
        gpioa.pa0, gpioa.pa1, gpioa.pa2, gpioa.pa3, gpioa.pa4, gpioa.pa5, gpioa.pa6, gpioa.pa7,
        gpioa.pa8, gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5,
    );

    let keypad = KeyPad::new(keymatrix);

    // Setup the LED (currently just using it with 7 colours or off).
    let led = Led::new(
        gpiob.pb4.into_push_pull_output(),
        gpiob.pb5.into_push_pull_output(),
        gpiob.pb0.into_push_pull_output(),
    );

    (external_flash, display, keypad, led, usb, delay, clocks)
}
