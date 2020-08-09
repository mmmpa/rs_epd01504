mod error;
mod mono8_image;
mod value;

pub use error::*;
pub use mono8_image::*;
pub use value::*;

use crate::*;
use async_trait::async_trait;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cmp::{max, min};
use std::io::Read;

pub type EpdResult<T> = Result<T, EpdError>;

//pub struct Epd;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum EpdColor {
    Black,
    White,
}

impl From<u8> for EpdColor {
    fn from(n: u8) -> Self {
        match n {
            1 => Self::Black,
            _ => Self::White,
        }
    }
}

pub trait EpdImage: Send + Sync + 'static + Sized {
    fn rect(&self) -> &EightSizedRectangle;
    fn update(&mut self, rect: NormalRectangle, data: &[EpdColor]) -> EpdResult<()>;
    fn as_vec(&self) -> &[u8];
    fn as_part_vec(&self, rect: NormalRectangle) -> (EightSizedRectangle, Vec<u8>);
    fn data_for_fill(rect: NormalRectangle, color: EpdColor) -> EpdResult<Self>;
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct NormalRectangle {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl NormalRectangle {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct EightSizedRectangle {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl EightSizedRectangle {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<NormalRectangle> for EightSizedRectangle {
    fn from(rect: NormalRectangle) -> Self {
        let NormalRectangle {
            x,
            y,
            width,
            height,
        } = rect;

        let width = match width >> 3 {
            m if m == 0 => 1,
            m if width % 8 == 0 => m,
            m => m + 1,
        };

        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[async_trait]
pub trait RawSpiAccessor: Send + Sync + 'static {
    type EpdGpioWriter: EpdGpioWriter;

    fn data_command_pin(&self) -> &Self::EpdGpioWriter;
    async fn send(&self, data: &[u8]) -> EpdResult<()>;

    async fn send_command(&self, command: u8) -> EpdResult<()> {
        self.data_command_pin().low().await?;
        self.send(&[command]).await?;

        Ok(())
    }

    async fn send_data(&self, data: &[u8]) -> EpdResult<()> {
        if data.len() == 0 {
            return Ok(());
        }

        self.data_command_pin().high().await?;

        // Buf limit is machine unique. cat /sys/module/spidev/parameters/bufsiz
        for d in data.chunks(4096).into_iter() {
            self.send(d).await?;
        }

        Ok(())
    }
}

#[async_trait]
pub trait EpdGpioWriter: Send + Sync + 'static {
    async fn write(&self, value: u8) -> EpdResult<()>;
    async fn high(&self) -> EpdResult<()> {
        self.write(1).await
    }
    async fn low(&self) -> EpdResult<()> {
        self.write(0).await
    }
}

#[async_trait]
pub trait EpdGpioReader: Send + Sync + 'static {
    async fn read(&self) -> EpdResult<u8>;
    async fn high(&self) -> EpdResult<bool> {
        Ok(!self.low().await?)
    }
    async fn low(&self) -> EpdResult<bool> {
        Ok(self.read().await? == 0)
    }
}

#[async_trait]
pub trait EpdSpiWrapper: Send + Sync + 'static {
    type EpdGpioReader: EpdGpioReader;
    type EpdGpioWriter: EpdGpioWriter;
    type RawSpiAccessor: RawSpiAccessor<
        EpdGpioWriter = Self::EpdGpioWriter, //
    >;
    type EpdImage: EpdImage;

    fn chip_select_pin(&self) -> &Self::EpdGpioWriter;
    fn reset_pin(&self) -> &Self::EpdGpioWriter;
    fn busy_pin(&self) -> &Self::EpdGpioReader;
    fn spi(&self) -> &Self::RawSpiAccessor;

    fn canvas_width(&self) -> u16 {
        200
    }

    fn canvas_height(&self) -> u16 {
        200
    }

    async fn send_command(&self, commend: Command) -> EpdResult<()> {
        self.spi().send_command(commend as u8).await?;

        Ok(())
    }

    async fn send(&self, commend: Command, data: &[u8]) -> EpdResult<()> {
        if data.len() < 10 {
            debug!(
                "commend: {:x} data: {:?}",
                commend as u8,
                data.iter()
                    .map(|x| format!("{:>02x}", x))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        } else {
            debug!("commend: {:x} data: snip", commend as u8,);
        }
        self.spi().send_command(commend as u8).await?;
        self.spi().send_data(data).await?;

        Ok(())
    }

    async fn init(&self) -> EpdResult<()> {
        self.init_all_pin().await?;
        self.reset_panel().await?;
        self.set_driver_output_control().await?;
        self.set_booster_soft_start_control().await?;
        self.write_vcom_register().await?;
        self.set_dummy_line_period().await?;
        self.set_gate_time().await?;
        self.set_data_entry_mode(DataEntryMode::default()).await?;
        self.set_lut(Lut::FullUpdate).await?;

        Ok(())
    }

    async fn init_all_pin(&self) -> EpdResult<()> {
        self.reset_pin().high().await?;
        self.chip_select_pin().high().await?;

        Ok(())
    }

    async fn reset_panel(&self) -> EpdResult<()> {
        self.reset_pin().low().await?;
        delay_for(RESET_TIME).await;
        self.reset_pin().high().await?;
        delay_for(RESET_TIME).await;

        Ok(())
    }

    async fn set_driver_output_control(&self) -> EpdResult<()> {
        debug!("set_driver_output_control");
        let h = self.canvas_height() - 1;
        self.send(
            Command::DriverOutputControl,
            &[(h & 0xff) as u8, (h >> 8 & 0xff) as u8, 0],
        )
        .await
    }

    async fn set_booster_soft_start_control(&self) -> EpdResult<()> {
        debug!("set_booster_soft_start_control");
        self.send(
            Command::BoosterSoftStartControl,
            &data::BOOSTER_SOFT_START_CONTROL,
        )
        .await
    }

    async fn write_vcom_register(&self) -> EpdResult<()> {
        debug!("write_vcom_register");
        self.send(Command::WriteVcomRegister, &data::WRITE_VCOM_REGISTER)
            .await
    }

    async fn set_dummy_line_period(&self) -> EpdResult<()> {
        debug!("set_dummy_line_period");
        self.send(Command::SetDummyLinePeriod, &data::SET_DUMMY_LINE_PERIOD)
            .await
    }

    async fn set_gate_time(&self) -> EpdResult<()> {
        debug!("set_gate_time");
        self.send(Command::SetGateTime, &data::SET_GATE_TIME).await
    }

    async fn set_data_entry_mode(&self, mode: DataEntryMode) -> EpdResult<()> {
        debug!("set_data_entry_mode");
        self.send(Command::DataEntryModeSetting, &vec![mode as u8])
            .await
    }

    async fn set_lut(&self, lut: Lut) -> EpdResult<()> {
        debug!("set_full_update_lut");
        self.send(Command::WriteLutRegister, lut.as_ref()).await
    }

    async fn fill(&self, color: EpdColor) -> EpdResult<()> {
        let rect = NormalRectangle::new(0, 0, self.canvas_width(), self.canvas_height());
        self.draw(Self::EpdImage::data_for_fill(rect, color)?).await
    }

    async fn draw(&self, image: Self::EpdImage) -> EpdResult<()> {
        self.upload_image(image).await?;
        self.refresh_display().await?;

        Ok(())
    }

    async fn upload_image(&self, image: Self::EpdImage) -> EpdResult<()> {
        self.set_ram_area(&image).await?;
        self.set_ram_counter(&image).await?;
        self.write_image_to_ram(image).await?;

        Ok(())
    }

    // TODO: compute area from image
    async fn set_ram_area(&self, image: &Self::EpdImage) -> EpdResult<()> {
        let EightSizedRectangle {
            x,
            y,
            width,
            height,
        } = *image.rect();

        self.send(
            Command::SetRamXAddressStartEndPosition,
            &[x as u8, (x + width - 1) as u8],
        )
        .await?;

        let h = height - 1;
        self.send(
            Command::SetRamYAddressStartEndPosition,
            &[
                (y & 0xff) as u8,
                ((y >> 8) & 0xff) as u8,
                (y + h & 0xff) as u8,
                ((y + h >> 8) & 0xff) as u8,
            ],
        )
        .await?;

        Ok((()))
    }

    // TODO: compute start point from image
    async fn set_ram_counter(&self, image: &Self::EpdImage) -> EpdResult<()> {
        let EightSizedRectangle { x, y, .. } = *image.rect();

        self.send(Command::SetRamXAddressCounter, &[x as u8])
            .await?;
        self.send(
            Command::SetRamYAddressCounter,
            &[(y & 0xff) as u8, ((y >> 8) & 0xff) as u8],
        )
        .await?;

        Ok((()))
    }

    async fn write_image_to_ram(&self, image: Self::EpdImage) -> EpdResult<()> {
        self.send(Command::WriteRam, image.as_vec()).await
    }

    async fn refresh_display(&self) -> EpdResult<()> {
        self.set_image_activation_option().await?;
        self.activate_uploaded_image().await?;
        self.inform_read_write_end().await?;
        self.wait_until_idle().await?;

        Ok(())
    }

    async fn set_image_activation_option(&self) -> EpdResult<()> {
        self.send(
            Command::DisplayUpdateControl2,
            &data::DISPLAY_UPDATE_CONTROL_2,
        )
        .await
    }

    async fn activate_uploaded_image(&self) -> EpdResult<()> {
        self.send_command(Command::MasterActivation).await
    }

    async fn inform_read_write_end(&self) -> EpdResult<()> {
        self.send_command(Command::NOP).await
    }

    async fn wait_until_idle(&self) -> EpdResult<()> {
        loop {
            delay_for(100).await;
            if self.busy_pin().low().await? {
                break;
            }
        }
        Ok(())
    }

    async fn sleep(&self, mode: DeepSleep) -> EpdResult<()> {
        self.send(Command::DeepSleepMode, &[mode as u8]).await
    }
}

#[async_trait]
pub trait Epd: Send + Sync + 'static {
    type EpdGpioWriter: EpdGpioWriter;
    type EpdGpioReader: EpdGpioReader;
    type EpdWrappedSpi: EpdSpiWrapper<
        EpdGpioReader = Self::EpdGpioReader, //
        EpdGpioWriter = Self::EpdGpioWriter,
        EpdImage = Self::EpdImager,
    >;
    type EpdImager: EpdImage;

    fn spi(&self) -> &Self::EpdWrappedSpi;

    async fn init(&self) -> EpdResult<()> {
        self.spi().init().await
    }

    async fn draw(&self, image: Self::EpdImager) -> EpdResult<()> {
        self.spi().draw(image).await
    }

    async fn fill(&self, color: EpdColor) -> EpdResult<()> {
        self.spi().fill(color).await
    }

    async fn clear(&self) -> EpdResult<()> {
        self.fill(EpdColor::White).await
    }

    async fn sleep(&self, mode: DeepSleep) -> EpdResult<()> {
        self.spi().sleep(mode).await
    }
}
