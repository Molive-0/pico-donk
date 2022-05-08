#![no_std]
#![no_main]
#![feature(inline_const)]
#![feature(generic_const_exprs)]

use core::{
    hint::spin_loop,
    sync::atomic::{AtomicU16, AtomicU8, Ordering::Relaxed},
};
use cortex_m_rt::{entry, exception};
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use pico_donk_core::{cst::Sample, Song};
use rp2040_hal as hal;

use hal::{clocks::init_clocks_and_plls, pac, watchdog::Watchdog};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// I'm using atomic values here, but the problem is that thumbv6m
// doesn't have CAS instructions, so I have to load and store them
// manually. :(
pub static W_POSITION: AtomicU8 = AtomicU8::new(0);
pub static R_POSITION: AtomicU8 = AtomicU8::new(0);
pub static AUDIO_BUF: [AtomicU16; 256] = [const { AtomicU16::new(0) }; 256];

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let _clocks = init_clocks_and_plls(
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

    let mut song = Song::new();

    // We're performing extremely low level stuff here because then I know it's working

    // Perform this reset madness, god knows what's going on here
    pac.RESETS
        .reset
        .modify(|_, w| w.pads_bank0().set_bit().io_bank0().set_bit());
    pac.RESETS
        .reset
        .modify(|_, w| w.pads_bank0().clear_bit().io_bank0().clear_bit());
    while pac.RESETS.reset_done.read().pads_bank0().bit_is_clear() {
        spin_loop();
    }
    while pac.RESETS.reset_done.read().io_bank0().bit_is_clear() {
        spin_loop();
    }

    // Disable input on all pins
    for pad in &pac.PADS_BANK0.gpio {
        pad.write(|w| {
            w.od()
                .bit(false)
                .ie()
                .bit(false)
                .pue()
                .bit(false)
                .pde()
                .bit(false)
        });
    }
    // Take output from SIO on all pins
    for i in &pac.IO_BANK0.gpio {
        i.gpio_ctrl.write(|f| f.funcsel().sio_0())
    }
    // Enable output on all pins
    pac.SIO.gpio_oe_set.write(|w| unsafe { w.bits(0xFFFFFFFF) });

    // Set up the systick registers
    let tenms = cortex_m::peripheral::SYST::get_ticks_per_10ms();
    core.SYST.set_reload(tenms / 240); //48000hz

    core.SYST.clear_current();
    core.SYST.enable_interrupt();
    core.SYST.enable_counter();

    // Start making music!
    loop {
        let pos = W_POSITION.load(Relaxed);
        W_POSITION.store(pos.wrapping_add(1), Relaxed);
        AUDIO_BUF[pos as usize].store(song.get_sample(), Relaxed);
        while R_POSITION.load(Relaxed) == W_POSITION.load(Relaxed).wrapping_add(1) {
            cortex_m::asm::wfi();
        }
    }
}

#[exception]
fn SysTick() {
    unsafe {
        (*pac::SIO::ptr()).gpio_out.write(|w| {
            let pos = R_POSITION.load(Relaxed);
            R_POSITION.store(pos.wrapping_add(1), Relaxed);
            w.bits(AUDIO_BUF[pos as usize].load(Relaxed) as u32)
        });
    }
}
