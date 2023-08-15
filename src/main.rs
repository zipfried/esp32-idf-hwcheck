mod led;

use std::{thread::sleep, time::Duration};

use anyhow::{bail, Result};
use log::info;
use rgb::RGB8;

use crate::led::WS2812RMT;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    psk: &'static str,
}

fn main() -> Result<()> {
    let config = CONFIG;

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = match Peripherals::take() {
        Some(p) => p,
        None => bail!("failed to take peripherals"),
    };
    let sysloop = EspSystemEventLoop::take()?;

    let mut led = WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0)?;

    led.set_pixel(RGB8::new(50, 0, 0))?;

    let mut esp_wifi = EspWifi::new(
        peripherals.modem,
        sysloop.clone(),
        Some(EspDefaultNvsPartition::take()?),
    )?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
    wifi.start()?;
    let aps = wifi.scan()?;
    let channel = match aps.into_iter().find(|ap| ap.ssid == config.ssid) {
        Some(ap) => {
            info!("found ap {}", config.ssid);
            Some(ap.channel)
        }
        None => bail!("failed to find ap {}", config.ssid),
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: config.ssid.into(),
        password: config.psk.into(),
        auth_method: AuthMethod::WPAWPA2Personal,
        channel,
        ..Default::default()
    }))?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("DHCP Info: {:?}", ip_info);

    loop {
        led.set_pixel(RGB8::new(0, 50, 0))?;
        sleep(Duration::from_millis(500));

        led.set_pixel(RGB8::new(0, 0, 0))?;
        sleep(Duration::from_millis(500));
    }
}
