//! Draws a controllable circle on the display.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use rp2040_hal::Spi;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_hal::digital::{InputPin, OutputPin};

use sh1106::prelude::GraphicsMode;
use sh1106::Builder;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use picoboy as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use embedded_graphics::primitives::{Circle, PrimitiveStyleBuilder};
use rp2040_hal::fugit::RateExtU32;

#[entry]
fn main() -> ! {
    info!("Program start");

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Define display width and height
    const DISPLAY_WIDTH: i32 = 128;
    const DISPLAY_HEIGHT: i32 = 64;

    // Configure SPI pins
    let spi_sclk = pins.sck.into_function::<rp2040_hal::gpio::FunctionSpi>(); // SCK
    let spi_mosi = pins.mosi.into_function::<rp2040_hal::gpio::FunctionSpi>(); // MOSI
    let spi_miso = pins.gpio16.into_function::<rp2040_hal::gpio::FunctionSpi>(); // MISO (not needed)

    // Create spi instance
    let spi = Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));

    // Init spi
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        8_000_000u32.Hz(),         // 8 MHz
        embedded_hal::spi::MODE_0, // MODE_0 for SH1106
    );

    // Configure display pins
    let dc = pins.dc.into_push_pull_output(); // Data/command pin
    let mut rst = pins.reset.into_push_pull_output(); // Reset pin
    let cs = pins.cs.into_push_pull_output(); // Chip select

    // Reset display
    rst.set_low().unwrap();
    delay.delay_ms(10);
    rst.set_high().unwrap();
    delay.delay_ms(10);

    // Create display interface
    let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc, cs).into();

    display.init().unwrap();
    display.flush().unwrap();

    // Create Hello world.
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline(
        "From Rust with love.",
        Point::new(0, 16),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    display.flush().unwrap();
    delay.delay_ms(2500);

    // Ball
    display.clear();

    // Configuring joystick buttons
    let mut joystick_up = pins.joystick_up.into_pull_up_input();
    let mut joystick_down = pins.joystick_down.into_pull_up_input();
    let mut joystick_left = pins.joystick_left.into_pull_up_input();
    let mut joystick_right = pins.joystick_right.into_pull_up_input();

    let mut x: i32 = DISPLAY_WIDTH / 2;
    let mut y: i32 = DISPLAY_HEIGHT / 2;

    // Clear display
    display.clear();

    // Status led
    let mut led_pin = pins.led_red.into_push_pull_output();
    led_pin.set_high().unwrap();

    loop {
        display.clear();

        // Check entries and adjust position
        if joystick_down.is_low().unwrap() {
            y = y.saturating_add(2);
        }

        if joystick_up.is_low().unwrap() {
            y = y.saturating_sub(2);
        }

        if joystick_right.is_low().unwrap() {
            x = x.saturating_add(2);
        }

        if joystick_left.is_low().unwrap() {
            x = x.saturating_sub(2);
        }

        // Draw a new circle
        let new_circle = Circle::new(Point::new(x, y), 25).into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::On)
                .build(),
        );

        new_circle.draw(&mut display).unwrap();

        display.flush().unwrap();

        delay.delay_ms(50);
    }
}

// End of file
