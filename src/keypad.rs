use heapless::Vec;
use stm32f7xx_hal::gpio::gpioc::{PC0, PC1, PC2, PC3, PC4, PC5};
use stm32f7xx_hal::gpio::{gpioa::Parts as PartsA, gpioc::Parts as PartsC};
use stm32f7xx_hal::gpio::{Input, Output, PullUp, PushPull};
use stm32f7xx_hal::{
    gpio::gpioa::{PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8},
    prelude::OutputPin,
};

pub enum Keys {
    Left,
    Right,
    Up,
    Down,
    Power,
    Home,
    Ok,
    Back,
    Shift,
    Alpha,
    XNT,
    Var,
    Toolbox,
    Delete,
    EX,
    LN,
    LOG,
    I,
    Comma,
    POW,
    SIN,
    COS,
    TAN,
    PI,
    SQRT,
    SQR,
    Seven,
    Eight,
    Nine,
    LBracket,
    RBracket,
    Four,
    Five,
    Six,
    Multiply,
    Divide,
    One,
    Two,
    Three,
    Add,
    Subtract,
    Zero,
    Dot,
    Exponent,
    Ans,
    EXE,
}

struct KeypadRows {
    row_a: PA1<Output<PushPull>>,
    row_b: PA0<Output<PushPull>>,
    row_c: PA2<Output<PushPull>>,
    row_d: PA3<Output<PushPull>>,
    row_e: PA4<Output<PushPull>>,
    row_f: PA5<Output<PushPull>>,
    row_g: PA6<Output<PushPull>>,
    row_h: PA7<Output<PushPull>>,
    row_i: PA8<Output<PushPull>>,
}

struct KeypadColumns {
    col_1: PC0<Input<PullUp>>,
    col_2: PC1<Input<PullUp>>,
    col_3: PC2<Input<PullUp>>,
    col_4: PC3<Input<PullUp>>,
    col_5: PC4<Input<PullUp>>,
    col_6: PC5<Input<PullUp>>,
}

pub struct Keypad {
    rows: KeypadRows,
    columns: KeypadColumns,
}

impl Keypad {
    pub fn new(gpioa: PartsA, gpioc: PartsC) -> Self {
        let mut row_a = gpioa.pa1.into_push_pull_output();
        let mut row_b = gpioa.pa0.into_push_pull_output();
        let mut row_c = gpioa.pa2.into_push_pull_output();
        let mut row_d = gpioa.pa3.into_push_pull_output();
        let mut row_e = gpioa.pa4.into_push_pull_output();
        let mut row_f = gpioa.pa5.into_push_pull_output();
        let mut row_g = gpioa.pa6.into_push_pull_output();
        let mut row_h = gpioa.pa7.into_push_pull_output();
        let mut row_i = gpioa.pa8.into_push_pull_output();

        row_a.set_low().unwrap();
        row_b.set_low().unwrap();
        row_c.set_low().unwrap();
        row_d.set_low().unwrap();
        row_e.set_low().unwrap();
        row_f.set_low().unwrap();
        row_g.set_low().unwrap();
        row_h.set_low().unwrap();
        row_i.set_low().unwrap();

        let rows = KeypadRows {
            row_a,
            row_b,
            row_c,
            row_d,
            row_e,
            row_f,
            row_g,
            row_h,
            row_i,
        };

        let columns = KeypadColumns {
            col_1: gpioc.pc0.into_pull_up_input(),
            col_2: gpioc.pc1.into_pull_up_input(),
            col_3: gpioc.pc2.into_pull_up_input(),
            col_4: gpioc.pc3.into_pull_up_input(),
            col_5: gpioc.pc4.into_pull_up_input(),
            col_6: gpioc.pc5.into_pull_up_input(),
        };

        Self { rows, columns }
    }

    pub fn poll(&self) -> Vec<Keys, 46> {
        Vec::new()
    }
}
