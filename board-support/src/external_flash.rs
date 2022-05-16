#![allow(dead_code)]

use core::marker::PhantomData;

use cortex_m::asm;
use stm32f7xx_hal::gpio::gpiob::{PB2, PB6};
use stm32f7xx_hal::gpio::gpioc::PC9;
use stm32f7xx_hal::gpio::gpiod::{PD12, PD13};
use stm32f7xx_hal::gpio::gpioe::PE2;
use stm32f7xx_hal::gpio::Alternate;
use stm32f7xx_hal::{pac::QUADSPI, pac::RCC};

pub const FLASH_START: u32 = 0x90000000;
pub const FLASH_END: u32 = 0x90800000;

// 2^23 = 8MB
const FLASH_ADDRESS_SIZE: u8 = 23;
const ADDRESS_WIDTH: u8 = 3;
const FLASH_SIZE: u32 = 0x800000;

const N_4K_SECTORS: u8 = 8;
const N_32K_SECTORS: u8 = 1;
const N_64K_SECTORS: u8 = 127;
const N_SECTORS: u8 = N_4K_SECTORS + N_32K_SECTORS + N_64K_SECTORS;
const ADDRESS_BITS_4K: u8 = 12;
const ADDRESS_BITS_32K: u8 = 15;
const ADDRESS_BITS_64K: u8 = 16;
const PAGE_SIZE: usize = 256;

//#[allow(dead_code)]
#[repr(u8)]
enum QspiWidth {
    None = 0b00,
    Single = 0b01,
    Dual = 0b10,
    Quad = 0b11,
}

/// The different QSPI functional modes.
#[repr(u8)]
enum QspiMode {
    IndirectWrite = 0b00,
    IndirectRead = 0b01,
    AutoPolling = 0b10,
    MemoryMapped = 0b11,
}

/// The number of bytes required to specify addresses on the chip.
#[repr(u8)]
enum QspiSize {
    OneByte = 0b00,
    TwoBytes = 0b01,
    ThreeBytes = 0b10,
    FourBytes = 0b11,
}

/// Commands (instructions) that can be sent to the flash chip.
#[repr(u8)]
pub enum Command {
    ReadStatusRegister1 = 0x05,
    ReadStatusRegister2 = 0x35,
    WriteStatusRegister = 0x01,
    WriteStatusRegister2 = 0x31,
    WriteEnable = 0x06,
    WriteEnableVolatile = 0x50,
    ReadData = 0x03,
    FastRead = 0x0B,
    FastReadQuadIO = 0xEB,
    PageProgram = 0x02,
    QuadPageProgram = 0x33,
    EnableQPI = 0x38,
    EnableReset = 0x66,
    Reset = 0x99,
    ChipErase = 0xC7,
    Erase4KbyteBlock = 0x20,
    Erase32KbyteBlock = 0x52,
    Erase64KbyteBlock = 0xD8,
    SetReadParameters = 0xC0,
    DeepPowerDown = 0xB9,
    ReleaseDeepPowerDown = 0xAB,
    ReadIds = 0x90,
    ReadJEDECID = 0x9F,
}

pub enum Uninitialized {}
pub enum Indirect {}
pub enum MemoryMapped {}

pub struct ExternalFlash<MODE> {
    qspi: QUADSPI,
    mode: PhantomData<MODE>,
}

impl ExternalFlash<Uninitialized> {
    pub fn new(
        rcc: &mut RCC,
        qspi: QUADSPI,
        _pins: (
            PB2<Alternate<9>>,
            PB6<Alternate<10>>,
            PC9<Alternate<9>>,
            PD12<Alternate<9>>,
            PD13<Alternate<9>>,
            PE2<Alternate<9>>,
        ),
    ) -> Self {
        rcc.ahb3enr.modify(|_, w| w.qspien().set_bit());
        // Single flash mode with a QSPI clock prescaler of 2 (216 / 2 = 108 MHz), FIFO
        // threshold only matters for DMA and is set to 4 to allow word sized DMA requests

        // Configure controller for flash chip.
        unsafe {
            qspi.dcr.write_with_zero(|w| {
                w.fsize()
                    .bits(FLASH_ADDRESS_SIZE - 1)
                    .csht()
                    .bits(2)
                    .ckmode()
                    .set_bit()
            });
            qspi.cr
                .write_with_zero(|w| w.prescaler().bits(3).en().set_bit());
        }

        Self {
            qspi,
            mode: PhantomData,
        }
    }

    /// Turns on the chip and tells it to switch to QPI mode.
    #[must_use]
    pub fn init(mut self) -> ExternalFlash<Indirect> {
        // Turn on the chip.
        self.send_spi_command(Command::ReleaseDeepPowerDown, None);

        // Enable writing to the chip so that the status register can be changed.
        self.send_spi_command(Command::WriteEnableVolatile, None);

        // Set QPI to enabled in the chip's status register.
        self.send_spi_command(Command::WriteStatusRegister2, Some(0x02));

        // Enable QPI on the chip.
        self.send_spi_command(Command::EnableQPI, None);

        let mut qpi = ExternalFlash {
            qspi: self.qspi,
            mode: PhantomData,
        };

        qpi.set_read_parameters();

        qpi
    }

    /// Sends a command with optional data in SPI mode.
    fn send_spi_command(&mut self, command: Command, data: Option<u8>) {
        self.qspi.dlr.reset();

        if let Some(data) = data {
            self.qspi.abr.write(|w| unsafe { w.bits(u32::from(data)) });
        }

        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .instruction()
                .bits(command as u8);

            if data.is_some() {
                w.abmode()
                    .bits(QspiWidth::Single as u8)
                    .absize()
                    .bits(QspiSize::OneByte as u8);
            }

            w
        });

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }
    }
}

impl ExternalFlash<Indirect> {
    /// Reads the manufacturer and device IDs.
    ///
    /// The first value is the manufacturer ID and the second one it the device ID.
    pub fn read_ids(&mut self) -> (u8, u8) {
        self.qspi.dlr.write(|w| unsafe { w.dl().bits(2 - 1) });
        self.qspi.ar.reset();

        // The STM32 doesn't seem to release the QSPI pins early enough, after the
        // address is transmitted. The short bus contention leads to invalid data on
        // the rising clock edge. Using a later sampling point fixes this problem.
        // TODO: can the bus contention be eliminated entirely?
        self.qspi.cr.modify(|_, w| w.sshift().set_bit());

        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectRead as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .instruction()
                .bits(Command::ReadIds as u8)
        });

        self.qspi.ar.reset();

        let data = self.qspi.dr.read().bits();

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        self.qspi.cr.modify(|_, w| w.sshift().clear_bit());

        (data as u8, (data >> 8) as u8)
    }

    /// Reads status register 1.
    pub fn read_status_register1(&mut self) -> u8 {
        self.read_status_register(Command::ReadStatusRegister1)
    }

    /// Reads status register 2.
    pub fn read_status_register2(&mut self) -> u8 {
        self.read_status_register(Command::ReadStatusRegister2)
    }

    fn read_status_register(&mut self, command: Command) -> u8 {
        self.qspi.dlr.write(|w| unsafe { w.dl().bits(1 - 1) });

        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectRead as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .instruction()
                .bits(command as u8)
        });

        let data = self.qspi.dr.read().bits();

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        data as u8
    }

    /// Reads a byte.
    pub fn read_byte(&mut self, address: u32) -> u8 {
        self.qspi.dlr.write(|w| unsafe { w.dl().bits(1 - 1) });

        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectRead as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .dcyc()
                .bits(6)
                .instruction()
                .bits(Command::FastRead as u8)
        });

        self.qspi.ar.write(|w| unsafe { w.bits(address) });

        let data = self.qspi.dr.read().bits();

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        data as u8
    }

    /// Programs a page.
    pub fn program_page(&mut self, address: u32, data: &[u8]) {
        assert!(!data.is_empty());

        self.qspi
            .dlr
            .write(|w| unsafe { w.dl().bits(data.len() as u32 - 1) });

        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .instruction()
                .bits(Command::PageProgram as u8)
        });

        self.qspi.ar.write(|w| unsafe { w.bits(address) });

        for byte in data {
            // while self.qspi.sr.read().ftf().bit_is_clear() {
            //     asm::nop();
            // }
            unsafe {
                core::ptr::write_volatile(&self.qspi.dr as *const _ as *mut u8, *byte);
            }
        }

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        self.wait_busy();
    }

    /// Enables writing.
    pub fn write_enable(&mut self) {
        self.command(Command::WriteEnable);
    }

    /// Disables writing.
    pub fn write_disable(&mut self) {
        self.command(Command::WriteEnable);
    }

    /// Erases the chip.
    pub fn chip_erase(&mut self) {
        self.command(Command::ChipErase);
        self.wait_busy();
    }

    pub fn block_erase_4k(&mut self, address: u32) {
        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .instruction()
                .bits(Command::Erase4KbyteBlock as u8)
        });

        self.qspi.ar.write(|w| unsafe { w.bits(address) });

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        self.wait_busy();
    }

    fn command(&mut self, command: Command) {
        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .instruction()
                .bits(command as u8)
        });

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }
    }

    /// Waits until the busy flag is cleared.
    fn wait_busy(&mut self) {
        while self.read_status_register1() & 0x01 != 0 {
            asm::nop();
        }
    }

    fn set_read_parameters(&mut self) {
        // 104Mhz -> 6 dummy clocks, 8-byte wrap
        self.qspi.abr.write(|w| unsafe { w.bits(0b0010_0000) });

        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .abmode()
                .bits(QspiWidth::Quad as u8)
                .absize()
                .bits(QspiSize::OneByte as u8)
                .instruction()
                .bits(Command::SetReadParameters as u8)
        });

        while self.qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }
    }

    pub fn into_memory_mapped(self) -> ExternalFlash<MemoryMapped> {
        self.qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::MemoryMapped as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .dcyc()
                .bits(6)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .imode()
                .bits(QspiWidth::Quad as u8)
                .instruction()
                .bits(Command::FastReadQuadIO as u8)
                .sioo()
                .set_bit()
        });

        ExternalFlash {
            qspi: self.qspi,
            mode: PhantomData,
        }
    }
}
