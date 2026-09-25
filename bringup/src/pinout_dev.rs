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
    clock: ClockResources {
        // FIXME: implement clock-out
        out: PIN_23,
    },

    i2c: I2CResources {
        i2c: I2C1,              // I2C1 on prod
        sda: PIN_6,             // 18 on prod
        scl: PIN_7,             // 19 on prod
        reset: PIN_19,
        interrupt: PIN_20,
    },

    dac: DacResources {
        dma: DMA_CH0,
        pio: PIO0,
        sclk: PIN_16,
        lrck: PIN_17,
        din: PIN_18,
    },

    // spi: SpiResources {
    //     // for sd
    //     spi: SPI0,
    //     sck: PIN_2,
    //     mosi: PIN_3,
    //     miso: PIN_4,

    //     cs_sd: PIN_5,
    //     cs_disp: PIN_9,
    // }

    spi: SpiResources {
        // for display
        spi: SPI1,
        sck: PIN_10,
        mosi: PIN_11,
        miso: PIN_24,

        cs_sd: PIN_5,
        cs_disp: PIN_9,
    }

    disp: DisplayResources {
        dc: PIN_13,
        res: PIN_12,
        blk: PIN_14,
    }
}
