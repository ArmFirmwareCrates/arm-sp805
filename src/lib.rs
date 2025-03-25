// SPDX-FileCopyrightText: Copyright 2023-2025 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Arm Watchdog Module (SP805) driver
//!
//! Driver implementation for the [SP805 watchdog module](https://developer.arm.com/documentation/ddi0270/latest/).

#![no_std]

use bitflags::bitflags;
use safe_mmio::{
    field,
    fields::{ReadPure, ReadPureWrite, WriteOnly},
    UniqueMmioPointer,
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Control Register
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
struct ControlRegister(u32);

/// Interrupt register.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
struct Interrupts(u32);

bitflags! {
    /// Control register
    impl ControlRegister : u32 {
        /// Enable Watchdog module reset output
        const RESEN = 1 << 1;
        /// Break error
        const INTEN = 1 << 0;
    }

    /// Raw Interrupt Status Register
    impl Interrupts : u32 {
        /// Raw interrupt status from the counter
        const WDOGRIS = 1 << 0;
    }
}

/// SP805 register map.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C, align(4))]
pub struct SP805Registers {
    /// 0x000 Load Register
    wdog_load: ReadPureWrite<u32>,
    /// 0x004 Value Register
    wdog_value: ReadPure<u32>,
    /// 0x008 Control register
    wdog_control: ReadPureWrite<ControlRegister>,
    /// 0x00c Interrupt Clear Register
    wdog_intclr: WriteOnly<u32>,
    /// 0x010 Raw Interrupt Status Register
    wdog_ris: ReadPure<Interrupts>,
    /// 0x014 Masked Interrupt Status Register
    wdog_mis: ReadPure<Interrupts>,
    /// 0x018 - 0xbfc
    reserved_18: [u32; 762],
    /// 0xc00 Lock Register
    wdog_lock: ReadPureWrite<u32>,
    /// 0xc04 - 0xefc
    reserved_c04: [u32; 191],
    /// 0xf00 Integration Test Control Register,
    wdog_itcr: ReadPureWrite<u32>,
    /// 0xf04 Integration Test Output Set
    wdog_itop: WriteOnly<u32>,
    /// 0xf08 - 0xfdc
    reserved_f08: [u32; 54],
    /// 0xfe0 Peripheral Identification Register 0
    wdog_periph_id0: ReadPure<u32>,
    /// 0xfe4 Peripheral Identification Register 1
    wdog_periph_id1: ReadPure<u32>,
    /// 0xfe8 Peripheral Identification Register 2
    wdog_periph_id2: ReadPure<u32>,
    /// 0xfec Peripheral Identification Register 3
    wdog_periph_id3: ReadPure<u32>,
    /// 0xff0 PrimeCell Identification Register 0
    wdog_pcell_id0: ReadPure<u32>,
    /// 0xff4 PrimeCell Identification Register 1
    wdog_pcell_id1: ReadPure<u32>,
    /// 0xff8 PrimeCell Identification Register 2
    wdog_pcell_id2: ReadPure<u32>,
    /// 0xffc PrimeCell Identification Register 3
    wdog_pcell_id3: ReadPure<u32>,
}

/// SP805 Watchdog driver implementation.
pub struct Watchdog<'a> {
    regs: UniqueMmioPointer<'a, SP805Registers>,
    load_value: u32,
}

impl<'a> Watchdog<'a> {
    const LOCK: u32 = 0x00000001;
    const UNLOCK: u32 = 0x1ACCE551;

    /// Create new watchdog instance
    pub fn new(regs: UniqueMmioPointer<'a, SP805Registers>, load_value: u32) -> Self {
        Self { regs, load_value }
    }

    /// Enable watchdog
    pub fn enable(&mut self) {
        let load_value = self.load_value;

        self.with_unlock(|mut regs| {
            field!(regs, wdog_load).write(load_value);
            field!(regs, wdog_intclr).write(1);
            field!(regs, wdog_control)
                .write(ControlRegister::INTEN | ControlRegister::RESEN);
        });
    }

    /// Disable watchdog
    pub fn disable(&mut self) {
        self.with_unlock(|mut regs| {
            field!(regs, wdog_control).write(ControlRegister::empty())
        });
    }

    /// Update watchdog
    pub fn update(&mut self) {
        let load_value = self.load_value;

        self.with_unlock(|mut regs| field!(regs, wdog_load).write(load_value));
    }

    fn with_unlock<F>(&mut self, f: F)
    where
        F: FnOnce(&mut UniqueMmioPointer<SP805Registers>),
    {
        field!(self.regs, wdog_lock).write(Self::UNLOCK);
        f(&mut self.regs);
        field!(self.regs, wdog_lock).write(Self::LOCK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::transmute_mut;

    const LOAD_VALUE : u32 = 0xabcd_ef01;

    #[repr(align(4096))]
    pub struct FakeSp805Registers {
        regs: [u32; 1024],
    }

    impl FakeSp805Registers {
        pub fn new() -> Self {
            Self { regs: [0u32; 1024] }
        }

        pub fn clear(&mut self) {
            self.regs.fill(0);
        }

        pub fn reg_write(&mut self, offset: usize, value: u32) {
            self.regs[offset / 4] = value;
        }

        pub fn reg_read(&self, offset: usize) -> u32 {
            self.regs[offset / 4]
        }

        fn get(&mut self) -> UniqueMmioPointer<SP805Registers> {
            UniqueMmioPointer::from(transmute_mut!(&mut self.regs))
        }

        pub fn system_for_test(&mut self) -> Watchdog {
            Watchdog::new(self.get(), LOAD_VALUE)
        }
    }

    #[test]
    fn register_block_size() {
        assert_eq!(0x1000, core::mem::size_of::<SP805Registers>());
    }

    #[test]
    fn enable() {
          let mut regs = FakeSp805Registers::new();

        {
            // Enable
            regs.reg_write(0x0c, 0xffff_ffff);
            let mut wdt = regs.system_for_test();
            wdt.enable();
        }

        assert_eq!(LOAD_VALUE, regs.reg_read(0x00));
        assert_eq!(0x0000_0003, regs.reg_read(0x08));
        assert!(regs.reg_read(0x0c) != 0);
        assert_eq!(Watchdog::LOCK, regs.reg_read(0xc00));

        regs.clear();

        {
            // Disable
            regs.reg_write(0x08, 0x0000_0003);
            let mut wdt = regs.system_for_test();
            wdt.disable();
        }

        assert_eq!(0x0000_0000, regs.reg_read(0x08));
        assert_eq!(Watchdog::LOCK, regs.reg_read(0xc00));

        regs.clear();

        {
            // Update
            let mut wdt = regs.system_for_test();
            wdt.update();
        }

        assert_eq!(LOAD_VALUE, regs.reg_read(0x00));
    }
}