# epd01504

This is to use GDEH0154D27 with Rust.

[GDEH0154D27,\(Discontinued\)1\.54 inch e\-paper display module partial refresh E\-ink screen panel G](http://www.e-paper-display.com/products_detail/productId=365.html)

# Usage

```sh
export CS="8"
export DC="25"
export RES="17"
export BUSY="24"
```

```rust
async fn prepare() -> impl Epd {
    let chip_select_pin = pin_writer(env!("CS")).await;
    let data_command_pin = pin_writer(env!("DC")).await;
    let reset_pin = pin_writer(env!("RES")).await;
    let busy_pin = pin_reader(env!("BUSY")).await;

    let raw_spi = SpiClient {
        spidev_cli: create_spi().unwrap().into(),
        data_command_pin,
    };

    let core = EpdCoreClient {
        raw_spi,
        chip_select_pin,
        reset_pin,
        busy_pin,
        width: 200,
        height: 200,
    };
    EpdClient { core }
}

fn create_spi() -> std::io::Result<Spidev> {
    let mut spi = Spidev::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(20_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options)?;
    Ok(spi)
}

async fn pin_writer(n: &'static str) -> GpioWriterClient {
    let config = config(n);
    let cli = poor_gpio::GpioWriterClient::open(config).await.unwrap();
    GpioWriterClient { cli }
}

async fn pin_reader(n: &'static str) -> GpioReaderClient {
    let config = config(n);
    let cli = poor_gpio::GpioReaderClient::open(config).await.unwrap();
    GpioReaderClient { cli }
}

fn config(n: &'static str) -> poor_gpio::Config {
    poor_gpio::Config {
        open: true,
        err_if_already_opened: false,
        close_if_open_self: false,
        gpio_n: n.parse().unwrap(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let epd = prepare().await;

    epd.init().await;
    epd.clear().await;

    epd.fill(Color::White).await.unwrap();
    epd.fill(Color::Black).await.unwrap();

    Ok(())
}
```