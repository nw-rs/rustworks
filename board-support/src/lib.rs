#![no_std]

use display::Display;
use external_flash::{ExternalFlash, Uninitialized};
use hal::{
    fmc_lcd::{ChipSelect1, LcdPins},
    gpio::{
        gpiob::{PB0, PB4, PB5},
        GpioExt, Output, PushPull, Speed,
    },
    pac,
    rcc::Clocks, otg_fs::UsbBus, flash::Flash, timer::SysTimerExt,
};
use keypad::{KeyMatrix, KeyPad};
use led::Led;

pub use stm32f7xx_hal as hal;
pub use clocks::init_clocks;

pub mod clocks;
pub mod display;
pub mod external_flash;
pub mod keypad;
pub mod led;

pub const HCLK: u32 = 192_000_000;

use hal::otg_fs::USB;
use usb_device::class_prelude::UsbBusAllocator;

pub fn get_internal_flash() -> Flash {
    let dp = unsafe { pac::Peripherals::steal() };

    Flash::new(dp.FLASH)
}

pub fn get_external_flash() -> ExternalFlash<Uninitialized> {
    let mut dp = unsafe { pac::Peripherals::steal() };

    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();
    let gpioe = dp.GPIOE.split();

    let qspi_pins = (
        gpiob.pb2.into_alternate().set_speed(Speed::VeryHigh),
        gpiob.pb6.into_alternate().set_speed(Speed::VeryHigh),
        gpioc.pc9.into_alternate().set_speed(Speed::VeryHigh),
        gpiod.pd12.into_alternate().set_speed(Speed::VeryHigh),
        gpiod.pd13.into_alternate().set_speed(Speed::VeryHigh),
        gpioe.pe2.into_alternate().set_speed(Speed::VeryHigh),
    );

    ExternalFlash::new(&mut dp.RCC, dp.QUADSPI, qspi_pins)
}

/// Init MPU before doing this.
pub fn get_display(clocks: &Clocks) -> Display {
    let dp = unsafe { pac::Peripherals::steal() };
    let cp = unsafe { cortex_m::Peripherals::steal() };

    let mut delay = cp.SYST.delay(clocks);

    let gpioc = dp.GPIOC.split();
    let gpiob = dp.GPIOB.split();
    let gpiod = dp.GPIOD.split();
    let gpioe = dp.GPIOE.split();

    let lcd_pins = LcdPins {
        data: (
            gpiod.pd14.into_alternate(),
            gpiod.pd15.into_alternate(),
            gpiod.pd0.into_alternate(),
            gpiod.pd1.into_alternate(),
            gpioe.pe7.into_alternate(),
            gpioe.pe8.into_alternate(),
            gpioe.pe9.into_alternate(),
            gpioe.pe10.into_alternate(),
            gpioe.pe11.into_alternate(),
            gpioe.pe12.into_alternate(),
            gpioe.pe13.into_alternate(),
            gpioe.pe14.into_alternate(),
            gpioe.pe15.into_alternate(),
            gpiod.pd8.into_alternate(),
            gpiod.pd9.into_alternate(),
            gpiod.pd10.into_alternate(),
        ),
        address: gpiod.pd11.into_alternate(),
        read_enable: gpiod.pd4.into_alternate(),
        write_enable: gpiod.pd5.into_alternate(),
        chip_select: ChipSelect1(gpiod.pd7.into_alternate()),
    };

    Display::new(
        lcd_pins,
        dp.FMC,
        gpioe.pe1.into_push_pull_output(),
        gpioc.pc8.into_push_pull_output(),
        gpioe.pe0.into_push_pull_output(),
        gpiob.pb11.into_floating_input(),
        gpiod.pd6.into_push_pull_output(),
        &mut delay,
        clocks,
    )
}

pub fn get_keypad() -> KeyPad {
    let dp = unsafe { pac::Peripherals::steal() };

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    let keymatrix = KeyMatrix::new(
        gpioa.pa0, gpioa.pa1, gpioa.pa2, gpioa.pa3, gpioa.pa4, gpioa.pa5, gpioa.pa6, gpioa.pa7,
        gpioa.pa8, gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5,
    );

    KeyPad::new(keymatrix)
}

pub fn get_led() -> Led<PB4<Output<PushPull>>, PB5<Output<PushPull>>, PB0<Output<PushPull>>> {
    let dp = unsafe { pac::Peripherals::steal() };

    let gpiob = dp.GPIOB.split();

    Led::new(
        gpiob.pb4.into_push_pull_output(),
        gpiob.pb5.into_push_pull_output(),
        gpiob.pb0.into_push_pull_output(),
    )
}

pub fn get_usb_bus_allocator(clocks: &Clocks, ep_memory: &'static mut [u32]) -> UsbBusAllocator<UsbBus<USB>> {
    let dp = unsafe { pac::Peripherals::steal() };

    let gpioa = dp.GPIOA.split();

    let usb = USB::new(
        dp.OTG_FS_GLOBAL,
        dp.OTG_FS_DEVICE,
        dp.OTG_FS_PWRCLK,
        (
            gpioa.pa11.into_alternate(),
            gpioa.pa12.into_alternate(),
        ),
        clocks,
    );

    UsbBus::new(usb, ep_memory)
}

pub fn init_mpu(mpu: &mut cortex_m::peripheral::MPU) {
    unsafe {
        const FULL_ACCESS: u32 = 0b011 << 24;
        const SIZE_512MB: u32 = 28 << 1;
        const SIZE_8MB: u32 = 22 << 1;
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

        // QSPI
        mpu.rnr.write(6);
        mpu.rbar.write(0x9000_0000);
        mpu.rasr.write(27 << 1 | 1 << 28 | 1);

        mpu.rnr.write(7);
        mpu.rbar.write(0x9000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_8MB | DEVICE_SHARED | 1);

        // Enable MPU
        mpu.ctrl.write(1);
    }
}
