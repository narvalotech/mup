//! Init SD card over SPI and list root directory

use defmt::*;
use defmt_rtt as _;
use embassy_time::{Delay};
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use core::ops::ControlFlow;
use panic_probe as _;

// spi stuff
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::spi::{Config as SpiConfig};
use embassy_rp::gpio::Output;

use crate::SpiBus;

/// embedded-sdmmc needs a time source for file timestamps. We don't have an
/// RTC here, so just return a fixed bogus time.
struct DummyTimesource();

impl TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

type SdSpi = SpiDeviceWithConfig<'static, NoopRawMutex, SpiBus, Output<'static>>;

pub async fn test_sd(sd_spi: SdSpi) {
    info!("Test SD card");

    let sdcard = SdCard::new(sd_spi, Delay);

    info!(
        "Card size: {} bytes",
        unwrap!(sdcard.num_bytes().map_err(|e| info!("sd error: {:?}", Debug2Format(&e))))
    );

    {
        // It's now safe to raise the clock for faster block reads/writes.
        let mut fast_config = SpiConfig::default();
        fast_config.frequency = 16_000_000;
        sdcard.spi(|spi_dev| spi_dev.set_config(fast_config));
    }

    let volume_mgr = VolumeManager::new(sdcard, DummyTimesource());
    let volume0 = unwrap!(volume_mgr.open_volume(VolumeIdx(0)).map_err(|_| ()));
    let root_dir = unwrap!(volume0.open_root_dir().map_err(|_| ()));

    info!("Files on SD card:");
    unwrap!(root_dir
    .iterate_dir(|entry| {
        info!(
            "  {} ({} bytes){}",
            Display2Format(&entry.name),
            entry.size,
            if entry.attributes.is_directory() { " [dir]" } else { "" }
        );
        ControlFlow::Continue(())
    })
    .map_err(|_| ()));
}
