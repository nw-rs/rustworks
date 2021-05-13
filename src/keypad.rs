use embedded_hal::blocking::delay::DelayUs;
use stm32f7xx_hal::gpio::gpioa::{PA, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8};
use stm32f7xx_hal::gpio::{Input, OpenDrain, Output, PullUp};
use stm32f7xx_hal::{
    gpio::{
        gpioc::{PC0, PC1, PC2, PC3, PC4, PC5},
        Floating,
    },
    prelude::OutputPin,
};

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
        columns as u8 & 0x3f
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
        let mut rows: [u8; 9] = [0; 9];
        for (n, row) in self.rows.iter_mut().enumerate() {
            row.set_low().unwrap();
            delay.delay_us(10);
            rows[n] = self.columns.read();
            row.set_high().unwrap();
        }
        rows[1] &= 0b000101;
        for row in rows[5..].iter_mut() {
            *row &= 0b011111;
        }
        rows
    }
}
