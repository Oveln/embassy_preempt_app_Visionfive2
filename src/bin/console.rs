#![no_main]
#![no_std]

//! # Embassy Preempt Console Application for JH7110
//!
//! This is a optimized console application for the VisionFive2 JH7110 development board
//! that demonstrates multi-hart communication and preemptive task scheduling.

use core::ffi::c_void;
use core::sync::atomic::Ordering;
use portable_atomic::AtomicBool;

use critical_section::Mutex;
use embassy_preempt_executor::{os_time::blockdelay::delay, os_time::timer::Timer, AsyncOSTaskCreate, OSInit, OSStart, SyncOSTaskCreate};
use embassy_preempt_log::task_log;
use embassy_preempt_platform::{get_platform, get_platform_trait};

// Import library modules
use embassy_preempt_app::{gpio, intercom, sync, system_info};

// ============================================================================
// Shared Memory Structures
// ============================================================================

/// Execution order tracking for debugging
static EXECUTION_ORDER: Mutex<[&'static str; 20]> = Mutex::new([""; 20]);
static mut ORDER_INDEX: usize = 0;

// ============================================================================
// Task Constants
// ============================================================================

const LONG_TIME: usize = 10;
const MID_TIME: usize = 5;
const SHORT_TIME: usize = 3;

// ============================================================================
// Task Definitions
// ============================================================================

/// Task 1 - Long delay task
fn task1(_args: *mut c_void) {
    task_log!(info, "---task1 begin---");
    delay(LONG_TIME);
    task_log!(info, "---task1 end---");
    delay(SHORT_TIME);
}

/// Task 2 - Medium delay task
fn task2(_args: *mut c_void) {
    task_log!(info, "---task2 begin---");
    delay(MID_TIME);
    task_log!(info, "---task2 end---");
    delay(SHORT_TIME);
}

/// Task 3 - Async task example
async fn task3(_args: *mut c_void) {
    task_log!(info, "---task3 begin---");
    // Timer::after_ticks(LONG_TIME as u64).await;
    task_log!(info, "---task3 end---");
    delay(SHORT_TIME);
}

/// Task 4 - Dynamic task creation test
fn task4(_args: *mut c_void) {
    task_log!(info, "---task4 begin---");
    SyncOSTaskCreate(task1, core::ptr::null_mut(), core::ptr::null_mut(), 34);
    delay(SHORT_TIME);
    task_log!(info, "---task4 end---");
    delay(SHORT_TIME);
}

/// Task 5 - Task stack pointer test
fn task5(_args: *mut c_void) {
    task_log!(info, "---task5 begin---");
    let ptos = core::ptr::null_mut::<usize>();
    task_log!(info, "ptos is {:p}", ptos);
    SyncOSTaskCreate(task1, core::ptr::null_mut(), ptos, 9);
    task_log!(info, "created task1 in task5");
    delay(SHORT_TIME);
    task_log!(info, "---task5 end---");
    delay(SHORT_TIME);
}

/// Task 6 - Same priority task creation test
fn task6(_args: *mut c_void) {
    task_log!(info, "---task6 begin---");
    SyncOSTaskCreate(task1, core::ptr::null_mut(), core::ptr::null_mut(), 35);
    delay(SHORT_TIME);
    task_log!(info, "---task6 end---");
    delay(SHORT_TIME);
}

static A: portable_atomic::AtomicUsize = portable_atomic::AtomicUsize::new(0);
static B: portable_atomic::AtomicUsize = portable_atomic::AtomicUsize::new(0);

async fn task7(_args: *mut c_void) {
    task_log!(info, "[InterCom] Task started, waiting for messages from StarryOS");

    loop {
        // Wait for IPI from StarryOS
        embassy_preempt_executor::ipi::wait_for_ipi().await;

        // Process any pending messages
        {
            // intercom::SWITCH_CONTEXT_CYCLE_COUNT = B - A;
            use portable_atomic::Ordering;
            intercom::SWITCH_CONTEXT_CYCLE_COUNT.store(B.load(Ordering::Acquire) - A.load(Ordering::Acquire), Ordering::Release);
        }
        task_log!(info, "SWITCH_CONTEXT_CYCLE_COUNT = {}", unsafe{intercom::SWITCH_CONTEXT_CYCLE_COUNT.load(Ordering::Acquire)});
        intercom::process_pending();
        unsafe{crate::gpio::gpio().set_high(45);}
        B.store(0, Ordering::Release);
        A.store(riscv::register::cycle::read(), Ordering::Release);
    }
}

async fn task8(_args: *mut c_void) {
    loop {
        if B.load(Ordering::Acquire) == 0 {
            B.store(riscv::register::cycle::read(), Ordering::Release);
        }
        unsafe{crate::gpio::gpio().set_low(37);}
        unsafe{crate::gpio::gpio().set_low(45);}
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[embassy_preempt_macros::entry]
fn main() -> ! {

    // Display early trap vector info for debugging
    system_info::print_trap_vector_info();
    unsafe {
        gpio::init_gpio();
    }

    // Initialize OS
    OSInit();

    // Initialize inter-system communication
    intercom::init();

    task_log!(info, "[OS Status] OSInit completed!");
    task_log!(info, "========================================");
    task_log!(info, "  Hello, Embassy Preempt on VisionFive2!");
    task_log!(info, "========================================\r\n");

    // Display comprehensive system information
    system_info::print_system_info();

    let hart_sync: &sync::HartSyncFlags = sync::get_hart_sync();
    task_log!(info, "HART_SYNC addr is {:#x}", core::ptr::addr_of!(*hart_sync) as usize);

    // Initialize HartSyncFlags (set magic number and initial state)
    hart_sync.init();

    task_log!(info, "HART_SYNC magic: {:#04x}, valid: {}", 0x0721, hart_sync.is_valid());

    // ========================================================================
    // Task Creation - Demonstrating priority-based scheduling
    // ========================================================================
    // Priority: Higher number = Higher priority

    SyncOSTaskCreate(task1, core::ptr::null_mut(), core::ptr::null_mut(), 30);
    SyncOSTaskCreate(task2, core::ptr::null_mut(), core::ptr::null_mut(), 25);
    AsyncOSTaskCreate(task3, core::ptr::null_mut(), core::ptr::null_mut(), 20);
    SyncOSTaskCreate(task4, core::ptr::null_mut(), core::ptr::null_mut(), 15);
    SyncOSTaskCreate(task5, core::ptr::null_mut(), core::ptr::null_mut(), 10);
    SyncOSTaskCreate(task6, core::ptr::null_mut(), core::ptr::null_mut(), 35);
    AsyncOSTaskCreate(task7, core::ptr::null_mut(), core::ptr::null_mut(), 36);
    AsyncOSTaskCreate(task8, core::ptr::null_mut(), core::ptr::null_mut(), 50);
    // Start OS (never returns)
    OSStart();

    loop {}
}
