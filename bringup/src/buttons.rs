use defmt::*;
use embassy_rp::adc::{Adc, Config, Channel, InterruptHandler};
use embassy_time::{Duration, Timer};
use embassy_rp::bind_interrupts;
use {defmt_rtt as _, panic_probe as _};

use crate::{AnalogResources};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[derive(Debug)]
enum Button {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
}

fn decode_dir(raw: i32) -> Option<Button> {
    let map = [(1036, Button::Up),
               (1668, Button::Down),
               (2066, Button::Left),
               (3280, Button::Right)];
    let tol = 100;

    let mut res = None;
    for m in map {
        let (thr, def) = m;
        if raw > (thr - tol) && (raw < (thr + tol)) {
            res = Some(def);
        }
    }

    res
}

fn decode_sel(raw: i32) -> Option<Button> {
    let map = [(1036, Button::Select),
               (1668, Button::Back)];
    let tol = 100;

    let mut res = None;
    for m in map {
        let (thr, def) = m;
        if raw > (thr - tol) && (raw < (thr + tol)) {
            res = Some(def);
        }
    }

    res
}

pub async fn test_buttons(r: AnalogResources) {
    let mut adc = Adc::new(r.adc, Irqs, Config::default());

    let mut dir = Channel::new_pin(r.direction_mux, embassy_rp::gpio::Pull::None);
    let mut sel = Channel::new_pin(r.select_mux, embassy_rp::gpio::Pull::None);

    loop {
        match adc.read(&mut dir).await {
            Ok(raw) => {
                if let Some(b) = decode_dir(raw as i32) {
                    info!("Raw ADC: {}, button: {:?}", raw, defmt::Debug2Format(&b));
                }
            }
            Err(e) => {
                error!("ADC read error: {:?}", defmt::Debug2Format(&e));
            }
        }

        match adc.read(&mut sel).await {
            Ok(raw) => {
                if let Some(b) = decode_sel(raw as i32) {
                    info!("Raw ADC: {}, button: {:?}", raw, defmt::Debug2Format(&b));
                }
            }
            Err(e) => {
                error!("ADC read error: {:?}", defmt::Debug2Format(&e));
            }
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}
