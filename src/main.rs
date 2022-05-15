#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(alloc_error_handler)]

extern crate cortex_m_rt as rt;
extern crate stm32f7xx_hal as hal;

extern crate alloc;

use alloc_cortex_m::CortexMHeap;

use hal::otg_fs::UsbBus;
use rt::entry;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_dfu::DFUClass;

use core::{fmt::Write, alloc::Layout};

use rtt_target::{rprintln, rtt_init_print};

use core::panic::PanicInfo;

use keypad::Key;
use rustworks::{*, dfu::QspiDfu};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {}
}

#[alloc_error_handler]
fn oom(layout: Layout) -> ! {
    rprintln!("{:?}", layout);
    loop {}
}

#[entry]
fn main() -> ! {
    // Initialize RTT printing (for debugging).
    rtt_init_print!(NoBlockTrim, 4096);

    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 2048;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

    let (external_flash, mut display, mut keypad, mut led, usb, mut delay, _clocks) =
        get_devices(false);

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });


    /* -- Disabled internal flash test write as it crashes probe-rs --
    use stm32f7xx_hal::flash::Flash;

    // Setup insternal flash for easy writing.
    let mut flash = Flash::new(dp.FLASH);

    let flash_test_data_str = "This is a message to test if writing to flash works.";
    let flash_test_data: &[u8] = flash_test_data_str.as_bytes();

    // The flash needs to be unlocked before any erase or program operations.
    flash.unlock();

    // Program the the test data into the internal flash memory starting at offset 0xC00 from
    // the beginning of the flash memory.
    flash.blocking_program(0x10000, flash_test_data).unwrap();

    // Lock the flash memory to prevent any accidental modification of the flash content.
    flash.lock();
    */

    // Initialize the external flash chip.
    let mut external_flash = external_flash.init(&mut delay);

    let (manufacturer, device) = external_flash.read_ids();
    assert_eq!(manufacturer, 0x1F);
    assert_eq!(device, 0x16);

    // Read the data at the pointer as an ascii hex encoded string.
    display.write_top_fmt(format_args!(
        "Manufacturer: {:#04x}\nDevice: {:#04x}",
        manufacturer, device
    ));

    display.write_top("\n\nBefore erase:\n");
    for i in 0..8 {
        let byte = external_flash.read_byte(i);
        display.write_top_fmt(format_args!("{:02x}", byte));
    }

    // This also works but is very slow.
    // From the datasheet: Chip Erase Time: 30s typ., 150s max
    // external_flash.write_enable();
    // external_flash.chip_erase();

    external_flash.write_enable();
    external_flash.block_erase_4k(0);

    display.write_top("\n\nAfter erase:\n");
    for i in 0..8 {
        let byte = external_flash.read_byte(i);
        display.write_top_fmt(format_args!("{:02x} ", byte));
    }

    external_flash.write_enable();
    external_flash.program_page(0, &[0x12, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);

    display.write_top("\n\nAfter write:\n");
    for i in 0..8 {
        let byte = external_flash.read_byte(i);
        display.write_top_fmt(format_args!("{:02x} ", byte));
    }

    // let _external_flash = external_flash.into_memory_mapped();

    // // Create a pointer to the first 8 bytes at the address 0x90000000 of external flash.
    // let read_slice = unsafe { slice::from_raw_parts(0x90000000 as *const u8, 8) };
    // display.write_top("\n\nMemory mapped:\n");
    // for byte in read_slice.iter() {
    //     display.write_top_fmt(format_args!("{:02x} ", byte));
    // }

    // Draw display contents
    display.draw_top(false);

    led.blue();

    let mut power_state = true;

    // Holds the keys pressed on the previous scan.
    let mut last_pressed: heapless::Vec<Key, 46> = heapless::Vec::new();

    // Whether the calculator is on or off, currently just disables the backlight, clears the
    // screen and stops any key presses except for `Key::Power` from being evaluated.
    let mut off = false;

    led.green();

    // Total number of keypresses.
    let mut key_count = 0usize;

    let dfu_mem = QspiDfu::new(external_flash);

    let mut dfu = DFUClass::new(&usb_bus, dfu_mem);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0483, 0xdf11))
        .manufacturer("Numworks")
        .product("RustWorks Bootloader")
        .serial_number("TEST")
        .device_class(0x02)
        .build();

    loop {
        usb_dev.poll(&mut [&mut dfu]);

        // Read the keys currently pressed.
        let keys = keypad.read(&mut delay);
        // Make sure that the keys currently pressed are not the same as the last scan (done to
        // ensure that keys are not repeated unintentionally).
        if keys != last_pressed {
            // If no keys are pressed there is no need to check for specific keys.
            if !keys.is_empty() {
                // Check if the power keys is pressed.
                if keys.contains(&Key::Power) {
                    // If the calculator is currently "on" (meaning the backlight is on and all
                    // keys are being scanned) turn it "off", otherwise turn it back "on".
                    if power_state {
                        // Disable the backlight and clear the screen to avoid burn in.
                        display.set_backlight(0);
                        led.off();
                        display.clear(display::BG_COLOUR);
                        off = true;
                        power_state = false;
                    } else {
                        // re-draw text boxes
                        display.draw_all();
                        // re-enable backlight
                        display.set_backlight(1);
                        led.green();
                        off = false;
                        power_state = true;
                    }
                }

                // Do not evaluate anything or draw anything to display if the calulator is
                // "off".
                if !off {
                    // If `Key::EXE` is pressed create a new line and do not do anything else.
                    if keys.contains(&Key::EXE) {
                        // Push the text in the input bar into the output display.
                        display = display.write_bottom_to_top();
                        // Write the key count (with padding so that it appears left alligned)
                        // to the output section of the display.
                        display
                            .write_fmt(format_args!("\n{: >52}", key_count))
                            .unwrap();
                        // Draw both sections of the display.
                        display.draw_all();
                    } else {
                        // Set `shift` to `true` if `Key::Shift` is pressed.
                        let shift = keys.contains(&Key::Shift);
                        // Evaluate all the keys pressed on the keypad.
                        for key in keys.iter() {
                            // Get the pressed key's corresponding character, will be `\0` if
                            // the key does not have a character, will probably change this in
                            // the future to be strings, or completely redesign the console...
                            let mut key_char = char::from(*key);
                            if key_char != '\0' {
                                if shift {
                                    key_char = key_char.to_ascii_uppercase();
                                }
                                let mut tmp = [0u8; 4];
                                if display.write_bottom(key_char.encode_utf8(&mut tmp), true) {
                                    key_count += 1;
                                }
                            // If `Key::Delete` is pressed, remove the last character from the
                            // input display box
                            } else if key == &Key::Delete {
                                display.pop_bottom(true);
                            // If `Key::Clear` is pressed (`Key::Delete` and `Key::Shift`)
                            // remove all text from the input display box.
                            } else if key == &Key::Clear {
                                display.clear_bottom(true);
                            }
                        }
                    }
                }
            }
            last_pressed = keys;
        }
    }
}
