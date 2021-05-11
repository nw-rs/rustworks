use stm32f7xx_hal::delay::Delay;
use stm32f7xx_hal::prelude::OutputPin;
use stm32f7xx_hal::prelude::_embedded_hal_blocking_delay_DelayUs;

pub struct PWMPin<P> {
    pin: P,
}

impl<P: OutputPin> PWMPin<P> {
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    pub fn send_pulses(&mut self, pulses: u32, delay: &mut Delay) {
        for _ in 0..pulses {
            let _ = self.pin.set_low();
            delay.delay_us(20u8);
            let _ = self.pin.set_high();
            delay.delay_us(20u8);
        }
    }

    pub fn off(&mut self) {
        let _ = self.pin.set_low();
    }
}

pub struct Led<RP: OutputPin, GP: OutputPin, BP: OutputPin> {
    red_pin: PWMPin<RP>,
    green_pin: PWMPin<GP>,
    blue_pin: PWMPin<BP>,
}

impl<RP: OutputPin, GP: OutputPin, BP: OutputPin> Led<RP, GP, BP> {
    pub fn new(red: RP, green: GP, blue: BP) -> Self {
        Self {
            red_pin: PWMPin::new(red),
            green_pin: PWMPin::new(green),
            blue_pin: PWMPin::new(blue),
        }
    }
    pub fn set_rgb(&mut self, delay: &mut Delay, red: bool, green: bool, blue: bool) {
        if red {
            self.red_pin.send_pulses(1, delay);
        } else {
            self.red_pin.off();
        }
        if green {
            self.green_pin.send_pulses(1, delay);
        } else {
            self.green_pin.off();
        }
        if blue {
            self.blue_pin.send_pulses(1, delay);
        } else {
            self.blue_pin.off();
        }
    }

    pub fn red(&mut self, delay: &mut Delay) {
        self.set_rgb(delay, true, false, false)
    }

    pub fn green(&mut self, delay: &mut Delay) {
        self.set_rgb(delay, false, true, false)
    }

    pub fn blue(&mut self, delay: &mut Delay) {
        self.set_rgb(delay, false, false, true)
    }

    pub fn off(&mut self) {
        self.red_pin.off();
        self.green_pin.off();
        self.blue_pin.off();
    }
}
