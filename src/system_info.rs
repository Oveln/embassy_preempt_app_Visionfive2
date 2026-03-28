//! System information display module
//!
//! Provides functions for displaying RISC-V system state including
//! trap vectors, hart information, machine status, interrupts, and stack.

use crate::csr::csr;
use embassy_preempt_log::task_log;

/// Display trap vector configuration
pub fn print_trap_vector_info() {
    let mtvec = unsafe { csr::mtvec() };
    let mode = mtvec & 0x3;
    let base = mtvec & !0x3;
    let mode_str = match mode {
        0 => "Direct (all traps to BASE)",
        1 => "Vectored (exceptions to BASE, interrupts to BASE+4*cause)",
        _ => "Reserved",
    };
    task_log!(info, "[Trap Vector]");
    task_log!(info, "  mtvec: {:#x}", mtvec);
    task_log!(info, "    Mode: {}", mode_str);
    task_log!(info, "    Base: {:#x}", base);
}

/// Display hardware thread information
pub fn print_hart_info() {
    let hartid = unsafe { csr::mhartid() };
    task_log!(info, "[Hart Information]");
    task_log!(info, "  mhartid (Hart ID): {:#x} ({})", hartid, hartid);
}

/// Display machine mode status
pub fn print_machine_status() {
    let mstatus = unsafe { csr::mstatus() };
    let mpp = match (mstatus >> 11) & 0x3 {
        0 => "User",
        1 => "Supervisor",
        3 => "Machine",
        _ => "Unknown",
    };
    task_log!(info, "[Machine Status]");
    task_log!(info, "  mstatus: {:#x}", mstatus);
    task_log!(
        info,
        "    MIE: {}, MPIE: {}, MPP: {}",
        if (mstatus >> 3) & 1 == 1 { "1" } else { "0" },
        if (mstatus >> 7) & 1 == 1 { "1" } else { "0" },
        mpp
    );
}

/// Display interrupt information (enable and pending)
pub fn print_interrupt_info() {
    // Interrupt enable
    let mie = unsafe { csr::mie() };
    task_log!(info, "[Interrupt Enable]");
    task_log!(info, "  mie: {:#x}", mie);
    task_log!(info, "    MIE bits:");
    task_log!(
        info,
        "      SSIP: {}, MSIP: {}, STIP: {}, MTIP: {}, SEIP: {}, MEIP: {}",
        if (mie >> 1) & 1 == 1 { "1" } else { "0" },
        if (mie >> 3) & 1 == 1 { "1" } else { "0" },
        if (mie >> 5) & 1 == 1 { "1" } else { "0" },
        if (mie >> 7) & 1 == 1 { "1" } else { "0" },
        if (mie >> 9) & 1 == 1 { "1" } else { "0" },
        if (mie >> 11) & 1 == 1 { "1" } else { "0" }
    );

    // Interrupt pending
    let mip = unsafe { csr::mip() };
    task_log!(info, "[Interrupt Pending]");
    task_log!(info, "  mip: {:#x}", mip);
    task_log!(info, "    MIP bits:");
    task_log!(
        info,
        "      SSIP: {}, MSIP: {}, STIP: {}, MTIP: {}, SEIP: {}, MEIP: {}",
        if (mip >> 1) & 1 == 1 { "1" } else { "0" },
        if (mip >> 3) & 1 == 1 { "1" } else { "0" },
        if (mip >> 5) & 1 == 1 { "1" } else { "0" },
        if (mip >> 7) & 1 == 1 { "1" } else { "0" },
        if (mip >> 9) & 1 == 1 { "1" } else { "0" },
        if (mip >> 11) & 1 == 1 { "1" } else { "0" }
    );
}

/// Display stack information
pub fn print_stack_info() {
    let mscratch = unsafe { csr::mscratch() };
    let sp = unsafe { csr::sp() };
    task_log!(info, "[Stack Information]");
    task_log!(info, "  mscratch: {:#x}", mscratch);
    task_log!(info, "  sp (stack pointer): {:#x}", sp);
}

/// Display exception program counter
pub fn print_epc_info() {
    let mepc = unsafe { csr::mepc() };
    task_log!(info, "[Exception Program Counter]");
    task_log!(info, "  mepc: {:#x}", mepc);
}

/// Display code location (return address)
pub fn print_code_location() {
    let ra = unsafe { csr::ra() };
    task_log!(info, "[Code Location]");
    task_log!(info, "  ra (return address): {:#x}", ra);
}

/// Display comprehensive system information
pub fn print_system_info() {
    task_log!(info, "========================================");
    task_log!(info, "  Embassy Preempt - System Info");
    task_log!(info, "  VisionFive2 JH7110 Platform");
    task_log!(info, "========================================");

    print_hart_info();
    print_trap_vector_info();
    print_machine_status();
    print_epc_info();
    print_interrupt_info();
    print_stack_info();
    print_code_location();

    task_log!(info, "========================================");
    task_log!(info, "  UART Logger Initialized");
    task_log!(info, "========================================");
}
