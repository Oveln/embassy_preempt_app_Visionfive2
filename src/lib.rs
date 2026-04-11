//! # Embassy Preempt Console Application Library
//!
//! This library provides common modules for the Embassy Preempt console
//! application running on VisionFive2 JH7110 development board.

#![no_std]

pub mod bss;
pub mod cpu_freq;
pub mod csr;
pub mod gpio;
pub mod intercom;
pub mod sync;
pub mod system_info;
