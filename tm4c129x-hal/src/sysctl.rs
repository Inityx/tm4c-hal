//! # System Control
//!
//! The SYSCTL peripheral controls clocks and power.
//!
//! The TM4C129x can be clocked from the Main Oscillator or the PLL, through a
//! divider. The Main Oscillator can be either the internal 16 MHz precision
//! oscillator, a 4 MHz derivation of the same, or an external crystal.
//!
//! SYSCTL includes the following registers:
//!
//! * Device ID (class, major, minor, family, package, temperature range, )
//! * Brown-out reset control
//! * Brown-out/PLL interrupt control
//! * Reset cause
//! * Run-time clock configuration
//! * GPIO high-performance bus control
//! * System Properties
//! * Registers to indicate whether peripherals are present
//! * Registers to reset peripherals
//! * Registers to enable/disable clocking of peripherals
//!
//! See the LM4F120 datasheet, page 228 for a full list.

pub use tm4c_hal::sysctl::*;

use crate::{
    bb,
    time::{Hertz, U32Ext},
};
use cortex_m::asm::nop;

/// Constrained SYSCTL peripheral.
pub struct Sysctl {
    /// Power control methods will require `&mut this.power_control` to
    /// prevent them from running concurrently.
    pub power_control: PowerControl,
    /// Clock configuration will consume this and give you `Clocks`.
    pub clock_setup: ClockSetup,
}

/// Used to gate access to the run-time power control features of the chip.
pub struct PowerControl {
    _0: (),
}

/// Used to configure the clock generators.
pub struct ClockSetup {
    /// The system oscillator configuration
    pub oscillator: Oscillator,
    // Make this type uncreatable
    _0: (),
}

/// Selects the system oscillator source
#[derive(Clone, Copy)]
pub enum Oscillator {
    /// Use the main oscillator (with the given crystal), into the PLL or a
    /// clock divider
    Main(CrystalFrequency, SystemClock),
    /// Use the 16 MHz precision internal oscillator, into the PLL or a clock
    /// divider
    PrecisionInternal(SystemClock),
    /// Use the 33 kHz internal oscillator, divided by the given value.
    LowFrequencyInternal(Divider),
}

/// Selects the source for the system clock
#[derive(Clone, Copy)]
pub enum SystemClock {
    /// Clock the system direct from the system oscillator
    UseOscillator(Divider),
    /// Clock the system from the PLL (which is driven by the system
    /// oscillator), divided down from 400MHz to the given frequency.
    UsePll(PllOutputFrequency),
}

/// Selects which crystal is fitted to the XOSC pins.
#[derive(Clone, Copy)]
pub enum CrystalFrequency {
    /// 4 MHz
    _4mhz,
    /// 4.096 MHz
    _4_09mhz,
    /// 4.9152 MHz
    _4_91mhz,
    /// 5 MHz
    _5mhz,
    /// 5.12 MHz
    _5_12mhz,
    /// 6 MHz
    _6mhz,
    /// 6.144 MHz
    _6_14mhz,
    /// 7.3728 MHz
    _7_37mhz,
    /// 8 MHz
    _8mhz,
    /// 8.192 MHz
    _8_19mhz,
    /// 10 MHz
    _10mhz,
    /// 12 MHz
    _12mhz,
    /// 12.288 MHz
    _12_2mhz,
    /// 13.56 MHz
    _13_5mhz,
    /// 14.31818 MHz
    _14_3mhz,
    /// 16 MHz
    _16mhz,
    /// 16.384 MHz
    _16_3mhz,
    /// 18.0 MHz (USB)
    _18mhz,
    /// 20.0 MHz (USB)
    _20mhz,
    /// 24.0 MHz (USB)
    _24mhz,
    /// 25.0 MHz (USB)
    _25mhz,
}

impl From<CrystalFrequency> for Hertz {
    fn from(freq: CrystalFrequency) -> Self {
        use CrystalFrequency::*;
        Self(match freq {
               _4mhz =>  4_000_000,
            _4_09mhz =>  4_090_000,
            _4_91mhz =>  4_910_000,
               _5mhz =>  5_000_000,
            _5_12mhz =>  5_120_000,
               _6mhz =>  6_000_000,
            _6_14mhz =>  6_140_000,
            _7_37mhz =>  7_370_000,
               _8mhz =>  8_000_000,
            _8_19mhz =>  8_190_000,
              _10mhz => 10_000_000,
              _12mhz => 12_000_000,
            _12_2mhz => 12_200_000,
            _13_5mhz => 13_500_000,
            _14_3mhz => 14_300_000,
              _16mhz => 16_000_000,
            _16_3mhz => 16_300_000,
              _18mhz => 18_000_000,
              _20mhz => 20_000_000,
              _24mhz => 24_000_000,
              _25mhz => 25_000_000,
        })
    }
}

/// Selects what to divide the PLL's 400MHz down to.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum PllOutputFrequency {
    _120mhz,
     _60mhz,
     _48mhz,
     _30mhz,
     _24mhz,
     _12mhz,
      _6mhz,
}

impl Into<Hertz> for PllOutputFrequency {
    fn into(self) -> Hertz {
        use PllOutputFrequency::*;
        Hertz(match self {
            _120mhz => 120_000_000,
             _60mhz =>  60_000_000,
             _48mhz =>  48_000_000,
             _30mhz =>  30_000_000,
             _24mhz =>  24_000_000,
             _12mhz =>  12_000_000,
              _6mhz =>   6_000_000,
        })
    }
}

/// Selects how much to divide the system oscillator down.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum Divider {
    _1 = 1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
    _16,
}

/// List of peripherals that can be enabled or disabled
#[derive(Copy, Clone)]
pub enum Domain {
    /// Watchdog 1
    Watchdog1,
    /// Watchdog 0
    Watchdog0,
    /// 32/16-bit Timer 5
    Timer5,
    /// 32/16-bit Timer 4
    Timer4,
    /// 32/16-bit Timer 3
    Timer3,
    /// 32/16-bit Timer 2
    Timer2,
    /// 32/16-bit Timer 1
    Timer1,
    /// 32/16-bit Timer 0
    Timer0,
    /// Gpio Q
    GpioQ,
    /// Gpio P
    GpioP,
    /// Gpio N
    GpioN,
    /// Gpio M
    GpioM,
    /// Gpio L
    GpioL,
    /// Gpio K
    GpioK,
    /// Gpio J
    GpioJ,
    /// Gpio H
    GpioH,
    /// Gpio G
    GpioG,
    /// Gpio F
    GpioF,
    /// GPIO E
    GpioE,
    /// GPIO D
    GpioD,
    /// GPIO C
    GpioC,
    /// GPIO B
    GpioB,
    /// GPIO A
    GpioA,
    /// µDMA
    MicroDma,
    /// Hibernation
    Hibernation,
    /// UART 7
    Uart7,
    /// UART 6
    Uart6,
    /// UART 5
    Uart5,
    /// UART 4
    Uart4,
    /// UART 3
    Uart3,
    /// UART 2
    Uart2,
    /// UART 1
    Uart1,
    /// UART 0
    Uart0,
    /// SSI 3
    Ssi3,
    /// SSI 2
    Ssi2,
    /// SSI 1
    Ssi1,
    /// SSI 0
    Ssi0,
    /// I2C 3
    I2c3,
    /// I2C 2
    I2c2,
    /// I2C 1
    I2c1,
    /// I2C 0
    I2c0,
    /// USB
    Usb,
    /// CAN
    Can,
    /// ADC 1
    Adc1,
    /// ADC 0
    Adc0,
    /// Analog Comparator
    AnalogComparator,
    /// EEPROM
    Eeprom,
    /// PWM0
    Pwm0,
    /// PWM1
    Pwm1,
    /// EMAC0
    Emac0,
    /// EPHY0
    Ephy0,
}

/// Reset a peripheral
pub fn reset(_lock: &PowerControl, pd: Domain) {
    // We use bit-banding to make an atomic write, so this is safe
    let p = unsafe { &*tm4c129x::SYSCTL::ptr() };
    use Domain::*;
    unsafe { match pd {
        Watchdog1 => {
            bb::toggle_bit(&p.srwd, 1);
            bb::spin_bit(&p.prwd, 1);
        }
        Watchdog0 => {
            bb::toggle_bit(&p.srwd, 0);
            bb::spin_bit(&p.prwd, 0);
        }
        Timer5 => {
            bb::toggle_bit(&p.srtimer, 5);
            bb::spin_bit(&p.prtimer, 5);
        }
        Timer4 => {
            bb::toggle_bit(&p.srtimer, 4);
            bb::spin_bit(&p.prtimer, 4);
        }
        Timer3 => {
            bb::toggle_bit(&p.srtimer, 3);
            bb::spin_bit(&p.prtimer, 3);
        }
        Timer2 => {
            bb::toggle_bit(&p.srtimer, 2);
            bb::spin_bit(&p.prtimer, 2);
        }
        Timer1 => {
            bb::toggle_bit(&p.srtimer, 1);
            bb::spin_bit(&p.prtimer, 1);
        }
        Timer0 => {
            bb::toggle_bit(&p.srtimer, 0);
            bb::spin_bit(&p.prtimer, 0);
        }
        GpioQ => {
            bb::toggle_bit(&p.srgpio, 14);
            bb::spin_bit(&p.prgpio, 14);
        }
        GpioP => {
            bb::toggle_bit(&p.srgpio, 13);
            bb::spin_bit(&p.prgpio, 13);
        }
        GpioN => {
            bb::toggle_bit(&p.srgpio, 12);
            bb::spin_bit(&p.prgpio, 12);
        }
        GpioM => {
            bb::toggle_bit(&p.srgpio, 11);
            bb::spin_bit(&p.prgpio, 11);
        }
        GpioL => {
            bb::toggle_bit(&p.srgpio, 10);
            bb::spin_bit(&p.prgpio, 10);
        }
        GpioK => {
            bb::toggle_bit(&p.srgpio, 9);
            bb::spin_bit(&p.prgpio, 9);
        }
        GpioJ => {
            bb::toggle_bit(&p.srgpio, 8);
            bb::spin_bit(&p.prgpio, 8);
        }
        GpioH => {
            bb::toggle_bit(&p.srgpio, 7);
            bb::spin_bit(&p.prgpio, 7);
        }
        GpioG => {
            bb::toggle_bit(&p.srgpio, 6);
            bb::spin_bit(&p.prgpio, 6);
        }
        GpioF => {
            bb::toggle_bit(&p.srgpio, 5);
            bb::spin_bit(&p.prgpio, 5);
        }
        GpioE => {
            bb::toggle_bit(&p.srgpio, 4);
            bb::spin_bit(&p.prgpio, 4);
        }
        GpioD => {
            bb::toggle_bit(&p.srgpio, 3);
            bb::spin_bit(&p.prgpio, 3);
        }
        GpioC => {
            bb::toggle_bit(&p.srgpio, 2);
            bb::spin_bit(&p.prgpio, 2);
        }
        GpioB => {
            bb::toggle_bit(&p.srgpio, 1);
            bb::spin_bit(&p.prgpio, 1);
        }
        GpioA => {
            bb::toggle_bit(&p.srgpio, 0);
            bb::spin_bit(&p.prgpio, 0);
        }
        MicroDma => {
            bb::toggle_bit(&p.srdma, 0);
            bb::spin_bit(&p.prdma, 0);
        }
        Hibernation => {
            bb::toggle_bit(&p.srhib, 0);
            bb::spin_bit(&p.prhib, 0);
        }
        Uart7 => {
            bb::toggle_bit(&p.sruart, 7);
            bb::spin_bit(&p.pruart, 7);
        }
        Uart6 => {
            bb::toggle_bit(&p.sruart, 6);
            bb::spin_bit(&p.pruart, 6);
        }
        Uart5 => {
            bb::toggle_bit(&p.sruart, 5);
            bb::spin_bit(&p.pruart, 5);
        }
        Uart4 => {
            bb::toggle_bit(&p.sruart, 4);
            bb::spin_bit(&p.pruart, 4);
        }
        Uart3 => {
            bb::toggle_bit(&p.sruart, 3);
            bb::spin_bit(&p.pruart, 3);
        }
        Uart2 => {
            bb::toggle_bit(&p.sruart, 2);
            bb::spin_bit(&p.pruart, 2);
        }
        Uart1 => {
            bb::toggle_bit(&p.sruart, 1);
            bb::spin_bit(&p.pruart, 1);
        }
        Uart0 => {
            bb::toggle_bit(&p.sruart, 0);
            bb::spin_bit(&p.pruart, 0);
        }
        Ssi3 => {
            bb::toggle_bit(&p.srssi, 3);
            bb::spin_bit(&p.prssi, 3);
        }
        Ssi2 => {
            bb::toggle_bit(&p.srssi, 2);
            bb::spin_bit(&p.prssi, 2);
        }
        Ssi1 => {
            bb::toggle_bit(&p.srssi, 1);
            bb::spin_bit(&p.prssi, 1);
        }
        Ssi0 => {
            bb::toggle_bit(&p.srssi, 0);
            bb::spin_bit(&p.prssi, 0);
        }
        I2c3 => {
            bb::toggle_bit(&p.sri2c, 3);
            bb::spin_bit(&p.pri2c, 3);
        }
        I2c2 => {
            bb::toggle_bit(&p.sri2c, 2);
            bb::spin_bit(&p.pri2c, 2);
        }
        I2c1 => {
            bb::toggle_bit(&p.sri2c, 1);
            bb::spin_bit(&p.pri2c, 1);
        }
        I2c0 => {
            bb::toggle_bit(&p.sri2c, 0);
            bb::spin_bit(&p.pri2c, 0);
        }
        Usb => {
            bb::toggle_bit(&p.srusb, 0);
            bb::spin_bit(&p.prusb, 0);
        }
        Can => {
            bb::toggle_bit(&p.srcan, 0);
            bb::spin_bit(&p.prcan, 0);
        }
        Adc1 => {
            bb::toggle_bit(&p.sradc, 1);
            bb::spin_bit(&p.pradc, 1);
        }
        Adc0 => {
            bb::toggle_bit(&p.sradc, 0);
            bb::spin_bit(&p.pradc, 0);
        }
        AnalogComparator => {
            bb::toggle_bit(&p.sracmp, 0);
            bb::spin_bit(&p.pracmp, 0);
        }
        Eeprom => {
            bb::toggle_bit(&p.sreeprom, 0);
            bb::spin_bit(&p.preeprom, 0);
        }
        Pwm0 => {
            bb::toggle_bit(&p.srpwm, 0);
            bb::spin_bit(&p.prpwm, 0);
        }
        Pwm1 => {
            bb::toggle_bit(&p.srpwm, 1);
            bb::spin_bit(&p.prpwm, 1);
        }
        Emac0 => {
            bb::toggle_bit(&p.sremac, 0);
            bb::spin_bit(&p.premac, 0);
        }
        Ephy0 => {
            bb::toggle_bit(&p.srephy, 0);
            bb::spin_bit(&p.prephy, 0);
        }
    }}
}

/// Activate or De-Activate clocks and power to the given peripheral in the
/// given run mode.
///
/// We take a reference to PowerControl as a permission check. We don't need
/// an &mut reference as we use atomic writes in the bit-banding area so it's
/// interrupt safe.
pub fn control_power(_lock: &PowerControl, pd: Domain, run_mode: RunMode, state: PowerState) {
    let on = match state {
        PowerState::On => true,
        PowerState::Off => false,
    };
    match run_mode {
        RunMode::Run => control_run_power(pd, on),
        RunMode::Sleep => control_sleep_power(pd, on),
        RunMode::DeepSleep => control_deep_sleep_power(pd, on),
    }
    // Section 5.2.6 - "There must be a delay of 3 system clocks after a
    // peripheral module clock is enabled in the RCGC register before any
    // module registers are accessed."
    nop();
    nop();
    nop();
}

fn control_run_power(pd: Domain, on: bool) {
    // We use bit-banding to make an atomic write, so this is safe
    let p = unsafe { &*tm4c129x::SYSCTL::ptr() };
    use Domain::*;
    unsafe { match pd {
        Watchdog1 => bb::change_bit(&p.rcgcwd, 1, on),
        Watchdog0 => bb::change_bit(&p.rcgcwd, 0, on),
        Timer5 => bb::change_bit(&p.rcgctimer, 5, on),
        Timer4 => bb::change_bit(&p.rcgctimer, 4, on),
        Timer3 => bb::change_bit(&p.rcgctimer, 3, on),
        Timer2 => bb::change_bit(&p.rcgctimer, 2, on),
        Timer1 => bb::change_bit(&p.rcgctimer, 1, on),
        Timer0 => bb::change_bit(&p.rcgctimer, 0, on),
        GpioQ => bb::change_bit(&p.rcgcgpio, 14, on),
        GpioP => bb::change_bit(&p.rcgcgpio, 13, on),
        GpioN => bb::change_bit(&p.rcgcgpio, 12, on),
        GpioM => bb::change_bit(&p.rcgcgpio, 11, on),
        GpioL => bb::change_bit(&p.rcgcgpio, 10, on),
        GpioK => bb::change_bit(&p.rcgcgpio, 9, on),
        GpioJ => bb::change_bit(&p.rcgcgpio, 8, on),
        GpioH => bb::change_bit(&p.rcgcgpio, 7, on),
        GpioG => bb::change_bit(&p.rcgcgpio, 6, on),
        GpioF => bb::change_bit(&p.rcgcgpio, 5, on),
        GpioE => bb::change_bit(&p.rcgcgpio, 4, on),
        GpioD => bb::change_bit(&p.rcgcgpio, 3, on),
        GpioC => bb::change_bit(&p.rcgcgpio, 2, on),
        GpioB => bb::change_bit(&p.rcgcgpio, 1, on),
        GpioA => bb::change_bit(&p.rcgcgpio, 0, on),
        MicroDma => bb::change_bit(&p.rcgcdma, 0, on),
        Hibernation => bb::change_bit(&p.rcgchib, 0, on),
        Uart7 => bb::change_bit(&p.rcgcuart, 7, on),
        Uart6 => bb::change_bit(&p.rcgcuart, 6, on),
        Uart5 => bb::change_bit(&p.rcgcuart, 5, on),
        Uart4 => bb::change_bit(&p.rcgcuart, 4, on),
        Uart3 => bb::change_bit(&p.rcgcuart, 3, on),
        Uart2 => bb::change_bit(&p.rcgcuart, 2, on),
        Uart1 => bb::change_bit(&p.rcgcuart, 1, on),
        Uart0 => bb::change_bit(&p.rcgcuart, 0, on),
        Ssi3 => bb::change_bit(&p.rcgcssi, 3, on),
        Ssi2 => bb::change_bit(&p.rcgcssi, 2, on),
        Ssi1 => bb::change_bit(&p.rcgcssi, 1, on),
        Ssi0 => bb::change_bit(&p.rcgcssi, 0, on),
        I2c3 => bb::change_bit(&p.rcgci2c, 3, on),
        I2c2 => bb::change_bit(&p.rcgci2c, 2, on),
        I2c1 => bb::change_bit(&p.rcgci2c, 1, on),
        I2c0 => bb::change_bit(&p.rcgci2c, 0, on),
        Usb => bb::change_bit(&p.rcgcusb, 0, on),
        Can => bb::change_bit(&p.rcgccan, 0, on),
        Adc1 => bb::change_bit(&p.rcgcadc, 1, on),
        Adc0 => bb::change_bit(&p.rcgcadc, 0, on),
        AnalogComparator => bb::change_bit(&p.rcgcacmp, 0, on),
        Eeprom => bb::change_bit(&p.rcgceeprom, 0, on),
        Pwm0 => bb::change_bit(&p.rcgcpwm, 0, on),
        Pwm1 => bb::change_bit(&p.rcgcpwm, 1, on),
        Emac0 => bb::change_bit(&p.rcgcemac, 0, on),
        Ephy0 => bb::change_bit(&p.rcgcephy, 0, on),
    }}
}

fn control_sleep_power(pd: Domain, on: bool) {
    // We use bit-banding to make an atomic write, so this is safe
    let p = unsafe { &*tm4c129x::SYSCTL::ptr() };
    use Domain::*;
    unsafe { match pd {
        Watchdog1 => bb::change_bit(&p.scgcwd, 1, on),
        Watchdog0 => bb::change_bit(&p.scgcwd, 0, on),
        Timer5 => bb::change_bit(&p.scgctimer, 5, on),
        Timer4 => bb::change_bit(&p.scgctimer, 4, on),
        Timer3 => bb::change_bit(&p.scgctimer, 3, on),
        Timer2 => bb::change_bit(&p.scgctimer, 2, on),
        Timer1 => bb::change_bit(&p.scgctimer, 1, on),
        Timer0 => bb::change_bit(&p.scgctimer, 0, on),
        GpioQ => bb::change_bit(&p.scgcgpio, 14, on),
        GpioP => bb::change_bit(&p.scgcgpio, 13, on),
        GpioN => bb::change_bit(&p.scgcgpio, 12, on),
        GpioM => bb::change_bit(&p.scgcgpio, 11, on),
        GpioL => bb::change_bit(&p.scgcgpio, 10, on),
        GpioK => bb::change_bit(&p.scgcgpio, 9, on),
        GpioJ => bb::change_bit(&p.scgcgpio, 8, on),
        GpioH => bb::change_bit(&p.scgcgpio, 7, on),
        GpioG => bb::change_bit(&p.scgcgpio, 6, on),
        GpioF => bb::change_bit(&p.scgcgpio, 5, on),
        GpioE => bb::change_bit(&p.scgcgpio, 4, on),
        GpioD => bb::change_bit(&p.scgcgpio, 3, on),
        GpioC => bb::change_bit(&p.scgcgpio, 2, on),
        GpioB => bb::change_bit(&p.scgcgpio, 1, on),
        GpioA => bb::change_bit(&p.scgcgpio, 0, on),
        MicroDma => bb::change_bit(&p.scgcdma, 0, on),
        Hibernation => bb::change_bit(&p.scgchib, 0, on),
        Uart7 => bb::change_bit(&p.scgcuart, 7, on),
        Uart6 => bb::change_bit(&p.scgcuart, 6, on),
        Uart5 => bb::change_bit(&p.scgcuart, 5, on),
        Uart4 => bb::change_bit(&p.scgcuart, 4, on),
        Uart3 => bb::change_bit(&p.scgcuart, 3, on),
        Uart2 => bb::change_bit(&p.scgcuart, 2, on),
        Uart1 => bb::change_bit(&p.scgcuart, 1, on),
        Uart0 => bb::change_bit(&p.scgcuart, 0, on),
        Ssi3 => bb::change_bit(&p.scgcssi, 3, on),
        Ssi2 => bb::change_bit(&p.scgcssi, 2, on),
        Ssi1 => bb::change_bit(&p.scgcssi, 1, on),
        Ssi0 => bb::change_bit(&p.scgcssi, 0, on),
        I2c3 => bb::change_bit(&p.scgci2c, 3, on),
        I2c2 => bb::change_bit(&p.scgci2c, 2, on),
        I2c1 => bb::change_bit(&p.scgci2c, 1, on),
        I2c0 => bb::change_bit(&p.scgci2c, 0, on),
        Usb => bb::change_bit(&p.scgcusb, 0, on),
        Can => bb::change_bit(&p.scgccan, 0, on),
        Adc1 => bb::change_bit(&p.scgcadc, 1, on),
        Adc0 => bb::change_bit(&p.scgcadc, 0, on),
        AnalogComparator => bb::change_bit(&p.scgcacmp, 0, on),
        Eeprom => bb::change_bit(&p.scgceeprom, 0, on),
        Pwm0 => bb::change_bit(&p.scgcpwm, 0, on),
        Pwm1 => bb::change_bit(&p.scgcpwm, 1, on),
        Emac0 => bb::change_bit(&p.scgcemac, 0, on),
        Ephy0 => bb::change_bit(&p.scgcephy, 0, on),
    }}
}

fn control_deep_sleep_power(pd: Domain, on: bool) {
    // We use bit-banding to make an atomic write, so this is safe
    let p = unsafe { &*tm4c129x::SYSCTL::ptr() };
    use Domain::*;
    unsafe { match pd {
        Watchdog1 => bb::change_bit(&p.dcgcwd, 1, on),
        Watchdog0 => bb::change_bit(&p.dcgcwd, 0, on),
        Timer5 => bb::change_bit(&p.dcgctimer, 5, on),
        Timer4 => bb::change_bit(&p.dcgctimer, 4, on),
        Timer3 => bb::change_bit(&p.dcgctimer, 3, on),
        Timer2 => bb::change_bit(&p.dcgctimer, 2, on),
        Timer1 => bb::change_bit(&p.dcgctimer, 1, on),
        Timer0 => bb::change_bit(&p.dcgctimer, 0, on),
        GpioQ => bb::change_bit(&p.dcgcgpio, 14, on),
        GpioP => bb::change_bit(&p.dcgcgpio, 13, on),
        GpioN => bb::change_bit(&p.dcgcgpio, 12, on),
        GpioM => bb::change_bit(&p.dcgcgpio, 11, on),
        GpioL => bb::change_bit(&p.dcgcgpio, 10, on),
        GpioK => bb::change_bit(&p.dcgcgpio, 9, on),
        GpioJ => bb::change_bit(&p.dcgcgpio, 8, on),
        GpioH => bb::change_bit(&p.dcgcgpio, 7, on),
        GpioG => bb::change_bit(&p.dcgcgpio, 6, on),
        GpioF => bb::change_bit(&p.dcgcgpio, 5, on),
        GpioE => bb::change_bit(&p.dcgcgpio, 4, on),
        GpioD => bb::change_bit(&p.dcgcgpio, 3, on),
        GpioC => bb::change_bit(&p.dcgcgpio, 2, on),
        GpioB => bb::change_bit(&p.dcgcgpio, 1, on),
        GpioA => bb::change_bit(&p.dcgcgpio, 0, on),
        MicroDma => bb::change_bit(&p.dcgcdma, 0, on),
        Hibernation => bb::change_bit(&p.dcgchib, 0, on),
        Uart7 => bb::change_bit(&p.dcgcuart, 7, on),
        Uart6 => bb::change_bit(&p.dcgcuart, 6, on),
        Uart5 => bb::change_bit(&p.dcgcuart, 5, on),
        Uart4 => bb::change_bit(&p.dcgcuart, 4, on),
        Uart3 => bb::change_bit(&p.dcgcuart, 3, on),
        Uart2 => bb::change_bit(&p.dcgcuart, 2, on),
        Uart1 => bb::change_bit(&p.dcgcuart, 1, on),
        Uart0 => bb::change_bit(&p.dcgcuart, 0, on),
        Ssi3 => bb::change_bit(&p.dcgcssi, 3, on),
        Ssi2 => bb::change_bit(&p.dcgcssi, 2, on),
        Ssi1 => bb::change_bit(&p.dcgcssi, 1, on),
        Ssi0 => bb::change_bit(&p.dcgcssi, 0, on),
        I2c3 => bb::change_bit(&p.dcgci2c, 3, on),
        I2c2 => bb::change_bit(&p.dcgci2c, 2, on),
        I2c1 => bb::change_bit(&p.dcgci2c, 1, on),
        I2c0 => bb::change_bit(&p.dcgci2c, 0, on),
        Usb => bb::change_bit(&p.dcgcusb, 0, on),
        Can => bb::change_bit(&p.dcgccan, 0, on),
        Adc1 => bb::change_bit(&p.dcgcadc, 1, on),
        Adc0 => bb::change_bit(&p.dcgcadc, 0, on),
        AnalogComparator => bb::change_bit(&p.dcgcacmp, 0, on),
        Eeprom => bb::change_bit(&p.dcgceeprom, 0, on),
        Pwm0 => bb::change_bit(&p.dcgcpwm, 0, on),
        Pwm1 => bb::change_bit(&p.dcgcpwm, 1, on),
        Emac0 => bb::change_bit(&p.dcgcemac, 0, on),
        Ephy0 => bb::change_bit(&p.dcgcephy, 0, on),
    }}
}

/// Extension trait that constrains the `SYSCTL` peripheral
pub trait SysctlExt {
    /// Constrains the `SYSCTL` peripheral so it plays nicely with the other
    /// abstractions
    fn constrain(self) -> Sysctl;
}

impl SysctlExt for tm4c129x::SYSCTL {
    fn constrain(self) -> Sysctl {
        Sysctl {
            power_control: PowerControl { _0: () },
            clock_setup: ClockSetup {
                oscillator: Oscillator::PrecisionInternal(SystemClock::UseOscillator(Divider::_1)),
                _0: (),
            },
        }
    }
}

impl ClockSetup {
    /// Fix the clock configuration and produce a record of the configuration
    /// so that other modules can calibrate themselves (e.g. the UARTs).
    pub fn freeze(self) -> Clocks {
        // We own the SYSCTL at this point - no one else can be running.
        let sysctl = unsafe { &*tm4c129x::SYSCTL::ptr() };

        let osc: Hertz;
        let sysclk: Hertz;

        match self.oscillator {
            Oscillator::PrecisionInternal(SystemClock::UseOscillator(div)) => {
                // 1. Once POR has completed, the PIOSC is acting as the system clock.
                osc = 16_000_000.hz();
                sysclk = (osc.0 / (div as u32)).hz();

                sysctl.rsclkcfg.write(|w| w.osysdiv().bits(div as u16 - 1));
            }
            Oscillator::PrecisionInternal(SystemClock::UsePll(output_frequency)) => {
                osc = 16_000_000.hz();
                sysclk = output_frequency.into();

                // 6. Write the PLLFREQ0 and PLLFREQ1 registers with the values of Q, N, MINT,
                // and MFRAC to the configure the desired VCO frequency setting.
                // Crystal, MINT, MINT, N, Ref MHZ, Pll MHZ

                sysctl.rsclkcfg.write(|w| w.pllsrc().piosc());

                sysctl.pllfreq0.write(|w| w
                    .pllpwr().set_bit()

                    .mfrac().bits(0)
                    .mint().bits(30)
                );

                sysctl.pllfreq1.write(|w| w
                    .q().bits(0)
                    .n().bits(0)
                );

                sysctl.rsclkcfg.write(|w| w.newfreq().set_bit());

                let (xbcht, xbce, xws) = match sysclk.0 {
                    0..=16_000_000 => (0, true, 0),
                    16_000_001..=40_000_000 => (2, false, 1),
                    40_000_001..=60_000_000 => (3, false, 2),
                    60_000_001..=80_000_000 => (4, false, 3),
                    80_000_001..=100_000_000 => (5, false, 4),
                    100_000_001..=120_000_000 => (6, false, 5),
                    _ => unreachable!(),
                };

                // 7. Write the MEMTIM0 register to correspond to the new system clock setting.
                sysctl.memtim0.write(|w| unsafe { w
                    .fbcht().bits(xbcht)
                    .ebcht().bits(xbcht)

                    .fbce().bit(xbce)
                    .ebce().bit(xbce)

                    .fws().bits(xws)
                    .ews().bits(xws)
                });

                // 8. Wait for the PLLSTAT register to indicate the PLL has reached lock at the
                // new operating point (or that a timeout period has passed and lock has failed,
                // in which case an error condition exists and this sequence is abandoned and
                // error processing is initiated).
                while sysctl.pllstat.read().lock().bit_is_clear() {
                    cortex_m::asm::nop();
                }

                // 9. Write the RSCLKCFG register's PSYSDIV value, set the USEPLL bit to
                // enabled, and MEMTIMU bit.
                sysctl.rsclkcfg.write(|w| w
                    .usepll().set_bit()
                    .memtimu().set_bit()
                    .psysdiv().bits((480_000_000 / sysclk.0 - 1) as u16)
                );
            }
            Oscillator::Main(crystal_frequency, SystemClock::UseOscillator(div)) => {
                osc = crystal_frequency.into();
                sysclk = (osc.0 / (div as u32)).hz();

                // 2. Power up the MOSC by clearing the NOXTAL bit in the MOSCCTL register.
                sysctl.moscctl.write(|w| w
                    .oscrng().set_bit()

                    .noxtal().clear_bit()
                    .pwrdn().clear_bit()
                );

                let (xbcht, xbce, xws) = match sysclk.0 {
                             0..=15_999_999 => (0, true,  0),
                    16_000_000..=39_999_999 => (2, false, 1),
                    _ => unreachable!(),
                };

                // 7. Write the MEMTIM0 register to correspond to the new system clock
                sysctl.memtim0.modify(|_, w| unsafe { w
                    .fbcht().bits(xbcht)
                    .ebcht().bits(xbcht)

                    .fbce().bit(xbce)
                    .ebce().bit(xbce)

                    .fws().bits(xws)
                    .ews().bits(xws)
                });

                // If single-ended MOSC mode is required, the MOSC is ready to use. If crystal
                // mode is required, clear the PWRDN bit and wait for the MOSCPUPRIS bit to be
                // set in the Raw Interrupt Status (RIS), indicating MOSC crystal mode is ready.
                while sysctl.ris.read().moscpupris().bit_is_clear() {
                    nop();
                }

                // 4. Set the OSCSRC field to 0x3 in the RSCLKCFG register at offset 0x0B0.
                sysctl.rsclkcfg.write(|w| w
                    .oscsrc().mosc()
                    .memtimu().set_bit()

                    .osysdiv().bits(div as u16 - 1)
                );
            }

            Oscillator::Main(crystal_frequency, SystemClock::UsePll(output_frequency)) => {
                osc = crystal_frequency.into();
                sysclk = output_frequency.into();

                // 2. Power up the MOSC by clearing the NOXTAL bit in the MOSCCTL register.
                sysctl.moscctl.write(|w| w
                    .oscrng().set_bit()

                    .noxtal().clear_bit()
                    .pwrdn().clear_bit()
                );

                // If single-ended MOSC mode is required, the MOSC is ready to use. If crystal
                // mode is required, clear the PWRDN bit and wait for the MOSCPUPRIS bit to be
                // set in the Raw Interrupt Status (RIS), indicating MOSC crystal mode is ready.
                while sysctl.ris.read().moscpupris().bit_is_clear() {
                    nop();
                }

                // 6. Write the PLLFREQ0 and PLLFREQ1 registers with the values of Q, N, MINT,
                // and MFRAC to the configure the desired VCO frequency setting.
                // Crystal, MINT, MINT, N, Ref MHZ, Pll MHZ

                sysctl.rsclkcfg.write(|w| w.pllsrc().mosc());

                sysctl.pllfreq1.write(|w| w
                    .q().bits(0)
                    .n().bits(4)
                );

                sysctl.pllfreq0.write(|w| w
                    .mfrac().bits(0)
                    .mint().bits(96)
                );

                sysctl.pllfreq0.modify(|_, w| w.pllpwr().set_bit());

                // 8. Wait for the PLLSTAT register to indicate the PLL has reached lock at the
                // new operating point (or that a timeout period has passed and lock has failed,
                // in which case an error condition exists and this sequence is abandoned and
                // error processing is initiated).

                while sysctl.pllstat.read().lock().bit_is_clear() {
                    cortex_m::asm::nop();
                }

                let (xbcht, xbce, xws) = match sysclk.0 {
                              0..= 16_000_000 => (0, true,  0),
                     16_000_001..= 40_000_000 => (2, false, 1),
                     40_000_001..= 60_000_000 => (3, false, 2),
                     60_000_001..= 80_000_000 => (4, false, 3),
                     80_000_001..=100_000_000 => (5, false, 4),
                    100_000_001..=120_000_000 => (6, false, 5),
                    _ => unreachable!(),
                };

                // 7. Write the MEMTIM0 register to correspond to the new system clock setting.
                sysctl.memtim0.write(|w| unsafe { w
                    .fbcht().bits(xbcht)
                    .ebcht().bits(xbcht)

                    .fbce().bit(xbce)
                    .ebce().bit(xbce)

                    .fws().bits(xws)
                    .ews().bits(xws)
                });

                // 9. Write the RSCLKCFG register's PSYSDIV value, set the USEPLL bit to
                // enabled, and MEMTIMU bit.
                sysctl.rsclkcfg.write(|w| w
                    .usepll().set_bit()
                    .memtimu().set_bit()
                    .psysdiv().bits((480_000_000 / sysclk.0 - 1) as u16)
                );
            }

            Oscillator::LowFrequencyInternal(_div) => unimplemented!(),
        }

        Clocks { osc, sysclk }
    }
}

/// This module is all about identifying the physical chip we're running on.
pub mod chip_id {
    pub use tm4c_hal::sysctl::chip_id::*;

    /// Read DID0 and DID1 to discover what sort of
    /// TM4C129 this is.
    pub fn get() -> Result<ChipId, Error> {
        // This is safe as it's read only
        let p = unsafe { &*tm4c129x::SYSCTL::ptr() };
        let did0 = p.did0.read();
        if did0.ver().bits() != 0x01 {
            return Err(Error::UnknownDid0Ver(did0.ver().bits()));
        }
        let device_class = match did0.class().bits() {
            0x05 => DeviceClass::StellarisBlizzard,
            0x0a => DeviceClass::Snowflake,
            _ => DeviceClass::Unknown,
        };
        let major = did0.maj().bits();
        let minor = did0.min().bits();
        let did1 = p.did1.read();
        if did1.ver().bits() != 0x01 {
            // Stellaris LM3F (0x00) is not supported
            return Err(Error::UnknownDid1Ver(did1.ver().bits()));
        }
        let part_no = match did1.prtno().bits() {
            0x1F => PartNo::Tm4c1294ncpdt,
            45 => PartNo::Tm4c129encpdt,
            e => PartNo::Unknown(e),
        };
        let pin_count = match did1.pincnt().bits() {
            0 => PinCount::_28,
            1 => PinCount::_48,
            2 => PinCount::_100,
            3 => PinCount::_64,
            4 => PinCount::_144,
            5 => PinCount::_157,
            6 => PinCount::_168,
            _ => PinCount::Unknown,
        };
        let temp_range = match did1.temp().bits() {
            0 => TempRange::Commercial,
            1 => TempRange::Industrial,
            2 => TempRange::Extended,
            3 => TempRange::IndustrialOrExtended,
            _ => TempRange::Unknown,
        };
        let package = match did1.pkg().bits() {
            0 => Package::Soic,
            1 => Package::Lqfp,
            2 => Package::Bga,
            _ => Package::Unknown,
        };
        let rohs_compliant = did1.rohs().bit_is_set();
        let qualification = match did1.qual().bits() {
            0 => Qualification::EngineeringSample,
            1 => Qualification::PilotProduction,
            2 => Qualification::FullyQualified,
            _ => Qualification::Unknown,
        };
        Ok(ChipId {
            device_class,
            major,
            minor,
            pin_count,
            temp_range,
            package,
            rohs_compliant,
            qualification,
            part_no,
        })
    }
}
