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
use embassy_preempt_platform::{chip::constants::interrupt::MSIP, get_platform, get_platform_trait};

// Import library modules
use embassy_preempt_app::{bss, sync, system_info};

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

async fn task7(_args: *mut c_void) {
    let hart_sync = sync::get_hart_sync();

    /// Trigger machine software interrupt for hart 1
    fn trigger_misp_hart1() {
        unsafe {
            task_log!(info, "Triggering IPI for hart 1");
            // Hart 1 MSIP address is 0x02000004
            let msip_hart1: *mut u32 = 0x0200_0004 as *mut u32;
            core::ptr::write_volatile(msip_hart1, 1);
        }
    }

    loop {
        task_log!(info, "Heartbeat from hart 0");
        // Timer::after_ticks(16_000_000).await;
        embassy_preempt_executor::ipi::wait_for_ipi().await;

        // Check if hart 1 is ready and send IPI
        if hart_sync.is_hart1_ready() {
            hart_sync.set_ipi_sent();
            trigger_misp_hart1();
        }
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[embassy_preempt_macros::entry]
fn main() -> ! {
    // Clear BSS section
    bss::clear_bss();

    // Display early trap vector info for debugging
    system_info::print_trap_vector_info();

    // Initialize OS
    OSInit();

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

    // Start OS (never returns)
    OSStart();
}
