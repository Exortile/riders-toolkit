//! A very simple utility module. Contains some of the widely used utility functionalities.

use num::{FromPrimitive, Unsigned};

/// Provides an easy way to align (round up) any unsigned integer to the given alignment.
///
/// Once an [`Alignment`] is constructed, you can call [`Alignment::unwrap()`] on it to receive the
/// resulting aligned value.
pub enum Alignment<T: Unsigned> {
    /// Aligns to a 4-bit alignment.
    A4(T),
    /// Aligns to a 8-bit alignment.
    A8(T),
    /// Aligns to a 16-bit alignment.
    A16(T),
    /// Aligns to a 32-bit alignment.
    A32(T),
}

impl<T> Alignment<T>
where
    T: Unsigned,
    T: FromPrimitive,
    T: Clone,
    T: std::ops::Not<Output = T>,
    T: std::ops::BitAnd<Output = T>,
{
    /// Aligns and returns the given value as per the given alignment variant.
    pub fn unwrap(&self) -> T {
        match self {
            Alignment::A4(val) => {
                (val.clone() + T::from_u8(3).unwrap()) & T::from_u8(3).map(|x| !x).unwrap()
            }
            Alignment::A8(val) => {
                (val.clone() + T::from_u8(7).unwrap()) & T::from_u8(7).map(|x| !x).unwrap()
            }
            Alignment::A16(val) => {
                (val.clone() + T::from_u8(15).unwrap()) & T::from_u8(15).map(|x| !x).unwrap()
            }
            Alignment::A32(val) => {
                (val.clone() + T::from_u8(31).unwrap()) & T::from_u8(31).map(|x| !x).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_4bit() {
        let alignment = Alignment::A4(1u32);
        assert_eq!(alignment.unwrap(), 4);
    }

    #[test]
    fn align_8bit() {
        let alignment = Alignment::A8(5u32);
        assert_eq!(alignment.unwrap(), 8);
    }

    #[test]
    fn align_16bit() {
        let alignment = Alignment::A16(9u32);
        assert_eq!(alignment.unwrap(), 16);
    }

    #[test]
    fn align_32bit() {
        let alignment = Alignment::A32(16u32);
        assert_eq!(alignment.unwrap(), 32);
    }
}
