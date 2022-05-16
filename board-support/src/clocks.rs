use fugit::HertzU32;
use stm32f7xx_hal::rcc::{HSEClock, HSEClockMode, RccExt, PLLP};

use crate::hal::pac::{FLASH, PWR, RCC};
use crate::hal::rcc::Clocks;

pub const HSE: u32 = 8;
pub const PLL_M: u8 = 8;
pub const PLL_N: u16 = 384;
pub const PLL_P: u32 = 2;
pub const PLL_Q: u8 = 8;
pub const SYSCLK: u32 = 192_000_000;
pub const SSCG_MODPER: u16 = 250;

pub const SSCG_INCSTEP: u16 = 25;

pub fn init_clocks(rcc: RCC, pwr: &mut PWR, flash: &mut FLASH) -> Clocks {
    /* System clock
     * Configure the CPU at 192 MHz and USB at 48 MHz. */

    /* After reset, the device is using the high-speed internal oscillator (HSI)
     * as a clock source, which runs at a fixed 16 MHz frequency. The HSI is not
     * accurate enough for reliable USB operation, so we need to use the external
     * high-speed oscillator (HSE). */

    unsafe {
        // Enable the HSI and wait for it to be ready
        rcc.cr.modify(|r, w| w.bits(r.bits()).hsion().set_bit());
        while !rcc.cr.read().hsirdy().bit_is_set() {}

        // Enable the HSE and wait for it to be ready
        rcc.cr.modify(|r, w| w.bits(r.bits()).hseon().set_bit());
        while !rcc.cr.read().hserdy().bit_is_set() {}

        // Enable PWR peripheral clock
        rcc.apb1enr
            .modify(|r, w| w.bits(r.bits()).pwren().set_bit());

        rcc.sscgr.write(|w| {
            w.modper()
                .bits(SSCG_MODPER)
                .incstep()
                .bits(SSCG_INCSTEP)
                .spreadsel()
                .center()
                .sscgen()
                .set_bit()
        });

        rcc.pllcfgr.modify(|r, w| {
            w.bits(r.bits())
                .pllm()
                .bits(PLL_M)
                .plln()
                .bits(PLL_N)
                .pllq()
                .bits(PLL_Q)
                .pllsrc()
                .set_bit()
        });

        rcc.cr.modify(|r, w| w.bits(r.bits()).pllon().set_bit());

        pwr.cr1.modify(|r, w| w.bits(r.bits()).oden().set_bit());
        while !pwr.csr1.read().odrdy().bit_is_set() {}

        pwr.cr1.modify(|r, w| w.bits(r.bits()).odswen().set_bit());
        while !pwr.csr1.read().odswrdy().bit_is_set() {}

        pwr.cr1.modify(|r, w| w.bits(r.bits()).vos().scale1());
        while !pwr.csr1.read().vosrdy().bit_is_set() {}

        flash.acr.modify(|r, w| {
            w.bits(r.bits())
                .latency()
                .ws7()
                .prften()
                .set_bit()
                .arten()
                .set_bit()
        });

        rcc.cfgr
            .modify(|r, w| w.bits(r.bits()).ppre1().div4().ppre2().div2());

        while !rcc.cr.read().pllrdy().bit_is_set() {}

        rcc.cfgr.modify(|r, w| w.bits(r.bits()).sw().pll());
        while !rcc.cfgr.read().sws().is_pll() {}

        rcc.cr.modify(|r, w| w.bits(r.bits()).hsion().clear_bit());

        rcc.ahb1enr.write(|w| {
            w.gpioaen()
                .set_bit()
                .gpioben()
                .set_bit()
                .gpiocen()
                .set_bit()
                .gpioden()
                .set_bit()
                .gpioeen()
                .set_bit()
                .dma2en()
                .set_bit()
        });

        rcc.ahb2enr
            .modify(|r, w| w.bits(r.bits()).otgfsen().set_bit());

        rcc.ahb3enr
            .modify(|r, w| w.bits(r.bits()).fmcen().set_bit());

        rcc.apb1enr.modify(|r, w| {
            w.bits(r.bits())
                .tim3en()
                .set_bit()
                .pwren()
                .set_bit()
                .rtcapben()
                .set_bit()
        });

        rcc.apb2enr.write(|w| {
            w.adc1en()
                .set_bit()
                .syscfgen()
                .set_bit()
                .usart6en()
                .set_bit()
        });

        rcc.ahb1lpenr.write(|w| {
            w.gpioalpen() // charging / usb / beyboard pins
                .set_bit()
                .gpioblpen() // led pins
                .set_bit()
                .gpioclpen() // led / keyboard pins
                .set_bit()
                .gpiodlpen() // display pins
                .set_bit()
                .gpioelpen() // keyboard / battery pins
                .set_bit()
        });

        rcc.ahb2lpenr.reset();

        rcc.ahb3lpenr.reset();

        rcc.apb1lpenr.write(|w| w.tim3lpen().set_bit());

        rcc.apb2lpenr.reset();
    }

    
    let rcc_constrained = rcc.constrain();
    let clocks = rcc_constrained
        .cfgr
        .hse(HSEClock::new(HertzU32::MHz(8), HSEClockMode::Oscillator))
        .use_pll()
        .pllm(PLL_M)
        .plln(PLL_N)
        .pllp(PLLP::Div2)
        .pllq(PLL_Q)
        .hclk(HertzU32::Hz(SYSCLK))
        .sysclk(HertzU32::Hz(SYSCLK))
        .freeze();

    clocks
}
