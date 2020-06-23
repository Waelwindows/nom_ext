use nom::bytes::complete::*;
use nom::combinator::map;
use nom::multi::count;
use nom::number::complete::*;
use nom::number::Endianness;
use nom::IResult;
use nom::{map, u32};

use core::convert::TryInto;
use std::borrow::Cow;

pub mod r#trait;
// mod global;

pub fn usize<'a, F, O, E>(f: F) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], usize, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
    O: TryInto<usize>,
    E: ParseError<&'a [u8]>,
{
    map(f, |v| v.try_into().ok().unwrap())
}

pub fn u16<'a, E: ParseError<&'a [u8]>>(
    endian: Endianness,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], u16, E> {
    match endian {
        Endianness::Little => le_u16,
        _ => be_u16,
    }
}
pub fn u32<'a, E: ParseError<&'a [u8]>>(
    endian: Endianness,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], u32, E> {
    match endian {
        Endianness::Little => le_u32,
        _ => be_u32,
    }
}
pub fn i32<'a, E: ParseError<&'a [u8]>>(
    endian: Endianness,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], i32, E> {
    match endian {
        Endianness::Little => le_i32,
        _ => be_i32,
    }
}

pub fn u32_usize<'a>(
    endian: Endianness,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], usize> {
    usize(u32(endian))
}

use nom::{InputIter, InputTake};
pub fn at_offset<I, O, F>(offset: usize, f: F) -> impl Fn(I) -> IResult<I, O>
where
    I: InputIter + InputTake + Clone,
    F: Fn(I) -> IResult<I, O>,
{
    use nom::bytes::complete::*;
    move |i: I| {
        let (i0, _) = take(offset)(i.clone())?;
        let (_, v) = f(i0)?;
        Ok((i, v))
    }
}

// read_at_offset2(256)(string)(i0)
// read_at_offset(i0)(string)(256)
// read_at_offset(i0)(256)(string)
// read_at_offset(le_u32(i)?.1, string)(i0)
// read_offset_at(le_u32, string)(i0)
// read_offset_at(u32(endian), string)(i0)

pub fn offset_read_then<'a, O, F, F1, U>(
    i0: &'a [u8],
    f1: F1,
    f: F,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], O>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O>,
    F1: Fn(&'a [u8]) -> IResult<&'a [u8], U>,
    U: TryInto<usize>,
{
    move |i: &'a [u8]| {
        let (i1, offset) = f1(i)?;
        let offset = offset.try_into().ok().unwrap();
        let f0 = |x| f(x);
        let (_, v) = at_offset(offset, f0)(i0)?;
        Ok((i1, v))
    }
}

pub fn offset_then<'a, O, F>(
    i0: &'a [u8],
    f: F,
    endian: Endianness,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], O>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O>,
{
    offset_read_then(i0, u32_usize(endian), f)
}

pub fn offset_read_table<'a, O, F, F1, U>(
    i0: &'a [u8],
    f1: F1,
    f: F,
    cnt: usize,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<O>>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O>,
    F1: Fn(&'a [u8]) -> IResult<&'a [u8], U>,
    U: TryInto<usize>,
{
    move |i: &[u8]| {
        // let (i1, offset) = f1(i)?;
        let f1 = |x| f1(x);
        let (i1, offsets) = count(usize(f1), cnt)(i)?;
        let f0 = |x| f(x);
        let mut res = vec![];
        for offset in offsets {
            let (_, val) = at_offset(offset, f0)(i0)?;
            res.push(val);
        }
        Ok((i1, res))
    }
}

pub fn offset_table<'a, O, F>(
    i0: &'a [u8],
    f: F,
    cnt: usize,
    endian: Endianness,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<O>>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O>,
{
    offset_read_table(i0, u32(endian), f, cnt)
}

pub fn count_then_offset<'a, O, F, F1, U>(
    i0: &'a [u8],
    f1: F1,
    f: F,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<O>>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O>,
    F1: Fn(&'a [u8]) -> IResult<&'a [u8], U>,
    U: TryInto<usize>,
{
    move |i: &[u8]| {
        let (i1, cnt) = f1(i)?;
        let cnt = cnt.try_into().ok().unwrap();
        let f1 = |x| f1(x);
        let f = |x| f(x);
        let (i1, v) = offset_read_then(i0, f1, count(f, cnt))(i1)?;
        Ok((i1, v))
    }
}

// fn test() -> Result<(), Box<dyn std::error::Error>> {
//     let i = &[0u8][..];
//     let endian = Endianness::Little;
//     let read_at = |f1: _, f: _| offset_then(i, f1, f);
//     let (o, t) = offset_then(i, u32_usize(endian), string)(i).unwrap();
//     let (o, t) = read_at(le_u32, string)(i)?;
//     Ok(())
// }

///string terminated
pub fn string<'a>(i: &'a [u8]) -> IResult<&'a [u8], Cow<'a, str>> {
    map(take_until("\0"), String::from_utf8_lossy)(i).map(|(i, v)| (&i[1..], v))
}

pub fn offset_read_string<'a, F, U>(
    i0: &'a [u8],
    f: F,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Cow<'a, str>>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], U>,
    U: TryInto<usize>,
{
    offset_read_then(i0, f, string)
}

pub fn offset_string<'a>(
    i0: &'a [u8],
    endian: Endianness,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Cow<'a, str>> {
    offset_then(i0, string, endian)
}

use nom::error::ParseError;
// #[cfg(feature = "alloc")]
pub fn many_until<I, O, E, F>(f: F, v: O) -> impl Fn(I) -> IResult<I, Vec<O>, E>
where
    I: Clone,
    O: PartialEq,
    F: Fn(I) -> IResult<I, O, E>,
    E: ParseError<I>,
{
    move |i: I| {
        let mut res = Vec::new();
        let mut i = i.clone();
        loop {
            let (i1, val) = f(i.clone())?;
            i = i1;
            if val == v {
                break;
            } else {
                res.push(val);
            }
        }
        Ok((i, res))
    }
}

pub fn many_until_nth<I, O, E, F>(
    f: F,
    v: O,
    occurance: usize,
) -> impl Fn(I) -> IResult<I, Vec<O>, E>
where
    I: Clone,
    O: PartialEq,
    F: Fn(I) -> IResult<I, O, E>,
    E: ParseError<I>,
{
    move |i: I| {
        let mut res = Vec::new();
        let mut i = i.clone();
        let mut o = 1;
        loop {
            let (i1, val) = f(i.clone())?;
            i = i1;
            if val == v {
                if o <= occurance {
                    o += 1;
                    res.push(val);
                } else {
                    break;
                }
            } else {
                res.push(val);
            }
        }
        Ok((i, res))
    }
}
