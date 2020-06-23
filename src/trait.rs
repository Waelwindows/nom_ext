use super::*;

pub trait ParseContext: Sized {
    type Context;

    fn parse(i: &[u8], ctx: Self::Context, endian: Endianness) -> IResult<&[u8], Self>;
}

pub trait ParseEndian: Sized {
    fn parse(i: &[u8], endian: Endianness) -> IResult<&[u8], Self>;
}

pub trait Parse: Sized {
    fn parse(i: &[u8]) -> IResult<&[u8], Self>;
}
