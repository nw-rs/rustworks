use stm32f7xx_hal::gpio::{self, Alternate, Floating, Input, Speed, AF12};

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
    ne3: gpio::gpiod::PD7<Alternate<AF12>>,
}

const FSMCMemoryBank: usize = 1;
const FSMCDataCommandAddressBit: usize = 16;
const FSMCBaseAddress: usize = 0x60_000_000;
const FSMCBankAddress: usize = FSMCBaseAddress + (FSMCMemoryBank - 1) * 0x04_000_000;
const DataAddress: usize = FSMCBankAddress | (1 << (FSMCDataCommandAddressBit + 1));

impl FSMC16BitInterface {
    /// Create new parallel GPIO interface for communication with a display driver
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        p0: gpio::gpiod::PD14<Input<Floating>>,
        p1: gpio::gpiod::PD15<Input<Floating>>,
        p2: gpio::gpiod::PD0<Input<Floating>>,
        p3: gpio::gpiod::PD1<Input<Floating>>,
        p4: gpio::gpioe::PE7<Input<Floating>>,
        p5: gpio::gpioe::PE8<Input<Floating>>,
        p6: gpio::gpioe::PE9<Input<Floating>>,
        p7: gpio::gpioe::PE10<Input<Floating>>,
        p8: gpio::gpioe::PE11<Input<Floating>>,
        p9: gpio::gpioe::PE12<Input<Floating>>,
        p10: gpio::gpioe::PE13<Input<Floating>>,
        p11: gpio::gpioe::PE14<Input<Floating>>,
        p12: gpio::gpioe::PE15<Input<Floating>>,
        p13: gpio::gpiod::PD8<Input<Floating>>,
        p14: gpio::gpiod::PD9<Input<Floating>>,
        p15: gpio::gpiod::PD10<Input<Floating>>,
        dc: gpio::gpiod::PD11<Input<Floating>>,
        wr: gpio::gpiod::PD5<Input<Floating>>,
        noe: gpio::gpiod::PD4<Input<Floating>>,
        ne3: gpio::gpiod::PD7<Input<Floating>>,
    ) -> Self {
        let p0 = p0
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p1 = p1
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p2 = p2
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p3 = p3
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p4 = p4
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p5 = p5
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p6 = p6
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p7 = p7
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p8 = p8
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p9 = p9
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p10 = p10
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p11 = p11
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p12 = p12
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p13 = p13
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p14 = p14
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let p15 = p15
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);

        let dc = dc
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let wr = wr
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);

        let noe = noe
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);
        let ne3 = ne3
            .into_pull_up_input()
            .into_alternate_af12()
            .set_speed(Speed::VeryHigh);

        let fsmc = unsafe { &*stm32f7xx_hal::pac::FMC::ptr() };
        fsmc.bcr1.modify(|_, w| {
            w.muxen()
                .disabled()
                .mtyp()
                .sram()
                .mwid()
                .bits16()
                .bursten()
                .disabled()
                .waitpol()
                .active_low()
                .waitcfg()
                .before_wait_state()
                .wren()
                .enabled()
                .waiten()
                .disabled()
                .extmod()
                .disabled()
                .asyncwait()
                .disabled()
                .cburstrw()
                .disabled()
                .cpsize()
                .no_burst_split()
                .cclken()
                .clear_bit()
                .wfdis()
                .set_bit()
                .mbken()
                .enabled()
        });

        unsafe {
            fsmc.btr1.modify(|_, w| {
                w.accmod()
                    .a()
                    .datlat()
                    .bits(2)
                    .clkdiv()
                    .bits(2)
                    .busturn()
                    .bits(1)
                    .addhld()
                    .bits(1)
                    .addset()
                    .bits(3)
            });

            fsmc.bwtr1.modify(|_, w| {
                w.accmod()
                    .a()
                    .datast()
                    .bits(4)
                    .busturn()
                    .bits(1)
                    .addhld()
                    .bits(1)
                    .addset()
                    .bits(3)
            });
        }

        fsmc.bcr3.modify(|_, w| {
            w.muxen()
                .disabled()
                .mtyp()
                .sram()
                .mwid()
                .bits16()
                .bursten()
                .disabled()
                .waitpol()
                .active_low()
                .waitcfg()
                .before_wait_state()
                .wren()
                .enabled()
                .waiten()
                .disabled()
                .extmod()
                .enabled()
                .asyncwait()
                .disabled()
                .cburstrw()
                .disabled()
                .cpsize()
                .no_burst_split()
                .faccen()
                .disabled()
                .mbken()
                .enabled()
        });
        unsafe {
            fsmc.btr3.modify(|_, w| {
                w.addset()
                    .bits(3)
                    .addhld()
                    .bits(1)
                    .busturn()
                    .bits(1)
                    .clkdiv()
                    .bits(2)
                    .datlat()
                    .bits(2)
                    .accmod()
                    .a()
            });

            fsmc.bwtr3.modify(|_, w| {
                w.accmod()
                    .a()
                    .datast()
                    .bits(4)
                    .busturn()
                    .bits(1)
                    .addhld()
                    .bits(1)
                    .addset()
                    .bits(3)
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
            ne3,
        }
    }

    fn send_command_u8(&mut self, command: u8) {
        unsafe {
            core::ptr::write_volatile(FSMCBankAddress as *mut u8, command);
        }
    }
    fn send_command_u16(&mut self, command: u16) {
        unsafe {
            core::ptr::write_volatile(FSMCBankAddress as *mut u16, command);
        }
    }
    fn send_data_u8(&mut self, data: u8) {
        unsafe {
            core::ptr::write_volatile(DataAddress as *mut u8, data);
        }
    }
    fn send_data_u16(&mut self, data: u16) {
        unsafe {
            core::ptr::write_volatile(DataAddress as *mut u16, data);
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
