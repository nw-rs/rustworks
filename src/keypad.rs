use embedded_hal::blocking::delay::DelayUs;
use heapless::Vec;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use stm32f7xx_hal::gpio::gpioa::{PA, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8};
use stm32f7xx_hal::gpio::{Input, OpenDrain, Output, PullUp};
use stm32f7xx_hal::{
    gpio::{
        gpioc::{PC0, PC1, PC2, PC3, PC4, PC5},
        Floating,
    },
    prelude::OutputPin,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Key {
    Left = 0x01,
    Up = 0x02,
    Down = 0x03,
    Right = 0x04,
    Ok = 0x05,
    Back = 0x06,
    Home = 0x11,
    Power = 0x13,
    Shift = 0x21,
    Alpha = 0x22,
    XNT = 0x23,
    Var = 0x24,
    Toolbox = 0x25,
    Delete = 0x26,
    E = 0x31,
    Ln = 0x32,
    Log = 0x33,
    I = 0x34,
    Comma = 0x35,
    Pow = 0x36,
    Sin = 0x41,
    Cos = 0x42,
    Tan = 0x43,
    Pi = 0x44,
    Sqrt = 0x45,
    Square = 0x46,
    Seven = 0x51,
    Eight = 0x52,
    Nine = 0x53,
    LBracket = 0x54,
    RBracket = 0x55,
    Four = 0x61,
    Five = 0x62,
    Six = 0x63,
    Multiply = 0x64,
    Divide = 0x65,
    One = 0x71,
    Two = 0x72,
    Three = 0x73,
    Add = 0x74,
    Subtract = 0x75,
    Zero = 0x81,
    Dot = 0x82,
    EE = 0x83,
    Ans = 0x84,
    EXE = 0x85,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Shift {
    UpperAlpha = 0x22,
    Cut = 0x23,
    Copy = 0x24,
    Paste = 0x25,
    Clear = 0x26,
    RSqBracket = 0x31,
    LSqBracket = 0x32,
    RCurlyBrace = 0x33,
    LCurlyBrace = 0x34,
    Underscore = 0x35,
    Sto = 0x36,
    ASin = 0x41,
    ACos = 0x42,
    ATan = 0x43,
    Equals = 0x44,
    Less = 0x45,
    Greater = 0x46,
}

impl From<Key> for Option<Shift> {
    fn from(key: Key) -> Option<Shift> {
        Shift::from_u8(key as u8)
    }
}

impl From<Shift> for char {
    fn from(shift: Shift) -> char {
        match shift {
            Shift::RSqBracket => '[',
            Shift::LSqBracket => ']',
            Shift::RCurlyBrace => '{',
            Shift::LCurlyBrace => '}',
            Shift::Underscore => '_',
            Shift::Equals => '=',
            Shift::Less => '<',
            Shift::Greater => '>',
            _ => '\0',
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Alpha {
    Colon = 0x23,
    SemiColon = 0x24,
    Quote = 0x25,
    Percent = 0x26,
    A = 0x31,
    B = 0x32,
    C = 0x33,
    D = 0x34,
    E = 0x35,
    F = 0x36,
    G = 0x41,
    H = 0x42,
    I = 0x43,
    J = 0x44,
    K = 0x45,
    L = 0x46,
    M = 0x51,
    N = 0x52,
    O = 0x53,
    P = 0x54,
    Q = 0x55,
    R = 0x61,
    S = 0x62,
    T = 0x63,
    U = 0x64,
    V = 0x65,
    W = 0x71,
    X = 0x72,
    Y = 0x73,
    Z = 0x74,
    Space = 0x75,
    Question = 0x81,
    Exclamation = 0x82,
}

impl From<Key> for Option<Alpha> {
    fn from(key: Key) -> Option<Alpha> {
        Alpha::from_u8(key as u8)
    }
}

impl From<Alpha> for char {
    fn from(alpha: Alpha) -> char {
        match alpha {
            Alpha::Colon => ':',
            Alpha::SemiColon => ';',
            Alpha::Quote => '"',
            Alpha::Percent => '%',
            Alpha::A => 'a',
            Alpha::B => 'b',
            Alpha::C => 'c',
            Alpha::D => 'd',
            Alpha::E => 'e',
            Alpha::F => 'f',
            Alpha::G => 'g',
            Alpha::H => 'h',
            Alpha::I => 'i',
            Alpha::J => 'j',
            Alpha::K => 'k',
            Alpha::L => 'l',
            Alpha::M => 'm',
            Alpha::N => 'n',
            Alpha::O => 'o',
            Alpha::P => 'p',
            Alpha::Q => 'q',
            Alpha::R => 'r',
            Alpha::S => 's',
            Alpha::T => 't',
            Alpha::U => 'u',
            Alpha::V => 'v',
            Alpha::W => 'w',
            Alpha::X => 'x',
            Alpha::Y => 'y',
            Alpha::Z => 'z',
            Alpha::Space => ' ',
            Alpha::Question => '?',
            Alpha::Exclamation => '!',
        }
    }
}
struct KeyColumns(
    PC0<Input<PullUp>>,
    PC1<Input<PullUp>>,
    PC2<Input<PullUp>>,
    PC3<Input<PullUp>>,
    PC4<Input<PullUp>>,
    PC5<Input<PullUp>>,
);

impl KeyColumns {
    fn read(&self) -> u8 {
        // SAFETY: Atomic read with no side effects
        let columns = unsafe { (*stm32f7xx_hal::pac::GPIOC::ptr()).idr.read().bits() };
        columns as u8
    }
}

pub struct Keypad {
    rows: [PA<Output<OpenDrain>>; 9],
    columns: KeyColumns,
}

type MODE = Input<Floating>;

impl Keypad {
    pub fn new(
        pa0: PA0<MODE>,
        pa1: PA1<MODE>,
        pa2: PA2<MODE>,
        pa3: PA3<MODE>,
        pa4: PA4<MODE>,
        pa5: PA5<MODE>,
        pa6: PA6<MODE>,
        pa7: PA7<MODE>,
        pa8: PA8<MODE>,
        pc0: PC0<MODE>,
        pc1: PC1<MODE>,
        pc2: PC2<MODE>,
        pc3: PC3<MODE>,
        pc4: PC4<MODE>,
        pc5: PC5<MODE>,
    ) -> Self {
        let mut r0 = pa1.into_open_drain_output();
        let mut r1 = pa0.into_open_drain_output();
        let mut r2 = pa2.into_open_drain_output();
        let mut r3 = pa3.into_open_drain_output();
        let mut r4 = pa4.into_open_drain_output();
        let mut r5 = pa5.into_open_drain_output();
        let mut r6 = pa6.into_open_drain_output();
        let mut r7 = pa7.into_open_drain_output();
        let mut r8 = pa8.into_open_drain_output();

        r0.set_high().unwrap();
        r1.set_high().unwrap();
        r2.set_high().unwrap();
        r3.set_high().unwrap();
        r4.set_high().unwrap();
        r5.set_high().unwrap();
        r6.set_high().unwrap();
        r7.set_high().unwrap();
        r8.set_high().unwrap();

        let rows = [
            r0.downgrade(),
            r1.downgrade(),
            r2.downgrade(),
            r3.downgrade(),
            r4.downgrade(),
            r5.downgrade(),
            r6.downgrade(),
            r7.downgrade(),
            r8.downgrade(),
        ];

        let columns = KeyColumns(
            pc0.into_pull_up_input(),
            pc1.into_pull_up_input(),
            pc2.into_pull_up_input(),
            pc3.into_pull_up_input(),
            pc4.into_pull_up_input(),
            pc5.into_pull_up_input(),
        );

        Self { rows, columns }
    }

    pub fn scan(&mut self, delay: &mut impl DelayUs<u32>) -> [u8; 9] {
        let mut state = [
            0b111111, 0b000101, 0b111111, 0b111111, 0b111111, 0b011111, 0b011111, 0b011111,
            0b011111,
        ];

        for (row_pin, row_state) in self.rows.iter_mut().zip(&mut state) {
            row_pin.set_low().unwrap();
            delay.delay_us(10);
            *row_state &= !self.columns.read();
            row_pin.set_high().unwrap();
        }

        state
    }

    pub fn pressed(&mut self, delay: &mut impl DelayUs<u32>) -> Vec<Key, 46> {
        let mut keys: Vec<Key, 46> = Vec::new();
        let state = self.scan(delay);
        for (n, row) in state.iter().enumerate() {
            let start = 0x10 * n as u8;
            for col in [1u8, 2, 4, 8, 16, 32].iter() {
                if let Some(key) = row_to_key(start, *row, *col) {
                    keys.push(key).unwrap();
                }
            }
        }
        keys
    }
}

fn row_to_key(start: u8, row: u8, col: u8) -> Option<Key> {
    let key = match col {
        1 => 1,
        2 => 2,
        4 => 3,
        8 => 4,
        16 => 5,
        32 => 6,
        _ => return None,
    };
    if row & col == col {
        Key::from_u8(start + key)
    } else {
        None
    }
}
