#![no_main]
#![no_std]

//! CPU频率测量测试

use core::ffi::c_void;

use embassy_preempt_executor::{OSInit, OSStart};
use embassy_preempt_app::{bss, cpu_freq};
use embassy_preempt_log::task_log;

#[embassy_preempt_macros::entry]
fn main() -> ! {
    bss::clear_bss();

    // 获取当前HART信息
    let hart_id = cpu_freq::get_hart_id();
    task_log!(info, "========================================");
    task_log!(info, "CPU频率测量工具");
    task_log!(info, "当前HART: {} ({})", hart_id, cpu_freq::hart_name(hart_id));
    task_log!(info, "mtime频率: 4 MHz (从OpenSBI确认)");
    task_log!(info, "========================================");

    // 快速测量 (~10ms)
    task_log!(info, "");
    task_log!(info, "[快速测量] 采样时间: ~10ms");
    let quick = cpu_freq::quick_measure();
    cpu_freq::print_cpu_freq(&quick);

    // 精确测量 (~100ms)
    task_log!(info, "");
    task_log!(info, "[精确测量] 采样时间: ~100ms");
    let precise = cpu_freq::precise_measure();
    cpu_freq::print_cpu_freq(&precise);

    // 根据HART类型判断频率范围
    task_log!(info, "");
    task_log!(info, "频率分析:");
    if hart_id == 0 {
        // S7监控核心
        task_log!(info, "  S7核心通常运行在固定频率");
        if precise.mhz >= 400.0 && precise.mhz <= 600.0 {
            task_log!(info, "  -> 可能是 500 MHz");
        } else if precise.mhz >= 600.0 && precise.mhz <= 900.0 {
            task_log!(info, "  -> 可能是 750 MHz (需要确认)");
        } else {
            task_log!(info, "  -> 频率: {:.2} MHz", precise.mhz);
        }
    } else {
        // U74应用核心
        task_log!(info, "  U74核心支持DVFS:");
        task_log!(info, "  -> 375 MHz (低功耗)");
        task_log!(info, "  -> 500 MHz (平衡)");
        task_log!(info, "  -> 750 MHz (性能)");
        task_log!(info, "  -> 1500 MHz (极速)");
        match precise.mhz {
            f if f >= 350.0 && f <= 400.0 => task_log!(info, "  -> 当前可能是 375 MHz"),
            f if f >= 450.0 && f <= 550.0 => task_log!(info, "  -> 当前可能是 500 MHz"),
            f if f >= 700.0 && f <= 800.0 => task_log!(info, "  -> 当前可能是 750 MHz"),
            f if f >= 1400.0 && f <= 1600.0 => task_log!(info, "  -> 当前可能是 1500 MHz"),
            _ => task_log!(info, "  -> 当前频率: {:.2} MHz", precise.mhz),
        }
    }

    task_log!(info, "");
    task_log!(info, "测量完成，系统将停止...");
    task_log!(info, "========================================");

    // 停止在这里
    loop {
        unsafe { riscv::asm::wfi() };
    }
}
