use crate::*;
use async_trait::async_trait;
use itertools::Itertools;
use poor_gpio;
use poor_gpio::{Gpio, GpioReader, GpioWriter};
use spidev::Spidev;
use std::cmp::max;
use std::io::Write;
use tokio::sync::RwLock;

pub struct EpdClient {
    pub core: EpdCoreClient,
}

impl Epd for EpdClient {
    type EpdCore = EpdCoreClient;

    fn core(&self) -> &Self::EpdCore {
        &self.core
    }
}

pub struct EpdCoreClient {
    pub raw_spi: SpiClient,
    pub chip_select_pin: GpioWriterClient,
    pub reset_pin: GpioWriterClient,
    pub busy_pin: GpioReaderClient,
}

impl EpdCommand for EpdCoreClient {
    type GpioReader = GpioReaderClient;
    type GpioWriter = GpioWriterClient;
    type Spi = SpiClient;

    fn chip_select_pin(&self) -> &Self::GpioWriter {
        &self.chip_select_pin
    }

    fn reset_pin(&self) -> &Self::GpioWriter {
        &self.reset_pin
    }

    fn busy_pin(&self) -> &Self::GpioReader {
        &self.busy_pin
    }

    fn spi(&self) -> &Self::Spi {
        &self.raw_spi
    }
}

pub struct GpioWriterClient {
    pub cli: poor_gpio::GpioWriterClient,
}

#[async_trait]
impl crate::GpioWriter for GpioWriterClient {
    async fn write(&self, value: u8) -> EpdResult<()> {
        self.cli.write(value as usize).await?;
        Ok(())
    }
}

pub struct GpioReaderClient {
    pub cli: poor_gpio::GpioReaderClient,
}

#[async_trait]
impl crate::GpioReader for GpioReaderClient {
    async fn read(&self) -> EpdResult<u8> {
        let value = self.cli.read().await?;
        Ok(value as u8)
    }
}

pub struct SpiClient {
    pub spidev_cli: RwLock<Spidev>,
    pub data_command_pin: GpioWriterClient,
}

#[async_trait]
impl Spi for SpiClient {
    type GpioWriter = GpioWriterClient;

    fn data_command_pin(&self) -> &Self::GpioWriter {
        &self.data_command_pin
    }

    async fn send(&self, data: &[u8]) -> EpdResult<()> {
        self.spidev_cli.write().await.write(data)?;
        Ok(())
    }
}
