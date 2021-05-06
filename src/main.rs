#![no_std]
#![no_main]

extern crate panic_halt;

use rtic::app;

use stm32f7xx_hal::fmc_lcd::{ChipSelect1, FsmcLcd, Lcd, LcdPins, SubBank, Timing};
use stm32f7xx_hal::{delay::Delay, flash::Flash, gpio::GpioExt, pac, prelude::*};

use core::convert::{Infallible, TryInto};
use core::iter;
use core::iter::{Cloned, Cycle};
use core::ops::RangeInclusive;
use core::slice::Iter;

use embedded_graphics::drawable::Pixel;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Circle;
use embedded_graphics::style::PrimitiveStyle;

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
        let clocks = rcc.cfgr.freeze();
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

        // Reset LCD controller
        lcd_reset.set_low().unwrap();
        delay.delay_ms(5u16);
        lcd_reset.set_high().unwrap();
        delay.delay_ms(10u16);
        lcd_reset.set_low().unwrap();
        delay.delay_ms(20u16);
        // Release from reset
        lcd_reset.set_high().unwrap();
        delay.delay_ms(10u16);

        // Add LCD controller driver
        let mut lcd = St7789::new(lcd, 240, 240);
        /*let mut id = [0u8; 3];
        lcd.read(0x04, &mut id);
        if id != [0x85, 0x85, 0x52] {
            panic!(
                "Unexpected LCD controller ID: {:#x} {:#x} {:#x}",
                id[0], id[1], id[2]
            );
        }*/

        backlight_control.send_pulses(10, &mut delay);

        // LCD controller setup
        configure_lcd(&mut lcd, &mut delay);

        // Clear
        lcd.clear(Rgb565::BLACK).unwrap();

        // Turn on display
        lcd.write(0x29, &[]);

        // Draw some circles
        let test_colors = [
            Rgb565::new(0x4e >> 3, 0x79 >> 2, 0xa7 >> 3),
            Rgb565::new(0xf2 >> 3, 0x8e >> 2, 0x2b >> 3),
            Rgb565::new(0xe1 >> 3, 0x57 >> 2, 0x59 >> 3),
            Rgb565::new(0x76 >> 3, 0xb7 >> 2, 0xb2 >> 3),
            Rgb565::new(0x59 >> 3, 0xa1 >> 2, 0x4f >> 3),
            Rgb565::new(0xed >> 3, 0xc9 >> 2, 0x48 >> 3),
            Rgb565::new(0xb0 >> 3, 0x7a >> 2, 0xa1 >> 3),
            Rgb565::new(0xff >> 3, 0x9d >> 2, 0xa7 >> 3),
            Rgb565::new(0x9c >> 3, 0x75 >> 2, 0x5f >> 3),
            Rgb565::new(0xba >> 3, 0xb0 >> 2, 0xac >> 3),
        ];
        let center_points = [
            Point::new(70, 70),
            Point::new(170, 70),
            Point::new(170, 170),
            Point::new(70, 170),
        ];
        let mut drawer = ColoredCircleDrawer::new(&center_points, &test_colors);

        loop {
            drawer.draw(&mut lcd).unwrap();
            delay.delay_ms(100u16);
        }
    }
};

fn configure_lcd<S>(lcd: &mut St7789<S>, delay: &mut Delay)
where
    S: SubBank,
{
    // Initialize LCD controller
    // Sleep in
    lcd.write(0x10, &[]);
    delay.delay_ms(10u16);
    // Software reset
    lcd.write(0x1, &[]);
    delay.delay_ms(200u16);
    // Sleep out
    lcd.write(0x11, &[]);
    delay.delay_ms(120u16);
    // Memory data access control:
    // Page address order top to bottom
    // Column address order left to right
    // Normal order
    // Refresh top to bottom
    // RGB, not BGR
    // Refresh left to right
    lcd.write(0x36, &[0x0]);
    // Color mode 16 bits/pixel
    lcd.write(0x3a, &[0x5]);
    // Display inversion on
    lcd.write(0x21, &[]);
    // Display resolution is 240x240 pixels
    // Column address range 0 through 239
    lcd.write(0x2a, &[0x0, 0x0, 0x0, 0xef]);
    // Row address range 0 through 239
    lcd.write(0x2b, &[0x0, 0x0, 0x0, 0xef]);
    // Porch control
    lcd.write(0xb2, &[0x0c, 0x0c, 0x00, 0x33, 0x33]);
    // Gate control
    lcd.write(0xb7, &[0x35]);
    // VCOM
    lcd.write(0xbb, &[0x1f]);
    // LCM control
    lcd.write(0xc0, &[0x2c]);
    // VDV and VRH enable
    lcd.write(0xc2, &[0x01, 0xc3]);
    // VDV set
    lcd.write(0xc4, &[0x20]);
    // Normal mode frame rate control
    lcd.write(0xc6, &[0x0f]);
    // Power control
    lcd.write(0xd0, &[0xa4, 0xa1]);
    // Positive gamma
    lcd.write(
        0xe0,
        &[
            0xd0, 0x08, 0x11, 0x08, 0x0c, 0x15, 0x39, 0x33, 0x50, 0x36, 0x13, 0x14, 0x29, 0x2d,
        ],
    );
    // Negative gamma
    lcd.write(
        0xe0,
        &[
            0xd0, 0x08, 0x10, 0x08, 0x06, 0x06, 0x39, 0x44, 0x51, 0x0b, 0x16, 0x14, 0x2f, 0x31,
        ],
    );
}

/// Draws colored circles of various locations and colors
struct ColoredCircleDrawer<'a> {
    /// Infinite iterator over circle center points
    centers: Cloned<Cycle<Iter<'a, Point>>>,
    /// Infinite iterator over Rgb565 colors
    colors: Cloned<Cycle<Iter<'a, Rgb565>>>,
}

impl<'a> ColoredCircleDrawer<'a> {
    pub fn new(centers: &'a [Point], colors: &'a [Rgb565]) -> Self {
        ColoredCircleDrawer {
            centers: centers.iter().cycle().cloned(),
            colors: colors.iter().cycle().cloned(),
        }
    }

    /// Draws one circle onto a target
    pub fn draw<T>(&mut self, target: &mut T) -> Result<(), T::Error>
    where
        T: DrawTarget<Rgb565>,
    {
        let center = self.centers.next().unwrap();
        let color = self.colors.next().unwrap();

        Circle::new(center, 50)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(target)
    }
}

/// A simple driver for ST7789-series LCD controllers
struct St7789<S> {
    inner: Lcd<S>,
    width: u16,
    height: u16,
}

impl<S> St7789<S>
where
    S: SubBank,
{
    /// Creates a driver object, but does not perform any initialization
    pub fn new(inner: Lcd<S>, width: u16, height: u16) -> Self {
        St7789 {
            inner,
            width,
            height,
        }
    }

    pub fn write(&mut self, command: u16, arguments: &[u8]) {
        // Write the command code
        self.inner.write_command(command);
        // Set data/command high to write parameters
        for &argument in arguments {
            // Extend argument to 16 bits (the 8 higher bits are ignored)
            let argument: u16 = argument.into();
            self.inner.write_data(argument);
        }
    }

    pub fn read(&mut self, parameter: u16, buffer: &mut [u8]) {
        // Write the parameter to read (as a command)
        self.inner.write_command(parameter);
        // Dummy read
        let _ = self.inner.read_data();
        // Read results
        for result in buffer {
            // Read as 16 bits
            let result_16: u16 = self.inner.read_data();
            // Truncate to 8 bits
            *result = result_16 as u8;
        }
    }

    fn write_frame_memory<D>(&mut self, data: D)
    where
        D: IntoIterator<Item = u16>,
    {
        let ramwr_command = 0x2c;
        self.inner.write_command(ramwr_command);
        // Set data/command high to write data
        for argument in data.into_iter() {
            self.inner.write_data(argument);
        }
    }

    /// Sets the ranges of rows and columns to be written by subsequent memory write operations
    pub fn set_pixel_ranges(&mut self, columns: RangeInclusive<u16>, rows: RangeInclusive<u16>) {
        // CASET
        self.write(0x2a, &range_to_args(columns));
        // RASET
        self.write(0x2b, &range_to_args(rows));
    }
}

/// Converts a range of u16s into 4 bytes of arguments in the form expected by the RASET and
/// CASET commands
fn range_to_args(range: RangeInclusive<u16>) -> [u8; 4] {
    let (min, max) = range.into_inner();
    // Min high byte, min low byte, max high byte, max low byte
    [(min >> 8) as u8, min as u8, (max >> 8) as u8, max as u8]
}

// embedded-graphics compatibility
impl<S> DrawTarget<Rgb565> for St7789<S>
where
    S: SubBank,
{
    type Error = Infallible;

    fn draw_pixel(&mut self, Pixel(point, color): Pixel<Rgb565>) -> Result<(), Self::Error> {
        let x: u16 = point.x.try_into().expect("Pixel X too large");
        let y: u16 = point.y.try_into().expect("Pixel Y too large");
        self.set_pixel_ranges(x..=x, y..=y);
        self.write_frame_memory(iter::once(color.into_storage()));
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(u32::from(self.width), u32::from(self.height))
    }

    fn clear(&mut self, color: Rgb565) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        self.set_pixel_ranges(0..=(self.width - 1), 0..=(self.height - 1));
        // Cover the whole display in width * height pixels of the same color
        let total_pixels = usize::from(self.width) * usize::from(self.height);
        self.write_frame_memory(iter::repeat(color.into_storage()).take(total_pixels));
        Ok(())
    }
}

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
