//! Simple CAN example.

#![no_main]
#![no_std]

use bw_r_drivers_tc37x::can::config::NodeInterruptConfig;
use bw_r_drivers_tc37x::can::msg::ReadFrom;
use bw_r_drivers_tc37x::can::pin_map::*;
use bw_r_drivers_tc37x::can::Frame;
use bw_r_drivers_tc37x::can::InterruptLine;
use bw_r_drivers_tc37x::can::MessageId;
use bw_r_drivers_tc37x::can::*;
use bw_r_drivers_tc37x::cpu::Priority;
use bw_r_drivers_tc37x::gpio::GpioExt;
use bw_r_drivers_tc37x::log::info;
use bw_r_drivers_tc37x::pac::can0::{Can0, N as Can0Node};
use bw_r_drivers_tc37x::scu::wdt::{disable_cpu_watchdog, disable_safety_watchdog};
use bw_r_drivers_tc37x::scu::wdt_call::call_without_endinit;
use bw_r_drivers_tc37x::{pac, ssw};
use core::arch::asm;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::time::Duration;
use critical_section::RawRestoreState;
use embedded_can::ExtendedId;

pub static CAN0_NODE0_NEW_MSG: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn __INTERRUPT_HANDLER_2() {
    CAN0_NODE0_NEW_MSG.store(true, Ordering::SeqCst);
}

fn setup_can0() -> Option<Node<Can0Node, Can0, Node0, Configured>> {
    let can_module = Module::new(Module0);
    let mut can_module = can_module.enable();

    let cfg = NodeConfig {
        baud_rate: BitTimingConfig::Auto(AutoBitTiming {
            baud_rate: 1_000_000,
            sample_point: 8_000,
            sync_jump_width: 3,
        }),
        ..Default::default()
    };

    let mut node = can_module.take_node(Node0, cfg)?;

    node.setup_tx(&TxConfig {
        mode: TxMode::DedicatedBuffers,
        dedicated_tx_buffers_number: 2,
        fifo_queue_size: 0,
        buffer_data_field_size: DataFieldSize::_8,
        event_fifo_size: 1,
        tx_event_fifo_start_address: 0x400,
        tx_buffers_start_address: 0x440,
    });

    node.setup_rx(RxConfig {
        mode: RxMode::SharedFifo0,
        buffer_data_field_size: DataFieldSize::_8,
        fifo0_data_field_size: DataFieldSize::_8,
        fifo1_data_field_size: DataFieldSize::_8,
        fifo0_operating_mode: RxFifoMode::Blocking,
        fifo1_operating_mode: RxFifoMode::Blocking,
        fifo0_watermark_level: 0,
        fifo1_watermark_level: 0,
        fifo0_size: 4,
        fifo1_size: 0,
        rx_fifo0_start_address: 0x100,
        rx_fifo1_start_address: 0x200,
        rx_buffers_start_address: 0x300,
    });

    // TODO Can we use gpio for this?
    {
        let gpio20 = pac::P20.split();
        let _tx = gpio20.p20_8;
        let _rx = gpio20.p20_7;
        // node.setup_pins(tx, rx);
    }

    node.setup_pins(&Pins {
        tx: PIN_TX_0_0_P20_8,
        rx: PIN_RX_0_0_P20_7,
    });

    node.setup_interrupt(&NodeInterruptConfig {
        interrupt_group: InterruptGroup::Rxf0n,
        interrupt: Interrupt::RxFifo0newMessage,
        line: InterruptLine::Line1,
        priority: Priority::try_from(2).unwrap(),
        tos: Tos::Cpu0,
    });

    Some(node.lock_configuration())
}

/// Initialize the STB pin for the CAN transceiver.
fn init_can_stb_pin() {
    let gpio20 = pac::P20.split();
    let mut stb = gpio20.p20_6.into_push_pull_output();
    stb.set_low();
}

#[export_name = "main"]
fn main() -> ! {
    info!("Start example: can_send");

    info!("Enable interrupts");

    unsafe {
        asm!("enable");
    }

    info!("Setup notification LED");
    let gpio00 = pac::P00.split();
    let mut led1 = gpio00.p00_5.into_push_pull_output();

    info!("Initialize CAN transceiver");
    init_can_stb_pin();

    info!("Create CAN module ... ");
    let can0 = match setup_can0() {
        Some(can) => can,
        None => {
            info!("Can initialization error");
            loop {}
        }
    };

    info!("Define a message to send");
    let tx_msg_id: MessageId = {
        let id = 0x0CFE6E00;
        let id: ExtendedId = ExtendedId::new(id).unwrap().into();
        id.into()
    };

    info!("Allocate a buffer for the message data");
    let mut tx_msg_data: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut rx_msg_data: [u8; 8] = Default::default();

    loop {
        // Transmit a different message each time (changing the first byte)
        tx_msg_data[0] = tx_msg_data[0].wrapping_add(1);

        let tx_frame = Frame::new(tx_msg_id, tx_msg_data.as_slice()).unwrap();

        if can0.transmit(&tx_frame).is_err() {
            info!("Cannot send frame");
        }

        // if can1.transmit(&tx_frame).is_err() {
        //     info!("Cannot send frame");
        // }

        led1.set_high();
        wait_nop(Duration::from_millis(100));
        led1.set_low();
        wait_nop(Duration::from_millis(900));

        let can0_node0_received = CAN0_NODE0_NEW_MSG.load(Ordering::SeqCst);
        if can0_node0_received {
            info!("msg received");
            CAN0_NODE0_NEW_MSG.store(false, Ordering::SeqCst);

            // TODO For symmetry, it should receive a frame, with can id too
            can0.receive(ReadFrom::RxFifo0, &mut rx_msg_data);

            tx_msg_data.copy_from_slice(&rx_msg_data);

            can0.clear_interrupt_flag(Interrupt::RxFifo0newMessage);
        }
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
