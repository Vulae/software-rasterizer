#![allow(unused)]

use std::io::Read;

/// False = Little Endian, True = Big Endian
pub type Endianness = bool;
pub const LITTLE_ENDIAN: Endianness = false;
pub const BIG_ENDIAN: Endianness = true;

pub trait Primitive: Sized {
    fn read<const E: Endianness>(data: &mut impl Read) -> std::io::Result<Self>;
}

pub struct Reader<const E: Endianness, R: Read> {
    data: R,
}

impl<R: Read> Reader<LITTLE_ENDIAN, R> {
    pub fn new_le(data: R) -> Self {
        Self { data }
    }
}

impl<R: Read> Reader<BIG_ENDIAN, R> {
    pub fn new_be(data: R) -> Self {
        Self { data }
    }
}

impl<const E: Endianness, R: Read> Reader<E, R> {
    pub fn read_prim<P: Primitive>(&mut self) -> std::io::Result<P> {
        P::read::<E>(&mut self.data)
    }

    pub fn read_buf(&mut self, length: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; length];
        self.data.read_exact(&mut buf)?;
        Ok(buf)
    }
}

macro_rules! impl_primitive_number {
    ($type:ty) => {
        impl Primitive for $type {
            fn read<const E: Endianness>(data: &mut impl Read) -> std::io::Result<Self> {
                let mut buf = [0; std::mem::size_of::<Self>()];
                data.read_exact(&mut buf)?;
                match E {
                    LITTLE_ENDIAN => Ok(Self::from_le_bytes(buf)),
                    BIG_ENDIAN => Ok(Self::from_be_bytes(buf)),
                }
            }
        }

        impl<const N: usize> Primitive for [$type; N] {
            fn read<const E: Endianness>(data: &mut impl Read) -> std::io::Result<Self> {
                let mut vals: [$type; N] = [<$type>::default(); N];
                for i in 0..N {
                    vals[i] = <$type>::read::<E>(data)?;
                }
                Ok(vals)
            }
        }
    };
}

impl_primitive_number!(u8);
impl_primitive_number!(u16);
impl_primitive_number!(u32);
impl_primitive_number!(u64);
impl_primitive_number!(i8);
impl_primitive_number!(i16);
impl_primitive_number!(i32);
impl_primitive_number!(i64);
impl_primitive_number!(f32);
impl_primitive_number!(f64);
