#![no_std]
#![no_main]
// pick a panicking behavior
use panic_probe as _;
#[allow(unused)]
use stm32f4xx_hal::stm32::interrupt;

use stm32f4xx_hal as hal;

use crate::hal::{
    gpio::{gpioc, Output, PushPull},
    prelude::*,
    stm32::{Interrupt, Peripherals, TIM2},
    timer::{Event, Timer},
};

use core::cell::RefCell;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use rtt_target::{rtt_init_print, rprintln};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = Peripherals::take().unwrap();
    // Enable debug in sleep mode and force clocking on AHB1
    dp.DBGMCU.cr.modify(|_, w| {
             w.dbg_sleep().set_bit();
             w.dbg_standby().set_bit();
             w.dbg_stop().set_bit()
         });
    dp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).pclk1(8.mhz()).freeze();

    // Configure PC13 pin to blink LED
    let gpioa = dp.GPIOC.split();
    let mut led = gpioa.pc13.into_push_pull_output();
    let _ = led.set_high(); // Turn off

    // Move the pin into our global storage
    cortex_m::interrupt::free(|cs| *G_LED.borrow(cs).borrow_mut() = Some(led));

    // Set up a timer expiring after 1s
    let mut timer = Timer::tim2(dp.TIM2, 1.hz(), clocks);

    // Generate an interrupt when the timer expires
    timer.listen(Event::TimeOut);

    // Move the timer into our global storage
    cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));


    //enable TIM2 interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    loop {
//        wfi();
    }
}

type LEDPIN = gpioc::PC13<Output<PushPull>>;

// Make LED pin globally available
static G_LED: Mutex<RefCell<Option<LEDPIN>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static G_TIM: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));

// Define an interupt handler, i.e. function to call when interrupt occurs.
// This specific interrupt will "trip" when the timer TIM2 times out
#[interrupt]
fn TIM2() {
    static mut LED: Option<LEDPIN> = None;
    static mut TIM: Option<Timer<TIM2>> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_LED.borrow(cs).replace(None).unwrap()
        })
    });

    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_TIM.borrow(cs).replace(None).unwrap()
        })
    });

    let _ = led.toggle();
    let _ = tim.wait();

    static mut NUM_TICKS: i32 = 0;
    let n = unsafe { &mut NUM_TICKS };
    *n += 1;
    if *n % 2 != 0 {
        rprintln!("Тик {} !", n);
    } else {
        rprintln!("Так {} !", n);
    }
}
