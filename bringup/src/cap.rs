use embedded_hal_1::digital::{InputPin};
use embedded_hal_async::{delay::DelayNs, i2c::I2c, i2c::Operation};

type I2CAddress = u8;

pub struct CAPInterface<I2C, I, D>
where
    I2C: I2c,
    I: InputPin,
    D: DelayNs,
{
    i2c: I2C,
    // 7-bit unshifted address
    // TODO: change to an enum for the ADDR pin
    i2c_address: I2CAddress,
    interrupt_pin: I,
    _delay: D,
}

use device_driver::{RegisterInterfaceBase, AsyncRegisterInterface};

#[derive(Debug)]
pub enum InterfaceError {
    InterruptPinError,
    CommunicationError,
}

impl<I2C: I2c, I: InputPin, D: DelayNs> RegisterInterfaceBase
    for CAPInterface<I2C, I, D>
{
    type Error = InterfaceError;
    type AddressType = u8;
}

impl<I2C: I2c, I: InputPin, D: DelayNs> AsyncRegisterInterface
    for CAPInterface<I2C, I, D>
{
    async fn write_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &device_driver::FieldsetMetadata,
    ) -> Result<(), InterfaceError> {
        // Write register, control and data without STOP in the middle
        self.i2c.transaction(self.i2c_address, &mut [
            Operation::Write(&[address]),
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
        // Write register address & control byte.
        self.i2c.transaction(self.i2c_address, &mut [
            Operation::Write(&[address]),
            Operation::Read(data),
        ]).await.map_err(|_| InterfaceError::CommunicationError)?;

        Ok(())
    }
}

impl<I2C: I2c, I: InputPin, D: DelayNs>
    CAPInterface<I2C, I, D>
{
    pub const fn new(i2c: I2C, interrupt_pin: I, delay: D) -> Self {
        Self {
            i2c,
            i2c_address: 0b0101_000, // TODO: make configurable
            interrupt_pin,
            _delay: delay,
        }
    }

    pub async fn reset(&mut self) -> Result<(), InterfaceError> {
        // Do a read of the interrupt pin
        if self.interrupt_pin.is_low().map_err(|_| InterfaceError::InterruptPinError)? {
            // do something, maybe
        }

        Ok(())
    }
}

use embassy_rp::gpio::{Input};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use defmt::*;
use defmt_rtt as _;

use crate::pinout_prod as pinout;

device_driver::compile!(manifest: "cap1296.ddsl");

type CapI2c = I2cDevice<'static, NoopRawMutex, pinout::I2cBus>;
type CapType = CAPInterface<CapI2c, Input<'static>, embassy_time::Delay>;


pub async fn init(i2c: CapI2c, interrupt: Input<'static>) -> Cap1296<CapType> {
    let interface = CAPInterface::new(i2c, interrupt, embassy_time::Delay);

    Cap1296::new(interface)
}

type TouchMask = u8;

pub async fn get_touch(cap: &mut Cap1296<CapType>) -> TouchMask {
    let r = cap.input_status().read_async().await.unwrap();
    cap.main_control().write_async(|w| {
        w.set_int(false);       // this clears the touch bitmask
    }).await.unwrap();

    r.bf() as TouchMask
}

pub async fn get_raw(cap: &mut Cap1296<CapType>) -> [i8; 6] {
    let vals: [SensorInputDeltaCount; 6] = cap.sensor_input_delta_count().read_array_at_async(0).await.unwrap();
    vals.map(|x| x.count().abs())
}

pub async fn wait_touch(cap: &mut Cap1296<CapType>, sleep: bool) {
    cap.main_control().write_async(|w| {
        w.set_int(false); // clear ISR first
        if sleep {
            w.set_stby(true);
        }
    }).await.unwrap();

    // suspend until LOW
    cap.interface.interrupt_pin.wait_for_low().await;

    if sleep {
        cap.main_control().write_async(|w| {
            w.set_stby(false);
        }).await.unwrap();
    }
}

pub async fn configure(cap: &mut Cap1296<CapType>) {
    cap.main_control().write_async(|w| {
        w.set_gain(Gain::Gain1);
    }).await.unwrap();

    cap.sensitivity_control().write_async(|w| {
        w.set_base_shift(BaseShift::X256);
        w.set_delta_sense(DeltaSense::X2);
    }).await.unwrap();

    cap.averaging_and_sampling_configuration().write_async(|w| {
        w.set_samp_time(SampTime::Us1280);
        w.set_avg(Avg::S8);
    }).await.unwrap();

    cap.standby_channel().write_async(|w| {
        w.set_cs_1_stby(false);
        w.set_cs_2_stby(true);
        w.set_cs_3_stby(true);
        w.set_cs_4_stby(true);
        w.set_cs_5_stby(true);
        w.set_cs_6_stby(true);
    }).await.unwrap();

    cap.standby_sensitivity().write_async(|w| {
        w.set_stby_sense(StbySense::X2);
    }).await.unwrap();
}

#[must_use]
pub async fn test_cap(i2c: CapI2c, interrupt: Input<'static>) -> Cap1296<CapType> {
    let mut cap = init(i2c, interrupt).await;

    info!("configure CAP");
    configure(&mut cap).await;

    info!("start reading CAP");

    for _ in 0..1000 {
        wait_touch(&mut cap, true).await;
        for _ in 0..10 {
            let counts = get_raw(&mut cap).await;
            let mask = get_touch(&mut cap).await;
            info!("{} | {} | {} | {} | {} | M {}",
                  counts[5],
                  counts[4],
                  counts[3],
                  counts[2],
                  counts[1],
                  mask,
            );
            embassy_time::Delay.delay_ms(100).await;
        }
    }

    info!("done reading CAP");

    cap
}
