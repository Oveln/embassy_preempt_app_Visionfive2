#![no_main]
#![no_std]

//! # 上下文切换性能测试
//!
//! - 高优先级任务: GPIO45=high → await
//! - 低优先级任务: GPIO45=low → await
//!
//! GPIO45脉冲宽度 = 上下文切换时间
//! GPIO37/39/40 = 内核自动标记

use core::ffi::c_void;

use embassy_preempt_executor::{os_time::timer::Timer, AsyncOSTaskCreate, OSInit, OSStart};
use embassy_preempt_log::task_log;

use embassy_preempt_app::system_info;

use aclint::SifiveClint;
/// CLINT base address for JH7110
const CLINT_BASE: usize = 0x02000000;

pub const CLINT: *const SifiveClint = CLINT_BASE as *const SifiveClint;


async fn task(_args: *mut c_void) {
    for i in 0..1000 {
        use riscv::register::{mstatus, mip};
        
        let mtimer_en = riscv::register::mie::read().mtimer();
        let mtimer_pending = mip::read().mtimer();  // 中断挂起位
        let global_irq_en = mstatus::read().mie();   // 全局中断使能
        
        let mtimercmp = unsafe {(*CLINT).read_mtimecmp(0)};
        let timer = unsafe {(*CLINT).read_mtime()};

        task_log!(info, "task1: mstatus.MIE={}, mie.MTIE={}, mip.MTIP={}, mtimercmp={}, timer={}", 
            global_irq_en, mtimer_en, mtimer_pending, mtimercmp, timer);
    
        Timer::after_micros(100000000).await;
    }
}

async fn task_2(_args: *mut c_void) {
    loop {
        use riscv::register::{mstatus, mip};
        
        let mtimer_en = riscv::register::mie::read().mtimer();
        let mtimer_pending = mip::read().mtimer();  // 中断挂起位
        let global_irq_en = mstatus::read().mie();   // 全局中断使能
        
        let mtimercmp = unsafe {(*CLINT).read_mtimecmp(0)};
        let timer = unsafe {(*CLINT).read_mtime()};

        task_log!(info, "task2: mstatus.MIE={}, mie.MTIE={}, mip.MTIP={}, mtimercmp={}, timer={}", 
            global_irq_en, mtimer_en, mtimer_pending, mtimercmp, timer);
    }
}

#[embassy_preempt_macros::entry]
fn main() -> ! {
    task_log!(info, "=== 定时器测试 ===");

    // 初始化GPIO
    unsafe {
        let ptr = 0x13040000_usize as *mut u32;
        // GPIO45,37,39,40 output enable
        ptr.add(0).write_volatile(0x00000000);
        ptr.add(1).write_volatile(0x00000000);
        ptr.add(2).write_volatile(0x00000000);
    }

    OSInit();
    system_info::print_trap_vector_info();
    AsyncOSTaskCreate(task, core::ptr::null_mut(), core::ptr::null_mut(), 20);
    AsyncOSTaskCreate(task_2, core::ptr::null_mut(), core::ptr::null_mut(), 30);
    task_log!(info, "启动...");

    OSStart()
}
