//! BSS section utilities
//!
//! Provides functions for clearing the BSS (Block Started by Symbol) section,
//! which contains zero-initialized data.

/// Clear the BSS section (zero-initialized data)
pub fn clear_bss() {
    extern "C" {
        static __sbss: u8;
        static __ebss: u8;
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            &__sbss as *const u8 as *mut u8,
            &__ebss as *const u8 as usize - &__sbss as *const u8 as usize,
        )
        .fill(0);
    }
}
