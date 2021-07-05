use cortex_m::asm;
use stm32f7xx_hal::delay::Delay;
use stm32f7xx_hal::gpio::gpiob::{PB2, PB6};
use stm32f7xx_hal::gpio::gpioc::PC9;
use stm32f7xx_hal::gpio::gpiod::{PD12, PD13};
use stm32f7xx_hal::gpio::gpioe::PE2;
use stm32f7xx_hal::gpio::{Alternate, AF10, AF9};
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::{pac::QUADSPI, pac::RCC};

use rtt_target::rprintln;

// 2^23 = 8MB
const FLASH_ADDRESS_SIZE: u8 = 23;
const ADDRESS_WIDTH: u8 = 3;
const FLASH_SIZE: u32 = 8388608;

const N_4K_SECTORS: u8 = 8;
const N_32K_SECTORS: u8 = 1;
const N_64K_SECTORS: u8 = 127;
const N_SECTORS: u8 = N_4K_SECTORS + N_32K_SECTORS + N_64K_SECTORS;
const ADDRESS_BITS_4K: u8 = 12;
const ADDRESS_BITS_32K: u8 = 15;
const ADDRESS_BITS_64K: u8 = 16;
const PAGE_SIZE: usize = 256;

struct QspiWidth;

#[allow(dead_code)]
impl QspiWidth {
    pub const NONE: u8 = 0b00;
    pub const SINGLE: u8 = 0b01;
    pub const DUAL: u8 = 0b10;
    pub const QUAD: u8 = 0b11;
}

/// The different QSPI functional modes.
struct QspiMode;

#[allow(dead_code)]
impl QspiMode {
    pub const INDIRECT_WRITE: u8 = 0b00;
    pub const INDIRECT_READ: u8 = 0b01;
    pub const AUTO_POLLING: u8 = 0b10;
    pub const MEMORY_MAPPED: u8 = 0b11;
}

/// The number of bytes required to specify addresses on the chip.
struct QspiSize;

#[allow(dead_code)]
impl QspiSize {
    pub const ONE_BYTE: u8 = 0b00;
    pub const TWO_BYTES: u8 = 0b01;
    pub const THREE_BYTES: u8 = 0b10;
    pub const FOUR_BYTES: u8 = 0b11;
}

/// Commands (instructions) that can be sent to the flash chip.
#[repr(u8)]
pub enum Command {
    ReadStatusRegister1 = 0x05,
    ReadStatusRegister2 = 0x35,
    WriteStatusRegister = 0x01,
    WriteStatusRegister2 = 0x31,
    WriteEnable = 0x06,
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
    ReadJEDECID = 0x9F,
}

pub struct ExternalFlash {
    qspi: QUADSPI,
    width: u8,
    initialised: bool,
}

impl ExternalFlash {
    pub fn new(
        rcc: &mut RCC,
        qspi: QUADSPI,
        _pins: (
            PB2<Alternate<AF9>>,
            PB6<Alternate<AF10>>,
            PC9<Alternate<AF9>>,
            PD12<Alternate<AF9>>,
            PD13<Alternate<AF9>>,
            PE2<Alternate<AF9>>,
        ),
    ) -> Self {
        rcc.ahb3enr.modify(|_, w| w.qspien().set_bit());
        // Single flash mode with a QSPI clock prescaler of 2 (216 / 2 = 108 MHz), FIFO
        // threshold only matters for DMA and is set to 4 to allow word sized DMA requests

        // Configure controller for flash chip.
        qspi.dcr.write_with_zero(|w| unsafe {
            w.fsize()
                .bits(FLASH_ADDRESS_SIZE - 1)
                .csht()
                .bits(2)
                .ckmode()
                .set_bit()
        });
        qspi.cr
            .write_with_zero(|w| unsafe { w.prescaler().bits(1).en().set_bit() });

        Self {
            qspi,
            // Chip initially expects commands in SPI mode.
            width: QspiWidth::SINGLE,
            initialised: false,
        }
    }

    /// Turns on the chip and tells it to switch to QPI mode.
    pub fn init(&mut self, delay: &mut Delay) {
        // The chip should not be initialised twice.
        assert!(!self.initialised);
        // Turn on the chip.
        self.send_command(Command::ReleaseDeepPowerDown);
        delay.delay_us(3_u32);
        // Enable writing to the chip so that the status register can be changed.
        self.send_command(Command::WriteEnable);
        self.wait();
        // Set QPI to enabled in the chip's status register.
        self.send_write_command(Command::WriteStatusRegister2, FLASH_SIZE, &mut [0b00000010]);
        self.wait();
        // Enable QPI on the chip.
        self.send_command(Command::EnableQPI);
        // Now that QPI is enabled the default command width should be QUAD.
        self.width = QspiWidth::QUAD;
        // Configure number of dummy cycles for QPI read instructions.
        self.send_write_command(Command::SetReadParameters, FLASH_SIZE, &mut [0b0010000]);
        self.initialised = true;
        self.set_memory_mapped();
    }

    pub fn send_command_full(
        &mut self,
        mode: u8,
        command: Command,
        address: u32,
        alt_bytes: u32,
        number_alt_bytes: u8,
        dummy_cycles: u8,
        data: Option<&mut [u32]>,
        data_length: u32,
    ) {
        assert!(mode < 4); // There are only 4 modes.

        let fmode = self.qspi.ccr.read().fmode().bits();

        if mode == QspiMode::MEMORY_MAPPED {
            if fmode == QspiMode::INDIRECT_WRITE || fmode == QspiMode::INDIRECT_READ {
                unsafe { self.qspi.ar.write(|w| w.bits(0)) }
                if fmode == QspiMode::INDIRECT_READ {
                    self.qspi.cr.write(|w| w.abort().set_bit());
                    while self.qspi.cr.read().abort().bit() {
                        asm::nop();
                    }
                }
            }
        } else if fmode == QspiMode::MEMORY_MAPPED {
            self.qspi.cr.write(|w| w.abort().set_bit());
            while self.qspi.cr.read().abort().bit() {
                asm::nop();
            }
        }

        assert!(fmode != QspiMode::MEMORY_MAPPED || self.qspi.sr.read().busy().bit_is_clear());

        self.qspi.ccr.write_with_zero(|w| {
            unsafe {
                w.fmode().bits(mode);
            }
            if data.is_some() || mode == QspiMode::MEMORY_MAPPED {
                unsafe {
                    w.dmode().bits(self.width);
                }
            }
            if mode != QspiMode::MEMORY_MAPPED {
                self.qspi.dlr.write(|w| unsafe {
                    w.bits(if data_length > 0 { data_length - 1 } else { 0 })
                });
            }
            unsafe {
                w.dcyc().bits(dummy_cycles);
            }
            if number_alt_bytes > 0 {
                unsafe {
                    w.abmode()
                        .bits(self.width)
                        .absize()
                        .bits(number_alt_bytes - 1);
                }
                self.qspi.abr.write(|w| unsafe { w.bits(alt_bytes) });
            }

            if address != FLASH_SIZE || mode == QspiMode::MEMORY_MAPPED {
                unsafe {
                    w.admode()
                        .bits(self.width)
                        .adsize()
                        .bits(QspiSize::THREE_BYTES);
                }
            }
            unsafe {
                w.imode().bits(self.width).instruction().bits(command as u8);
            }
            if mode == QspiMode::MEMORY_MAPPED {
                w.sioo().set_bit();
            }
            w
        });

        if address != FLASH_SIZE {
            self.qspi.ar.write(|w| unsafe { w.bits(address) });
        }

        if let Some(data) = data {
            if mode == QspiMode::INDIRECT_WRITE {
                for num in data.iter() {
                    self.qspi.dr.write(|w| unsafe { w.bits(*num) });
                }
            } else if mode == QspiMode::INDIRECT_READ {
                for i in 0..(data_length as usize) {
                    data[i] = self.qspi.dr.read().bits();
                }
            }
        }

        // Wait for command to be sent unless in memory mapped mode because then busy does not fall
        // unless there is a timeout, an abort or the peripheral is disabled.
        if mode != QspiMode::MEMORY_MAPPED {
            while self.qspi.sr.read().busy().bit_is_set() {
                asm::nop();
            }
        }
    }

    pub fn send_command(&mut self, command: Command) {
        self.send_command_full(
            QspiMode::INDIRECT_WRITE,
            command,
            FLASH_SIZE,
            0,
            0,
            0,
            None,
            0,
        )
    }

    pub fn send_write_command(&mut self, command: Command, address: u32, data: &mut [u32]) {
        let data_length = data.len() as u32;
        self.send_command_full(
            QspiMode::INDIRECT_WRITE,
            command,
            address,
            0,
            0,
            0,
            Some(data),
            data_length,
        )
    }

    pub fn send_read_command(&mut self, command: Command, address: u32, data: &mut [u32]) {
        let data_length = data.len() as u32;
        self.send_command_full(
            QspiMode::INDIRECT_READ,
            command,
            address,
            0,
            0,
            0,
            Some(data),
            data_length,
        )
    }

    pub fn set_memory_mapped(&mut self) {
        self.send_command_full(
            QspiMode::MEMORY_MAPPED,
            Command::FastReadQuadIO,
            FLASH_SIZE,
            0xA0,
            1,
            4,
            None,
            0,
        );
    }

    pub fn unset_memory_mapped(&mut self) {
        self.send_command_full(
            QspiMode::INDIRECT_READ,
            Command::FastReadQuadIO,
            0,
            !(0xA0),
            1,
            4,
            Some(&mut [0u32]),
            1,
        );
    }

    pub fn wait(&mut self) {
        let mut status_one = [0u32];
        loop {
            self.send_read_command(Command::ReadStatusRegister1, FLASH_SIZE, &mut status_one);
            if status_one[0] & 1 != 1 {
                break;
            }
        }
    }

    pub fn unlock_flash(&mut self) {
        self.send_command(Command::WriteEnable);
        self.wait();
        let mut status_two = [0u32];
        self.send_read_command(Command::ReadStatusRegister2, FLASH_SIZE, &mut status_two);
        let mut registers = [0u32, 2 & status_two[0]];
        self.send_write_command(Command::WriteStatusRegister, FLASH_SIZE, &mut registers);
    }

    pub fn mass_erase(&mut self) {
        self.unset_memory_mapped();
        self.unlock_flash();
        self.send_command(Command::WriteEnable);
        self.wait();
        self.send_command(Command::ChipErase);
        self.wait();
        self.set_memory_mapped();
    }

    pub fn sector_at_address(address: u32) -> i32 {
        let mut i: i32 = address as i32 >> ADDRESS_BITS_64K;
        if i > 127 {
            return -1;
        }
        if i >= 1 {
            return (N_4K_SECTORS + N_32K_SECTORS) as i32 - 1 + i;
        }
        i = address as i32 >> ADDRESS_BITS_32K;
        if i >= 1 {
            i = N_4K_SECTORS as i32 - 1 + i;
            assert!(i >= 0 && i <= N_32K_SECTORS as i32);
            return i;
        }
        i = address as i32 >> ADDRESS_BITS_4K;
        assert!(i <= N_4K_SECTORS as i32);
        i
    }

    #[inline(never)]
    pub fn erase_sector(&mut self, i: i32) {
        assert!(i >= 0 && i < N_SECTORS as i32);
        self.unset_memory_mapped();
        self.unlock_flash();
        self.send_command(Command::WriteEnable);
        self.wait();
        if i < N_4K_SECTORS as i32 {
            self.send_write_command(
                Command::Erase4KbyteBlock,
                (i << ADDRESS_BITS_4K) as u32,
                &mut [],
            );
        } else if i < (N_4K_SECTORS + N_32K_SECTORS) as i32 {
            self.send_write_command(
                Command::Erase32KbyteBlock,
                ((i - N_4K_SECTORS as i32 + 1) << ADDRESS_BITS_32K) as u32,
                &mut [],
            );
        } else {
            self.send_write_command(
                Command::Erase64KbyteBlock,
                ((i - N_4K_SECTORS as i32 - N_32K_SECTORS as i32 + 1) << ADDRESS_BITS_64K) as u32,
                &mut [],
            );
        }
        self.wait();
        self.set_memory_mapped();
    }

    #[inline(never)]
    pub fn write_memory(&mut self, mut destination: u32, source: &mut [u32]) {
        self.unset_memory_mapped();
        let mut length_fits_page = PAGE_SIZE - (destination as usize & (PAGE_SIZE - 1));
        for slice in source.chunks_mut(length_fits_page) {
            self.send_command(Command::WriteEnable);
            self.wait();
            self.send_write_command(Command::QuadPageProgram, destination, slice);
            destination += length_fits_page as u32;
            length_fits_page = PAGE_SIZE;
            self.wait();
        }
        self.set_memory_mapped();
    }
}
