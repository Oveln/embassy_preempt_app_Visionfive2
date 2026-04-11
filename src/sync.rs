//! Multi-hart synchronization primitives
//!
//! Provides shared memory structures for communication between
//! hart0 and hart1 on the JH7110 dual-core RISC-V processor.

use core::sync::atomic::Ordering;
use portable_atomic::AtomicBool;

/// Hart synchronization flags for multi-hart communication
///
/// This structure is placed in shared memory (0xc8000000) to enable
/// communication between hart0 and hart1 on the JH7110 dual-core RISC-V processor.
#[repr(C)]
pub struct HartSyncFlags {
    /// Magic number for validation (0x0721)
    pub magic_number: u16,
    /// Flag indicating if hart0 OS has started
    pub hart0_os_ready: AtomicBool,
    /// Flag indicating if hart1 OS has started
    pub hart1_os_ready: AtomicBool,
    /// Flag indicating if hart0 has sent an IPI to hart1
    pub hart0_ipi_sent: AtomicBool,
}

impl HartSyncFlags {
    /// Magic number for shared memory validation
    const MAGIC: u16 = 0x0721;

    /// Initialize the HartSyncFlags structure
    #[inline]
    pub fn init(&self) {
        unsafe {
            // Use volatile write to ensure magic number is written
            (self as *const Self as *mut u16).write_volatile(Self::MAGIC);
        }
        self.hart0_os_ready.store(true, Ordering::SeqCst);
        self.hart1_os_ready.store(false, Ordering::SeqCst);
        self.hart0_ipi_sent.store(false, Ordering::SeqCst);
    }

    /// Validate the magic number
    #[inline]
    pub fn is_valid(&self) -> bool {
        unsafe {
            (self as *const Self as *const u16).read_volatile() == Self::MAGIC
        }
    }

    /// Check if hart1 is ready (hart0 perspective)
    #[inline]
    pub fn is_hart1_ready(&self) -> bool {
        self.hart1_os_ready.load(Ordering::SeqCst)
    }

    /// Set hart1 ready flag (hart1 calls this)
    #[inline]
    pub fn set_hart1_ready(&self) {
        self.hart1_os_ready.store(true, Ordering::SeqCst);
    }

    /// Check if hart0 is ready (hart1 perspective)
    #[inline]
    pub fn is_hart0_ready(&self) -> bool {
        self.hart0_os_ready.load(Ordering::SeqCst)
    }

    /// Set hart0 ready flag (hart0 calls this)
    #[inline]
    pub fn set_hart0_ready(&self) {
        self.hart0_os_ready.store(true, Ordering::SeqCst);
    }

    /// Set IPI sent flag (hart0 calls this)
    #[inline]
    pub fn set_ipi_sent(&self) {
        self.hart0_ipi_sent.store(true, Ordering::SeqCst);
    }

    /// Clear IPI sent flag (hart1 calls this)
    #[inline]
    pub fn clear_ipi_sent(&self) {
        self.hart0_ipi_sent.store(false, Ordering::SeqCst);
    }

    /// Check if IPI was sent (hart1 calls this)
    #[inline]
    pub fn is_ipi_sent(&self) -> bool {
        self.hart0_ipi_sent.load(Ordering::SeqCst)
    }
}

/// Get reference to hart synchronization flags in shared memory
pub fn get_hart_sync() -> &'static HartSyncFlags {
    let addr: usize = 0xc8000000;
    unsafe {
        &*(addr as *const HartSyncFlags)
    }
}

/// Shared memory base address for inter-system communication
/// 0xc8000000 + 256 (after HartSyncFlags which is smaller than 256 bytes)
pub const INTERCOM_SHM_BASE: usize = 0xc8000100;
