//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;
use rp2040_hal as hal;

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{self, bank0::*, Output},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

macro_rules! pin {
    ($x:ident, $y:expr, $self:ident, $i:ident) => {
        if ($i & 1 << $y) > 0 {
            $self.$x.set_high().unwrap();
        } else {
            $self.$x.set_low().unwrap();
        }
    };
}

macro_rules! note {
    ($x:expr) => {
        ($x / 44000.0 * 65536.0) as u16
    };
}

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let drums = include_bytes!("cw_amen08_165.raw");

    let sio = Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut disaster = Disaster::new(pins);

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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let mut i = 0;
    let mut bass = 0;
    let mut leads = [0; 4];
    let length = drums.len() / 2;
    let mut first = true;
    loop {
        let mut output: u16 = 0;
        let mut drums_vol = ((drums[(i * 4) % length] as u16)
            + ((drums[(i * 4 + 1) % length] as u16) << 8))
            .wrapping_add(0x8000);
        if drums_vol > 50000 {
            drums_vol = ((drums_vol - 50000) / 32) + 50000;
        }
        output = output.saturating_add(drums_vol / 2);
        if first {
            output = output.saturating_add(get_bass(i, length, &mut bass) / 3);
            output = output.saturating_add(get_lead(i, length, &mut leads) / 3);
        } else {
            output = output.saturating_add(get_second_lead(i, length, &mut leads) / 3);
        }
        disaster.pins_from_u16(output);
        delay.delay_us(44);
        i += 1;
        if i >= length {
            i = 0;
            bass = 0;
            leads = [0; 4];
            first = !first;
        }
    }
}

fn get_bass(i: usize, length: usize, bass: &mut u16) -> u16 {
    const NOTES: [u16; 8] = [
        note!(130.81), //C3
        note!(130.81), //C3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(174.61), //F3
        note!(174.61), //F3
        note!(207.65), //Ab3
        note!(196.00), //G3
    ];
    let note = NOTES[i * 8 / length];
    *bass = bass.wrapping_add(note * 2);
    if *bass > 32767 {
        return (65535 - (*bass as i32)) as u16 * 2;
    } else {
        return *bass * 2;
    }
}

fn get_lead(i: usize, length: usize, leads: &mut [u16; 4]) -> u16 {
    const NOTES: [u16; 32] = [
        note!(130.81), //C3
        note!(130.81),
        note!(130.81),
        note!(130.81),
        note!(130.81),
        note!(123.47), //B2
        note!(130.81), //C3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(155.56), //Eb3
        note!(174.61), //F3
        note!(174.61), //F3
        note!(174.61), //F3
        note!(174.61), //F3
        note!(174.61), //F3
        note!(155.56), //Eb3
        note!(174.61), //F3
        note!(207.65), //Ab3
        note!(207.65), //Ab3
        note!(207.65), //Ab3
        note!(207.65), //Ab3
        note!(207.65), //Ab3
        note!(196.00), //G3
        note!(196.00), //G3
        note!(196.00), //G3
        note!(196.00), //G3
    ];
    let note = NOTES[i * 32 / length];
    let mut offset = -2;
    let mut output = 0;
    for lead in leads {
        *lead = lead.wrapping_add(((note * 4) as i32 + offset) as u16);
        output += *lead / 4;
        offset += 1;
    }
    output
}

fn get_second_lead(i: usize, length: usize, leads: &mut [u16; 4]) -> u16 {
    const NOTES: [u16; 32] = [
        note!(0.0),
        note!(0.0),
        note!(196.00),
        note!(0.0),
        note!(0.0),
        note!(196.00),
        note!(0.0),
        note!(0.0),
        note!(0.0),
        note!(0.0),
        note!(196.00),
        note!(0.0),
        note!(196.00),
        note!(174.61),
        note!(196.00),
        note!(0.0),
        note!(0.0),
        note!(0.0),
        note!(196.00),
        note!(0.0),
        note!(0.0),
        note!(196.00),
        note!(0.0),
        note!(0.0),
        note!(0.0),
        note!(0.0),
        note!(196.00),
        note!(0.0),
        note!(196.00),
        note!(207.65),
        note!(196.00),
        note!(0.0),
    ];
    let note = NOTES[i * 32 / length];
    let mut offset = -6;
    let mut output = 0;
    for lead in leads {
        *lead = lead.wrapping_add(((note * 4) as i32 + offset) as u16);
        output += *lead / 4;
        offset += 3;
    }
    output
}
struct Disaster {
    pin0: gpio::Pin<Gpio0, Output<gpio::PushPull>>,
    pin1: gpio::Pin<Gpio1, Output<gpio::PushPull>>,
    pin2: gpio::Pin<Gpio2, Output<gpio::PushPull>>,
    pin3: gpio::Pin<Gpio3, Output<gpio::PushPull>>,
    pin4: gpio::Pin<Gpio4, Output<gpio::PushPull>>,
    pin5: gpio::Pin<Gpio5, Output<gpio::PushPull>>,
    pin6: gpio::Pin<Gpio6, Output<gpio::PushPull>>,
    pin7: gpio::Pin<Gpio7, Output<gpio::PushPull>>,
    pin8: gpio::Pin<Gpio8, Output<gpio::PushPull>>,
    pin9: gpio::Pin<Gpio9, Output<gpio::PushPull>>,
    pin10: gpio::Pin<Gpio10, Output<gpio::PushPull>>,
    pin11: gpio::Pin<Gpio11, Output<gpio::PushPull>>,
    pin12: gpio::Pin<Gpio12, Output<gpio::PushPull>>,
    pin13: gpio::Pin<Gpio13, Output<gpio::PushPull>>,
    pin14: gpio::Pin<Gpio14, Output<gpio::PushPull>>,
    pin15: gpio::Pin<Gpio15, Output<gpio::PushPull>>,
}

impl Disaster {
    fn new(pins: Pins) -> Disaster {
        Disaster {
            pin0: pins.gpio0.into_push_pull_output(),
            pin1: pins.gpio1.into_push_pull_output(),
            pin2: pins.gpio2.into_push_pull_output(),
            pin3: pins.gpio3.into_push_pull_output(),
            pin4: pins.gpio4.into_push_pull_output(),
            pin5: pins.gpio5.into_push_pull_output(),
            pin6: pins.gpio6.into_push_pull_output(),
            pin7: pins.gpio7.into_push_pull_output(),
            pin8: pins.gpio8.into_push_pull_output(),
            pin9: pins.gpio9.into_push_pull_output(),
            pin10: pins.gpio10.into_push_pull_output(),
            pin11: pins.gpio11.into_push_pull_output(),
            pin12: pins.gpio12.into_push_pull_output(),
            pin13: pins.gpio13.into_push_pull_output(),
            pin14: pins.gpio14.into_push_pull_output(),
            pin15: pins.gpio15.into_push_pull_output(),
        }
    }
    fn pins_from_u16(&mut self, i: u16) {
        pin!(pin0, 0, self, i);
        pin!(pin1, 1, self, i);
        pin!(pin2, 2, self, i);
        pin!(pin3, 3, self, i);
        pin!(pin4, 4, self, i);
        pin!(pin5, 5, self, i);
        pin!(pin6, 6, self, i);
        pin!(pin7, 7, self, i);
        pin!(pin8, 8, self, i);
        pin!(pin9, 9, self, i);
        pin!(pin10, 10, self, i);
        pin!(pin11, 11, self, i);
        pin!(pin12, 12, self, i);
        pin!(pin13, 13, self, i);
        pin!(pin14, 14, self, i);
        pin!(pin15, 15, self, i);
    }
}
