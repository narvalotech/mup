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


#[must_use]
pub async fn test_cap(i2c: CapI2c, interrupt: Input<'static>) -> Cap1296<CapType> {
    let interface = CAPInterface::new(i2c, interrupt, embassy_time::Delay);
    let mut cap = Cap1296::new(interface);

    {
        info!("configure CAP");

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

        info!("start reading CAP");

        for _ in 0..1000 {
            let val: [SensorInputDeltaCount; 6] = cap.sensor_input_delta_count().read_array_at_async(0).await.unwrap();
            info!("{} | {} | {} | {} | {}",
                  val[5].count().abs(),
                  val[4].count().abs(),
                  val[3].count().abs(),
                  val[2].count().abs(),
                  val[1].count().abs(),
            );
            embassy_time::Delay.delay_ms(100).await;
        }

        info!("done reading CAP");
    }

    cap
}
