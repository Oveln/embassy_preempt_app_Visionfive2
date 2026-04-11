//! # CPU频率测量
//!
//! 使用mcycle寄存器 + mtime定时器计算CPU频率
//!
//! mtime频率 = 4MHz (从OpenSBI启动信息确认)
//! mcycle = CPU周期计数器

use core::arch::asm;
use core::time::Duration;

/// mtime寄存器地址 (JH7110 CLINT)
const CLINT_BASE: usize = 0x2000000;
const MTIME_ADDR: usize = CLINT_BASE + 0xbff8;

use embassy_preempt_log::task_log;

/// 读取mtime (64位)
#[inline]
fn read_mtime() -> u64 {
    unsafe {
        let ptr = MTIME_ADDR as *const u32;
        let mut high: u32;
        let mut low: u32;

        // 防止进位时的竞争条件
        loop {
            high = ptr.add(1).read_volatile();
            low = ptr.read_volatile();
            let high2 = ptr.add(1).read_volatile();
            if high == high2 {
                break;
            }
        }

        ((high as u64) << 32) | (low as u64)
    }
}

/// 读取mcycle寄存器 (RISC-V csr)
#[inline]
fn read_mcycle() -> u64 {
    unsafe {
        let cycles: u64;
        asm!(
            "csrr {}, mcycle",
            out(reg) cycles,
            options(nomem, nostack)
        );
        cycles
    }
}

/// CPU频率测量结果
#[derive(Debug, Clone, Copy)]
pub struct CpuFreqInfo {
    /// 频率 (Hz)
    pub hz: u64,
    /// 频率 (MHz)
    pub mhz: f64,
    /// 测量的周期数
    pub cycles: u64,
    /// 测量的时间 (us)
    pub elapsed_us: u64,
}

/// 测量CPU频率
///
/// # 参数
/// * `duration_us` - 测量时长 (微秒)，建议 >= 10000
///
/// # 返回
/// CPU频率信息
pub fn measure_cpu_frequency(duration_us: u64) -> CpuFreqInfo {
    // mtime频率 = 4MHz (从OpenSBI确认)
    const MTIME_FREQ: u64 = 4_000_000;

    // 计算需要等待的mtime ticks
    let target_ticks = (MTIME_FREQ * duration_us) / 1_000_000;

    // 开始测量
    let start_mtime = read_mtime();
    let start_cycle = read_mcycle();

    // 等待指定时长 (忙等待以获得准确测量)
    let target_mtime = start_mtime.wrapping_add(target_ticks);
    let mut current_mtime: u64 = read_mtime();

    // 处理mtime溢出情况
    let overflowed = target_mtime < start_mtime;

    if overflowed {
        // 等待mtime回绕
        while current_mtime >= start_mtime {
            current_mtime = read_mtime();
        }
    }

    // 等待达到目标时间
    loop {
        current_mtime = read_mtime();
        if overflowed {
            if current_mtime >= target_mtime {
                break;
            }
        } else {
            if current_mtime >= target_mtime && current_mtime >= start_mtime {
                break;
            }
        }
    }

    // 结束测量
    let end_cycle = read_mcycle();
    let end_mtime = read_mtime();

    // 计算结果
    let cycles = end_cycle.wrapping_sub(start_cycle);
    let elapsed_ticks = end_mtime.wrapping_sub(start_mtime);
    let elapsed_us = (elapsed_ticks * 1_000_000) / MTIME_FREQ;

    // CPU频率 = 周期数 / 实际时间
    let hz = if elapsed_us > 0 {
        (cycles * 1_000_000) / elapsed_us
    } else {
        0
    };

    let mhz = hz as f64 / 1_000_000.0;

    CpuFreqInfo {
        hz,
        mhz,
        cycles,
        elapsed_us,
    }
}

/// 获取当前HART ID
pub fn get_hart_id() -> u64 {
    unsafe {
        let hart_id: u64;
        asm!(
            "csrr {}, mhartid",
            out(reg) hart_id,
            options(nomem, nostack)
        );
        hart_id
    }
}

/// HART名称映射
pub fn hart_name(hart_id: u64) -> &'static str {
    match hart_id {
        0 => "S7_0 (Monitor)",
        1 => "U74_1",
        2 => "U74_2",
        3 => "U74_3",
        4 => "U74_4",
        _ => "Unknown",
    }
}

/// 打印CPU频率信息
pub fn print_cpu_freq(info: &CpuFreqInfo) {
    let hart_id = get_hart_id();
    task_log!(info, "=== CPU频率测量结果 ===");
    task_log!(info, "HART: {} ({})", hart_id, hart_name(hart_id));
    task_log!(info, "频率: {} Hz ({:.2} MHz)", info.hz, info.mhz);
    task_log!(info, "测量周期: {} cycles", info.cycles);
    task_log!(info, "测量时间: {} us", info.elapsed_us);
}

/// 快速测量 (约10ms)
pub fn quick_measure() -> CpuFreqInfo {
    measure_cpu_frequency(10000)
}

/// 精确测量 (约100ms)
pub fn precise_measure() -> CpuFreqInfo {
    measure_cpu_frequency(100000)
}