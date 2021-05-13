use stm32f7xx_hal::prelude::OutputPin;

pub struct Led<RP: OutputPin, GP: OutputPin, BP: OutputPin> {
    red_pin: RP,
    green_pin: GP,
    blue_pin: BP,
}

impl<RP: OutputPin, GP: OutputPin, BP: OutputPin> Led<RP, GP, BP> {
    pub fn new(red: RP, green: GP, blue: BP) -> Self {
        Self {
            red_pin: red,
            green_pin: green,
            blue_pin: blue,
        }
    }
    pub fn set_rgb(&mut self, red: bool, green: bool, blue: bool) {
        if red {
            let _ = self.red_pin.set_high();
        } else {
            let _ = self.red_pin.set_low();
        }
        if green {
            let _ = self.green_pin.set_high();
        } else {
            let _ = self.green_pin.set_low();
        }
        if blue {
            let _ = self.blue_pin.set_high();
        } else {
            let _ = self.blue_pin.set_low();
        }
    }

    pub fn red(&mut self) {
        self.set_rgb(true, false, false)
    }

    pub fn green(&mut self) {
        self.set_rgb(false, true, false)
    }

    pub fn blue(&mut self) {
        self.set_rgb(false, false, true)
    }

    pub fn off(&mut self) {
        let _ = self.red_pin.set_low();
        let _ = self.green_pin.set_low();
        let _ = self.blue_pin.set_low();
    }
}
