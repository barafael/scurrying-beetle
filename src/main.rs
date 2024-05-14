#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

include!("../beetle_map.rs");

use ch32_hal::{
    adc, bind_interrupts,
    debug::SDIPrint,
    delay::Delay,
    dma::NoDma,
    gpio::{self, Level, Output},
    i2c::{self, I2c},
    peripherals::USART1,
    println,
    time::Hertz,
    usart::{self, InterruptHandler, Uart},
};
use display_interface_i2c::I2CInterface;
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either, Either3};
use embassy_time::Duration;
use embedded_graphics::{draw_target::DrawTarget, geometry::Point, pixelcolor::BinaryColor, Drawable, Pixel};
use ssd1309::{displayrotation::DisplayRotation, mode::GraphicsMode, prelude::DisplaySize, Builder};
use wanderer::Wanderer;
use wyrand::WyRand;

mod wanderer;

bind_interrupts!(
    struct Irqs {
        USART1 => InterruptHandler<USART1>;
    }
);

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(_spawner: Spawner) -> ! {
    SDIPrint::enable();

    let p = ch32_hal::init(ch32_hal::Config::default());
    ch32_hal::embassy::init();

    let uart_config = usart::Config::default();
    let uart = Uart::new(p.USART1, p.PC1, p.PC0, Irqs, p.DMA1_CH4, p.DMA1_CH5, uart_config).unwrap();

    let sda1 = p.PC6;
    let scl1 = p.PC5;

    let mut rst1 = Output::new(p.PC3, Level::Low, gpio::Speed::default());

    let i2c1 = I2c::new::<2>(
        p.I2C1,
        scl1,
        sda1,
        NoDma,
        NoDma,
        Hertz::khz(600),
        i2c::Config::default(),
    );

    let display_interface = I2CInterface::new(i2c1, 0x3C, 0x40);
    let mut display: GraphicsMode<_> = Builder::new()
        .with_size(DisplaySize::Display128x64)
        .with_rotation(DisplayRotation::Rotate0)
        .connect(display_interface)
        .into();

    rst1.set_high();
    Delay.delay_ms(10);
    rst1.set_low();
    Delay.delay_ms(10);
    rst1.set_high();
    Delay.delay_ms(10);

    display.init().unwrap();

    let seed = {
        let mut adc = ch32_hal::adc::Adc::new(p.ADC1, adc::Config::default());
        let mut analog_input = p.PD4;
        adc.convert(&mut analog_input, ch32_hal::adc::SampleTime::CYCLES9)
    };
    let mut rng = WyRand::new(u64::from(seed));

    let mut led = Output::new(p.PD0, Level::Low, Default::default());

    let mut display_ticker = embassy_time::Ticker::every(Duration::from_millis(50));
    let mut blink_ticker = embassy_time::Ticker::every(Duration::from_millis(500));
    let mut uart_ticker = embassy_time::Ticker::every(Duration::from_millis(1000));

    let mut receive_buffer = [0u8; 16];
    let (mut tx, mut rx) = uart.split();

    let mut target = random_target(&mut rng);
    let mut x = 16;
    let mut y = 16;
    let mut wanderer = Wanderer::new(x, y, target.0, target.1);

    loop {
        match select3(display_ticker.next(), blink_ticker.next(), rx.read(&mut receive_buffer)).await {
            Either3::First(()) => match wanderer.next() {
                Some((x, y)) => {
                    draw(&mut display, x, y).unwrap();
                    display.flush().unwrap();
                    display.clear();
                }
                None => {
                    (x, y) = target;
                    target = random_target(&mut rng);
                    wanderer = Wanderer::new(x, y, target.0, target.1);
                }
            },
            Either3::Second(()) => {
                led.toggle();
            }
            Either3::Third(r) => {}
        }
    }
}

fn random_target(rng: &mut WyRand) -> (i32, i32) {
    let width = 128 - 32;
    let height = 64 - 32;
    let x = rng.rand() % width as u64;
    let y = rng.rand() % height as u64;

    let x = x + 16;
    let y = y + 16;

    (x as i32, y as i32)
}

fn draw<T>(display: &mut T, offset_x: i32, offset_y: i32) -> Result<(), T::Error>
where
    T: DrawTarget<Color = BinaryColor>,
    T::Error: core::fmt::Debug,
{
    let offset_x = offset_x - 16;
    let offset_y = offset_y - 16;
    for (x, y) in BEETLE_MAP {
        Pixel(Point::new(offset_x + x as i32, offset_y + y as i32), BinaryColor::On).draw(display)?;
    }
    Ok(())
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // println!("\n\n\n{}", info);
    loop {}
}
