use crate::tree::ProgressStep;
use std::{fmt, fmt::Write, ops::Deref};

#[cfg(feature = "unit-bytes")]
mod bytes;
#[cfg(feature = "unit-bytes")]
pub use bytes::Bytes;

pub trait DisplayValue {
    fn display_current_value(
        &self,
        f: &mut fmt::Formatter,
        value: ProgressStep,
        _upper: Option<ProgressStep>,
    ) -> fmt::Result {
        write!(f, "{}", value)
    }
    fn display_upper_bound(
        &self,
        f: &mut fmt::Formatter,
        upper_bound: ProgressStep,
        _value: ProgressStep,
    ) -> fmt::Result {
        write!(f, "{}", upper_bound)
    }
    fn display_unit(&self, f: &mut fmt::Formatter, value: ProgressStep) -> fmt::Result;
    fn display_percentage(&self, f: &mut fmt::Formatter, percentage: f64) -> fmt::Result {
        write!(f, "[{}%]", percentage as usize)
    }
}

impl DisplayValue for &'static str {
    fn display_unit(&self, f: &mut fmt::Formatter, _value: usize) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub enum Unit {
    Label(&'static str, Option<Mode>),
    Dynamic(Box<dyn DisplayValue>, Option<Mode>),
}

/// Construction
impl Unit {
    pub fn label(label: &'static str) -> Self {
        Unit::Label(label, None)
    }
    pub fn label_and_mode(label: &'static str, mode: Mode) -> Self {
        Unit::Label(label, Some(mode))
    }
    pub fn dynamic(label: impl DisplayValue + 'static) -> Self {
        Unit::Dynamic(Box::new(label), None)
    }
    pub fn dynamic_and_mode(label: impl DisplayValue + 'static, mode: Mode) -> Self {
        Unit::Dynamic(Box::new(label), Some(mode))
    }
}

/// Display and utilities
impl Unit {
    pub fn display(&self, current_value: ProgressStep, upper_bound: Option<ProgressStep>) -> UnitDisplay {
        UnitDisplay {
            current_value,
            upper_bound,
            parent: self,
            display: WhatToDisplay::ValuesAndUnit,
        }
    }

    pub fn as_display_value(&self) -> (&dyn DisplayValue, Option<Mode>) {
        match self {
            Unit::Label(unit, mode) => (unit, *mode),
            Unit::Dynamic(unit, mode) => (unit.deref(), *mode),
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Mode {
    PercentageBeforeValue,
    PercentageAfterUnit,
}

pub struct UnitDisplay<'a> {
    current_value: ProgressStep,
    upper_bound: Option<ProgressStep>,
    parent: &'a Unit,
    display: WhatToDisplay,
}

enum WhatToDisplay {
    ValuesAndUnit,
    Unit,
    Values,
}

impl WhatToDisplay {
    fn values(&self) -> bool {
        match self {
            WhatToDisplay::Values | WhatToDisplay::ValuesAndUnit => true,
            _ => false,
        }
    }
    fn unit(&self) -> bool {
        match self {
            WhatToDisplay::Unit | WhatToDisplay::ValuesAndUnit => true,
            _ => false,
        }
    }
}

impl<'a> UnitDisplay<'a> {
    pub fn values(mut self) -> Self {
        self.display = WhatToDisplay::Values;
        self
    }
    pub fn unit(mut self) -> Self {
        self.display = WhatToDisplay::Unit;
        self
    }
}

impl<'a> fmt::Display for UnitDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (unit, mode): (&dyn DisplayValue, _) = self.parent.as_display_value();

        let mode_and_fraction = mode.and_then(|mode| {
            self.upper_bound
                .map(|upper| (mode, (self.current_value as f64 / upper as f64) * 100.0))
        });
        if self.display.values() {
            if let Some((Mode::PercentageBeforeValue, fraction)) = mode_and_fraction {
                unit.display_percentage(f, fraction)?;
                f.write_char(' ')?;
            }
            unit.display_current_value(f, self.current_value, self.upper_bound)?;
            if let Some(upper) = self.upper_bound {
                f.write_char('/')?;
                unit.display_upper_bound(f, upper, self.current_value)?;
            }
        }
        if self.display.unit() {
            if self.display.values() {
                f.write_char(' ')?;
            }
            unit.display_unit(f, self.current_value)?;

            if let Some((Mode::PercentageAfterUnit, fraction)) = mode_and_fraction {
                f.write_char(' ')?;
                unit.display_percentage(f, fraction)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
