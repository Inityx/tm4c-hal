//! Inter-Integrated Circuit (I2C) bus

use core::hint::unreachable_unchecked;
use cortex_m::asm::delay;
use tm4c123x::{I2C0, I2C1, I2C2, I2C3};

use crate::{
    gpio::{
        gpioa::{PA6, PA7},
        gpiob::{PB2, PB3},
        gpiod::{PD0, PD1},
        gpioe::{PE4, PE5},
        AlternateFunction, Floating, OpenDrain, OutputMode, AF3,
    },
    hal::blocking::i2c::{Read, Write, WriteRead},
    sysctl::{self, Clocks},
    time::Hertz,
};

/// I2C error
#[derive(Debug)]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration loss
    Arbitration,

    /// Missing Data ACK
    DataAck,

    /// Missing Addrees ACK
    AdrAck,

    #[doc(hidden)]
    _Extensible,
}

// FIXME these should be "closed" traits
/// SCL pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SclPin<I2C> {}

/// SDA pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SdaPin<I2C> {}

unsafe impl<T> SclPin<I2C0> for PB2<AlternateFunction<AF3, T>> where T: OutputMode {}
unsafe impl SdaPin<I2C0> for PB3<AlternateFunction<AF3, OpenDrain<Floating>>> {}

unsafe impl<T> SclPin<I2C1> for PA6<AlternateFunction<AF3, T>> where T: OutputMode {}
unsafe impl SdaPin<I2C1> for PA7<AlternateFunction<AF3, OpenDrain<Floating>>> {}

unsafe impl<T> SclPin<I2C2> for PE4<AlternateFunction<AF3, T>> where T: OutputMode {}
unsafe impl SdaPin<I2C2> for PE5<AlternateFunction<AF3, OpenDrain<Floating>>> {}

unsafe impl<T> SclPin<I2C3> for PD0<AlternateFunction<AF3, T>> where T: OutputMode {}
unsafe impl SdaPin<I2C3> for PD1<AlternateFunction<AF3, OpenDrain<Floating>>> {}

/// I2C peripheral operating in master mode
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

macro_rules! busy_wait {
    ($i2c:expr, $flag:ident, $op:ident) => {
        // in 'release' builds, the time between setting the `run` bit and checking the `busy`
        // bit is too short and the `busy` bit is not reliably set by the time you get there,
        // it can take up to 8 clock cycles for the `run` to begin so this delay allows time
        // for that hardware synchronization
        delay(2);

        loop {
            let mcs = $i2c.mcs.read();

            if mcs.error().bit_is_set() {
                return Err(
                    if mcs.adrack().bit_is_set() {
                        Error::AdrAck
                    } else if mcs.datack().bit_is_set() {
                        Error::DataAck
                    } else {
                        Error::Bus
                    }
                );
            }

            if mcs.arblst().bit_is_set() {
                return Err(Error::Arbitration);
            }

            if mcs.$flag().$op() {
                break;
            }
        }
    };
}

macro_rules! hal {
    ($($I2CX:ident: ($powerDomain:ident, $i2cX:ident),)+) => {
        $(
            impl<SCL, SDA> I2c<$I2CX, (SCL, SDA)> {
                /// Configures the I2C peripheral to work in master mode
                pub fn $i2cX<F>(
                    i2c: $I2CX,
                    pins: (SCL, SDA),
                    freq: F,
                    clocks: &Clocks,
                    pc: &sysctl::PowerControl,
                ) -> Self
                where
                    F: Into<Hertz>,
                    SCL: SclPin<$I2CX>,
                    SDA: SdaPin<$I2CX>,
                {
                    use sysctl::{Domain, RunMode, PowerState};

                    sysctl::control_power(
                        pc,
                        Domain::$powerDomain,
                        RunMode::Run,
                        PowerState::On,
                    );
                    sysctl::reset(pc, Domain::$powerDomain);

                    // set Master Function Enable, and clear other bits.
                    i2c.mcr.write(|w| w.mfe().set_bit());

                    // Write TimerPeriod configuration and clear other bits.
                    let freq = freq.into().0;
                    let tpr = ((clocks.sysclk.0/(2*10*freq))-1) as u8;

                    i2c.mtpr.write(|w| unsafe {w.tpr().bits(tpr)});

                    I2c { i2c, pins }
                }

                /// Releases the I2C peripheral and associated pins
                pub fn free(self) -> ($I2CX, (SCL, SDA)) {
                    (self.i2c, self.pins)
                }
            }

            impl<PINS> Write for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
                    if bytes.is_empty() { return Ok(()); }

                    // Write Slave address and clear Receive bit
                    self.i2c.msa.write(|w| unsafe { w.sa().bits(addr) });

                    match bytes {
                        [] => unsafe { unreachable_unchecked() } // Explicit check for empty buffer at top
                        [byte] => {
                            self.i2c.mdr.write(|w| unsafe { w.data().bits(*byte) });

                            busy_wait!(self.i2c, busbsy, bit_is_clear);

                            self.i2c.mcs.write(|w| w
                                .stop().set_bit()
                                .start().set_bit()
                                .run().set_bit()
                            );
                        }
                        [first, middle @ .., last] => {
                            self.i2c.mdr.write(|w| unsafe { w.data().bits(*first) });

                            busy_wait!(self.i2c, busbsy, bit_is_clear);

                            self.i2c.mcs.write(|w| w
                                .start().set_bit()
                                .run().set_bit()
                            );

                            for &byte in middle.iter() {
                                busy_wait!(self.i2c, busy, bit_is_clear);

                                self.i2c.mdr.write(|w| unsafe { w.data().bits(byte) });
                                self.i2c.mcs.write(|w| w.run().set_bit());
                            }

                            busy_wait!(self.i2c, busy, bit_is_clear);

                            self.i2c.mdr.write(|w| unsafe { w.data().bits(*last) });
                            self.i2c.mcs.write(|w| w
                                .stop().set_bit()
                                .run().set_bit()
                            );
                        }
                    }

                    busy_wait!(self.i2c, busy, bit_is_clear);

                    Ok(())
                }
            }

            impl<PINS> Read for I2c<$I2CX, PINS> {
                type Error = Error;

                fn read(
                    &mut self,
                    addr: u8,
                    buffer: &mut [u8],
                ) -> Result<(), Error> {
                    if buffer.is_empty() { return Ok(()); }

                    // Write Slave address and set Receive bit
                    self.i2c.msa.write(|w| unsafe { w
                        .sa().bits(addr)
                        .rs().set_bit()
                    });

                    busy_wait!(self.i2c, busbsy, bit_is_clear);

                    match buffer {
                        [] => unsafe { unreachable_unchecked() } // Explicit check for empty buffer at top
                        [byte] => {
                            self.i2c.mcs.write(|w| w
                                .run().set_bit()
                                .start().set_bit()
                                .stop().set_bit()
                            );

                            busy_wait!(self.i2c, busy, bit_is_clear);
                            *byte = self.i2c.mdr.read().data().bits();
                        }
                        [first, middle @ .., last] => {
                            self.i2c.mcs.write(|w| w
                                .start().set_bit()
                                .run().set_bit()
                                .ack().set_bit()
                            );

                            busy_wait!(self.i2c, busy, bit_is_clear);
                            *first = self.i2c.mdr.read().data().bits();

                            for byte in middle.iter_mut() {
                                self.i2c.mcs.write(|w| w
                                    .run().set_bit()
                                    .ack().set_bit()
                                );

                                busy_wait!(self.i2c, busy, bit_is_clear);
                                *byte = self.i2c.mdr.read().data().bits();
                            }

                            self.i2c.mcs.write(|w| w
                                .run().set_bit()
                                .stop().set_bit()
                            );

                            busy_wait!(self.i2c, busy, bit_is_clear);
                            *last = self.i2c.mdr.read().data().bits();
                        }
                    }

                    Ok(())
                }
            }

            impl<PINS> WriteRead for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write_read(
                    &mut self,
                    addr: u8,
                    bytes: &[u8],
                    buffer: &mut [u8],
                ) -> Result<(), Error> {
                    match (bytes, buffer) {
                        ([], []) => Ok(()),
                        (bytes, []) => self.write(addr, bytes),
                        ([], buffer) => self.read(addr, buffer),
                        (
                            [send_first, send_rest @ ..],
                            [recv_first, recv_rest @ ..],
                        ) => {
                            // Write Slave address and clear Receive bit
                            self.i2c.msa.write(|w| unsafe { w.sa().bits(addr) });

                            self.i2c.mdr.write(|w| unsafe { w.data().bits(*send_first) });

                            busy_wait!(self.i2c, busbsy, bit_is_clear);

                            self.i2c.mcs.write(|w| w
                                .start().set_bit()
                                .run().set_bit()
                            );

                            busy_wait!(self.i2c, busy, bit_is_clear);

                            for byte in send_rest.iter() {
                                self.i2c.mdr.write(|w| unsafe { w.data().bits(*byte) });
                                self.i2c.mcs.write(|w| w.run().set_bit());

                                busy_wait!(self.i2c, busy, bit_is_clear);
                            }

                            // Write Slave address and set Receive bit
                            self.i2c.msa.write(|w| unsafe { w
                                .sa().bits(addr)
                                .rs().set_bit()
                            });

                            match recv_rest {
                                [] => {
                                    // emit Repeated START and STOP
                                    self.i2c.mcs.write(|w| w
                                        .run().set_bit()
                                        .start().set_bit()
                                        .stop().set_bit()
                                    );

                                    busy_wait!(self.i2c, busy, bit_is_clear);
                                    *recv_first = self.i2c.mdr.read().data().bits();
                                }
                                [recv_middle @ .., recv_last] => {
                                    // emit Repeated START
                                    self.i2c.mcs.write(|w| w
                                        .run().set_bit()
                                        .start().set_bit()
                                        .ack().set_bit()
                                    );

                                    busy_wait!(self.i2c, busy, bit_is_clear);
                                    *recv_first = self.i2c.mdr.read().data().bits();

                                    for byte in recv_middle.iter_mut() {
                                        self.i2c.mcs.write(|w| w
                                            .run().set_bit()
                                            .ack().set_bit()
                                        );
                                        busy_wait!(self.i2c, busy, bit_is_clear);
                                        *byte = self.i2c.mdr.read().data().bits();
                                    }

                                    self.i2c.mcs.write(|w| w
                                        .run().set_bit()
                                        .stop().set_bit()
                                    );

                                    busy_wait!(self.i2c, busy, bit_is_clear);
                                    *recv_last = self.i2c.mdr.read().data().bits();
                                }
                            }

                            Ok(())
                        }
                    }
                }
            }
        )+
    }
}

hal! {
    I2C0: (I2c0, i2c0),
    I2C1: (I2c1, i2c1),
    I2C2: (I2c2, i2c2),
    I2C3: (I2c3, i2c3),
}
