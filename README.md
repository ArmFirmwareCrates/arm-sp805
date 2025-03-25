# Arm Watchdog Module (SP805) driver

Driver implementation for the [SP805 watchdog module](https://developer.arm.com/documentation/ddi0270/latest/).

## Implemented features

* Enable/disable watchdog timer
* Update timer value

## Example

```rust
use arm_sp805::{SP805Registers, Watchdog, UniqueMmioPointer};
use core::ptr::NonNull;
# use zerocopy::transmute_mut;
# let mut fake_registers = [0u32; 1024];
# let WATCHDOG_ADDRESS : *mut SP805Registers = transmute_mut!(&mut fake_registers);
# fn handler() {}

// SAFETY: `WATCHDOG_ADDRESS` is the base address of a SP805 watchdog register block. It remains
// valid for the lifetime of the application and nothing else references this address range.
let watchdog_pointer = unsafe { UniqueMmioPointer::new(NonNull::new(WATCHDOG_ADDRESS).unwrap()) };

let mut watchdog = Watchdog::new(watchdog_pointer, 0x0001_0000);
watchdog.enable();

loop {
  handler();
  watchdog.update();
  # break
}
```

## License

The project is MIT and Apache-2.0 dual licensed, see `LICENSE-APACHE` and `LICENSE-MIT`.

## Maintainers

arm-sp805 is a trustedfirmware.org maintained project. All contributions are ultimately merged by the maintainers
listed below.

* Bálint Dobszay <balint.dobszay@arm.com>
  [balint-dobszay-arm](https://github.com/balint-dobszay-arm)
* Imre Kis <imre.kis@arm.com>
  [imre-kis-arm](https://github.com/imre-kis-arm)
* Sandrine Afsa <sandrine.afsa@arm.com>
  [sandrine-bailleux-arm](https://github.com/sandrine-bailleux-arm)

## Contributing

Please follow the directions of the [Trusted Firmware Processes](https://trusted-firmware-docs.readthedocs.io/en/latest/generic_processes/index.html)

Contributions are handled through [review.trustedfirmware.org](https://review.trustedfirmware.org/q/project:rust-spmc/arm-pl011-uart).

## Reporting Security Issues

Please follow the directions of the [Trusted Firmware Security Center](https://trusted-firmware-docs.readthedocs.io/en/latest/security_center/index.html)

--------------

*Copyright 2025 Arm Limited and/or its affiliates <open-source-office@arm.com>*
