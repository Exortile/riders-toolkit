use num::{FromPrimitive, Unsigned};

pub enum Alignment<T: Unsigned> {
    A8(T),
    A16(T),
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
    pub fn unwrap(&self) -> T {
        match self {
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
