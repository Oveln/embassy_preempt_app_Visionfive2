#!/usr/bin/env cargo
/*
# 上下文切换精确性能测试

## 测试原理
1. 两个相同优先级的任务交替执行
2. 使用 mcycle CSR 寄存器精确测量周期数
3. 使用极短的 Timer 触发上下文切换

## 编译运行
cargo build --release --bin ctx_switch_test --features "jh7110"
*/

#![no_main]
#![no_std]

use core::ffi::c_void;
use core::sync::atomic::{AtomicU64, Ordering};

use embassy_preempt_app::{gpio, system_info};
use embassy_preempt_executor::{os_time::timer::Timer, AsyncOSTaskCreate, OSInit, OSStart};
use embassy_preempt_log::task_log;

// 存储测量结果
static mut CYCLES_A2B: [u64; 100] = [0; 100];
static mut CYCLES_B2A: [u64; 100] = [0; 100];
static mut COUNT: usize = 0;

/// 读取 mcycle 寄存器 (64位)
#[inline(always)]
fn read_mcycle() -> u64 {
    let mut cycles: u64;
    unsafe {
        core::arch::asm!(
            "csrrs {}, mcycle, x0",
            out(reg) cycles
        );
    }
    cycles
}

/// 任务 A: 高优先级，设置 GPIO 高，触发切换
async fn task_a(_args: *mut c_void) {
    for i in 0..100 {
        // 设置 GPIO45 为高
        unsafe { gpio().set_high(45); }

        // 记录切换前的时间
        let start = read_mcycle();

        // 使用极短的 Timer 触发上下文切换
        Timer::after_micros(1).await;

        // 被唤醒后，记录从 A 到 B 再回到 A 的时间
        let end = read_mcycle();
        unsafe {
            CYCLES_A2B[i] = end - start;
        }
    }

    // 打印统计结果
    unsafe {
        task_log!(info, "=== 上下文切换测试完成 ===");
        task_log!(info, "总测试次数: {}", COUNT);

        // 计算平均值
        let mut sum_a2b = 0u64;
        let mut sum_b2a = 0u64;
        for i in 0..COUNT {
            sum_a2b += CYCLES_A2B[i];
            sum_b2a += CYCLES_B2A[i];
        }

        let avg_a2b = sum_a2b / COUNT as u64;
        let avg_b2a = sum_b2a / COUNT as u64;

        task_log!(info, "A->B 平均周期: {} ({:.3} μs @ 1GHz)", avg_a2b, avg_a2b as f64 / 1000.0);
        task_log!(info, "B->A 平均周期: {} ({:.3} μs @ 1GHz)", avg_b2a, avg_b2a as f64 / 1000.0);

        // 估算单向上下文切换时间（假设 A->B 和 B->A 对称）
        let ctx_sw_cycles = (avg_a2b + avg_b2a) / 4;
        task_log!(info, "估算单向上下文切换时间: {} ({:.3} μs @ 1GHz)", ctx_sw_cycles, ctx_sw_cycles as f64 / 1000.0);
    }

    loop {
        Timer::after_micros(1000000).await;
    }
}

/// 任务 B: 低优先级，记录时间
async fn task_b(_args: *mut c_void) {
    for i in 0..100 {
        // 被唤醒时，记录从 A 切换过来的时间
        let start = read_mcycle();

        // 设置 GPIO45 为低
        unsafe { gpio().set_low(45); }

        // 立即触发切换回 A
        Timer::after_micros(1).await;

        // 记录 B->A 的切换时间
        let end = read_mcycle();
        unsafe {
            CYCLES_B2A[i] = end - start;
            COUNT = i + 1;
        }
    }

    loop {
        Timer::after_micros(1000000).await;
    }
}

#[embassy_preempt_macros::entry]
fn main() -> ! {
    task_log!(info, "=== 上下文切换精确性能测试 ===");
    task_log!(info, "测试方法: 使用 mcycle 计数器测量上下文切换周期数");
    task_log!(info, "GPIO45: 任务A运行时为高，任务B运行时为低");

    // 初始化 GPIO
    unsafe { gpio::init_gpio(); }

    OSInit();
    // A 优先级稍高，确保先执行
    AsyncOSTaskCreate(task_a, core::ptr::null_mut(), core::ptr::null_mut(), 20);
    AsyncOSTaskCreate(task_b, core::ptr::null_mut(), core::ptr::null_mut(), 21);
    task_log!(info, "启动测试...");

    OSStart()
}
