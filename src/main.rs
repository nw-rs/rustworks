#![no_std]
#![no_main]
use cortex_m::asm;
use cortex_m_rt::entry;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::style::*;
use st7789::{Orientation, ST7789};
#[allow(unused_imports)]
use stm32f7::stm32f730::{self, interrupt, Interrupt, NVIC};
use stm32f7xx_hal::pac::Peripherals;
use stm32f7xx_hal::{
    delay::Delay,
    rcc::{Rcc},
    time::MegaHertz,
};
use stm32f7xx_hal::{
    gpio::{GpioExt},
    pac::CorePeripherals,
    prelude::OutputPin,
    rcc::{HSEClock, HSEClockMode, RccExt},
    spi::{self, Spi},
};

extern crate panic_halt;

#[entry]
fn main() -> ! {
    let core = CorePeripherals::take().unwrap();
    let peripherals = Peripherals::take().unwrap();
    let _cp = cortex_m::Peripherals::take().unwrap();
    let mut rcc: Rcc = peripherals.RCC.constrain();

    let clocks = rcc
        .cfgr
        .hse(HSEClock::new(MegaHertz(25), HSEClockMode::Bypass))
        .sysclk(MegaHertz(216))
        .hclk(MegaHertz(216))
        .freeze();
    let mut delay = Delay::new(core.SYST, clocks);

    let gpioa = peripherals.GPIOA.split();
    let gpiob = peripherals.GPIOB.split();
    let gpioc = peripherals.GPIOC.split();
    let gpiod = peripherals.GPIOD.split();
    let gpioe = peripherals.GPIOE.split();

    let mut backlight = gpioe.pe0.into_push_pull_output();
    backlight.set_low();

    let rst = gpioe.pe1.into_push_pull_output();

    let mut spi_ncs = gpiob.pb6.into_push_pull_output();

    let sck = gpioc.pc10.into_alternate_af6();
    let miso = gpioc.pc11.into_alternate_af6();
    let mosi = gpioc.pc12.into_alternate_af6();
    let dc = gpioc.pc15.into_push_pull_output();

    spi_ncs.set_high().unwrap();

    let spi = Spi::new(peripherals.SPI3, (sck, miso, mosi)).enable::<u8>(
        &mut rcc.apb1,
        spi::ClockDivider::DIV32,
        spi::Mode {
            polarity: spi::Polarity::IdleHigh,
            phase: spi::Phase::CaptureOnSecondTransition,
        },
    );

    // display interface abstraction from SPI and DC
    let di = SPIInterfaceNoCS::new(spi, dc);

    // create driver
    let mut display = ST7789::new(di, rst, 240, 240);

    // initialize
    display.init(&mut delay).unwrap();
    // set default orientation
    display.set_orientation(Orientation::Landscape).unwrap();

    let circle1 =
        Circle::new(Point::new(128, 64), 64).into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    let circle2 = Circle::new(Point::new(64, 64), 64)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1));

    let mut blue_with_red_outline = PrimitiveStyle::new();
    blue_with_red_outline.fill_color = Some(Rgb565::BLUE);
    blue_with_red_outline.stroke_color = Some(Rgb565::RED);
    blue_with_red_outline.stroke_width = 1;
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
        asm::nop()
    }
}
