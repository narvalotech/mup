#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Timer};
use panic_probe as _;

// spi stuff
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_rp::spi::{Spi, Config as SpiConfig};
use embassy_rp::spi;
use embassy_rp::gpio::{Output, Level};
use static_cell::StaticCell;
use core::cell::RefCell;

pub mod sd;
pub mod disp;
pub mod dac;
pub mod dac_init;
pub mod pinout_prod;
// pub mod pinout_dev;

use crate::pinout_prod as pinout;
// use crate::pinout_dev as pinout;
use pinout::{SpiBus, SpiResources, ClockResources, DisplayResources, I2CResources, DacResources, AssignedResources};

#[allow(unused_imports)]
use crate::sd::test_sd;
#[allow(unused_imports)]
use crate::disp::test_display;
#[allow(unused_imports)]
use crate::dac::test_dac;
#[allow(unused_imports)]
use crate::dac_init::test_dac_init;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: embassy_rp::block::ImageDef = embassy_rp::block::ImageDef::secure_exe();

const DISPLAY_FREQ: u32 = 32_000_000;
const SD_FREQ: u32 = 400_000;

static SPI_BUS: StaticCell<Mutex<NoopRawMutex, RefCell<SpiBus>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    let spi = Spi::new_blocking(
        r.spi.spi,
        r.spi.sck,
        r.spi.mosi,
        r.spi.miso,
        SpiConfig::default(),
    );
    let spi_bus: &'static Mutex<NoopRawMutex, RefCell<SpiBus>> =
    SPI_BUS.init(Mutex::new(RefCell::new(spi)));

    info!("Hello!");
    Timer::after_millis(500).await;

    let mut display_cfg = SpiConfig::default();
    display_cfg.frequency = DISPLAY_FREQ;
    display_cfg.phase = spi::Phase::CaptureOnSecondTransition;
    display_cfg.polarity = spi::Polarity::IdleHigh;
    let cs_display = Output::new(r.spi.cs_disp, Level::High);
    let _spi_dev_display = SpiDeviceWithConfig::new(spi_bus, cs_display, display_cfg);
    // test_display(r.disp, spi_dev_display).await;

    let mut sd_cfg = SpiConfig::default();
    sd_cfg.frequency = SD_FREQ;
    let cs_sd = Output::new(r.spi.cs_sd, Level::High);
    let spi_dev_sd = SpiDeviceWithConfig::new(spi_bus, cs_sd, sd_cfg);
    test_sd(spi_dev_sd).await;

    let _dac = test_dac_init(r.i2c).await; // keep dac driver alive
    test_dac(r.dac).await;

    loop {
        Timer::after_secs(1).await;
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
