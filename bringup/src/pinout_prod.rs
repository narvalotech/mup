// Motherboard pinout
//
// DISP
// P09 - DISP_DC
// P10 - DISP_RES
// P11 - DISP_BLK
//
// SD/DISP (SPI-1)
// P13 - DISP_CS
// P17 - SD_CS
// P14 - SD_SCK
// P15 - SD_MOSI (DI)
// P12 - SD_MISO (DO)
//
// EXT/DSP (SPI-PIO)
// P00 - EXT_CS
// P01 - DSP_EXT_INT
// P05 - DSP_CS
// P06 - DSP_SCK
// P07 - DSP_MISO
// P08 - DSP_MOSI
//
// USB HOST
// P02 - USB_PIO_N
// P03 - USB_PIO_P
//
// CAPSENSE - I2C-1
// P04 - CAP_INT
// P16 - DAC_INT
// P18 - I2C_SDA
// P19 - I2C_SCL
//
// DAC
// P24 - DAC_RST
// P25 - DAC_CLKOUT (is this an input?)
// P21 - SCLK
// P20 - LRCK // FIXME: swap with SCLK
// P22 - SDIN
// P23 - MCLK
//
// BUTTONS
// P26 - A0 - SW_SELECT
// P27 - A1 - SW_DIRECTION
// P28 - A2 - CHRG
// P29 - A3 - VBAT_DIV


use embassy_rp::{Peri, peripherals};
use assign_resources::assign_resources;


assign_resources! {
    clock: ClockResources {
        // FIXME: implement clock-out
        out: PIN_23,
    },

    i2c: I2CResources {
        i2c: I2C1,
        sda: PIN_18,
        scl: PIN_19,
        reset: PIN_24,
        interrupt: PIN_16,
    },

    dac: DacResources {
        dma: DMA_CH0,
        pio: PIO0,
        sclk: PIN_20,
        lrck: PIN_21,
        din: PIN_22,
    },

    spi: SpiResources {
        spi: SPI1,
        sck: PIN_14,
        mosi: PIN_15,
        miso: PIN_12,

        cs_sd: PIN_17,
        cs_disp: PIN_13,
    },

    disp: DisplayResources {
        dc: PIN_9,
        res: PIN_10,
        blk: PIN_11,
    }
}

use embassy_rp::spi::{Spi, Blocking};

use embassy_rp::peripherals::SPI1;
pub type SpiBus = Spi<'static, SPI1, Blocking>;
