use core::mem;

use defmt_rtt as _;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::i2s::{PioI2sOut, PioI2sOutProgram};
use embassy_rp::{bind_interrupts, dma};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use panic_probe as _;
use static_cell::StaticCell;

use crate::DacResources;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

const SAMPLE_RATE: u32 = 48_000;
const BIT_DEPTH: u32 = 16;

const LEFT_FREQ: u32 = 220; // A3
const RIGHT_FREQ: u32 = 440; // A4, one octave above left

fn phase_step(freq_hz: u32) -> i32 {
    ((freq_hz << 16) / SAMPLE_RATE) as i32
}

fn triangle_sample(phase: i32) -> i32 {
    (phase as i16 as i32).abs() - 16384
}

pub async fn test_dac(rd: DacResources) {
    // Setup pio state machine for i2s output
    let Pio { mut common, sm0, .. } = Pio::new(rd.pio, Irqs);

    // Library pinout limitation:
    // LRCK = BCLK + 1
    // => motherboard has LRCK = BCLK - 1 :'(
    // To test PIO mod, swap the two wires on the devboard
    let bit_clock_pin = rd.sclk;
    let left_right_clock_pin = rd.lrck;
    let data_pin = rd.din;

    let program = PioI2sOutProgram::new(&mut common);
    let mut i2s = PioI2sOut::new(
        &mut common,
        sm0,
        rd.dma,
        Irqs,
        data_pin,
        bit_clock_pin,
        left_right_clock_pin,
        SAMPLE_RATE,
        BIT_DEPTH,
        &program,
    );
    i2s.start();

    // create two audio buffers (back and front) which will take turns being
    // filled with new audio data and being sent to the pio fifo using dma
    const BUFFER_SIZE: usize = 960;
    static DMA_BUFFER: StaticCell<[u32; BUFFER_SIZE * 2]> = StaticCell::new();
    let dma_buffer = DMA_BUFFER.init_with(|| [0u32; BUFFER_SIZE * 2]);
    let (mut back_buffer, mut front_buffer) = dma_buffer.split_at_mut(BUFFER_SIZE);

    let left_step = phase_step(LEFT_FREQ);
    let right_step = phase_step(RIGHT_FREQ);
    let mut left_phase: i32 = 0;
    let mut right_phase: i32 = 0;

    loop {
        // trigger transfer of front buffer data to the pio fifo
        // but don't await the returned future, yet
        let dma_future = i2s.write(front_buffer);

        // fill back buffer with fresh audio samples before awaiting the dma future
        for s in back_buffer.iter_mut() {
            left_phase = (left_phase + left_step) & 0xffff;
            right_phase = (right_phase + right_step) & 0xffff;

            let left_sample = triangle_sample(left_phase);
            let right_sample = triangle_sample(right_phase);

            // pack left into upper 16 bits, right into lower 16 bits;
            *s = ((left_sample as u16 as u32) << 16) | (right_sample as u16 as u32);
        }

        // now await the dma future. once the dma finishes, the next buffer needs to be queued
        // within DMA_DEPTH / SAMPLE_RATE = 8 / 48000 seconds = 166us
        dma_future.await;
        mem::swap(&mut back_buffer, &mut front_buffer);
    }
}
