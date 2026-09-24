    // I2C details:
    // - 100/400kHz speed
    // - I2C address: 0b01100xxy
    //   last two bits: 00 (ADR = GND) -> 0b0110000y
    //
    // Write:
    // | 1       | 3        | 1       | n    |
    // | address | register | control | data |
    // control b0: 1 for autoincrement
    // register is big-endian

    // Changes for using PLL clock input through the XTI/MCLK pin
    // Will feed 12 MHz (using GPOUT1)
    // For 22.5792 MCLK:
    // - PLL_REF_PREDIV: 0x02 (factor 4)
    // - PLL_DIV_INT:    0x49
    // - PLL_DIV_FRAC:   0x80 0000
    // - PLL_OUT_DIV:    0x0A
    // - PLL_MODE:       0
    // - PLL_OUT:        22.5792 MHz
    // - PLL_CAL_RATIO:  120
    //
    // MCLK_SRC_SEL = 0x01 (PLL)
    // MCLK_INT = 0 (f 24.576MHz)
    // PDN_PLL = 0

    // PLL power-up sequence:
    // 1. PDN_PLL = 0
    // 2. Set PLL_REF_PREDIV
    // 3. Set PLL_OUT_DIV
    // 4. Set PLL_DIV_FRAC
    // 5. Set PLL_DIV_INT
    // 6. Set PLL_MODE & PLL_CAL_RATION
    // 7. Clear PLL_READY_INT_MASK & PLL_ERROR_INT_MASK
    //    Wait for PLL_READY_INT or PLL_ERROR_INT
    // 8. PLL_START = 1

    // Init sequence with PLL
    // 1. Apply power, assert RESET
    // 2. Wait 1.5ms
    // 3. Configure PLL:
    // 4. [20000] PDN_PLL = 0
    // 5. [40002] PLL_REF_PREDIV
    // 6. [30008] PLL_OUT_DIV
    // 7. [30002] PLL_DIV_FRAC_0 (LSB)
    //    [30003] PLL_DIV_FRAC_1
    //    [30004] PLL_DIV_FRAC_2 (MSB)
    // 8. [30005] PLL_DIV_INT
    // 9. [3001B] PLL_MODE
    // 10.[F0000] read ISR
    // 11.[3000A] PLL_CAL_RATIO
    // 12.[F0010] Enable PLL_READ_INT & PLL_ERROR_INT (=0)
    // 13.[30001] PLL_START=1
    // .. configure ASP
    // 26.[F0000] Wait for PLL_READY_INT=1
    // .. configure DSD & HP
    // .. enable ASP, DSD, DoP interrupts
    // 42.[F0000] Wait for PLL_READY_INT=1
    // 43.[10006] Set MCLK source to 22.5792/PLL = 0x05
    // 45.[1000D] Enable ASP clocks
    // .. power up HP

    // Initialization sequence: Power to I2S PCM playback
    // 1. Apply power, assert RESET
    // 2. Wait 1.5ms
    //
    // 3. Configure XTAL driver [CHANGE TO EXT MCLK]
    // 4. Configure XTAL bias
    //
    // 5. Clear interrupts
    //    [F0000] read reg
    //
    // 6. [F0010] Enable XTAL interrupts = 0xE7
    // 7. [20000] Start XTAL = 0xF6
    //
    // 8. Configure ASP interface
    // 9. [1000B] Set ASP sample rate. 44.1kHz = 0x01
    // 10. [1000C] Set ASP sample bit size. 32b = 0x04
    // 11. ASP numerator (LE)
    //     [40010] = 0x01
    //     [40011] = 0x00
    // 12. ASP denominator (LE)
    //     [40012] = 0x08
    //     [40013] = 0x00
    // 13. ASP LRCK high time (LE)
    //     [40014] = 0x1F
    //     [40015] = 0x00
    // 14. ASP LRCK period (LE)
    //     [40016] = 0x3F
    //     [40017] = 0x00
    // 15. Configure ASP clock [CHANGE TO SLAVE]
    //     [40018] = 0x1C is master
    // 16. Configure ASP frame (for I2S input)
    //     [40019] = 0x0A
    // 17. Set ASP channel location
    //     [50000] = 0x00 (ch1 on sclk0)
    //     [50001] = 0x00 (ch2 on sclk0)
    // 18. Set channel size & enable
    //     [5000A] = 0x07 (size 32b)
    //     [5000B] = 0x0F (size 32b)
    //
    // 19. Configure PCM interface. HPF used. De-emphasis off.
    // 20. [90000] Enable HPF = 0x02
    // 21. [90001] Set vol channel B = 0x00 (0dB)
    // 22. [90002] Set vol channel A = 0x00 (0dB)
    // 23. Configure PCM path signal control
    //     [90003] = 0xEC
    //     [90004] = 0x00
    //
    // 24. Configure HP
    // 25. [B0000] Class H amplifier control = 0x1E
    // 26. [80000] Set HP output to full-scale = 0x30
    // 27. [D0000] Configure HP detect = 0x04
    // 28. [D0000] Enable HP detect = 0xC4
    //
    // 29. Enable interrupts
    // 30. [F0000] read reg (clears IRQ)
    // 31. [F0010] Enable headphone detect IRQ = 0x87
    // 32. [F0011] Enable ASP IRQ = 0x07
    // 33. [F0000] Wait for IRQ (XTAL_READ_INT=1) [CHANGE]
    // 34. [10006] Switch MCLK src to XTAL = 0x04 [CHANGE]
    // 35. Wait 150us
    // 36. [1000D] Enable ASP clocks = 0x02
    //
    // 37. Power up HP (see below)

    // Headphone power up sequence
    // 1. Pop-free power-up settings
    //    [10010] = 0x99
    //    [80032] = 0x20
    // 2. [20000] Power up ASP PDN_ASP = 0
    // 3. [20000] Power up amplifier PDN_HP = 0
    // 4. Wait 12ms
    // 5. Restore default settings
    //    [80032] = 0x00
    //    [10010] = 0x00

    // Headphone power down sequence
    // 1. [F0010] Enable PDN_DONE interrupt
    // 2. [20000] Power down amplifier PDN_HP = 1
    // 3. [F0000] Wait for PDN_DONE_INT
    // 4. [20000] Power down ASP PDN_ASP = 1

use crate::I2CResources;
use embedded_hal_1::digital::{OutputPin, InputPin};
use embedded_hal_async::{delay::DelayNs, i2c::I2c, i2c::Operation};

type I2CAddress = u8;

pub struct DACInterface<I2C, R, I, D>
where
    I2C: I2c,
    R: OutputPin,
    I: InputPin,
    D: DelayNs,
{
    i2c: I2C,
    // 7-bit unshifted address
    // TODO: change to an enum for the ADDR pin
    i2c_address: I2CAddress,
    reset_pin: R,
    interrupt_pin: I,
    delay: D,
}

use device_driver::{RegisterInterfaceBase, AsyncRegisterInterface};

#[derive(Debug)]
pub enum InterfaceError {
    ResetPinError,
    InterruptPinError,
    CommunicationError,
}

impl<I2C: I2c, R: OutputPin, I: InputPin, D: DelayNs> RegisterInterfaceBase
    for DACInterface<I2C, R, I, D>
{
    type Error = InterfaceError;
    type AddressType = u32;
}

impl<I2C: I2c, R: OutputPin, I: InputPin, D: DelayNs> AsyncRegisterInterface
    for DACInterface<I2C, R, I, D>
{
    async fn write_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &device_driver::FieldsetMetadata,
    ) -> Result<(), InterfaceError> {

        let mut reg_addr = [0u8; 3]; // Register addresses are 24 bits
        reg_addr.copy_from_slice(&address.to_be_bytes()[1..4]);

        let control_byte = [0x01u8]; // Enable register address auto-increment

        // Write register, control and data without STOP in the middle
        self.i2c.transaction(self.i2c_address, &mut [
            Operation::Write(&reg_addr),
            Operation::Write(&control_byte),
            Operation::Write(data),
        ]).await.map_err(|_| InterfaceError::CommunicationError)?;

        Ok(())
    }

    async fn read_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &device_driver::FieldsetMetadata,
    ) -> Result<(), InterfaceError> {
        let mut reg_addr = [0u8; 3]; // Register addresses are 24 bits
        reg_addr.copy_from_slice(&address.to_be_bytes()[1..4]);

        let control_byte = [0x01u8]; // Enable register address auto-increment

        // Write register address & control byte.
        self.i2c.transaction(self.i2c_address, &mut [
            Operation::Write(&reg_addr),
            Operation::Write(&control_byte),
        ]).await.map_err(|_| InterfaceError::CommunicationError)?;

        // Datasheet says STOP expected between read & write, so we start
        // another transaction.

        // Read the register value. Mostly a single u8. Multiple registers can
        // be combined as a single "logical register" spanning multiple bytes.
        // In that case, the buffer will be n bytes and the auto-increment
        // (control byte) makes sure we read the next registers.
        self.i2c.transaction(self.i2c_address, &mut [
            Operation::Read(data),
        ]).await.map_err(|_| InterfaceError::CommunicationError)?;

        Ok(())
    }
}

impl<I2C: I2c, R: OutputPin, I: InputPin, D: DelayNs>
    DACInterface<I2C, R, I, D>
{
    pub const fn new(i2c: I2C, reset_pin: R, interrupt_pin: I, delay: D) -> Self {
        Self {
            i2c,
            i2c_address: 0b0110000, // TODO: make configurable
            reset_pin,
            interrupt_pin,
            delay,
        }
    }

    pub async fn reset(&mut self) -> Result<(), InterfaceError> {
        // Reset the chip
        {
            self.reset_pin
                .set_low()
                .map_err(|_| InterfaceError::ResetPinError)?;

            self.delay.delay_us(200).await;

            self.reset_pin
                .set_high()
                .map_err(|_| InterfaceError::ResetPinError)?;

            self.delay.delay_ms(2).await;
        }

        // Do a read of the interrupt pin
        if self.interrupt_pin.is_low().map_err(|_| InterfaceError::InterruptPinError)? {
            // do something, maybe
        }

        Ok(())
    }
}

use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{Config, InterruptHandler};
use embassy_rp::{peripherals::I2C1};
use embassy_rp::gpio::{Level, Pull, Input, Output};
use defmt::*;
use defmt_rtt as _;

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

device_driver::compile!(manifest: "cs43131.ddsl");

pub async fn test_dac_init(rd: I2CResources) {
    let i2c = embassy_rp::i2c::I2c::new_async(rd.i2c, rd.scl, rd.sda, Irqs, Config::default());

    let interface = DACInterface::new(
        i2c,
        Output::new(rd.reset, Level::Low),
        Input::new(rd.interrupt, Pull::Up),
        embassy_time::Delay,
    );

    let mut dac = Cs43131::new(interface);
    info!("toggle reset");
    dac.interface.reset().await.unwrap();

    {
        info!("reading serial number");
        let snr = dac.global().device_id()
                              .read_async().await.unwrap().dev_id();
        info!("serial number: {=u32:x}", snr);
    }

    {
        info!("start init sequence");
        embassy_time::Delay.delay_ms(2).await;

        dac.global().power_down_control()
                    .write_async(|w| {
                        w.set_pdn_clkout(true);
                        w.set_pdn_pll(false);
                        w.set_pdn_xtal(true);
                        w.set_pdn_hp(true);
                        w.set_pdn_dsdif(true);
                        w.set_pdn_asp(true);
                        w.set_pdn_xsp(true);
                    })
                    .await
                    .unwrap();

        embassy_time::Delay.delay_ms(1000).await;

        dac.asp().pll_setting_9().write_async(|w| {
            w.set_pll_ref_prediv(PllRefPrediv::Div1)
        }).await.unwrap();

        dac.pll().pll_setting_6().write_async(|w| {
            w.set_pll_out_div(0x0A)
        }).await.unwrap();

        dac.pll().pll_setting_234().write_async(|w| {
            w.set_pll_div_frac(0x80_0000)
        }).await.unwrap();

        dac.pll().pll_setting_5().write_async(|w| {
            w.set_pll_div_int(0x49)
        }).await.unwrap();

        dac.pll().pll_setting_8().write_async(|w| {
            w.set_pll_mode_bypass(false)
        }).await.unwrap();

        // clear any old interrupts
        let _ = dac.interrupts().status_1().read_async().await.unwrap();
        let _ = dac.interrupts().status_2().read_async().await.unwrap();

        dac.pll().pll_setting_7().write_async(|w| {
            w.set_pll_cal_ratio(120)
        }).await.unwrap();

        dac.interrupts().mask_1().write_async(|w| {
            w.set_pll_ready(true);
            w.set_pll_error(true);
        }).await.unwrap();

        dac.pll().pll_setting_1().write_async(|w| {
            w.set_pll_start(true);
        }).await.unwrap();

        dac.global().serial_port_sample_rate().write_async(|w| {
            w.set_asp_sprate(AspRateKhz::Rate441);
        }).await.unwrap();

        dac.global().serial_port_sample_bit_size().write_async(|w| {
            w.set_asp_spsize(AspBitSize::Bs16);
        }).await.unwrap();

        dac.asp().asp_numerator().write_async(|w| {
            w.set_asp_n(1);
        }).await.unwrap();

        dac.asp().asp_denominator().write_async(|w| {
            w.set_asp_m(8);
        }).await.unwrap();

        dac.asp().asp_lrck_high_time().write_async(|w| {
            // datasheet says 31 for 32-bits
            // we configure 16-bits instead
            w.set_asp_lchi(15);
        }).await.unwrap();

        dac.asp().asp_lrck_period().write_async(|w| {
            // datasheet says 63 for 32-bits
            // we configure 16-bits instead
            w.set_asp_lcpr(31);
        }).await.unwrap();

        dac.asp().asp_clock_configuration().write_async(|w| {
            w.set_asp_lcpol_in(false);
            w.set_asp_lcpol_out(false);
            w.set_asp_scpol_in(true);
            w.set_asp_scpol_out(true);
            w.set_asp_m_sb(false)
        }).await.unwrap();

        dac.asp().asp_frame_configuration().write_async(|w| {
            w.set_asp_fsd(2);   // 1.0 delay
            w.set_asp_5050(false);
            w.set_asp_stp(false);
        }).await.unwrap();

        dac.asp().asp_channel_1_location().write_async(|w| {
            w.set_asp_rx_ch_1(0);
        }).await.unwrap();

        dac.asp().asp_channel_2_location().write_async(|w| {
            w.set_asp_rx_ch_2(0);
        }).await.unwrap();

        dac.asp().asp_channel_1_size_enable().write_async(|w| {
            w.set_asp_rx_ch_1_res(AspRxCh1Res::Bs16);
            w.set_asp_rx_ch_1_en(true);
            w.set_asp_rx_ch_1_ap(false);
        }).await.unwrap();

        dac.asp().asp_channel_2_size_enable().write_async(|w| {
            w.set_asp_rx_ch_2_res(AspRxCh2Res::Bs16);
            w.set_asp_rx_ch_2_en(true);
            w.set_asp_rx_ch_2_ap(true);
        }).await.unwrap();

        dac.headphone_pcm().pcm_filter_option().write_async(|w| {
            w.set_high_pass(true);
        }).await.unwrap();

        dac.headphone_pcm().pcm_volume_b().write_async(|w| {
            w.set_pcm_volume_b(0);  // 0dB
        }).await.unwrap();
        dac.headphone_pcm().pcm_volume_a().write_async(|w| {
            w.set_pcm_volume_a(0);  // 0dB
        }).await.unwrap();

        dac.headphone_pcm().pcm_path_signal_control_1().write_async(|w| {
            w.set_pcm_ramp_down(true);
            w.set_pcm_vol_beqa(true);
            w.set_pcm_szc(PcmSzc::SoftRamp);
            w.set_pcm_amute(true);
            w.set_pcm_amutebeqa(true);
            w.set_pcm_mute_a(false);
            w.set_pcm_mute_b(false);
        }).await.unwrap();

        dac.headphone_pcm().pcm_path_signal_control_2().write_async(|w| {
            w.set_pcm_copy_chan(false);
            w.set_pcm_swap_chan(false);
            w.set_pcm_inv_b(false);
            w.set_pcm_inv_a(false);
        }).await.unwrap();

        dac.headphone_pcm().class_h_control().write_async(|w| {
            w.set_ext_vcpfilt(false);
            w.set_hv_en(true);
            w.set_adpt_pwr(AdptPwr::AdaptToSignal);
        }).await.unwrap();

        dac.headphone_pcm().hp_output_control_1().write_async(|w| {
            w.set_plus_1_db_en(false);
            w.set_hp_in_lp(false);
            w.set_hp_in_en(false);
        }).await.unwrap();

        dac.headphone_pcm().hp_detect().write_async(|w| {
            w.set_hpdetect_fall_dbc_time(HpdetectFallDbcTime::Fall0Ms);
            w.set_hpdetect_rise_dbc_time(HpdetectRiseDbcTime::Rise250Ms);
            w.set_hpdetect_inv(false);
        }).await.unwrap();
        dac.headphone_pcm().hp_detect().write_async(|w| {
            // This write should output 0xC4
            w.set_hpdetect_ctrl(HpdetectCtrl::Enabled);
        }).await.unwrap();

        // clear any old interrupts
        let _ = dac.interrupts().status_1().read_async().await.unwrap();
        let _ = dac.interrupts().status_2().read_async().await.unwrap();
        dac.interrupts().mask_1().write_async(|w| {
            w.set_pll_done(false);
            w.set_pll_ready(false);
            w.set_pll_error(false);
            w.set_dac_ovfl(false);

            w.set_xtal_error(true);
            w.set_xtal_ready(true);
            w.set_hp_detect_plug(true);
            w.set_hp_detect_unplug(true);
        }).await.unwrap();
        dac.interrupts().mask_2().write_async(|w| {
            w.set_asp_nolrck(false);
            w.set_asp_early(false);
            w.set_asp_late(false);
            w.set_asp_error(false);
            w.set_asp_ovfl(false);
        }).await.unwrap();

        embassy_time::Delay.delay_ms(2).await;
        let _ = dac.interrupts().status_1().read_async().await.unwrap();
        // FIXME: loop here waiting for PLL_READY

        dac.global().system_clocking_control().write_async(|w| {
            w.set_mclk_int(false);
            w.set_mclk_src_sel(MclkSrcSel::PllMode);
        }).await.unwrap();

        embassy_time::Delay.delay_ms(2).await;

        // // Don't need to do this as we're in slave mode
        // dac.global().pad_interface_configuration().write_async(|w| {
        //     w.asp_3st(false);
        // }).await.unwrap();

        dac.undocumented().undocumented_10010().write_async(|w| {
            w.set_data(0x99);
        }).await.unwrap();
        dac.undocumented().undocumented_80032().write_async(|w| {
            w.set_data(0x20);
        }).await.unwrap();

        dac.global().power_down_control().write_async(|w| {
            w.set_pdn_asp(false);
        }).await.unwrap();
        dac.global().power_down_control().write_async(|w| {
            w.set_pdn_hp(false);
        }).await.unwrap();

        embassy_time::Delay.delay_ms(12).await;

        dac.undocumented().undocumented_80032().write_async(|w| {
            w.set_data(0x00);
        }).await.unwrap();
        dac.undocumented().undocumented_10010().write_async(|w| {
            w.set_data(0x00);
        }).await.unwrap();

        // Can now start sending I2S data
        info!("end init sequence");
    }
}
