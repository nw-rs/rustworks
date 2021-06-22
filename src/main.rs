#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

extern crate alloc;

use alloc::format;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

use rtt_target::{rprintln, rtt_init_print};

use core::panic::PanicInfo;

use rtic::app;

use stm32f7xx_hal::rcc::{HSEClock, HSEClockMode};
use stm32f7xx_hal::{
    delay::Delay,
    fmc_lcd::{ChipSelect1, LcdPins},
    gpio::GpioExt,
    pac,
    prelude::*,
};

mod display;
mod external_flash;
mod keypad;
mod led;

use keypad::{Key, KeyMatrix, KeyPad};
use led::Led;

use crate::display::Display;

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

        let mut dp: pac::Peripherals = cx.device;

        init_mpu(&mut cp.MPU);

        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();
        let gpiob = dp.GPIOB.split();
        let gpiod = dp.GPIOD.split();
        let gpioe = dp.GPIOE.split();

        let qspi_pins = (
            gpiob.pb2.into_alternate_af9(),
            gpiob.pb6.into_alternate_af10(),
            gpioc.pc9.into_alternate_af9(),
            gpiod.pd12.into_alternate_af9(),
            gpiod.pd13.into_alternate_af9(),
            gpioe.pe2.into_alternate_af9(),
        );

        let mut external_flash =
            external_flash::ExternalFlash::new(&mut dp.RCC, dp.QUADSPI, qspi_pins);

        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .hse(HSEClock::new(8.mhz(), HSEClockMode::Oscillator))
            .use_pll()
            .sysclk(HCLK.hz())
            .freeze();
        let mut delay = Delay::new(cp.SYST, clocks);

        delay.delay_ms(100_u8);

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

        let mut power_state = true;

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

        let mut display = Display::new(
            lcd_pins,
            dp.FMC,
            gpioe.pe1.into_push_pull_output(),
            gpioc.pc8.into_push_pull_output(),
            gpioe.pe0.into_push_pull_output(),
            gpiob.pb11.into_floating_input(),
            gpiod.pd6.into_push_pull_output(),
            &mut delay,
        );

        let mut last_pressed: heapless::Vec<Key, 46> = heapless::Vec::new();

        let mut off = false;

        led.green();

        let mut key_count = 0usize;

        loop {
            let keys = keypad.read(&mut delay);
            if keys != last_pressed {
                if !keys.is_empty() {
                    if keys.contains(&Key::Power) {
                        if power_state {
                            display.set_backlight(0);
                            led.off();
                            display.clear(display::BG_COLOUR);
                            off = true;
                            power_state = false;
                        } else {
                            display.set_backlight(1);
                            led.green();
                            off = false;
                            power_state = true;
                        }
                    }

                    if !off {
                        if keys.contains(&Key::EXE) {
                            //let result = rcas::parse_eval(&display.bottom);
                            display.write_bottom_to_top();
                            //if let Ok(num) = result {
                            display.write_top(&format!("\n{: >52}", key_count));
                            //}
                            display.draw_all();
                        } else {
                            let shift = keys.contains(&Key::Shift);
                            for key in keys.iter() {
                                let mut key_char = char::from(*key);
                                if key_char != '\0' {
                                    if shift {
                                        key_char = key_char.to_ascii_uppercase();
                                    }
                                    let mut tmp = [0u8; 4];
                                    if display.write_bottom(key_char.encode_utf8(&mut tmp), true) {
                                        key_count += 1;
                                    }
                                } else if key == &Key::Delete {
                                    display.pop_bottom(true);
                                } else if key == &Key::Clear {
                                    display.clear_bottom(true);
                                }
                            }
                        }
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
