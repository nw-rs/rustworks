#![allow(dead_code)]

use cortex_m::asm;
use stm32f7xx_hal::pac::{GPIOB, GPIOC, GPIOD, GPIOE};
use stm32f7xx_hal::{pac::QUADSPI, pac::RCC};

// 2^23 = 8MB
const FLASH_ADDRESS_SIZE: u8 = 23;
const ADDRESS_WIDTH: u8 = 3;
const FLASH_SIZE: u32 = 1 << 23; // 0x800000

pub const FLASH_START: u32 = 0x90000000;
pub const FLASH_END: u32 = FLASH_START + FLASH_SIZE;

const N_4K_SECTORS: u8 = 8;
const N_32K_SECTORS: u8 = 1;
const N_64K_SECTORS: u8 = 127;
const N_SECTORS: u8 = N_4K_SECTORS + N_32K_SECTORS + N_64K_SECTORS;
const ADDRESS_BITS_4K: u8 = 12;
const ADDRESS_BITS_32K: u8 = 15;
const ADDRESS_BITS_64K: u8 = 16;
const PAGE_SIZE: usize = 256;

#[derive(PartialEq, Eq, Clone, Copy)]
enum OperatingModes {
    Modes100,
    Modes101,
    Modes110,
    Modes111,
    Modes114,
    Modes144,
}

impl OperatingModes {
    fn imode(&self) -> QspiWidth {
        QspiWidth::Single
    }

    fn amode(&self) -> QspiWidth {
        match *self {
            OperatingModes::Modes100 => QspiWidth::None,
            OperatingModes::Modes101 => QspiWidth::None,
            OperatingModes::Modes110 => QspiWidth::Single,
            OperatingModes::Modes111 => QspiWidth::Single,
            OperatingModes::Modes114 => QspiWidth::Single,
            OperatingModes::Modes144 => QspiWidth::Quad,
        }
    }

    fn dmode(&self) -> QspiWidth {
        match *self {
            OperatingModes::Modes100 => QspiWidth::None,
            OperatingModes::Modes101 => QspiWidth::Single,
            OperatingModes::Modes110 => QspiWidth::None,
            OperatingModes::Modes111 => QspiWidth::Single,
            OperatingModes::Modes114 => QspiWidth::Quad,
            OperatingModes::Modes144 => QspiWidth::Quad,
        }
    }
}

//#[allow(dead_code)]
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
enum QspiWidth {
    None = 0,
    Single = 1,
    Dual = 2,
    Quad = 3,
}

/// The different QSPI functional modes.
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
enum QspiMode {
    IndirectWrite = 0,
    IndirectRead = 1,
    AutoPolling = 2,
    MemoryMapped = 3,
}

/// The number of bytes required to specify addresses on the chip.
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
enum QspiSize {
    OneByte = 0,
    TwoBytes = 1,
    ThreeBytes = 2,
    FourBytes = 3,
}

/// Commands (instructions) that can be sent to the flash chip.
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
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

/// Mostly taken from the flash aglo repo: https://github.com/willemml/rsworks-flash-algo/blob/a69c2a6eb31d6d50fefcec3d99a8be4da4ca6e8e/src/main.rs

/// Initialize flash chip and QSPI peripheral.
pub fn init() {
    unsafe {
        init_gpio();
        init_qspi();
    }
    init_chip();
}

pub fn shutdown() {
    shutdown_chip();
    unsafe {
        shutdown_qspi();
        shutdown_gpio();
    }
}

unsafe fn init_qspi() {
    let rcc = &(*RCC::ptr());

    rcc.ahb3enr.modify(|_, w| w.qspien().set_bit());

    rcc.ahb3rstr.modify(|_, w| w.qspirst().reset());
    rcc.ahb3rstr.modify(|_, w| w.qspirst().clear_bit());

    let qspi = &(*QUADSPI::ptr());
    // Single flash mode with a QSPI clock prescaler of 2 (216 / 2 = 108 MHz), FIFO
    // threshold only matters for DMA and is set to 4 to allow word sized DMA requests

    // Configure controller for flash chip.
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

unsafe fn shutdown_qspi() {
    let rcc = &(*RCC::ptr());

    rcc.ahb3rstr.modify(|_, w| w.qspirst().reset());
    rcc.ahb3rstr.modify(|_, w| w.qspirst().clear_bit());

    rcc.ahb3enr.modify(|_, w| w.qspien().clear_bit());
}

unsafe fn init_gpio() {
    let rcc = &(*RCC::ptr());

    rcc.ahb1enr.modify(|_, w| {
        w.gpioben()
            .set_bit()
            .gpiocen()
            .set_bit()
            .gpioden()
            .set_bit()
            .gpioeen()
            .set_bit()
    });

    let gpiob = &(*GPIOB::ptr());
    let gpioc = &(*GPIOC::ptr());
    let gpiod = &(*GPIOD::ptr());
    let gpioe = &(*GPIOE::ptr());

    gpiob.afrl.modify(|_, w| w.afrl2().af9().afrl6().af10());
    gpioc.afrh.modify(|_, w| w.afrh9().af9());
    gpiod.afrh.modify(|_, w| w.afrh12().af9().afrh13().af9());
    gpioe.afrl.modify(|_, w| w.afrl2().af9());

    gpiob
        .moder
        .modify(|_, w| w.moder2().alternate().moder6().alternate());
    gpioc.moder.modify(|_, w| w.moder9().alternate());
    gpiod
        .moder
        .modify(|_, w| w.moder12().alternate().moder13().alternate());
    gpioe.moder.modify(|_, w| w.moder2().alternate());

    gpiob
        .ospeedr
        .modify(|_, w| w.ospeedr2().very_high_speed().ospeedr6().very_high_speed());
    gpioc.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());
    gpiod.ospeedr.modify(|_, w| {
        w.ospeedr12()
            .very_high_speed()
            .ospeedr13()
            .very_high_speed()
    });
    gpioe.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());
}

unsafe fn shutdown_gpio() {
    let gpiob = &(*GPIOB::ptr());
    let gpioc = &(*GPIOC::ptr());
    let gpiod = &(*GPIOD::ptr());
    let gpioe = &(*GPIOE::ptr());

    gpiob.moder.modify(|_, w| w.moder2().analog());
    gpioc.moder.modify(|_, w| w.moder9().analog());
    gpiod
        .moder
        .modify(|_, w| w.moder12().analog().moder13().analog());
    gpioe.moder.modify(|_, w| w.moder2().analog());

    gpiob
        .ospeedr
        .modify(|_, w| w.ospeedr2().low_speed().ospeedr6().low_speed());
    gpioc.ospeedr.modify(|_, w| w.ospeedr9().low_speed());
    gpiod
        .ospeedr
        .modify(|_, w| w.ospeedr12().low_speed().ospeedr13().low_speed());
    gpioe.ospeedr.modify(|_, w| w.ospeedr2().low_speed());
}

fn init_chip() {
    // Turn on the chip.
    send_command(Command::ReleaseDeepPowerDown);

    asm::delay(1000);

    // Enable writing to the chip so that the status register can be changed.
    send_command(Command::WriteEnable);

    wait();

    // Set enable quad in the chip's status register.
    send_write_command(
        Command::WriteStatusRegister2,
        FLASH_SIZE,
        [2u8].as_slice(),
        OperatingModes::Modes101,
    );

    wait();

    // Enable QPI on the chip.
    // send_command(Command::EnableQPI);

    wait();

    set_memory_mapped();
}

fn shutdown_chip() {
    unset_memory_mapped();
    send_command(Command::EnableReset);
    send_command(Command::Reset);

    asm::delay(7000);

    send_command(Command::DeepPowerDown);

    asm::delay(1000);
}

fn wait() {
    let mut reg = [1u8];
    let sr1 = reg.as_mut_slice();

    while sr1[0] & 1 == 1 {
        sr1[0] = 0;
        send_read_command(Command::ReadStatusRegister1, FLASH_SIZE, sr1, 1);
    }
}

fn send_command_full(
    mode: QspiMode,
    op_mode: OperatingModes,
    command: Command,
    address: u32,
    alt_bytes: u32,
    alt_byte_count: usize,
    dummy_cycles: u8,
    data: Option<&[u8]>,
    read: Option<&mut [u8]>,
    data_length: usize,
) {
    let qspi = unsafe { &(*QUADSPI::ptr()) };

    let previous_mode = qspi.ccr.read().fmode().bits();

    if mode == QspiMode::MemoryMapped {
        if previous_mode == QspiMode::IndirectWrite as u8
            || previous_mode == QspiMode::IndirectRead as u8
        {
            qspi.ar.reset();
            if previous_mode == QspiMode::IndirectRead as u8 {
                qspi.cr.modify(|_, w| w.abort().set_bit());
                while qspi.cr.read().abort().bit_is_set() {
                    cortex_m::asm::nop();
                }
            }
        }
    } else if previous_mode == QspiMode::MemoryMapped as u8 {
        qspi.cr.modify(|_, w| w.abort().set_bit());
        while qspi.cr.read().abort().bit_is_set() {
            cortex_m::asm::nop();
        }
    }

    assert!(
        qspi.ccr.read().fmode().bits() != QspiMode::MemoryMapped as u8
            || qspi.sr.read().busy().bit_is_clear()
    );

    qspi.ccr.write(|ccr| unsafe {
        ccr.fmode().bits(mode as u8);
        if data.is_some() || mode == QspiMode::MemoryMapped {
            ccr.dmode().bits(op_mode.dmode() as u8);
        }
        if mode != QspiMode::MemoryMapped {
            qspi.dlr.modify(|_, w| {
                w.dl().bits({
                    if data_length > 0 {
                        data_length as u32 - 1
                    } else {
                        0
                    }
                })
            });
        }
        ccr.dcyc().bits(dummy_cycles);
        if alt_byte_count > 0 {
            ccr.abmode().bits(op_mode.amode() as u8);
            ccr.absize().bits(alt_byte_count as u8 - 1);
            qspi.abr.write(|w| w.bits(alt_bytes));
        }
        if address != FLASH_SIZE || mode == QspiMode::MemoryMapped {
            ccr.admode().bits(op_mode.amode() as u8);
            ccr.adsize().bits(QspiSize::ThreeBytes as u8);
        }
        ccr.imode().bits(op_mode.imode() as u8);
        ccr.instruction().bits(command as u8);
        if mode == QspiMode::MemoryMapped {
            ccr.sioo().set_bit();
        }
        ccr
    });

    if address != FLASH_SIZE {
        qspi.ar.write(|w| unsafe { w.bits(address) });
    }

    if mode == QspiMode::IndirectWrite {
        if let Some(data) = data {
            for i in data {
                let ptr = qspi.dr.as_ptr() as *mut u8;
                unsafe { ptr.write_volatile(*i) }
            }
        }
    } else if mode == QspiMode::IndirectRead {
        let read = read.expect("Cannot read to null.");
        for i in 0..(data_length - 1) {
            let ptr = qspi.dr.as_ptr() as *mut u8;
            read[i] = unsafe { ptr.read_volatile() };
        }
    }

    if mode != QspiMode::MemoryMapped {
        while qspi.sr.read().busy().bit() {
            asm::nop();
        }
    }
}

fn send_read_command(command: Command, address: u32, buffer: &mut [u8], length: usize) {
    send_command_full(
        QspiMode::IndirectRead,
        OperatingModes::Modes101,
        command,
        address,
        0,
        0,
        0,
        None,
        Some(buffer),
        length,
    )
}

fn send_write_command(command: Command, address: u32, data: &[u8], op_mode: OperatingModes) {
    send_command_full(
        QspiMode::IndirectWrite,
        op_mode,
        command,
        address,
        0,
        0,
        0,
        Some(data),
        None,
        data.len(),
    )
}

fn send_command(command: Command) {
    send_command_full(
        QspiMode::IndirectWrite,
        OperatingModes::Modes100,
        command,
        FLASH_SIZE,
        0,
        0,
        0,
        None,
        None,
        0,
    )
}

pub fn set_memory_mapped() {
    send_command_full(
        QspiMode::MemoryMapped,
        OperatingModes::Modes144,
        Command::FastReadQuadIO,
        FLASH_SIZE,
        0xA0,
        1,
        4,
        None,
        None,
        0,
    )
}

pub fn unset_memory_mapped() {
    let dummy_data = &mut [0u8; 0];
    send_command_full(
        QspiMode::IndirectRead,
        OperatingModes::Modes144,
        Command::FastReadQuadIO,
        0,
        !0xA0,
        1,
        4,
        None,
        Some(dummy_data),
        1,
    )
}

pub fn reset() {
    send_command(Command::EnableReset);

    send_command(Command::Reset);
}

fn unlock_flash() {
    send_command(Command::WriteEnable);
    wait();

    let mut reg = [0u8];
    let sr2 = reg.as_mut_slice();

    send_read_command(Command::ReadStatusRegister2, FLASH_SIZE, sr2, 1);
    if sr2[0] & 2 == 2 {
        sr2[0] = 2;
    }

    send_write_command(
        Command::WriteStatusRegister,
        FLASH_SIZE,
        [0, sr2[0]].as_slice(),
        OperatingModes::Modes101,
    );
}

pub fn sector_at_address(mut address: u32) -> u8 {
    address -= FLASH_START;
    let mut index = address >> ADDRESS_BITS_64K;
    if index > N_64K_SECTORS as u32 {
        panic!("Sector does not exist.");
    } else if index >= 1 {
        return N_4K_SECTORS + N_32K_SECTORS + index as u8 - 1;
    }
    index = address >> ADDRESS_BITS_32K;
    if index >= 1 {
        index += N_4K_SECTORS as u32 - 1;
        assert!(index >= N_4K_SECTORS as u32 && index <= (N_4K_SECTORS + N_32K_SECTORS) as u32);
        return index as u8;
    }
    index = address >> ADDRESS_BITS_4K;
    assert!(index <= N_4K_SECTORS as u32);
    index as u8
}

pub fn erase_all() {
    unset_memory_mapped();
    unlock_flash();
    send_command(Command::WriteEnable);
    wait();
    send_command(Command::ChipErase);
    wait();
    set_memory_mapped();
}

pub fn erase_sector(sector: u8) {
    assert!(sector < N_SECTORS);
    unset_memory_mapped();
    unlock_flash();
    send_command(Command::WriteEnable);
    wait();
    if sector < N_4K_SECTORS {
        send_write_command(
            Command::Erase4KbyteBlock,
            (sector as u32) << ADDRESS_BITS_4K,
            &mut [],
            OperatingModes::Modes110,
        );
    } else if sector < N_4K_SECTORS + N_32K_SECTORS {
        send_write_command(
            Command::Erase32KbyteBlock,
            (sector as u32) << ADDRESS_BITS_32K,
            &mut [],
            OperatingModes::Modes110,
        );
    } else {
        send_write_command(
            Command::Erase64KbyteBlock,
            (sector as u32) << ADDRESS_BITS_64K,
            &mut [],
            OperatingModes::Modes110,
        );
    }
    wait();
    set_memory_mapped();
}

pub fn write_memory(mut address: u32, source: &[u8]) {
    address -= FLASH_START;
    unset_memory_mapped();

    let offset: u8 = (address & (PAGE_SIZE - 1) as u32) as u8;
    let mut fits_in_page: usize = PAGE_SIZE - offset as usize;
    let mut length = source.len();
    let mut start = 0;
    while length > 0 {
        if fits_in_page > length {
            fits_in_page = length;
        }

        send_command(Command::WriteEnable);
        wait();

        send_write_command(
            Command::QuadPageProgram,
            address,
            &source[start..(start + fits_in_page)],
            OperatingModes::Modes114,
        );

        length -= fits_in_page;
        address += fits_in_page as u32;
        start += fits_in_page;
        fits_in_page = PAGE_SIZE;

        wait();
    }
    set_memory_mapped();
}
