use crate::*;
use async_trait::async_trait;
use poor_gpio;
use poor_gpio::*;
use spidev::Spidev;
use std::io::Write;
use tokio::sync::RwLock;

pub struct EpdClient {
    pub wrapped_spi: EpdWrappedSpiClient,
}

impl Epd for EpdClient {
    type EpdGpioReader = EpdGpioReaderClient;
    type EpdGpioWriter = EpdGpioWriterClient;
    type EpdWrappedSpi = EpdWrappedSpiClient;

    fn spi(&self) -> &Self::EpdWrappedSpi {
        &self.wrapped_spi
    }

    type EpdImager = EpdMonoColorImage;
}

pub struct EpdWrappedSpiClient {
    pub raw_spi: RawSpiAccessorSpiClient,
    pub chip_select_pin: EpdGpioWriterClient,
    pub reset_pin: EpdGpioWriterClient,
    pub busy_pin: EpdGpioReaderClient,
}

impl EpdSpiWrapper for EpdWrappedSpiClient {
    type EpdGpioReader = EpdGpioReaderClient;
    type EpdGpioWriter = EpdGpioWriterClient;
    type RawSpiAccessor = RawSpiAccessorSpiClient;
    type EpdImage = EpdMonoColorImage;

    fn chip_select_pin(&self) -> &Self::EpdGpioWriter {
        &self.chip_select_pin
    }

    fn reset_pin(&self) -> &Self::EpdGpioWriter {
        &self.reset_pin
    }

    fn busy_pin(&self) -> &Self::EpdGpioReader {
        &self.busy_pin
    }

    fn spi(&self) -> &Self::RawSpiAccessor {
        &self.raw_spi
    }
}

pub struct EpdGpioWriterClient {
    pub cli: poor_gpio::GpioWriterClient,
}

#[async_trait]
impl EpdGpioWriter for EpdGpioWriterClient {
    async fn write(&self, value: u8) -> EpdResult<()> {
        self.cli.write(value as usize).await?;
        Ok(())
    }
}

pub struct EpdGpioReaderClient {
    pub cli: poor_gpio::GpioReaderClient,
}

#[async_trait]
impl EpdGpioReader for EpdGpioReaderClient {
    async fn read(&self) -> EpdResult<u8> {
        let value = self.cli.read().await?;
        Ok(value as u8)
    }
}

pub struct RawSpiAccessorSpiClient {
    pub spidev_cli: RwLock<Spidev>,
    pub data_command_pin: EpdGpioWriterClient,
}

#[async_trait]
impl RawSpiAccessor for RawSpiAccessorSpiClient {
    type EpdGpioWriter = EpdGpioWriterClient;

    fn data_command_pin(&self) -> &Self::EpdGpioWriter {
        &self.data_command_pin
    }

    async fn send(&self, data: &[u8]) -> EpdResult<()> {
        self.spidev_cli.write().await.write(data)?;
        Ok(())
    }
}
