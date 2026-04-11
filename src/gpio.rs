//! GPIO控制模块用于JH7110平台调度性能测试
//!
//! 使用GPIO 45, 37, 39, 40引脚标记任务切换时间点
//! 通过逻辑分析仪测量embassy_preempt调度性能
//!
//! # JH7110 GPIO寄存器布局 (基于U-Boot)
//! - 基地址: 0x13040000
//! - DOEN (Output Enable): 0=使能输出, 1=禁用输出
//! - DOUT (Data Output): 控制0/1输出电平
//! - DIN (Data Input): 读取输入电平
//! - 每4个GPIO一组，每组4字节，每GPIO占8位

use core::ptr::{read_volatile, write_volatile};

// GPIO寄存器基地址 (sysgpio: pinctrl@13040000)
const SYS_GPIO_BASE: usize = 0x13040000;

// 基于 U-Boot 的准确寄存器偏移 (arch/riscv/include/asm/arch-jh7110/gpio.h)
const GPIO_DOEN: usize = 0x00;   // Output Enable寄存器基地址
const GPIO_DOUT: usize = 0x40;   // Data Output寄存器基地址
const GPIO_DIN: usize = 0x80;    // Data Input寄存器基地址

// 掩码定义 (基于 U-Boot)
const GPIO_DOEN_MASK: u32 = 0x3f;
const GPIO_DOUT_MASK: u32 = 0x7f;

// 常量定义
const GPOUT_LOW: u8 = 0;
const GPOUT_HIGH: u8 = 1;
const GPOEN_ENABLE: u8 = 0;  // 0 = 使能输出
const GPOEN_DISABLE: u8 = 1; // 1 = 禁用输出

/// 性能测试使用的GPIO引脚
pub enum TestPin {
    /// GPIO45 - 标记任务切换开始
    TaskSwitchStart = 45,
    /// GPIO37 - 标记任务切换结束
    TaskSwitchEnd = 37,
    /// GPIO39 - 标记高优先级任务运行
    HighPrioTask = 39,
    /// GPIO40 - 标记中优先级任务运行
    MidPrioTask = 40,
}

/// JH7110 GPIO控制器
pub struct GpioController {
    base: usize,
}

impl GpioController {
    /// 创建新的GPIO控制器
    pub const unsafe fn new() -> Self {
        Self {
            base: SYS_GPIO_BASE,
        }
    }

    /// 读取GPIO寄存器
    #[inline]
    unsafe fn read_reg(&self, offset: usize) -> u32 {
        read_volatile((self.base + offset) as *const u32)
    }

    /// 写入GPIO寄存器
    #[inline]
    unsafe fn write_reg(&self, offset: usize, value: u32) {
        write_volatile((self.base + offset) as *mut u32, value);
    }

    /// 修改寄存器位: 清除clr_mask并设置set_mask
    #[inline]
    unsafe fn clrsetbits(&self, offset: usize, clr_mask: u32, set_mask: u32) {
        let current = self.read_reg(offset);
        self.write_reg(offset, (current & !clr_mask) | set_mask);
    }

    /// 计算GPIO引脚的寄存器偏移
    /// U-Boot宏: #define gpio_offset(gpio) (((gpio) >> 2) << 2)
    #[inline]
    const fn gpio_offset(gpio: u32) -> usize {
        ((gpio >> 2) << 2) as usize
    }

    /// 计算GPIO引脚的位移
    /// U-Boot宏: #define gpio_shift(gpio) ((gpio) & 0x3) << 3
    #[inline]
    const fn gpio_shift(gpio: u32) -> u32 {
        (gpio & 0x3) << 3
    }

    /// 设置DOEN寄存器 (基于U-Boot的sys_iomux_doen)
    #[inline]
    unsafe fn set_doen(&self, gpio: u32, oen: u32) {
        let offset = Self::gpio_offset(gpio);
        let shift = Self::gpio_shift(gpio);
        self.clrsetbits(
            GPIO_DOEN + offset,
            GPIO_DOEN_MASK << shift,
            oen << shift,
        );
    }

    /// 设置DOUT寄存器 (基于U-Boot的sys_iomux_dout)
    #[inline]
    unsafe fn set_dout(&self, gpio: u32, gpo: u32) {
        let offset = Self::gpio_offset(gpio);
        let shift = Self::gpio_shift(gpio);
        self.clrsetbits(
            GPIO_DOUT + offset,
            GPIO_DOUT_MASK << shift,
            (gpo & GPIO_DOUT_MASK) << shift,
        );
    }

    /// 读取DIN寄存器 (基于U-Boot的sys_iomux_din_read)
    #[inline]
    unsafe fn read_din(&self, gpio: u32) -> bool {
        let offset = GPIO_DIN + (((gpio >> 5) * 4) as usize);
        let value = self.read_reg(offset);
        ((value >> (gpio & 0x1F)) & 0x1) != 0
    }

    /// 配置GPIO引脚为输出模式（低电平）
    pub fn set_output(&self, pin: u32) {
        unsafe {
            // 设置为输出模式 (oen = 0)
            self.set_doen(pin, GPOEN_ENABLE as u32);
            // 初始输出低电平
            self.set_dout(pin, GPOUT_LOW as u32);
        }
    }

    /// 设置GPIO引脚输出高电平
    pub fn set_high(&self, pin: u32) {
        unsafe {
            self.set_dout(pin, GPOUT_HIGH as u32);
        }
    }

    /// 设置GPIO引脚输出低电平
    pub fn set_low(&self, pin: u32) {
        unsafe {
            self.set_dout(pin, GPOUT_LOW as u32);
        }
    }

    /// 翻转GPIO引脚状态
    pub fn toggle(&self, pin: u32) {
        unsafe {
            let offset = Self::gpio_offset(pin);
            let shift = Self::gpio_shift(pin);
            let dout_offset = GPIO_DOUT + offset;

            // 读取当前值并翻转
            let reg_val = self.read_reg(dout_offset);
            let current = (reg_val >> shift) & 0x7f;

            let new_value = if current == 0 { 1 } else { 0 };
            self.clrsetbits(
                dout_offset,
                GPIO_DOUT_MASK << shift,
                new_value << shift,
            );
        }
    }

    /// 读取GPIO引脚的输入值（调试用）
    pub fn read_input(&self, pin: u32) -> bool {
        unsafe { self.read_din(pin) }
    }
}

/// 全局GPIO控制器
static mut GPIO_CONTROLLER: Option<GpioController> = None;

/// 初始化GPIO控制器
pub unsafe fn init_gpio() {
    GPIO_CONTROLLER = Some(GpioController::new());

    // 初始化测试引脚为输出模式，初始状态为低
    let gpio = GPIO_CONTROLLER.as_ref().unwrap();
    gpio.set_output(TestPin::TaskSwitchStart as u32);
    gpio.set_output(TestPin::TaskSwitchEnd as u32);
    gpio.set_output(TestPin::HighPrioTask as u32);
    gpio.set_output(TestPin::MidPrioTask as u32);

    gpio.set_low(TestPin::TaskSwitchStart as u32);
    gpio.set_low(TestPin::TaskSwitchEnd as u32);
    gpio.set_low(TestPin::HighPrioTask as u32);
    gpio.set_low(TestPin::MidPrioTask as u32);
}

/// 获取GPIO控制器
pub unsafe fn gpio() -> &'static GpioController {
    GPIO_CONTROLLER.as_ref().expect("GPIO controller not initialized")
}

/// 性能测试GPIO操作
pub struct TestGpio;

impl TestGpio {
    /// 标记任务切换开始 (GPIO45)
    #[inline]
    pub unsafe fn task_switch_start() {
        gpio().set_high(TestPin::TaskSwitchStart as u32);
    }

    /// 标记任务切换结束 (GPIO45)
    #[inline]
    pub unsafe fn task_switch_end() {
        gpio().set_low(TestPin::TaskSwitchStart as u32);
    }

    /// 标记任务切换完成脉冲 (GPIO37)
    #[inline]
    pub unsafe fn task_switch_pulse() {
        let gpio = gpio();
        gpio.set_high(TestPin::TaskSwitchEnd as u32);
        core::arch::asm!("nop");
        gpio.set_low(TestPin::TaskSwitchEnd as u32);
    }

    /// 标记高优先级任务运行 (GPIO39)
    #[inline]
    pub unsafe fn high_prio_task_on() {
        gpio().set_high(TestPin::HighPrioTask as u32);
    }

    /// 标记高优先级任务停止 (GPIO39)
    #[inline]
    pub unsafe fn high_prio_task_off() {
        gpio().set_low(TestPin::HighPrioTask as u32);
    }

    /// 标记中优先级任务运行 (GPIO40)
    #[inline]
    pub unsafe fn mid_prio_task_on() {
        gpio().set_high(TestPin::MidPrioTask as u32);
    }

    /// 标记中优先级任务停止 (GPIO40)
    #[inline]
    pub unsafe fn mid_prio_task_off() {
        gpio().set_low(TestPin::MidPrioTask as u32);
    }
}
