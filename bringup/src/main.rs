//! This example shows reading the OTP constants on the RP235x.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::otp;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _ = embassy_rp::init(Default::default());

    Timer::after_millis(10).await; // flash read delay. might be unnecessary.

    let chip_id = unwrap!(otp::get_chipid().map_err(|_| ()));
    let private_rand = unwrap!(otp::get_private_random_number().map_err(|_| ()));

    info!("Unique id:{:X}", chip_id);
    info!("Private Rand:{:X}", private_rand);

    loop {
        Timer::after_secs(1).await;
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
