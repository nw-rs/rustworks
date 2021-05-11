use crate::scan_cols;
use heapless::Vec;
use stm32f7xx_hal::gpio::{
    gpioc::{PC0, PC1, PC2, PC3, PC4, PC5},
    Floating,
};
use stm32f7xx_hal::gpio::{Input, Output, PullUp, PushPull};
use stm32f7xx_hal::{
    gpio::gpioa::{PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8},
    prelude::{InputPin, OutputPin},
};

#[derive(Debug)]
pub enum Key {
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,

    B1,
    B3,

    C1,
    C2,
    C3,
    C4,
    C5,
    C6,

    D1,
    D2,
    D3,
    D4,
    D5,
    D6,

    E1,
    E2,
    E3,
    E4,
    E5,
    E6,

    F1,
    F2,
    F3,
    F4,
    F5,

    G1,
    G2,
    G3,
    G4,
    G5,

    H1,
    H2,
    H3,
    H4,
    H5,

    I1,
    I2,
    I3,
    I4,
    I5,
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
        let mut row_a = pa1.into_push_pull_output();
        let mut row_b = pa0.into_push_pull_output();
        let mut row_c = pa2.into_push_pull_output();
        let mut row_d = pa3.into_push_pull_output();
        let mut row_e = pa4.into_push_pull_output();
        let mut row_f = pa5.into_push_pull_output();
        let mut row_g = pa6.into_push_pull_output();
        let mut row_h = pa7.into_push_pull_output();
        let mut row_i = pa8.into_push_pull_output();

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
            col_1: pc0.into_pull_up_input(),
            col_2: pc1.into_pull_up_input(),
            col_3: pc2.into_pull_up_input(),
            col_4: pc3.into_pull_up_input(),
            col_5: pc4.into_pull_up_input(),
            col_6: pc5.into_pull_up_input(),
        };

        Self { rows, columns }
    }

    pub fn scan(&mut self) -> Vec<Key, 46> {
        let mut keys: Vec<Key, 46> = Vec::new();
        self.rows.row_a.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (A1, col_1),
            (A2, col_2),
            (A3, col_3),
            (A4, col_4),
            (A5, col_5),
            (A6, col_6),
        );
        self.rows.row_a.set_low().unwrap();
        self.rows.row_b.set_high().unwrap();
        scan_cols!(self, keys, (B1, col_1), (B3, col_3),);
        self.rows.row_b.set_low().unwrap();
        self.rows.row_c.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (C1, col_1),
            (C2, col_2),
            (C3, col_3),
            (C4, col_4),
            (C5, col_5),
            (C6, col_6),
        );
        self.rows.row_c.set_low().unwrap();
        self.rows.row_d.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (D1, col_1),
            (D2, col_2),
            (D3, col_3),
            (D4, col_4),
            (D5, col_5),
            (D6, col_6),
        );
        self.rows.row_d.set_low().unwrap();
        self.rows.row_e.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (E1, col_1),
            (E2, col_2),
            (E3, col_3),
            (E4, col_4),
            (E5, col_5),
            (E6, col_6),
        );
        self.rows.row_e.set_low().unwrap();
        self.rows.row_f.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (F1, col_1),
            (F2, col_2),
            (F3, col_3),
            (F4, col_4),
            (F5, col_5),
        );
        self.rows.row_f.set_low().unwrap();
        self.rows.row_g.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (G1, col_1),
            (G2, col_2),
            (G3, col_3),
            (G4, col_4),
            (G5, col_5),
        );
        self.rows.row_g.set_low().unwrap();
        self.rows.row_h.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (H1, col_1),
            (H2, col_2),
            (H3, col_3),
            (H4, col_4),
            (H5, col_5),
        );
        self.rows.row_h.set_low().unwrap();
        self.rows.row_i.set_high().unwrap();
        scan_cols!(
            self,
            keys,
            (I1, col_1),
            (I2, col_2),
            (I3, col_3),
            (I4, col_4),
            (I5, col_5),
        );
        self.rows.row_i.set_low().unwrap();

        keys
    }
}

#[macro_export]
macro_rules! scan_cols {
    ($matrix:expr, $keys:expr, $(($l:ident, $n:ident),)*) => {
        $(
            {
                let is_low = ($matrix.columns.$n).is_low().unwrap();
                if is_low {
                    $keys.push(Key::$l).unwrap();
                }
            }
        )+
    };
}
