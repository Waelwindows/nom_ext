use super::*;

use nom::number::Endianness;

use core::convert::TryFrom;
use core::convert::TryInto;

use std::io;

// type IResult<I, O, E=io::Error> = Result<(I, O), E>;

#[derive(Debug)]
pub struct ParsingContext<'a> {
    global: &'a [u8],
    endian: Endianness,
}

impl<'a> ParsingContext<'a> {
    pub fn read_at<F, O>(&self, offset: usize, f: F) -> IResult<&[u8], O>
    where
        F: Fn(&[u8]) -> IResult<&[u8], O>,
    {
        f(&self.global[offset..])
    }

    pub fn read_offset_then<P, U, F, O>(&'a self, p: P, f: F) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], O> + 'a
    where
        F: Fn(&[u8]) -> IResult<&[u8], O> + 'a,
        P: Fn(&[u8]) -> IResult<&[u8], U> + 'a,
        usize: TryFrom<U>,
        <usize as std::convert::TryFrom<U>>::Error: std::fmt::Debug,
    {
        move |i| {
            let (i, ptr) = p(i)?;
            let ptr = ptr.try_into().unwrap();
            let (_, val) = f(&self.global[ptr..])?;
            Ok((i, val))
        }
    }
    pub fn read_then<F, O>(&'a self, f: F) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], O> + 'a
    where
        F: Fn(&[u8]) -> IResult<&[u8], O> + 'a,
    {
        self.read_offset_then(le_u32, f)
    }
}