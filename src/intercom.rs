//! Inter-system communication module for embassy_preempt
//!
//! Shared memory layout at 0xc8000100:
//! - Channel 0: StarryOS -> embassy_preempt (requests/notifications)
//! - Channel 1: embassy_preempt -> StarryOS (responses/notifications)

#![no_std]

use ov_channal::{ChannelId, Message, MsgType, SharedMemory};
use embassy_preempt_log::task_log;

/// Shared memory base address: 0xc8000000 + 256
pub const SHM_BASE_ADDR: usize = 0xc8000100;

/// Initialize shared memory
pub fn init() {
    unsafe {
        let shm = SharedMemory::at(SHM_BASE_ADDR);
        shm.init();
    }
    task_log!(info, "[InterCom] Initialized at {:#x}", SHM_BASE_ADDR);
}

/// Check if StarryOS sent us a message
pub fn has_pending() -> bool {
    unsafe {
        let shm = SharedMemory::at(SHM_BASE_ADDR);
        shm.receiver(ChannelId::new(0))
            .map_or(false, |rx| rx.has_pending())
    }
}

/// Receive and process all pending messages
pub fn process_pending() {
    unsafe {
        let shm = SharedMemory::at(SHM_BASE_ADDR);
        if let Ok(rx) = shm.receiver(ChannelId::new(0)) {
            while let Some(msg) = rx.try_recv() {
                handle_message(msg);
            }
        }
    }
}

/// Handle a single message
fn handle_message(msg: Message) {
    match msg.ty() {
        Some(MsgType::Notification) => {
            if let Some(id) = msg.as_notification() {
                task_log!(info, "[InterCom] Notification: {}", id);
                // Echo back
                send_notification(id);
            }
        }
        Some(MsgType::Request) => {
            if let Some(method_id) = msg.method_id() {
                task_log!(info, "[InterCom] Request: method={}", method_id);
                handle_request(method_id, msg);
            }
        }
        _ => {
            task_log!(warn, "[InterCom] Unknown msg type");
        }
    }
}

/// Handle RPC request
fn handle_request(method_id: u64, msg: Message) {
    const HELLO_WORLD: u64 = 0;
    const ADD: u64 = 1;

    match method_id {
        HELLO_WORLD => {
            task_log!(info, "[InterCom] HELLO_WORLD!");
            let request_id = match msg.as_request::<()>() {
                Some((rid, _, _)) => rid,
                None => {
                    task_log!(warn, "[InterCom] Invalid HELLO_WORLD request");
                    return;
                }
            };
            let result = 0;
            if let Ok(msg) = Message::response(request_id, &result) {
                send_message(msg);
            }
        }
        ADD => {
            task_log!(info, "[InterCom] ADD method called");
            let (request_id, args) = match msg.as_request::<(i32, i32)>() {
                Some((rid, _, args)) => (rid, args),
                None => {
                    task_log!(warn, "[InterCom] Invalid ADD request");
                    return;
                }
            };
            let (a, b) = args;
            let result: i32 = a + b;
            if let Ok(msg) = Message::response(request_id, &result) {
                send_message(msg);
            }
        }
        _ => {
            task_log!(warn, "[InterCom] Unknown method: {}", method_id);
        }
    }
}

/// Send message to StarryOS
fn send_message(msg: Message) {
    unsafe {
        let shm = SharedMemory::at(SHM_BASE_ADDR);
        if let Ok(tx) = shm.sender(ChannelId::new(1)) {
            if tx.try_send(&msg).is_ok() {
                trigger_ipi_hart1();
            }
        }
    }
}

/// Send notification to StarryOS
pub fn send_notification(id: u32) {
    let msg = Message::notification(id);
    send_message(msg);
}

/// Trigger IPI to hart 1
fn trigger_ipi_hart1() {
    crate::sync::get_hart_sync().set_ipi_sent();
    unsafe {
        let msip: *mut u32 = 0x0200_0004 as *mut u32;
        core::ptr::write_volatile(msip, 1);
    }
}