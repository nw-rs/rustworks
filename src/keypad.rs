use stm32f7xx_hal::gpio::{
    gpioc::{PC, PC0, PC1, PC2, PC3, PC4, PC5},
    Floating,
};
use stm32f7xx_hal::gpio::{Input, Output, PullUp, PushPull};
use stm32f7xx_hal::{
    gpio::gpioa::{PA, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8},
    prelude::{InputPin, OutputPin},
};

pub struct Keypad {
    rows: [PA<Output<PushPull>>; 9],
    columns: [PC<Input<PullUp>>; 6],
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
        let rows = [
            pa1.into_push_pull_output().downgrade(),
            pa0.into_push_pull_output().downgrade(),
            pa2.into_push_pull_output().downgrade(),
            pa3.into_push_pull_output().downgrade(),
            pa4.into_push_pull_output().downgrade(),
            pa5.into_push_pull_output().downgrade(),
            pa6.into_push_pull_output().downgrade(),
            pa7.into_push_pull_output().downgrade(),
            pa8.into_push_pull_output().downgrade(),
        ];

        let columns = [
            pc0.into_pull_up_input().downgrade(),
            pc1.into_pull_up_input().downgrade(),
            pc2.into_pull_up_input().downgrade(),
            pc3.into_pull_up_input().downgrade(),
            pc4.into_pull_up_input().downgrade(),
            pc5.into_pull_up_input().downgrade(),
        ];

        Self { rows, columns }
    }

    pub fn scan(&mut self) -> [bool; 54] {
        let mut keys: [bool; 54] = [false; 54];
        let mut pos = 0usize;
        for row in self.rows.iter_mut() {
            row.set_high().unwrap();
            for column in self.columns.iter() {
                *keys.get_mut(pos).unwrap() = column.is_low().unwrap();
                pos = pos + 1;
            }
            row.set_low().unwrap();
        }
        keys
    }
}
