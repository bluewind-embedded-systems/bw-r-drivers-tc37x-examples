//! Blinks LED1 and LED2 on Aurix Lite Kit V2. Blinks faster when BUTTON1 is pressed.

#![no_main]
#![no_std]

use bw_r_drivers_tc37x::gpio::GpioExt;
use bw_r_drivers_tc37x::log::info;
use bw_r_drivers_tc37x::scu::wdt::{disable_cpu_watchdog, disable_safety_watchdog};
use bw_r_drivers_tc37x::scu::wdt_call::call_without_endinit;
use bw_r_drivers_tc37x::{pac, ssw};
use core::arch::asm;
use core::time::Duration;
use critical_section::RawRestoreState;
use embedded_hal::digital::StatefulOutputPin;

pub enum State {
    NotChanged = 0,
    High = 1,
    Low = 1 << 16,
    Toggled = (1 << 16) | 1,
}

#[export_name = "main"]
fn main() -> ! {
    let gpio00 = pac::P00.split();

    let mut led1 = gpio00.p00_5.into_push_pull_output();
    let mut led2 = gpio00.p00_6.into_push_pull_output();
    let button1 = gpio00.p00_7.into_input();

    let mut was_pressed = false;

    loop {
        let is_pressed = button1.is_low();

        if is_pressed != was_pressed {
            was_pressed = is_pressed;
            if is_pressed {
                info!("Button pressed");
            } else {
                info!("Button released");
            }
        }

        let period = Duration::from_millis(if is_pressed { 50 } else { 500 });

        // Test set_low
        led1.set_low();

        // Test toggle
        led2.toggle();

        info!("Wait for {:?}", period);
        wait_nop(period);
        info!("Wait done");

        // Test high
        led1.set_high();

        // Test is_set_high
        if led1.is_set_high().unwrap_or_default() {
            led2.set_low();
        }

        // Test is_set_low
        if led1.is_set_low().unwrap_or_default() {
            led2.set_high();
        }

        wait_nop(period);
    }
}

/// Wait for a number of cycles roughly calculated from a duration.
#[inline(always)]
pub fn wait_nop(period: Duration) {
    let ns: u32 = period.as_nanos() as u32;
    let n_cycles = ns / 920;
    for _ in 0..n_cycles {
        // SAFETY: nop is always safe
        unsafe { core::arch::asm!("nop") };
    }
}

// Note: without this, the watchdog will reset the CPU
#[export_name = "Crt0PreInit"]
fn pre_init_fn() {
    let cpu_core_id: u32;
    unsafe {
        core::arch::asm!("mfcr {0}, 0xFE1C", out(reg32) cpu_core_id);
    }
    if cpu_core_id == 0 {
        disable_safety_watchdog();
    }
    disable_cpu_watchdog();
}

#[export_name = "Crt0PostInit"]
fn post_init_fn() {
    if let Err(_) = ssw::init_clock() {
        info!("Error in ssw init");
        loop {}
    }

    load_interrupt_table();
}

#[allow(unused_variables)]
#[panic_handler]
fn panic(panic: &core::panic::PanicInfo<'_>) -> ! {
    // defmt::error!("Panic! {}", defmt::Display2Format(panic));
    #[allow(clippy::empty_loop)]
    loop {}
}

struct Section;

critical_section::set_impl!(Section);

unsafe impl critical_section::Impl for Section {
    unsafe fn acquire() -> RawRestoreState {
        unsafe { asm!("disable") };
        true
    }

    unsafe fn release(token: RawRestoreState) {
        if token {
            unsafe { asm!("enable") }
        }
    }
}

extern "C" {
    static __INTERRUPT_TABLE: u8;
}

pub fn load_interrupt_table() {
    call_without_endinit(|| unsafe {
        let interrupt_table = &__INTERRUPT_TABLE as *const u8 as u32;
        asm!("mtcr	$biv, {0}", in(reg32) interrupt_table);
        asm!("isync");
    });
}
