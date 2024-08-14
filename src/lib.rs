// SPDX-FileCopyrightText: Copyright 2023-2024 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Arm Watchdog Module (SP805) driver
//!
//! Driver implementation for the [SP805 watchdog module](https://developer.arm.com/documentation/ddi0270/latest/).

#![no_std]

use core::ops::Deref;

use bitflags::bitflags;
use volatile_register::{RO, RW, WO};

bitflags! {
    /// Control register
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct ControlRegister : u32 {
        /// Enable Watchdog module reset output
        const RESEN = 1 << 1;
        /// Break error
        const INTEN = 1 << 0;
    }

    /// Raw Interrupt Status Register
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct RawInterruptStatusRegister : u32 {
        /// Raw interrupt status from the counter
        const WDOGRIS = 1 << 0;
    }

    /// Masked Interrupt Status Register
      struct MaskedInterruptStatusRegister : u32 {
        /// Enabled interrupt status from the counter
        const WDOGMIS = 1 << 0;
    }
}

/// SP805 register map
#[repr(C, align(4))]
pub struct SP805Registers {
    wdog_load: RW<u32>,       // 0x000 Load Register
    wdog_value: RO<u32>,      // 0x004 Value Register
    wdog_control: RW<u32>,    // 0x008 Control register
    wdog_intclr: WO<u32>,     // 0x00c Interrupt Clear Register
    wdog_ris: RO<u32>,        // 0x010 Raw Interrupt Status Register
    wdog_mis: RO<u32>,        // 0x014 Masked Interrupt Status Register
    reserved_18: [u32; 762],  // 0x018 - 0xbfc
    wdog_lock: RW<u32>,       // 0xc00 Lock Register
    reserved_c04: [u32; 191], // 0xc04 - 0xefc
    wdog_itcr: RW<u32>,       // 0xf00 Integration Test Control Register,
    wdog_itop: WO<u32>,       // 0xf04 Integration Test Output Set
    reserved_f08: [u32; 54],  // 0xf08 - 0xfdc
    wdog_periph_id0: RO<u32>, // 0xfe0 Peripheral Identification Register 0
    wdog_periph_id1: RO<u32>, // 0xfe4 Peripheral Identification Register 1
    wdog_periph_id2: RO<u32>, // 0xfe8 Peripheral Identification Register 2
    wdog_periph_id3: RO<u32>, // 0xfec Peripheral Identification Register 3
    wdog_pcell_id0: RO<u32>,  // 0xff0 PrimeCell Identification Register 0
    wdog_pcell_id1: RO<u32>,  // 0xff4 PrimeCell Identification Register 1
    wdog_pcell_id2: RO<u32>,  // 0xff8 PrimeCell Identification Register 2
    wdog_pcell_id3: RO<u32>,  // 0xffc PrimeCell Identification Register 3
}

struct WatchdogUnlockGuard<'a, R>
where
    R: Deref<Target = SP805Registers>,
{
    regs: &'a R,
}

impl<'a, R> WatchdogUnlockGuard<'a, R>
where
    R: Deref<Target = SP805Registers>,
{
    const LOCK: u32 = 0x00000001;
    const UNLOCK: u32 = 0x1ACCE551;

    pub fn new(regs: &'a R) -> Self {
        unsafe {
            regs.wdog_lock.write(Self::UNLOCK);
        }
        Self { regs }
    }
}

impl<'a, R> Deref for WatchdogUnlockGuard<'a, R>
where
    R: Deref<Target = SP805Registers>,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.regs
    }
}

impl<'a, R> Drop for WatchdogUnlockGuard<'a, R>
where
    R: Deref<Target = SP805Registers>,
{
    fn drop(&mut self) {
        unsafe {
            self.regs.wdog_lock.write(Self::LOCK);
        }
    }
}

/// SP805 Watchdog implementation
pub struct Watchdog<R>
where
    R: Deref<Target = SP805Registers>,
{
    regs: R,
    load_value: u32,
}

impl<R> Watchdog<R>
where
    R: Deref<Target = SP805Registers>,
{
    /// Create new watchdog instance
    pub fn new(regs: R, load_value: u32) -> Self {
        Self { regs, load_value }
    }

    /// Enable watchdog
    pub fn enable(&self) {
        let regs = self.unlock();

        unsafe {
            regs.wdog_load.write(self.load_value);
            regs.wdog_intclr.write(1);
            regs.wdog_control
                .write((ControlRegister::INTEN | ControlRegister::RESEN).bits());
        }
    }

    /// Disable watchdog
    pub fn disable(&self) {
        unsafe {
            self.unlock()
                .wdog_control
                .write(ControlRegister::empty().bits());
        }
    }

    /// Update watchdog
    pub fn update(&self) {
        unsafe {
            self.unlock().wdog_load.write(self.load_value);
        }
    }

    fn unlock(&self) -> WatchdogUnlockGuard<R> {
        WatchdogUnlockGuard::new(&self.regs)
    }
}
