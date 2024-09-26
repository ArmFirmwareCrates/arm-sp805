// SPDX-FileCopyrightText: Copyright 2023-2024 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Arm Watchdog Module (SP805) driver
//!
//! Driver implementation for the [SP805 watchdog module](https://developer.arm.com/documentation/ddi0270/latest/).

#![no_std]

use core::{
    ops::{Deref, DerefMut},
    ptr::addr_of_mut,
};

use bitflags::bitflags;

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
    wdog_load: u32,           // 0x000 Load Register
    wdog_value: u32,          // 0x004 Value Register
    wdog_control: u32,        // 0x008 Control register
    wdog_intclr: u32,         // 0x00c Interrupt Clear Register
    wdog_ris: u32,            // 0x010 Raw Interrupt Status Register
    wdog_mis: u32,            // 0x014 Masked Interrupt Status Register
    reserved_18: [u32; 762],  // 0x018 - 0xbfc
    wdog_lock: u32,           // 0xc00 Lock Register
    reserved_c04: [u32; 191], // 0xc04 - 0xefc
    wdog_itcr: u32,           // 0xf00 Integration Test Control Register,
    wdog_itop: u32,           // 0xf04 Integration Test Output Set
    reserved_f08: [u32; 54],  // 0xf08 - 0xfdc
    wdog_periph_id0: u32,     // 0xfe0 Peripheral Identification Register 0
    wdog_periph_id1: u32,     // 0xfe4 Peripheral Identification Register 1
    wdog_periph_id2: u32,     // 0xfe8 Peripheral Identification Register 2
    wdog_periph_id3: u32,     // 0xfec Peripheral Identification Register 3
    wdog_pcell_id0: u32,      // 0xff0 PrimeCell Identification Register 0
    wdog_pcell_id1: u32,      // 0xff4 PrimeCell Identification Register 1
    wdog_pcell_id2: u32,      // 0xff8 PrimeCell Identification Register 2
    wdog_pcell_id3: u32,      // 0xffc PrimeCell Identification Register 3
}

struct WatchdogUnlockGuard<'a, R>
where
    R: DerefMut<Target = SP805Registers>,
{
    regs: &'a mut R,
}

impl<'a, R> WatchdogUnlockGuard<'a, R>
where
    R: DerefMut<Target = SP805Registers>,
{
    const LOCK: u32 = 0x00000001;
    const UNLOCK: u32 = 0x1ACCE551;

    pub fn new(regs: &'a mut R) -> Self {
        unsafe {
            addr_of_mut!(regs.wdog_lock).write_volatile(Self::UNLOCK);
        }
        Self { regs }
    }
}

impl<'a, R> Deref for WatchdogUnlockGuard<'a, R>
where
    R: DerefMut<Target = SP805Registers>,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.regs
    }
}

impl<'a, R> DerefMut for WatchdogUnlockGuard<'a, R>
where
    R: DerefMut<Target = SP805Registers>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.regs
    }
}

impl<'a, R> Drop for WatchdogUnlockGuard<'a, R>
where
    R: DerefMut<Target = SP805Registers>,
{
    fn drop(&mut self) {
        unsafe {
            addr_of_mut!(self.regs.wdog_lock).write_volatile(Self::LOCK);
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
    R: DerefMut<Target = SP805Registers>,
{
    /// Create new watchdog instance
    pub fn new(regs: R, load_value: u32) -> Self {
        Self { regs, load_value }
    }

    /// Enable watchdog
    pub fn enable(&mut self) {
        let load_value = self.load_value;
        let mut regs = self.unlock();

        unsafe {
            addr_of_mut!(regs.wdog_load).write_volatile(load_value);
            addr_of_mut!(regs.wdog_intclr).write_volatile(1);
            addr_of_mut!(regs.wdog_control)
                .write_volatile((ControlRegister::INTEN | ControlRegister::RESEN).bits());
        }
    }

    /// Disable watchdog
    pub fn disable(&mut self) {
        unsafe {
            addr_of_mut!(self.unlock().wdog_control)
                .write_volatile(ControlRegister::empty().bits());
        }
    }

    /// Update watchdog
    pub fn update(&mut self) {
        let load_value = self.load_value;

        unsafe {
            addr_of_mut!(self.unlock().wdog_load).write_volatile(load_value);
        }
    }

    fn unlock(&mut self) -> WatchdogUnlockGuard<R> {
        WatchdogUnlockGuard::new(&mut self.regs)
    }
}
