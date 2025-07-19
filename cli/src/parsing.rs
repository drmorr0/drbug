use anyhow::{
    anyhow,
    bail,
};
use libdrbug::prelude::*;
use paste::paste;

pub fn parse_for_register(info: &RegisterInfo, val_str: &str) -> anyhow::Result<RegisterValue> {
    match info.format {
        RegisterFormat::Uint => parse_uint(val_str, info.size),
        RegisterFormat::DoubleFloat => parse_double(val_str),
        RegisterFormat::LongDouble => parse_long_double(val_str),
        RegisterFormat::Vector => parse_vector(val_str, info.size),
    }
}

macro_rules! make_parse_uint {
    ($t:ty) => {
        paste! {
            fn [<parse_ $t>](input: &str) -> anyhow::Result<$t> {
                let trimmed_input = input.trim();
                let (radix, start_idx) = match trimmed_input.get(..2) {
                    Some("0x") | Some("0X") => (16, 2),
                    Some("0o") | Some("0O") => (8, 2),
                    Some("0b") | Some("0B") => (2, 2),
                    _ => (10, 0),
                };

                let val_str = trimmed_input.get(start_idx..).ok_or(anyhow!("parse error: {}", trimmed_input))?.trim();
                $t::from_str_radix(val_str, radix).map_err(|e| e.into())
            }
        }
    };
}

make_parse_uint!(u8);
make_parse_uint!(u16);
make_parse_uint!(u32);
make_parse_uint!(u64);

pub(crate) fn parse_uint(input: &str, size: usize) -> anyhow::Result<RegisterValue> {
    Ok(match size {
        1 => RegisterValue::U8(parse_u8(input)?),
        2 => RegisterValue::U16(parse_u16(input)?),
        4 => RegisterValue::U32(parse_u32(input)?),
        8 => RegisterValue::U64(parse_u64(input)?),
        _ => bail!("invalid register size: {size}"),
    })
}

pub(crate) fn parse_double(input: &str) -> anyhow::Result<RegisterValue> {
    Ok(RegisterValue::F64(input.trim().parse::<f64>()?))
}

pub(crate) fn parse_long_double(_input: &str) -> anyhow::Result<RegisterValue> {
    bail!("long double unsupported");
}

pub(crate) fn parse_vector(input: &str, size: usize) -> anyhow::Result<RegisterValue> {
    let trimmed_input = input.trim();
    let len = trimmed_input.len();
    if trimmed_input.get(..1) != Some("[") || trimmed_input.get(len - 1..) != Some("]") {
        bail!("missing opening/closing brackets");
    }

    let bytes = trimmed_input
        .get(1..len - 1)
        .ok_or(anyhow!("parse error: {}", trimmed_input))?
        .split(",")
        .map(parse_u8)
        .collect::<anyhow::Result<Vec<u8>>>()?;

    Ok(match size {
        8 => RegisterValue::B64(
            bytes
                .try_into()
                .map_err(|v: Vec<u8>| anyhow!("incorrect size for vector register: {}", v.len()))?,
        ),
        16 => RegisterValue::B128(
            bytes
                .try_into()
                .map_err(|v: Vec<u8>| anyhow!("incorrect size for vector register: {}", v.len()))?,
        ),
        _ => bail!("invalid register size: {size}"),
    })
}

#[cfg(test)]
mod tests {
    use assertables::*;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("0x2a", 1, RegisterValue::U8(42))]
    #[case(" 0x2a  ", 1, RegisterValue::U8(42))]
    #[case("0o52", 1, RegisterValue::U8(42))]
    #[case("0b00101010", 1, RegisterValue::U8(42))]
    #[case("0x2a2a", 2, RegisterValue::U16(10794))]
    #[case("0x2a2a2a2a", 4, RegisterValue::U32(707406378))]
    #[case("0x2a2a2a2a2a2a2a2a", 8, RegisterValue::U64(3038287259199220266))]
    fn test_parse_uint(#[case] input: &str, #[case] size: usize, #[case] expected: RegisterValue) {
        assert_eq!(parse_uint(input, size).unwrap(), expected);
    }

    #[rstest]
    #[case("0x2a2a", 1)]
    #[case("0x", 1)]
    #[case("0xzzzz", 1)]
    #[case("0x2a", 5)]
    fn test_parse_uint_fails(#[case] input: &str, #[case] size: usize) {
        assert_err!(parse_uint(input, size));
    }

    #[rstest]
    #[case("[0, 0b1, 2, 3, 0o4, 0x5, 6, 7]", 8, RegisterValue::B64([0, 1, 2, 3, 4, 5, 6, 7]))]
    #[case(
        "[0, 0b1, 2, 3, 0o4, 0x5, 6, 7, 8, 9, 0x0a, 11, 12, 13, 14, 15]", 16,
        RegisterValue::B128([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
    )]
    fn test_parse_vector(#[case] input: &str, #[case] size: usize, #[case] expected: RegisterValue) {
        assert_eq!(parse_vector(input, size).unwrap(), expected);
    }
}
