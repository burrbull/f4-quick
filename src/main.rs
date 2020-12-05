#![no_std]
#![no_main]
// pick a panicking behavior
use panic_probe as _;
#[allow(unused)]
use stm32f4xx_hal::stm32::interrupt;

use cortex_m::asm;
use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    rprintln!("Hello, world!");
    loop { asm::bkpt() }
}
