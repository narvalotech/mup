use core::cell::RefCell;

use defmt::*;
use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi;
use embassy_rp::spi::{Blocking, Spi};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Timer, Delay};
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use mipidsi::Builder;
use mipidsi::models::ST7789;
use mipidsi::options::{Orientation, Rotation};
use static_cell::StaticCell;
use panic_probe as _;

use crate::DisplayResources;

const DISPLAY_FREQ: u32 = 32_000_000;

pub async fn test_display(rd: DisplayResources) {
    info!("Test display");

    let display_cs_output = Output::new(rd.cs, Level::High);

    // create SPI
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    let spi = Spi::new_blocking_txonly(rd.spi, rd.sck, rd.mosi, display_config.clone());

    // I don't really understand all this mutex/refcell stuff yet..
    static SPI_BUS: StaticCell<Mutex<NoopRawMutex, RefCell<Spi<'static, embassy_rp::peripherals::SPI1, Blocking>>>> =
        StaticCell::new();
    let spi_bus_mutex = SPI_BUS.init(Mutex::new(RefCell::new(spi)));
    let display_spi = SpiDeviceWithConfig::new(spi_bus_mutex, display_cs_output, display_config);

    let dcx = Output::new(rd.dc, Level::Low);
    let rst = Output::new(rd.res, Level::Low);
    // dcx: 0 = command, 1 = data

    // Enable LCD backlight
    let _bl = Output::new(rd.blk, Level::High);

    // display interface abstraction from SPI and DC
    let di = SPIInterface::new(display_spi, dcx);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(ST7789, di)
        .display_size(172, 320)
        .reset_pin(rst)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut Delay)
        .unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    let raw_image_data = ImageRawLE::new(include_bytes!("./ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(34, 68));

    // Display the image
    ferris.draw(&mut display).unwrap();

    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    Text::new(
        "Hello embedded_graphics \n + embassy + RP235x!",
        Point::new(10, 20),
        style,
    )
    .draw(&mut display)
    .unwrap();

    Timer::after_secs(3).await;

    info!("Display test over");
}
