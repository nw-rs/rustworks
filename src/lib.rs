#![no_std]

pub use stm32f7xx_hal;

pub mod display;
pub mod external_flash;
pub mod keypad;
pub mod led;

pub const HCLK: u32 = 216_000_000;
