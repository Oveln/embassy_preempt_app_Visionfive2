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
use crate::gpio::gpio;

use embassy_preempt_executor::{os_time::timer::Timer, AsyncOSTaskCreate, OSInit, OSStart};
use embassy_preempt_log::task_log;

use embassy_preempt_app::{bss, gpio, system_info};

async fn high_task(_args: *mut c_void) {
    for i in 0..1000 {
        task_log!(info, "切换次数: {}", i);
        unsafe { gpio().toggle(45); }
        Timer::after_micros(10000000).await;
    }
}

async fn low_task(_args: *mut c_void) {
    loop {
        unsafe { gpio().toggle(37); }
    }
}

#[embassy_preempt_macros::entry]
fn main() -> ! {
    bss::clear_bss();

    task_log!(info, "=== 上下文切换测试 ===");
    task_log!(info, "GPIO45 - 切换时间");
    task_log!(info, "GPIO37/39/40 - 内核标记");

    // 初始化GPIO
    unsafe { gpio::init_gpio(); }

    OSInit();
    AsyncOSTaskCreate(high_task, core::ptr::null_mut(), core::ptr::null_mut(), 20);
    AsyncOSTaskCreate(low_task, core::ptr::null_mut(), core::ptr::null_mut(), 30);
    task_log!(info, "启动...");

    OSStart()
}
