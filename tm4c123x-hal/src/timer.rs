//! Timers

use crate::{
    hal::timer::{CountDown, Periodic},
    sysctl::{self, Clocks, PowerControl},
};

#[rustfmt::skip]
use tm4c123x::{
    TIMER0, TIMER1, TIMER2, TIMER3, TIMER4, TIMER5,
    WTIMER0, WTIMER1, WTIMER2, WTIMER3, WTIMER4, WTIMER5,
};
use tm4c_hal::time::{Hertz, U32Ext};
use void::Void;

/// Hardware timers
pub struct Timer<TIM> {
    tim: TIM,
    clocks: Clocks,
    timeout: Hertz,
}

/// Interrupt events
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

macro_rules! hal {
    ($($TIM:ident: ($tim:ident, $powerDomain:ident),)+) => {
        $(
            impl Periodic for Timer<$TIM> {}

            impl CountDown for Timer<$TIM> {
                type Time = Hertz;

                fn start<T: Into<Hertz>>(&mut self, timeout: T) {
                    // Disable timer
                    self.tim.ctl.modify(|_, w| w
                        .taen().clear_bit()
                        .tben().clear_bit()
                    );
                    self.timeout = timeout.into();

                    let frequency = self.timeout.0;
                    let ticks = self.clocks.sysclk.0 / frequency;

                    self.tim.tav.write(|w| unsafe { w.bits(ticks) });
                    self.tim.tailr.write(|w| unsafe { w.bits(ticks) });

                    // start counter
                    self.tim.ctl.modify(|_, w| w.taen().set_bit());
                }

                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.tim.ris.read().tatoris().bit_is_set() {
                        Ok(self.tim.icr.write(|w| w.tatocint().set_bit()))
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl Timer<$TIM> {
                /// Configures a TIM peripheral as a periodic count down timer
                pub fn $tim<T: Into<Hertz>>(tim: $TIM, timeout: T, pc: &PowerControl, clocks: Clocks) -> Self {
                    use sysctl::{Domain, RunMode, PowerState};

                    // power up
                    sysctl::control_power(
                        pc,
                        Domain::$powerDomain,
                        RunMode::Run,
                        PowerState::On,
                    );
                    sysctl::reset(pc, Domain::$powerDomain);

                    // Stop Timers
                    tim.ctl.write(|w| w
                        .taen().clear_bit()
                        .tben().clear_bit()
                        .tastall().set_bit()
                    );

                    // GPTMCFG = 0x0 (chained - 2x16 = 32bits) This
                    // will not force 32bits wide timer, this will
                    // really force the wider range to be used (32 for
                    // 16/32bits timers, 64 for 32/64).
                    tim.cfg.write(|w| w.cfg()._32_bit_timer());

                    tim.tamr.write(|w| w.tamr().period());

                    let mut timer = Timer {
                        tim,
                        clocks,
                        timeout: 0.hz(),
                    };
                    timer.start(timeout);

                    timer
                }

                /// Starts listening for an `event`
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.imr.modify(|_, w|  w.tatoim().set_bit());
                        }
                    }
                }

                /// Stops listening for an `event`
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.imr.modify(|_, w| w.tatoim().clear_bit());
                        }
                    }
                }

                /// Releases the TIM peripheral
                pub fn free(self) -> $TIM {
                    // pause counter
                    self.tim.ctl.write(|w| w
                        .taen().clear_bit()
                        .tben().clear_bit()
                    );
                    self.tim
                }
            }
        )+
    }
}

hal! {
    TIMER0: (timer0, Timer0),
    TIMER1: (timer1, Timer1),
    TIMER2: (timer2, Timer2),
    TIMER3: (timer3, Timer3),
    TIMER4: (timer4, Timer4),
    TIMER5: (timer5, Timer5),

    WTIMER0: (wtimer0, WideTimer0),
    WTIMER1: (wtimer1, WideTimer1),
    WTIMER2: (wtimer2, WideTimer2),
    WTIMER3: (wtimer3, WideTimer3),
    WTIMER4: (wtimer4, WideTimer4),
    WTIMER5: (wtimer5, WideTimer5),
}
