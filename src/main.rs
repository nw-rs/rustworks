#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

extern crate alloc;

use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use stm32f7xx_hal::rcc::{HSEClock, HSEClockMode};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

use rtt_target::{rprintln, rtt_init_print};

use core::panic::PanicInfo;

use rtic::app;

use stm32f7xx_hal::fmc_lcd::{AccessMode, ChipSelect1, FmcLcd, LcdPins, Timing};
use stm32f7xx_hal::{delay::Delay, gpio::GpioExt, pac, prelude::*};

use embedded_graphics::fonts::Font6x8;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_text::prelude::*;

use st7789::{Orientation, ST7789};

use alloc::string::String;

mod keypad;
mod led;

use keypad::{Key, KeyMatrix, KeyPad};
use led::Led;

const HCLK: u32 = 216_000_000;
const HEAP: usize = 32768;

#[app(device = stm32f7xx_hal::pac, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        rtt_init_print!();
        let start = cortex_m_rt::heap_start() as usize;
        unsafe { ALLOCATOR.init(start, HEAP) }

        let mut cp: cortex_m::Peripherals = cx.core;

        let dp: pac::Peripherals = cx.device;

        init_mpu(&mut cp.MPU);

        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .hse(HSEClock::new(8.mhz(), HSEClockMode::Oscillator))
            .use_pll()
            .sysclk(HCLK.hz())
            .freeze();
        let mut delay = Delay::new(cp.SYST, clocks);

        delay.delay_ms(100_u8);

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

        let mut led = Led::new(
            gpiob.pb4.into_push_pull_output(),
            gpiob.pb5.into_push_pull_output(),
            gpiob.pb0.into_push_pull_output(),
        );

        led.blue();

        let mut backlight_control = gpioe.pe0.into_push_pull_output();

        let mut backlight_state = true;
        backlight_control.set_high().unwrap();

        let mut lcd_power = gpioc.pc8.into_push_pull_output();

        lcd_power.set_high().unwrap();

        let mut lcd_extd_command = gpiod.pd6.into_push_pull_output();

        lcd_extd_command.set_high().unwrap();

        let _lcd_tearing_effect = gpiob.pb11.into_floating_input();

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

        rprintln!("timings start");

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

        rprintln!(
            "tedge: {}, twc: {}, trcfm: {}, twrl: {}, trdlfm: {}, trdatast: {}",
            tedge,
            twc,
            trcfm,
            twrl,
            trdlfm,
            trdatast
        );
        rprintln!("read: {:?}", read_timing);
        rprintln!("write: {:?}", write_timing);

        let (_fmc, lcd) = FmcLcd::new(dp.FMC, HCLK.hz(), lcd_pins, &read_timing, &write_timing);

        /*let mut lcd_chip_select = gpiod.pd7.into_push_pull_output();

        lcd_chip_select.set_low().unwrap();

        let mut lcd_read_enable = gpiod.pd4.into_push_pull_output();

        lcd_read_enable.set_high().unwrap();

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

        let lcd = display_interface_parallel_gpio::PGPIO16BitInterface::new(
            lcd_bus,
            gpiod.pd11.into_push_pull_output(),
            gpiod.pd5.into_push_pull_output(),
        );*/

        let mut lcd_reset = gpioe.pe1.into_push_pull_output();

        lcd_reset.set_low().unwrap();
        delay.delay_ms(5u16);
        lcd_reset.set_high().unwrap();
        delay.delay_ms(10u16);
        lcd_reset.set_low().unwrap();
        delay.delay_ms(20u16);
        // Release from reset
        lcd_reset.set_high().unwrap();
        delay.delay_ms(10u16);

        let display_width = 320i32;
        let display_height = 240i32;

        rprintln!("display create");

        let mut display = ST7789::new(lcd, lcd_reset, display_width as u16, display_height as u16);

        rprintln!("display init");

        display.init(&mut delay).unwrap();

        display
            .set_orientation(Orientation::LandscapeSwapped)
            .unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(Rgb565::GREEN)
            .background_color(Rgb565::BLACK)
            .vertical_alignment(Scrolling)
            .build();

        let bounds = Rectangle::new(
            Point::new(3, 5),
            Point::new(display_width - 3, display_height - 5),
        );

        led.green();

        let mut last_pressed: heapless::Vec<Key, 46> = heapless::Vec::new();

        let mut string = String::with_capacity(2132);

        let mut off = false;

        loop {
            let keys = keypad.read(&mut delay);
            if keys != last_pressed {
                if !keys.is_empty() {
                    if keys.contains(&Key::Power) {
                        if backlight_state {
                            backlight_control.set_low().unwrap();
                            led.off();
                            display.clear(Rgb565::BLACK).unwrap();
                            off = true;
                            backlight_state = false;
                        } else {
                            backlight_control.set_high().unwrap();
                            led.green();
                            off = false;
                            backlight_state = true;
                        }
                    }

                    if !off {
                        let shift = keys.contains(&Key::Shift);
                        for key in keys.iter() {
                            let mut key_char = char::from(*key);
                            if key_char != '\0' {
                                if shift {
                                    key_char = key_char.to_ascii_uppercase();
                                }
                                let lines = string.lines().count();
                                rprintln!("l: {}, -40: {}", lines, lines - 40);
                                if lines > 40 {
                                    string = string
                                        .lines()
                                        .skip(lines - 40)
                                        .collect::<alloc::vec::Vec<&str>>()
                                        .join("\n");
                                }
                                if string.len() > 2100 {
                                    string = string.chars().skip(string.len() - 2100).collect();
                                }
                                rprintln!("lines: {}, len: {}", lines, string.len());
                                string.push(key_char);
                            } else if key == &Key::Delete {
                                string.pop();
                            } else if key == &Key::Clear {
                                string.clear();
                            }
                        }
                        rprintln!("{:?}", keys);
                        display.clear(Rgb565::BLACK).unwrap();
                        let text_box = TextBox::new(&string, bounds).into_styled(textbox_style);
                        text_box.draw(&mut display).unwrap();
                    }
                }
                last_pressed = keys;
            }
        }
    }
};

#[inline(never)]
#[alloc_error_handler]
fn oom(layout: Layout) -> ! {
    panic!("OOM: {:?}", layout);
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    cortex_m::peripheral::SCB::sys_reset();
}

fn init_mpu(mpu: &mut cortex_m::peripheral::MPU) {
    unsafe {
        const FULL_ACCESS: u32 = 0b011 << 24;
        const SIZE_512MB: u32 = (29 - 1) << 1;
        const DEVICE_SHARED: u32 = 0b000001 << 16;
        const NORMAL_SHARED: u32 = 0b000110 << 16;

        // Flash
        mpu.rnr.write(0);
        mpu.rbar.write(0x0000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | 1);

        // SRAM
        mpu.rnr.write(1);
        mpu.rbar.write(0x2000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | NORMAL_SHARED | 1);

        // Peripherals
        mpu.rnr.write(2);
        mpu.rbar.write(0x4000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // FSMC
        mpu.rnr.write(3);
        mpu.rbar.write(0x6000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // FSMC
        mpu.rnr.write(4);
        mpu.rbar.write(0xA000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // Core peripherals
        mpu.rnr.write(5);
        mpu.rbar.write(0xE000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | 1);

        // Enable MPU
        mpu.ctrl.write(1);
    }
}
