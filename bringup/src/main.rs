#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Timer};
use panic_probe as _;

pub mod sd;
pub mod disp;

// use crate::sd::test_sd;
use crate::disp::test_display;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: embassy_rp::block::ImageDef = embassy_rp::block::ImageDef::secure_exe();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Hello!");
    Timer::after_millis(100).await;

    // test_sd(p).await;
    test_display(p).await;

    loop {
        Timer::after_secs(1).await;
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
