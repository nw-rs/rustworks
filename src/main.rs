#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate panic_halt;
use rtt_target::{rprintln, rtt_init_print};

use core::fmt::Write;

use rtic::app;

use stm32f7xx_hal::{delay::Delay, gpio::GpioExt, pac, prelude::*};

use embedded_graphics::fonts::Font6x8;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_text::prelude::*;

use heapless::{String, Vec};

use st7789::{Orientation, ST7789};

mod keypad;
mod led;

use keypad::{Key, KeyMatrix, KeyPad};
use led::Led;

const HCLK_MHZ: u32 = 216;

#[app(device = stm32f7xx_hal::pac, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        rtt_init_print!();
        rprintln!("Starting...");
        let cp: cortex_m::Peripherals = cx.core;

        let dp: pac::Peripherals = cx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.hclk(HCLK_MHZ.mhz()).freeze();
        let mut delay = Delay::new(cp.SYST, clocks);

        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();
        let gpiob = dp.GPIOB.split();
        let gpiod = dp.GPIOD.split();
        let gpioe = dp.GPIOE.split();

        let keymatrix = KeyMatrix::new(
            gpioa.pa0, gpioa.pa1, gpioa.pa2, gpioa.pa3, gpioa.pa4, gpioa.pa5, gpioa.pa6, gpioa.pa7,
            gpioa.pa8, gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5,
        );

        let mut keypad = KeyPad::new(keymatrix);

        let mut backlight_control = gpioe.pe0.into_push_pull_output();

        let mut backlight_state = true;
        backlight_control.set_high().unwrap();

        let mut lcd_power = gpioc.pc8.into_push_pull_output();

        lcd_power.set_high().unwrap();

        let mut lcd_chip_select = gpiod.pd7.into_push_pull_output();

        lcd_chip_select.set_low().unwrap();

        let mut lcd_read_enable = gpiod.pd4.into_push_pull_output();

        lcd_read_enable.set_high().unwrap();

        let mut lcd_extd_command = gpiod.pd6.into_push_pull_output();

        lcd_extd_command.set_low().unwrap();

        let mut lcd_tearing_effect = gpiob.pb11.into_push_pull_output();

        lcd_tearing_effect.set_low().unwrap();

        let mut led = Led::new(
            gpiob.pb4.into_push_pull_output(),
            gpiob.pb5.into_push_pull_output(),
            gpiob.pb0.into_push_pull_output(),
        );

        led.blue();

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

        let lcd_reset = gpioe.pe1.into_push_pull_output();

        let display_width = 320i32;
        let display_height = 240i32;

        let mut display = ST7789::new(
            lcd_interface,
            lcd_reset,
            display_width as u16,
            display_height as u16,
        );

        display.init(&mut delay).unwrap();

        display
            .set_orientation(Orientation::LandscapeSwapped)
            .unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(Rgb565::GREEN)
            .background_color(Rgb565::BLACK)
            .height_mode(FitToText)
            .build();

        let bounds = Rectangle::new(Point::new(4, 4), Point::new(display_width, 4));

        let text_box =
            TextBox::new("Hello from Rust on Numworks!", bounds).into_styled(textbox_style);

        text_box.draw(&mut display).unwrap();

        led.green();

        let text_box_tl = Point::new(4, 12);
        let text_box_tr = Point::new(display_width - 4, 12);

        let bounds = Rectangle::new(text_box_tl, text_box_tr);

        let mut last_pressed: Vec<Key, 46> = Vec::new();

        let mut string: String<52> = String::new();

        loop {
            let keys = keypad.read(&mut delay);
            if keys != last_pressed {
                if !keys.is_empty() {
                    if keys.contains(&Key::Power) {
                        if backlight_state {
                            backlight_control.set_low().unwrap();
                            led.off();
                            backlight_state = false;
                        } else {
                            backlight_control.set_high().unwrap();
                            led.green();
                            backlight_state = true;
                        }
                    }
                    for key in keys.iter() {
                        let key_char = char::from(*key);
                        if key_char != '\0' {
                            if string.len() >= 52 {
                                string.clear();
                            }
                            string.push(key_char).unwrap();
                        }
                    }
                    let mut pressed_string: String<184> = String::new();
                    write!(&mut pressed_string, "{:?}", keys).unwrap();
                    rprintln!("{}", pressed_string);
                    let mut tmp_string: String<52> = String::new();
                    write!(&mut tmp_string, "{}", string).unwrap();
                    for _ in 0..(52 - tmp_string.len()) {
                        tmp_string.push(' ').unwrap();
                    }
                    let text_box = TextBox::new(&tmp_string, bounds).into_styled(textbox_style);
                    text_box.draw(&mut display).unwrap();
                }
                last_pressed = keys;
            }
        }
    }
};
