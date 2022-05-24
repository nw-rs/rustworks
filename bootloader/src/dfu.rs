use usbd_dfu::DFUMemIO;

use heapless::Vec;

use nw_board_support::external_flash;

pub struct QspiDfu {
    buffer: Vec<u8, { Self::TRANSFER_SIZE as usize * 8 }>,
}

impl QspiDfu {
    pub fn new() -> Self {
        QspiDfu {
            buffer: Vec::new(),
        }
    }
}

impl DFUMemIO for QspiDfu {
    const INITIAL_ADDRESS_POINTER: u32 = 0x90000000;

    const MEM_INFO_STRING: &'static str =
        "@ExternalFlash/0x90000000/08*004Kg,01*032Kg,63*064Kg,64*064Kg";

    const HAS_DOWNLOAD: bool = true;

    const HAS_UPLOAD: bool = true;

    const MANIFESTATION_TOLERANT: bool = false;

    const PROGRAM_TIME_MS: u32 = 5;

    const ERASE_TIME_MS: u32 = 60;

    const FULL_ERASE_TIME_MS: u32 = 30000;

    const MANIFESTATION_TIME_MS: u32 = 1;

    const DETACH_TIMEOUT: u16 = 250;

    const TRANSFER_SIZE: u16 = 128;

    fn store_write_buffer(&mut self, src: &[u8]) -> Result<(), ()> {
        self.buffer.clear();
        if let Ok(()) = self.buffer.extend_from_slice(src) {
            Ok(())
        } else {
            Err(())
        }
    }

    fn read(&mut self, address: u32, length: usize) -> Result<&[u8], usbd_dfu::DFUMemError> {
        self.buffer.clear();

        Ok(unsafe { core::slice::from_raw_parts(address as *const u8, length) })
    }

    fn program(&mut self, address: u32, length: usize) -> Result<(), usbd_dfu::DFUMemError> {
        let slice = &self.buffer.as_slice()[0..length];

        external_flash::write_memory(address, slice);

        Ok(())
    }

    fn erase(&mut self, address: u32) -> Result<(), usbd_dfu::DFUMemError> {
        external_flash::erase_sector(external_flash::sector_at_address(address));
        Ok(())
    }

    fn erase_all(&mut self) -> Result<(), usbd_dfu::DFUMemError> {
        external_flash::erase_all();
        Ok(())
    }

    fn manifestation(&mut self) -> Result<(), usbd_dfu::DFUManifestationError> {
        loop {}
    }
}
