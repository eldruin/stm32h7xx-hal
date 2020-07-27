//! Example for using ADC3 to read the internal temperature sensor
//!
//! For an example of using ADC1, see examples/adc.rs
//! For an example of using ADC1 and ADC2 together, see examples/adc12.rs

#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_itm;

use cortex_m_rt::entry;

use stm32h7xx_hal::{
    adc,
    delay::Delay,
    pac,
    prelude::*,
    signature::{TS_CAL_110, TS_CAL_30},
};

use cortex_m_log::println;
use cortex_m_log::{
    destination::Itm, printer::itm::InterruptSync as InterruptSyncItm,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut log = InterruptSyncItm::new(Itm::new(cp.ITM));

    // Constrain and Freeze power
    println!(log, "Setup PWR...                  ");
    let pwr = dp.PWR.constrain();
    let vos = pwr.freeze();

    // Constrain and Freeze clock
    println!(log, "Setup RCC...                  ");
    let rcc = dp.RCC.constrain();

    let ccdr = rcc
        .sys_ck(100.mhz())
        .pll2_p_ck(4.mhz()) // Default adc_ker_ck_input
        .freeze(vos, &dp.SYSCFG);

    println!(log, "");
    println!(log, "stm32h7xx-hal example - Temperature");
    println!(log, "");

    let mut delay = Delay::new(cp.SYST, ccdr.clocks);

    // Setup ADC
    let mut adc3 =
        adc::Adc::adc3(dp.ADC3, &mut delay, ccdr.peripheral.ADC3, &ccdr.clocks);
    adc3.set_resolution(adc::Resolution::SIXTEENBIT);

    // Setup Temperature Sensor on the disabled ADC
    let mut channel = adc::Temperature::new();
    channel.enable(&adc3);
    delay.try_delay_us(25_u16).unwrap();
    let mut adc3 = adc3.enable();

    let vdda = 2.500; // Volts

    loop {
        let word: u32 = adc3
            .try_read(&mut channel)
            .expect("Temperature read failed.");

        // Average slope
        let cal = (110.0 - 30.0)
            / (TS_CAL_110::get().read() - TS_CAL_30::get().read()) as f32;
        // Calibration values are measured at VDDA = 3.3 V ± 10 mV
        let word_3v3 = word as f32 * vdda / 3.3;
        // Linear interpolation
        let temperature =
            cal * (word_3v3 - TS_CAL_30::get().read() as f32) + 30.0;

        println!(
            log,
            "ADC reading: {}, Temperature: {:.1} °C", word, temperature
        );
    }
}
