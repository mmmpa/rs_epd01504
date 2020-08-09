mod error;
mod value;

pub use error::*;
pub use value::*;

use crate::*;
use async_trait::async_trait;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cmp::{max, min};
use std::io::Read;

pub type EpdResult<T> = Result<T, EpdError>;

//pub struct Epd;

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Color {
    Black = 1,
    White = 0,
}

impl From<u8> for Color {
    fn from(n: u8) -> Self {
        match n {
            1 => Self::Black,
            _ => Self::White,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct EightSizedRectangle {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
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

#[derive(Copy, Clone, Default)]
pub struct DisplayUpdateControlParams {
    pub enabling: DisplayUpdateEnablingStep,
    pub target_display: DisplayUpdateTarget,
    pub disabling: DisplayUpdateDisablingStep,
}

#[async_trait]
pub trait Spi: Send + Sync {
    type GpioWriter: GpioWriter;

    fn data_command_pin(&self) -> &Self::GpioWriter;
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
pub trait GpioWriter: Send + Sync {
    async fn write(&self, value: u8) -> EpdResult<()>;
    async fn high(&self) -> EpdResult<()> {
        self.write(1).await
    }
    async fn low(&self) -> EpdResult<()> {
        self.write(0).await
    }
}

#[async_trait]
pub trait GpioReader: Send + Sync {
    async fn read(&self) -> EpdResult<u8>;
    async fn high(&self) -> EpdResult<bool> {
        Ok(!self.low().await?)
    }
    async fn low(&self) -> EpdResult<bool> {
        Ok(self.read().await? == 0)
    }
}

#[async_trait]
pub trait EpdCommand: Send + Sync {
    type GpioReader: GpioReader;
    type GpioWriter: GpioWriter;
    type Spi: Spi<
        GpioWriter = Self::GpioWriter, //
    >;

    fn chip_select_pin(&self) -> &Self::GpioWriter;
    fn reset_pin(&self) -> &Self::GpioWriter;
    fn busy_pin(&self) -> &Self::GpioReader;
    fn spi(&self) -> &Self::Spi;

    fn canvas_width(&self) -> usize {
        200
    }

    fn canvas_height(&self) -> usize {
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
        self.set_data_entry_mode(
            DataEntryAddressDirection::default(),
            DataEntryAddressCounterDirection::default(),
        )
        .await?;
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
            &static_data::BOOSTER_SOFT_START_CONTROL,
        )
        .await
    }

    async fn write_vcom_register(&self) -> EpdResult<()> {
        debug!("write_vcom_register");
        self.send(
            Command::WriteVcomRegister,
            &static_data::WRITE_VCOM_REGISTER,
        )
        .await
    }

    async fn set_dummy_line_period(&self) -> EpdResult<()> {
        debug!("set_dummy_line_period");
        self.send(
            Command::SetDummyLinePeriod,
            &static_data::SET_DUMMY_LINE_PERIOD,
        )
        .await
    }

    async fn set_gate_time(&self) -> EpdResult<()> {
        debug!("set_gate_time");
        self.send(Command::SetGateTime, &static_data::SET_GATE_TIME)
            .await
    }

    async fn set_data_entry_mode(
        &self,
        mode: DataEntryAddressDirection,
        counter: DataEntryAddressCounterDirection,
    ) -> EpdResult<()> {
        debug!("set_data_entry_mode");
        self.send(
            Command::DataEntryModeSetting,
            &vec![counter as u8 | mode as u8],
        )
        .await
    }

    async fn set_lut(&self, lut: Lut) -> EpdResult<()> {
        debug!("set_full_update_lut");
        self.send(Command::WriteLutRegister, lut.as_ref()).await
    }

    async fn fill(&self, color: Color, params: DisplayUpdateControlParams) -> EpdResult<()> {
        let hex = match color {
            Color::Black => 0xFF,
            Color::White => 0x00,
        };

        let w = self.canvas_width() >> 3;
        let h = self.canvas_height();

        self.draw(
            &EightSizedRectangle::new(0, 0, w as u16, h as u16),
            &vec![hex; w * h],
            params,
        )
        .await
    }

    async fn draw(
        &self,
        rect: &EightSizedRectangle,
        data: &[u8],
        params: DisplayUpdateControlParams,
    ) -> EpdResult<()> {
        self.upload_image(rect, data).await?;
        self.refresh_display(params).await?;

        Ok(())
    }

    async fn upload_image(&self, rect: &EightSizedRectangle, data: &[u8]) -> EpdResult<()> {
        self.set_ram_area(&rect).await?;
        self.set_ram_counter(&rect).await?;
        self.write_image_to_ram(data).await?;

        Ok(())
    }

    // TODO: compute area from image
    async fn set_ram_area(&self, rect: &EightSizedRectangle) -> EpdResult<()> {
        let EightSizedRectangle {
            x,
            y,
            width,
            height,
        } = *rect;

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

    async fn set_ram_counter(&self, rect: &EightSizedRectangle) -> EpdResult<()> {
        let EightSizedRectangle { x, y, .. } = *rect;

        self.send(Command::SetRamXAddressCounter, &[x as u8])
            .await?;
        self.send(
            Command::SetRamYAddressCounter,
            &[(y & 0xff) as u8, ((y >> 8) & 0xff) as u8],
        )
        .await?;

        Ok((()))
    }

    async fn write_image_to_ram(&self, data: &[u8]) -> EpdResult<()> {
        self.send(Command::WriteRam, data).await
    }

    async fn refresh_display(&self, params: DisplayUpdateControlParams) -> EpdResult<()> {
        self.set_image_activation_option(params).await?;
        self.activate_uploaded_image().await?;
        self.inform_read_write_end().await?;
        self.wait_until_idle().await?;

        Ok(())
    }

    async fn set_image_activation_option(
        &self,
        params: DisplayUpdateControlParams,
    ) -> EpdResult<()> {
        let DisplayUpdateControlParams {
            enabling,
            target_display,
            disabling,
        } = params;

        self.send(
            Command::DisplayUpdateControl2,
            &[enabling as u8 | target_display as u8 | disabling as u8],
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
pub trait Epd: Send + Sync {
    type EpdCore: EpdCommand;

    fn core(&self) -> &Self::EpdCore;

    fn display_update_control(&self) -> DisplayUpdateControlParams {
        DisplayUpdateControlParams::default()
    }

    async fn init(&self) -> EpdResult<()> {
        self.core().init().await
    }

    async fn draw(&self, rect: &EightSizedRectangle, data: &[u8]) -> EpdResult<()> {
        self.core()
            .draw(rect, data, self.display_update_control())
            .await
    }

    async fn fill(&self, color: Color) -> EpdResult<()> {
        self.core().fill(color, self.display_update_control()).await
    }

    async fn clear(&self) -> EpdResult<()> {
        self.fill(Color::White).await
    }

    async fn sleep(&self, mode: DeepSleep) -> EpdResult<()> {
        self.core().sleep(mode).await
    }
}
