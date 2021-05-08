use stm32f7xx_hal::gpio::{self, Alternate, Speed, AF12};

pub use display_interface::{DisplayError, WriteOnlyDataCommand};

#[allow(dead_code)]
pub struct FSMC16BitInterface {
    p0: gpio::gpiod::PD14<Alternate<AF12>>,
    p1: gpio::gpiod::PD15<Alternate<AF12>>,
    p2: gpio::gpiod::PD0<Alternate<AF12>>,
    p3: gpio::gpiod::PD1<Alternate<AF12>>,
    p4: gpio::gpioe::PE7<Alternate<AF12>>,
    p5: gpio::gpioe::PE8<Alternate<AF12>>,
    p6: gpio::gpioe::PE9<Alternate<AF12>>,
    p7: gpio::gpioe::PE10<Alternate<AF12>>,
    p8: gpio::gpioe::PE11<Alternate<AF12>>,
    p9: gpio::gpioe::PE12<Alternate<AF12>>,
    p10: gpio::gpioe::PE13<Alternate<AF12>>,
    p11: gpio::gpioe::PE14<Alternate<AF12>>,
    p12: gpio::gpioe::PE15<Alternate<AF12>>,
    p13: gpio::gpiod::PD8<Alternate<AF12>>,
    p14: gpio::gpiod::PD9<Alternate<AF12>>,
    p15: gpio::gpiod::PD10<Alternate<AF12>>,
    dc: gpio::gpiod::PD11<Alternate<AF12>>,
    wr: gpio::gpiod::PD5<Alternate<AF12>>,
    noe: gpio::gpiod::PD4<Alternate<AF12>>,
    ne1: gpio::gpiod::PD7<Alternate<AF12>>,
}

const MEMORY_BANK: usize = 1;
const DATA_COMMAND_ADDRESS_BIT: usize = 16;
const BASE_ADDRESS: usize = 0x60_000_000;
const BANK_ADDRESS: usize = BASE_ADDRESS + (MEMORY_BANK - 1) * 0x04_000_000;
const DATA_ADDRESS: usize = BANK_ADDRESS | (1 << (DATA_COMMAND_ADDRESS_BIT + 1));

impl FSMC16BitInterface {
    /// Create new parallel GPIO interface for communication with a display driver
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        hclk_mhz: u32,
        p0: gpio::gpiod::PD14<Alternate<AF12>>,
        p1: gpio::gpiod::PD15<Alternate<AF12>>,
        p2: gpio::gpiod::PD0<Alternate<AF12>>,
        p3: gpio::gpiod::PD1<Alternate<AF12>>,
        p4: gpio::gpioe::PE7<Alternate<AF12>>,
        p5: gpio::gpioe::PE8<Alternate<AF12>>,
        p6: gpio::gpioe::PE9<Alternate<AF12>>,
        p7: gpio::gpioe::PE10<Alternate<AF12>>,
        p8: gpio::gpioe::PE11<Alternate<AF12>>,
        p9: gpio::gpioe::PE12<Alternate<AF12>>,
        p10: gpio::gpioe::PE13<Alternate<AF12>>,
        p11: gpio::gpioe::PE14<Alternate<AF12>>,
        p12: gpio::gpioe::PE15<Alternate<AF12>>,
        p13: gpio::gpiod::PD8<Alternate<AF12>>,
        p14: gpio::gpiod::PD9<Alternate<AF12>>,
        p15: gpio::gpiod::PD10<Alternate<AF12>>,
        dc: gpio::gpiod::PD11<Alternate<AF12>>,
        wr: gpio::gpiod::PD5<Alternate<AF12>>,
        noe: gpio::gpiod::PD4<Alternate<AF12>>,
        ne1: gpio::gpiod::PD7<Alternate<AF12>>,
    ) -> Self {
        let p0 = p0.set_speed(Speed::VeryHigh);
        let p1 = p1.set_speed(Speed::VeryHigh);
        let p2 = p2.set_speed(Speed::VeryHigh);
        let p3 = p3.set_speed(Speed::VeryHigh);
        let p4 = p4.set_speed(Speed::VeryHigh);
        let p5 = p5.set_speed(Speed::VeryHigh);
        let p6 = p6.set_speed(Speed::VeryHigh);
        let p7 = p7.set_speed(Speed::VeryHigh);
        let p8 = p8.set_speed(Speed::VeryHigh);
        let p9 = p9.set_speed(Speed::VeryHigh);
        let p10 = p10.set_speed(Speed::VeryHigh);
        let p11 = p11.set_speed(Speed::VeryHigh);
        let p12 = p12.set_speed(Speed::VeryHigh);
        let p13 = p13.set_speed(Speed::VeryHigh);
        let p14 = p14.set_speed(Speed::VeryHigh);
        let p15 = p15.set_speed(Speed::VeryHigh);

        let dc = dc.set_speed(Speed::VeryHigh);
        let wr = wr.set_speed(Speed::VeryHigh);

        let noe = noe.set_speed(Speed::VeryHigh);
        let ne1 = ne1.set_speed(Speed::VeryHigh);

        let fsmc = unsafe { &*stm32f7xx_hal::pac::FMC::ptr() };
        fsmc.bcr1.modify(|_, w| {
            w.muxen()
                .disabled()
                .mtyp()
                .sram()
                .mwid()
                .bits16()
                .wren()
                .enabled()
                .extmod()
                .enabled()
                .mbken()
                .enabled()
        });

        let ns_to_cycles = |ns: u32| ns * hclk_mhz / 1000 + 1;

        let tedge: u32 = 15;
        let twc: u32 = 66;
        let trcfm: u32 = 450;
        let twrl: u32 = 15;
        let trdlfm: u32 = 355;

        let trdatast = trdlfm + tedge;
        let twdatast = twrl + tedge;

        let read_data_cycles = ns_to_cycles(trdatast);

        let read_addrsetup_cycles = ns_to_cycles(trcfm - trdatast);

        let write_data_cycles = ns_to_cycles(twdatast);

        let write_addrsetup_cycles = ns_to_cycles(twc - twdatast) - 1;

        unsafe {
            fsmc.btr1.modify(|_, w| {
                w.accmod()
                    .a()
                    .datast()
                    .bits(read_data_cycles as u8)
                    .busturn()
                    .bits(0)
                    .addhld()
                    .bits(0)
                    .addset()
                    .bits(read_addrsetup_cycles as u8)
            });

            fsmc.bwtr1.modify(|_, w| {
                w.accmod()
                    .a()
                    .datast()
                    .bits(write_data_cycles as u8)
                    .busturn()
                    .bits(0)
                    .addhld()
                    .bits(0)
                    .addset()
                    .bits(write_addrsetup_cycles as u8)
            });
        }

        Self {
            p0,
            p1,
            p2,
            p3,
            p4,
            p5,
            p6,
            p7,
            p8,
            p9,
            p10,
            p11,
            p12,
            p13,
            p14,
            p15,
            dc,
            wr,
            noe,
            ne1,
        }
    }

    fn send_command_u8(&mut self, command: u8) {
        unsafe {
            core::ptr::write_volatile(BANK_ADDRESS as *mut u8, command);
        }
    }
    fn send_command_u16(&mut self, command: u16) {
        unsafe {
            core::ptr::write_volatile(BANK_ADDRESS as *mut u16, command);
        }
    }
    fn send_data_u8(&mut self, data: u8) {
        unsafe {
            core::ptr::write_volatile(DATA_ADDRESS as *mut u8, data);
        }
    }
    fn send_data_u16(&mut self, data: u16) {
        unsafe {
            core::ptr::write_volatile(DATA_ADDRESS as *mut u16, data);
        }
    }
}

use display_interface::DataFormat;

impl WriteOnlyDataCommand for FSMC16BitInterface {
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        match cmds {
            DataFormat::U8(slice) => {
                for cmd in slice {
                    self.send_command_u8(*cmd)
                }
            }
            DataFormat::U8Iter(iter) => {
                for cmd in iter {
                    self.send_command_u8(cmd)
                }
            }
            DataFormat::U16(slice) => {
                for cmd in slice {
                    self.send_command_u16(*cmd)
                }
            }
            DataFormat::U16BE(slice) | DataFormat::U16LE(slice) => {
                for cmd in slice {
                    self.send_command_u16(*cmd)
                }
            }
            DataFormat::U16BEIter(iter) | DataFormat::U16LEIter(iter) => {
                for cmd in iter {
                    self.send_command_u16(cmd)
                }
            }
            _ => Err(display_interface::DisplayError::DataFormatNotImplemented)?,
        }
        Ok(())
    }

    fn send_data(&mut self, buf: DataFormat) -> Result<(), DisplayError> {
        match buf {
            DataFormat::U8(slice) => {
                for d in slice {
                    self.send_data_u8(*d)
                }
            }
            DataFormat::U8Iter(iter) => {
                for d in iter {
                    self.send_data_u8(d)
                }
            }
            DataFormat::U16(slice) => {
                for d in slice {
                    self.send_data_u16(*d)
                }
            }
            DataFormat::U16BE(slice) | DataFormat::U16LE(slice) => {
                for d in slice {
                    self.send_data_u16(*d)
                }
            }
            DataFormat::U16BEIter(iter) | DataFormat::U16LEIter(iter) => {
                for d in iter {
                    self.send_data_u16(d)
                }
            }
            _ => Err(display_interface::DisplayError::DataFormatNotImplemented)?,
        }
        Ok(())
    }
}
