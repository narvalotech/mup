#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Timer};
use panic_probe as _;

pub mod sd;
pub mod disp;
pub mod dac;
pub mod dac_init;
pub mod pinout_dev;

use crate::pinout_dev as pinout;
use pinout::{DisplayResources, I2CResources, SdResources, DacResources, AssignedResources};

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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Hello!");
    Timer::after_millis(100).await;

    // test_sd(r.sd).await;
    // test_display(r.disp).await;
    // test_dac(r.dac).await;
    test_dac_init(r.i2c).await;

    loop {
        Timer::after_secs(1).await;
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
