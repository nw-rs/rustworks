use core::convert::TryInto;

use alloc::vec::Vec;
use cortex_m::asm;
use stm32f7xx_hal::delay::Delay;
use stm32f7xx_hal::gpio::gpiob::{PB2, PB6};
use stm32f7xx_hal::gpio::gpioc::PC9;
use stm32f7xx_hal::gpio::gpiod::{PD12, PD13};
use stm32f7xx_hal::gpio::gpioe::PE2;
use stm32f7xx_hal::gpio::{Alternate, AF10, AF9};
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::{pac::QUADSPI, pac::RCC};

// 2^23 = 8MB
const FLASH_ADDRESS_SIZE: u8 = 23;
const ADDRESS_WIDTH: u8 = 3;
const FLASH_SIZE: u32 = 8388608;
const DEFAULT_WIDTH: u8 = QspiWidth::QUAD;

struct QspiWidth;

#[allow(dead_code)]
impl QspiWidth {
    pub const NONE: u8 = 0b00;
    pub const SINGLE: u8 = 0b01;
    pub const DUAL: u8 = 0b10;
    pub const QUAD: u8 = 0b11;
}

struct QspiMode;

#[allow(dead_code)]
impl QspiMode {
    pub const INDIRECT_WRITE: u8 = 0b00;
    pub const INDIRECT_READ: u8 = 0b01;
    pub const AUTO_POLLING: u8 = 0b10;
    pub const MEMORY_MAPPED: u8 = 0b11;
}

struct QspiSize;

#[allow(dead_code)]
impl QspiSize {
    pub const ONE_BYTE: u8 = 0b00;
    pub const TWO_BYTES: u8 = 0b01;
    pub const THREE_BYTES: u8 = 0b10;
    pub const FOUR_BYTES: u8 = 0b11;
}

#[repr(u8)]
enum Command {
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
        unsafe {
            // Single flash mode with a QSPI clock prescaler of 2 (216 / 2 = 108 MHz), FIFO
            // threshold only matters for DMA and is set to 4 to allow word sized DMA requests
            qspi.cr
                .write_with_zero(|w| w.prescaler().bits(1).fthres().bits(3).en().set_bit());

            // Set the device size
            qspi.dcr.write(|w| w.fsize().bits(FLASH_ADDRESS_SIZE - 1));
            // Set chip select high time
            qspi.dcr.write(|w| w.csht().bits(2));
            qspi.dcr.write(|w| w.ckmode().set_bit());
        }

        Self {
            qspi,
            width: QspiWidth::SINGLE,
        }
    }

    pub fn init(&mut self, delay: &mut Delay) {
        self.send_command(Command::ReleaseDeepPowerDown, self.width);
        delay.delay_us(3_u32);
        if self.width == QspiWidth::SINGLE && DEFAULT_WIDTH == QspiWidth::QUAD {
            self.send_command(Command::WriteEnable, self.width);
            let mut status_two = [2u32];
            self.wait();
            self.send_write_command(
                Command::WriteStatusRegister2,
                FLASH_SIZE,
                &mut status_two,
                self.width,
            );
            self.wait();
            self.send_command(Command::EnableQPI, self.width);
            self.width = QspiWidth::QUAD;
        }
    }

    fn send_command_full(
        &mut self,
        mode: u8,
        width: u8,
        command: Command,
        address: u32,
        alt_bytes: u32,
        number_alt_bytes: u8,
        dummy_cycles: u8,
        data: Option<&mut [u32]>,
        data_length: u32,
    ) {
        assert!(mode < 4); // There are only 4 modes.
        assert!(width < 4); // There are only 4 valid widths
        if mode == QspiMode::MEMORY_MAPPED {
            let previous_mode = self.qspi.ccr.read().fmode().bits();
            if previous_mode == QspiMode::INDIRECT_WRITE || previous_mode == QspiMode::INDIRECT_READ
            {
                unsafe { self.qspi.ar.write(|w| w.bits(0)) }
                if previous_mode == QspiMode::INDIRECT_READ {
                    self.qspi.cr.write(|w| w.abort().set_bit());
                    while self.qspi.cr.read().abort().bit() {
                        asm::nop();
                    }
                }
            }
        } else if self.qspi.ccr.read().fmode() == QspiMode::MEMORY_MAPPED {
            self.qspi.cr.write(|w| w.abort().set_bit());
            while self.qspi.cr.read().abort().bit() {
                asm::nop();
            }
        }
        assert!(
            self.qspi.ccr.read().fmode() != QspiMode::MEMORY_MAPPED
                || self.qspi.sr.read().busy().bit()
        );
        unsafe {
            self.qspi.ccr.write(|w| w.fmode().bits(mode));
            if data.is_some() || mode == QspiMode::MEMORY_MAPPED {
                self.qspi.ccr.write(|w| w.dmode().bits(width));
            }
            if mode != QspiMode::MEMORY_MAPPED {
                self.qspi
                    .dlr
                    .write(|w| w.bits(if data_length > 0 { data_length - 1 } else { 0 }));
            }
            self.qspi.ccr.write(|w| w.dcyc().bits(dummy_cycles));
            if number_alt_bytes > 0 {
                self.qspi
                    .ccr
                    .write(|w| w.abmode().bits(width).absize().bits(number_alt_bytes - 1));
                self.qspi.abr.write(|w| w.bits(alt_bytes));
            }
            if address != FLASH_SIZE || mode == QspiMode::MEMORY_MAPPED {
                self.qspi
                    .ccr
                    .write(|w| w.admode().bits(width).adsize().bits(QspiSize::THREE_BYTES));
            }
            self.qspi
                .ccr
                .write(|w| w.imode().bits(width).instruction().bits(command as u8));
            if mode == QspiMode::MEMORY_MAPPED {
                self.qspi.ccr.write(|w| w.sioo().set_bit());
            }
            if address != FLASH_SIZE {
                self.qspi.ar.write(|w| w.bits(address));
            }
            if let Some(data) = data {
                if mode == QspiMode::INDIRECT_WRITE {
                    for num in data.iter() {
                        self.qspi.dr.write(|w| w.bits(*num));
                    }
                } else if mode == QspiMode::INDIRECT_READ {
                    for i in 0..(data_length as usize) {
                        data[i] = self.qspi.dr.read().bits();
                    }
                }
            }
            if mode != QspiMode::MEMORY_MAPPED {
                while self.qspi.sr.read().busy().bit() {
                    asm::nop();
                }
            }
        }
    }

    fn send_command(&mut self, command: Command, width: u8) {
        self.send_command_full(
            QspiMode::INDIRECT_WRITE,
            width,
            command,
            FLASH_SIZE,
            0,
            0,
            0,
            None,
            0,
        )
    }

    fn send_write_command(&mut self, command: Command, address: u32, data: &mut [u32], width: u8) {
        let data_length = data.len() as u32;
        self.send_command_full(
            QspiMode::INDIRECT_WRITE,
            width,
            command,
            address,
            0,
            0,
            0,
            Some(data),
            data_length,
        )
    }

    fn send_read_command(&mut self, command: Command, address: u32, data: &mut [u32], width: u8) {
        let data_length = data.len() as u32;
        self.send_command_full(
            QspiMode::INDIRECT_READ,
            width,
            command,
            address,
            0,
            0,
            0,
            Some(data),
            data_length,
        )
    }

    fn wait(&mut self) {
        let mut status_one = [0u32];
        loop {
            self.send_read_command(
                Command::ReadStatusRegister1,
                FLASH_SIZE,
                &mut status_one,
                self.width,
            );
            if status_one[0] & 1 == 1 {
                break;
            }
        }
    }
}
