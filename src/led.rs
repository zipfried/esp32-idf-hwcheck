use std::time::Duration;

use anyhow::Result;
use rgb::RGB8;

use esp_idf_hal::{
    gpio::OutputPin,
    peripheral::Peripheral,
    rmt::{config::TransmitConfig, FixedLengthSignal, PinState, Pulse, RmtChannel, TxRmtDriver},
};

pub struct WS2812RMT<'a> {
    t0h: Pulse,
    t0l: Pulse,
    t1h: Pulse,
    t1l: Pulse,
    tx_rmt_driver: TxRmtDriver<'a>,
}

impl<'a> WS2812RMT<'a> {
    pub fn new(
        led: impl Peripheral<P = impl OutputPin> + 'a,
        channel: impl Peripheral<P = impl RmtChannel> + 'a,
    ) -> Result<Self> {
        let config = TransmitConfig::new().clock_divider(2);
        let mut tx_rmt_driver = TxRmtDriver::new(channel, led, &config)?;

        let ticks_hz = tx_rmt_driver.counter_clock()?;
        let t0h = Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?;
        let t0l = Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?;
        let t1h = Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?;
        let t1l = Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?;

        tx_rmt_driver.start_blocking(&FixedLengthSignal::<24>::new())?; // reset

        Ok(Self {
            t0h,
            t0l,
            t1h,
            t1l,
            tx_rmt_driver,
        })
    }

    pub fn set_pixel(&mut self, rgb: RGB8) -> Result<()> {
        let color: u32 = (rgb.g as u32) << 16 | (rgb.r as u32) << 8 | (rgb.b as u32); // GRB
        let mut signal = FixedLengthSignal::<24>::new();
        for i in (0..24).rev() {
            let sel = 2u32 << i;
            let bit = sel & color != 0;
            let (high, low) = if bit {
                (self.t1h, self.t1l)
            } else {
                (self.t0h, self.t0l)
            };
            signal.set(23 - i, &(high, low))?;
        }
        self.tx_rmt_driver.start_blocking(&signal)?;

        Ok(())
    }
}
