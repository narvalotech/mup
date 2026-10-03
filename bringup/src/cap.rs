use crate::I2CResources;
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

use embassy_rp::i2c::{Config};
use embassy_rp::{peripherals::I2C1};
use embassy_rp::gpio::{Pull, Input};
use defmt::*;
use defmt_rtt as _;

use crate::pinout_prod as pinout;

device_driver::compile!(manifest: "cap1296.ddsl");

type CapType = CAPInterface<embassy_rp::i2c::I2c<'static, I2C1, embassy_rp::i2c::Async>, Input<'static>, embassy_time::Delay>;

#[must_use]
pub async fn test_cap(rd: I2CResources) -> Cap1296<CapType> {
    let i2c = embassy_rp::i2c::I2c::new_async(rd.i2c, rd.scl, rd.sda, pinout::Irqs, Config::default());

    let interface = CAPInterface::new(
        i2c,
        Input::new(rd.interrupt, Pull::Up),
        embassy_time::Delay,
    );

    let mut cap = Cap1296::new(interface);

    {
        info!("start init sequence");

        for _ in 0..1000 {
            let val = cap.sensor_input_delta_count().read_at_async(3).await.unwrap().count();
            info!("val: {}", val);
            embassy_time::Delay.delay_ms(100).await;
        }

        info!("end init sequence");
    }

    cap
}
