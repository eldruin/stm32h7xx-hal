//! Power Configuration
//!
//! This module configures the PWR unit to provide the core voltage
//! `VCORE`. The voltage scaling mode is fixed at VOS1 (High
//! Performance).
//!
//! When the system starts up, it is in Run* mode. After the call to
//! `freeze`, it will be in Run mode. See RM0433 Rev 7 Section 6.6.1
//! "System/D3 domain modes".
//!
//! # Example
//!
//! ```rust
//!     let dp = pac::Peripherals::take().unwrap();
//!
//!     let pwr = dp.PWR.constrain();
//!     let vos = pwr.freeze();
//!
//!     assert_eq!(vos, VoltageScale::Scale1);
//! ```
//!
//! # SMPS
//!
//! Some parts include an integrated Switched Mode Power Supply (SMPS)
//! to supply VCORE. For these parts, the method of supplying VCORE
//! can be specified. Refer to RM0399 Rev 3 Table 32. for a more
//! detailed descriptions of the possible modes.
//!
//! - Low Dropout Regulator [ldo](Pwr#ldo)
//! - Switch Mode Power Supply [smps](Pwr#smps)
//! - Bypass [bypass](Pwr#pypass)
//! - SMPS Output at 1.8V, then LDO [smps_1v8_feeds_ldo](Pwr#smps_1v8_feeds_ldo)
//! - SMPS Output at 2.5V, then LDO [smps_2v5_feeds_ldo](Pwr#smps_2v5_feeds_ldo)
//!
//! **Note**: Specifying the wrong mode for your hardware will cause
//! undefined results.
//!
//! ```rust
//!     let dp = pac::Peripherals::take().unwrap();
//!
//!     let pwr = dp.PWR.constrain();
//!     let vos = pwr.smps().freeze();
//!
//!     assert_eq!(vos, VoltageScale::Scale1);
//! ```
//!
//! The VCORE supply configuration can only be set once after each
//! POR, and this is enforced by hardware. If you add or change the
//! power supply method, `freeze` will panic until you power on reset
//! your board.

use crate::stm32::PWR;
#[cfg(feature = "revision_v")]
use crate::stm32::{RCC, SYSCFG};

/// Extension trait that constrains the `PWR` peripheral
pub trait PwrExt {
    fn constrain(self) -> Pwr;
}

impl PwrExt for PWR {
    fn constrain(self) -> Pwr {
        Pwr {
            rb: self,
            #[cfg(any(feature = "dualcore"))]
            supply_configuration: SupplyConfiguration::Default,
            #[cfg(feature = "revision_v")]
            enable_vos0: false,
        }
    }
}

/// Constrained PWR peripheral
///
/// Generated by calling `constrain` on the PAC's PWR peripheral.
pub struct Pwr {
    pub(crate) rb: PWR,
    #[cfg(any(feature = "dualcore"))]
    supply_configuration: SupplyConfiguration,
    #[cfg(feature = "revision_v")]
    enable_vos0: bool,
}

/// Voltage Scale
///
/// Generated when the PWR peripheral is frozen. The existence of this
/// value indicates that the voltage scaling configuration can no
/// longer be changed.
#[derive(PartialEq)]
pub enum VoltageScale {
    Scale0,
    Scale1,
    Scale2,
    Scale3,
}

/// SMPS Supply Configuration - Dual Core parts
///
/// Refer to RM0399 Rev 3 Table 32.
#[cfg(any(feature = "dualcore"))]
enum SupplyConfiguration {
    Default = 0,
    LDOSupply,
    DirectSMPS,
    SMPSFeedsIntoLDO1V8,
    SMPSFeedsIntoLDO2V5,
    // External SMPS loads not supported
    Bypass,
}

#[cfg(any(feature = "dualcore"))]
macro_rules! supply_configuration_setter {
    ($($config:ident: $name:ident, $doc:expr,)*) => {
        $(
            #[doc=$doc]
            pub fn $name(mut self) -> Self {
                self.supply_configuration = SupplyConfiguration::$config;
                self
            }
        )*
    };
}

impl Pwr {
    #[cfg(any(feature = "dualcore"))]
    supply_configuration_setter! {
        LDOSupply: ldo, "VCORE power domains supplied from the LDO. \
                         LDO voltage adjusted by VOS. \
                         LDO power mode will follow the system \
                         low-power mode.",
        DirectSMPS: smps, "VCORE power domains are supplied from the \
                           SMPS step-down converter. SMPS output voltage \
                           adjusted by VOS. SMPS power mode will follow \
                           the system low-power mode",
        Bypass: bypass, "VCORE is supplied from an external source",
        SMPSFeedsIntoLDO1V8:
        smps_1v8_feeds_ldo, "VCORE power domains supplied from the LDO. \
                         LDO voltage adjusted by VOS. \
                         LDO power mode will follow the system \
                         low-power mode. SMPS output voltage set to \
                         1.8V. SMPS power mode will follow \
                         the system low-power mode",
        SMPSFeedsIntoLDO2V5:
        smps_2v5_feeds_ldo, "VCORE power domains supplied from the LDO. \
                         LDO voltage adjusted by VOS. \
                         LDO power mode will follow the system \
                         low-power mode. SMPS output voltage set to \
                         2.5V. SMPS power mode will follow \
                         the system low-power mode",
    }

    /// Verify that the lower byte of CR3 reads as written
    #[cfg(any(feature = "dualcore"))]
    fn verify_supply_configuration(&self) {
        use SupplyConfiguration::*;
        let error = "Values in lower byte of PWR.CR3 do not match the \
                     configured power mode. These values can only be set \
                     once for each POR (Power-on-Reset). Try removing power \
                     to your board.";

        match self.supply_configuration {
            LDOSupply => {
                assert!(self.rb.cr3.read().sden().bit_is_clear(), error);
                assert!(self.rb.cr3.read().ldoen().bit_is_set(), error);
            }
            DirectSMPS => {
                assert!(self.rb.cr3.read().sden().bit_is_set(), error);
                assert!(self.rb.cr3.read().ldoen().bit_is_clear(), error);
            }
            SMPSFeedsIntoLDO1V8 => {
                assert!(self.rb.cr3.read().sden().bit_is_set(), error);
                assert!(self.rb.cr3.read().ldoen().bit_is_clear(), error);
                assert!(self.rb.cr3.read().sdlevel().bits() == 1, error);
            }
            SMPSFeedsIntoLDO2V5 => {
                assert!(self.rb.cr3.read().sden().bit_is_set(), error);
                assert!(self.rb.cr3.read().ldoen().bit_is_clear(), error);
                assert!(self.rb.cr3.read().sdlevel().bits() == 2, error);
            }
            Bypass => {
                assert!(self.rb.cr3.read().sden().bit_is_clear(), error);
                assert!(self.rb.cr3.read().ldoen().bit_is_clear(), error);
                assert!(self.rb.cr3.read().bypass().bit_is_set(), error);
            }
            Default => {} // Default configuration is NOT verified
        }
    }

    #[cfg(feature = "revision_v")]
    pub fn vos0(mut self, _: &SYSCFG) -> Self {
        self.enable_vos0 = true;
        self
    }

    pub fn freeze(self) -> VoltageScale {
        // NB. The lower bytes of CR3 can only be written once after
        // POR, and must be written with a valid combination. Refer to
        // RM0433 Rev 7 6.8.4. This is partially enforced by dropping
        // `self` at the end of this method, but of course we cannot
        // know what happened between the previous POR and here.

        #[cfg(any(feature = "singlecore"))]
        self.rb.cr3.modify(|_, w| {
            w.scuen().set_bit().ldoen().set_bit().bypass().clear_bit()
        });

        #[cfg(any(feature = "dualcore"))]
        self.rb.cr3.modify(|_, w| {
            use SupplyConfiguration::*;

            match self.supply_configuration {
                LDOSupply => w.sden().clear_bit().ldoen().set_bit(),
                DirectSMPS => w.sden().set_bit().ldoen().clear_bit(),
                SMPSFeedsIntoLDO1V8 => unsafe {
                    w.sden().set_bit().ldoen().set_bit().sdlevel().bits(1)
                },
                SMPSFeedsIntoLDO2V5 => unsafe {
                    w.sden().set_bit().ldoen().set_bit().sdlevel().bits(2)
                },
                Bypass => {
                    w.sden().clear_bit().ldoen().clear_bit().bypass().set_bit()
                }
                Default => {
                    // Default configuration. The actual reset value of
                    // CR3 varies between packages (See RM0399 Section
                    // 7.8.4 Footnote 2). Therefore we do not modify
                    // anything here.
                    w
                }
            }
        });
        // Verify supply configuration, panics if these values read
        // from CR3 do not match those written.
        #[cfg(any(feature = "dualcore"))]
        self.verify_supply_configuration();

        // Validate the supply configuration. If you are stuck here, it is
        // because the voltages on your board do not match those specified
        // in the D3CR.VOS and CR3.SDLEVEL fields.  By default after reset
        // VOS = Scale 3, so check that the voltage on the VCAP pins =
        // 1.0V.
        while self.rb.csr1.read().actvosrdy().bit_is_clear() {}

        // We have now entered Run mode. See RM0433 Rev 7 Section 6.6.1

        // go to VOS1 voltage scale for high performance
        self.rb.d3cr.write(|w| unsafe { w.vos().bits(0b11) });
        while self.rb.d3cr.read().vosrdy().bit_is_clear() {}

        // Enable overdrive for maximum clock
        // Syscfgen required to set enable overdrive
        #[cfg(feature = "revision_v")]
        if self.enable_vos0 {
            unsafe {
                &(*RCC::ptr()).apb4enr.modify(|_, w| w.syscfgen().enabled())
            };
            #[cfg(any(feature = "dualcore"))]
            unsafe {
                &(*SYSCFG::ptr()).pwrcr.modify(|_, w| w.oden().set_bit())
            };
            #[cfg(not(any(feature = "dualcore")))]
            unsafe {
                &(*SYSCFG::ptr()).pwrcr.modify(|_, w| w.oden().bits(1))
            };
            while self.rb.d3cr.read().vosrdy().bit_is_clear() {}
            return VoltageScale::Scale0;
        }

        VoltageScale::Scale1
    }
}
