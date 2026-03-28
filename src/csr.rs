//! CSR (Control and Status Register) access module
//!
//! Provides safe(ish) wrappers around inline assembly for reading
//! RISC-V machine-mode registers.

pub mod csr {
    use core::arch::asm;

    /// Read current hardware thread ID
    #[inline]
    pub unsafe fn mhartid() -> usize {
        let mut value: usize;
        asm!("csrr {}, mhartid", out(reg) value);
        value
    }

    /// Read machine trap-vector base address
    #[inline]
    pub unsafe fn mtvec() -> usize {
        let mut value: usize;
        asm!("csrr {}, mtvec", out(reg) value);
        value
    }

    /// Read machine status register
    #[inline]
    pub unsafe fn mstatus() -> usize {
        let mut value: usize;
        asm!("csrr {}, mstatus", out(reg) value);
        value
    }

    /// Read machine exception program counter
    #[inline]
    pub unsafe fn mepc() -> usize {
        let mut value: usize;
        asm!("csrr {}, mepc", out(reg) value);
        value
    }

    /// Read machine interrupt-enable register
    #[inline]
    pub unsafe fn mie() -> usize {
        let mut value: usize;
        asm!("csrr {}, mie", out(reg) value);
        value
    }

    /// Read machine interrupt pending register
    #[inline]
    pub unsafe fn mip() -> usize {
        let mut value: usize;
        asm!("csrr {}, mip", out(reg) value);
        value
    }

    /// Read machine scratch register
    #[inline]
    pub unsafe fn mscratch() -> usize {
        let mut value: usize;
        asm!("csrr {}, mscratch", out(reg) value);
        value
    }

    /// Read current stack pointer
    #[inline]
    pub unsafe fn sp() -> usize {
        let mut value: usize;
        asm!("mv {}, sp", out(reg) value);
        value
    }

    /// Read return address
    #[inline]
    pub unsafe fn ra() -> usize {
        let mut value: usize;
        asm!("mv {}, ra", out(reg) value);
        value
    }
}
