use std::path::Path;

use anyhow::{Result, bail};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    psk: &'static str,
}

fn main() -> Result<()> {
    if !Path::new("cfg.toml").exists() {
        bail!("You need to create a `cfg.toml` file with your Wi-Fi credentials.");
    }

    let config = CONFIG;
    if config.ssid == "foo" || config.psk == "bar" {
        bail!("You need to set the Wi-Fi credentials in `cfg.toml`.");
    }

    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;
    Ok(())
}
