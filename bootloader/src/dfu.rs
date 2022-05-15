use usbd_dfu::DFUMemIO;

use crate::external_flash::{ExternalFlash, Indirect};

pub struct QspiDfu {
    flash: ExternalFlash<Indirect>,
    buffer: [u8; Self::TRANSFER_SIZE as usize],
}

impl QspiDfu {
    const EMPTY_BUFFER: [u8; Self::TRANSFER_SIZE as usize] = [0; Self::TRANSFER_SIZE as usize];
    pub fn new(flash: ExternalFlash<Indirect>) -> Self {
        QspiDfu {
            flash,
            buffer: [0; Self::TRANSFER_SIZE as usize],
        }
    }
}

impl DFUMemIO for QspiDfu {
    const INITIAL_ADDRESS_POINTER: u32 = 0x90000000;

    const MEM_INFO_STRING: &'static str =
        "@ExternalFlash/0x90000000/08*004Kg,01*032Kg,63*064Kg,64*064Kg";

    const HAS_DOWNLOAD: bool = true;

    const HAS_UPLOAD: bool = true;

    const MANIFESTATION_TOLERANT: bool = true;

    const PROGRAM_TIME_MS: u32 = 5;

    const ERASE_TIME_MS: u32 = 60;

    const FULL_ERASE_TIME_MS: u32 = 30000;

    const MANIFESTATION_TIME_MS: u32 = 1;

    const DETACH_TIMEOUT: u16 = 250;

    const TRANSFER_SIZE: u16 = 128;

    fn store_write_buffer(&mut self, src: &[u8]) -> Result<(), ()> {
        self.buffer = Self::EMPTY_BUFFER;
        self.buffer.copy_from_slice(src);
        Ok(())
    }

    fn read(&mut self, address: u32, length: usize) -> Result<&[u8], usbd_dfu::DFUMemError> {
        self.buffer = Self::EMPTY_BUFFER;
        self.flash.write_disable();
        for i in 0..(length as u32) {
            self.buffer[i as usize] = self
                .flash
                .read_byte(address - Self::INITIAL_ADDRESS_POINTER + i);
        }
        Ok(&self.buffer)
    }

    fn program(&mut self, address: u32, length: usize) -> Result<(), usbd_dfu::DFUMemError> {
        self.flash.write_enable();
        self.flash.program_page(
            address - Self::INITIAL_ADDRESS_POINTER,
            &self.buffer[0..length],
        );
        Ok(())
    }

    fn erase(&mut self, address: u32) -> Result<(), usbd_dfu::DFUMemError> {
        self.flash
            .block_erase_4k(address - Self::INITIAL_ADDRESS_POINTER);
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
