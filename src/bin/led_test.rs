#![no_main]
#![no_std]

//! # LED控制测试程序
//!
//! 使用GPIO 45, 37, 39, 40控制4个LED
//! 验证GPIO功能是否正常工作
//!
//! ## GPIO引脚分配
//! - **GPIO45**: LED1
//! - **GPIO37**: LED2
//! - **GPIO39**: LED3
//! - **GPIO40**: LED4
//!
//! ## 测试模式
//! 1. 所有LED同时闪烁
//! 2. LED依次点亮（流水灯）
//! 3. LED交替闪烁
//! 4. LED随机闪烁

use core::ffi::c_void;

use embassy_preempt_executor::{OSInit, OSStart, SyncOSTaskCreate};
use embassy_preempt_log::task_log;

use embassy_preempt_app::{gpio::TestGpio, system_info};

// ============================================================================
// 全局变量
// ============================================================================

/// 测试模式选择
static mut TEST_MODE: u32 = 0;

// ============================================================================
// GPIO辅助函数
// ============================================================================

/// 简单延时函数 (使用忙等待)
fn simple_delay(ms: u32) {
    // JH7110运行在较高频率，这里需要根据实际频率调整
    // 假设每毫秒大约需要几万次循环
    let loops_per_ms = 50000;
    for _ in 0..(ms * loops_per_ms) {
        core::hint::spin_loop();
        unsafe {
            core::arch::asm!("nop");
        }
    }
}

/// 控制单个LED
fn led_on(pin: u32) {
    unsafe {
        embassy_preempt_app::gpio::gpio().set_high(pin);
    }
}

fn led_off(pin: u32) {
    unsafe {
        embassy_preempt_app::gpio::gpio().set_low(pin);
    }
}

fn led_toggle(pin: u32) {
    unsafe {
        embassy_preempt_app::gpio::gpio().toggle(pin);
    }
}

// ============================================================================
// 测试模式函数
// ============================================================================

/// 模式1: 所有LED同时闪烁
fn mode_all_blink(iterations: u32) {
    task_log!(info, "[LED] 模式1: 所有LED同时闪烁 - {}次", iterations);

    for i in 0..iterations {
        // 所有LED亮
        led_on(45); // GPIO45 - LED1
        led_on(37); // GPIO37 - LED2
        led_on(39); // GPIO39 - LED3
        led_on(40); // GPIO40 - LED4

        simple_delay(500); // 亮500ms

        // 所有LED灭
        led_off(45);
        led_off(37);
        led_off(39);
        led_off(40);

        simple_delay(500); // 灭500ms

        task_log!(info, "[LED] 闪烁循环 {}/{}", i + 1, iterations);
    }
}

/// 模式2: 流水灯效果
fn mode_running_light(iterations: u32) {
    task_log!(info, "[LED] 模式2: 流水灯效果 - {}次", iterations);

    let pins = [45u32, 37, 39, 40];

    for _ in 0..iterations {
        // 从左到右
        for &pin in &pins {
            led_on(pin);
            simple_delay(200);
            led_off(pin);
        }

        // 从右到左
        for &pin in pins.iter().rev() {
            led_on(pin);
            simple_delay(200);
            led_off(pin);
        }
    }
}

/// 模式3: LED交替闪烁
fn mode_alternate_blink(iterations: u32) {
    task_log!(info, "[LED] 模式3: LED交替闪烁 - {}次", iterations);

    for _ in 0..iterations {
        // LED1和LED3亮
        led_on(45); // GPIO45 - LED1
        led_on(39); // GPIO39 - LED3
        led_off(37); // GPIO37 - LED2
        led_off(40); // GPIO40 - LED4

        simple_delay(500);

        // LED2和LED4亮
        led_off(45);
        led_off(39);
        led_on(37);
        led_on(40);

        simple_delay(500);
    }

    // 全部关闭
    led_off(45);
    led_off(37);
    led_off(39);
    led_off(40);
}

/// 模式4: 逐个LED依次点亮然后全部熄灭
fn mode_sequential_on_off(iterations: u32) {
    task_log!(info, "[LED] 模式4: 逐个点亮然后全部熄灭 - {}次", iterations);

    let pins = [45u32, 37, 39, 40];

    for _ in 0..iterations {
        // 逐个点亮
        for &pin in &pins {
            led_on(pin);
            simple_delay(300);
        }

        simple_delay(500);

        // 全部熄灭
        for &pin in &pins {
            led_off(pin);
            simple_delay(300);
        }

        simple_delay(500);
    }
}

/// 模式5: 快速随机闪烁
fn mode_random_blink(iterations: u32) {
    task_log!(info, "[LED] 模式5: 快速闪烁 - {}次", iterations);

    let pins = [45u32, 37, 39, 40];

    for i in 0..iterations {
        // 随机选择一个LED翻转
        let pin = pins[(i as usize) % pins.len()];
        led_toggle(pin);

        simple_delay(100); // 快速闪烁
    }

    // 全部关闭
    for &pin in &pins {
        led_off(pin);
    }
}

/// 测试单个LED
fn test_single_led() {
    task_log!(info, "[LED] ========== 单个LED测试 ==========");

    let pins = [45u32, 37, 39, 40];
    let names = ["GPIO45", "GPIO37", "GPIO39", "GPIO40"];

    for (i, &pin) in pins.iter().enumerate() {
        task_log!(info, "[LED] 测试 {} (LED{})", names[i], i + 1);

        // 亮3次
        for _ in 0..3 {
            led_on(pin);
            simple_delay(300);
            led_off(pin);
            simple_delay(300);
        }
    }

    task_log!(info, "[LED] 单个LED测试完成");
}

// ============================================================================
// 任务函数
// ============================================================================

/// LED控制任务
fn led_control_task(_args: *mut c_void) {
    task_log!(info, "[LED] LED控制任务启动");
    task_log!(info, "[LED] ========================================");
    task_log!(info, "[LED] GPIO引脚分配:");
    task_log!(info, "[LED]   GPIO45 -> LED1");
    task_log!(info, "[LED]   GPIO37 -> LED2");
    task_log!(info, "[LED]   GPIO39 -> LED3");
    task_log!(info, "[LED]   GPIO40 -> LED4");
    task_log!(info, "[LED] ========================================");

    simple_delay(2000); // 启动延迟

    // 测试单个LED
    test_single_led();

    simple_delay(1000);

    // 运行各种测试模式
    loop {
        task_log!(info, "[LED] ========================================");
        task_log!(info, "[LED] 开始LED测试序列");
        task_log!(info, "[LED] ========================================");

        // 模式1: 同时闪烁 (5次)
        mode_all_blink(5);
        simple_delay(1000);

        // 模式2: 流水灯 (3次)
        mode_running_light(3);
        simple_delay(1000);

        // 模式3: 交替闪烁 (5次)
        mode_alternate_blink(5);
        simple_delay(1000);

        // 模式4: 逐个点亮 (3次)
        mode_sequential_on_off(3);
        simple_delay(1000);

        // 模式5: 快速闪烁 (50次)
        mode_random_blink(50);
        simple_delay(1000);

        task_log!(info, "[LED] ========================================");
        task_log!(info, "[LED] LED测试序列完成，3秒后重新开始");
        task_log!(info, "[LED] ========================================");

        simple_delay(3000);
    }
}

/// GPIO状态监控任务
fn gpio_monitor_task(_args: *mut c_void) {
    task_log!(info, "[MONITOR] GPIO监控任务启动");

    loop {
        // 读取GPIO状态
        unsafe {
            let gpio = embassy_preempt_app::gpio::gpio();

            // 这里可以添加GPIO输入测试
            // 目前只监控输出状态
            task_log!(info, "[MONITOR] GPIO状态检查完成");
        }

        embassy_preempt_executor::os_time::blockdelay::delay(100);
    }
}

// ============================================================================
// 主函数
// ============================================================================

#[embassy_preempt_macros::entry]
fn main() -> ! {
    // 显示系统信息
    system_info::print_trap_vector_info();

    task_log!(info, "========================================");
    task_log!(info, "  LED控制测试程序");
    task_log!(info, "========================================");
    task_log!(info, "GPIO引脚: 45, 37, 39, 40");
    task_log!(info, "功能: 验证GPIO基本功能");
    task_log!(info, "========================================\n");

    // 初始化GPIO
    task_log!(info, "[INIT] 初始化GPIO...");

    unsafe {
        embassy_preempt_app::gpio::init_gpio();
    }

    task_log!(info, "[INIT] GPIO初始化完成");

    // 初始化OS
    OSInit();

    task_log!(info, "[INIT] OS初始化完成");
    task_log!(info, "[INIT] 创建LED控制任务...\n");

    // 创建LED控制任务 (高优先级)
    SyncOSTaskCreate(led_control_task, core::ptr::null_mut(), core::ptr::null_mut(), 10);

    // 创建GPIO监控任务 (低优先级)
    SyncOSTaskCreate(gpio_monitor_task, core::ptr::null_mut(), core::ptr::null_mut(), 20);

    task_log!(info, "[START] 启动OS调度...\n");
    task_log!(info, "========================================");
    task_log!(info, "LED测试开始!");
    task_log!(info, "观察LED是否按照预期模式闪烁");
    task_log!(info, "========================================\n");

    // 启动OS (永不返回)
    OSStart()
}
