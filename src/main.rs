#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

include!("../embeetle_32x32.rs");

use ch32_hal::{
    adc,
    debug::SDIPrint,
    delay::Delay,
    dma::NoDma,
    gpio::{self, Level, Output},
    i2c::{self, I2c},
    time::Hertz,
};
use display_interface_i2c::I2CInterface;
use embedded_graphics::{draw_target::DrawTarget, geometry::Point, pixelcolor::BinaryColor, Drawable, Pixel};
use itertools::Itertools;
use ssd1309::{displayrotation::DisplayRotation, mode::GraphicsMode, prelude::DisplaySize, Builder};
use wanderer::Wanderer;
use wyrand::WyRand;

mod wanderer;

#[ch32_hal::entry]
fn main() -> ! {
    SDIPrint::enable();

    let p = ch32_hal::init(ch32_hal::Config::default());

    let sda1 = p.PC1;
    let scl1 = p.PC2;

    let mut rst1 = Output::new(p.PC3, Level::Low, gpio::Speed::default());

    let i2c1 = I2c::new::<0>(
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

    display.init().expect("Display connected?");

    let seed = {
        let mut adc = ch32_hal::adc::Adc::new(p.ADC1, adc::Config::default());
        let mut analog_input = p.PD4;
        adc.convert(&mut analog_input, ch32_hal::adc::SampleTime::CYCLES9)
    };
    let mut rng = WyRand::new(u64::from(seed));

    let mut target;
    let mut x = 16;
    let mut y = 16;
    loop {
        target = random_target(&mut rng);
        let wanderer = Wanderer::new(x, y, target.0, target.1);
        for (x, y) in wanderer {
            draw(&mut display, x, y).unwrap();
            display.flush().unwrap();
            // Delay.delay_ms(5);
            display.clear();
        }
        (x, y) = target;
    }
}

fn random_target(rng: &mut WyRand) -> (i32, i32) {
    let width = 128 - BEETLE[0].len();
    let height = 64 - BEETLE.len();
    let x = rng.rand() % width as u64;
    let y = rng.rand() % height as u64;

    let x = x + BEETLE[0].len() as u64 / 2u64;
    let y = y + BEETLE.len() as u64 / 2u64;

    (x as i32, y as i32)
}

fn draw<T>(display: &mut T, offset_x: i32, offset_y: i32) -> Result<(), T::Error>
where
    T: DrawTarget<Color = BinaryColor>,
    T::Error: core::fmt::Debug,
{
    let offset_x = offset_x - 16;
    let offset_y = offset_y - 16;
    for (x, y) in (0..BEETLE.len()).cartesian_product(0..BEETLE[0].len()) {
        if BEETLE[y][x] == 1 {
            Pixel(Point::new(offset_x + x as i32, offset_y + y as i32), BinaryColor::On).draw(display)?;
        }
    }
    Ok(())
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // println!("\n\n\n{}", info);

    loop {}
}
