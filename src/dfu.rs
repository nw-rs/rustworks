use alloc::vec::Vec;
use rtt_target::rprintln;
use usbd_dfu::DFUMemIO;

extern crate alloc;

use crate::external_flash::{ExternalFlash, Indirect};

pub struct QspiDfu {
    flash: ExternalFlash<Indirect>,
    buffer: Vec<u8>,
    read_buf: Vec<u8>,
}

impl QspiDfu {
    pub fn new(flash: ExternalFlash<Indirect>) -> Self {
        QspiDfu {
            flash,
            buffer: Vec::new(),
            read_buf: Vec::new()
        }
    }
}

impl DFUMemIO for QspiDfu {
    const INITIAL_ADDRESS_POINTER: u32 = 0x90000000;

    const MEM_INFO_STRING: &'static str = "@ExternalFlash/0x90000000/08*004Kg,01*032Kg,63*064Kg,64*064Kg";

    const HAS_DOWNLOAD: bool = true;

    const HAS_UPLOAD: bool = true;

    const MANIFESTATION_TOLERANT: bool = true;

    const PROGRAM_TIME_MS: u32 = 5;

    const ERASE_TIME_MS: u32 = 60;

    const FULL_ERASE_TIME_MS: u32 =  30000;

    const MANIFESTATION_TIME_MS: u32 = 1;

    const DETACH_TIMEOUT: u16 = 250;

    const TRANSFER_SIZE: u16 = 128;

    fn store_write_buffer(&mut self, src: &[u8]) -> Result<(), ()> {
        self.buffer.extend_from_slice(src);
        Ok(())
    }

    fn read(&mut self, address: u32, length: usize) -> Result<&[u8], usbd_dfu::DFUMemError> {
        self.read_buf.clear();
        self.read_buf.shrink_to_fit();
        self.flash.write_disable();
        for i in 0..(length as u32) {
            let byte = self.flash.read_byte(address + i);
            rprintln!("Read byte: {}", byte);
            self.read_buf.push(byte);
        }
        let result = &self.read_buf.as_slice()[0..length];
        Ok(result)
    }

    fn program(&mut self, address: u32, length: usize) -> Result<(), usbd_dfu::DFUMemError> {
        let drain = self.buffer.drain(0..(length - 1));
        let slice = drain.as_slice();
        rprintln!("Program: {:?}", slice);
        self.flash.write_enable();
        self.flash.program_page(address, slice);
        Ok(())
    }

    fn erase(&mut self, address: u32) -> Result<(), usbd_dfu::DFUMemError> {
        self.flash.block_erase_4k(address);
        Ok(())
    }

    fn erase_all(&mut self) -> Result<(), usbd_dfu::DFUMemError> {
        self.flash.chip_erase();
        Ok(())
    }

    fn manifestation(&mut self) -> Result<(), usbd_dfu::DFUManifestationError> {
        cortex_m::peripheral::SCB::sys_reset();
    }
}