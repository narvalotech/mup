// Perfboard pinout
//
// DISP
// P13 - DISP_DC
// P12 - DISP_RES
// P14 - DISP_BLK
//
// DISP (SPI-1)
// P09 - DISP_CS
// P10 - DISP_SCK
// P11 - DISP_MOSI (DI)
//
// SD (SPI-0)
// P05 - SD_CS
// P02 - SD_SCK
// P03 - SD_MOSI
// P04 - SD_MISO
//
// DAC - I2S "0"
// P17 - LRCK / WS
// P16 - SCLK / SCLK
// P18 - SDIN / SD

use embassy_rp::{Peri, peripherals};
use assign_resources::assign_resources;


assign_resources! {
    dac: DacResources {
        dma: DMA_CH0,
        pio: PIO0,
        sclk: PIN_16,
        lrck: PIN_17,
        din: PIN_18,
    },

    sd: SdResources {
        spi: SPI0,              // this will be moved out for prod
        cs: PIN_5,
        sck: PIN_2,
        mosi: PIN_3,
        miso: PIN_4,
    },

    disp: DisplayResources {
        spi: SPI1,
        dc: PIN_13,
        res: PIN_12,
        blk: PIN_14,

        cs: PIN_9,
        sck: PIN_10,
        mosi: PIN_11,
    }
}
