#![no_main]
#![no_std]

use defmt_rtt as _;

use bsp::hal;
use hal::pac;
use rp_pico as bsp;

use panic_probe as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    exit()
}
