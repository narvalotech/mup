
use defmt::*;
use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_rp::gpio::{Level, Output};
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
use panic_probe as _;

// spi stuff
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;

use crate::{SpiBus, DisplayResources};

type DisplaySpi = SpiDeviceWithConfig<'static, NoopRawMutex, SpiBus, Output<'static>>;

pub async fn test_display(rd: DisplayResources, display_spi: DisplaySpi) {
    info!("Test display");

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
