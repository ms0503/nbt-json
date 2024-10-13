use super::TagResolvingError;
use super::ANONYMOUS_KEY;
use super::{LongNumber, TagType};
use serde_json::Map;
use serde_json::Number;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

fn resolve_byte<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &Number,
) -> Result<(), TagResolvingError> {
    let v = v.as_i64();
    match v {
        Some(v) => {
            if !(-0x80..0x80).contains(&v) {
                return Err(TagResolvingError::UnsupportedValue);
            }
            nbt.insert(name, TagType::Byte(v as i8));
        }
        None => return Err(TagResolvingError::UnsupportedValue),
    }
    Ok(())
}

fn resolve_short<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &Number,
) -> Result<(), TagResolvingError> {
    let v = v.as_i64();
    match v {
        Some(v) => {
            if !(-0x8000..0x8000).contains(&v) {
                return Err(TagResolvingError::UnsupportedValue);
            }
            nbt.insert(name, TagType::Short(v as i16));
        }
        None => return Err(TagResolvingError::UnsupportedValue),
    }
    Ok(())
}

fn resolve_int<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &Number,
) -> Result<(), TagResolvingError> {
    let v = v.as_i64();
    match v {
        Some(v) => {
            if !(-0x80000000..0x80000000).contains(&v) {
                return Err(TagResolvingError::UnsupportedValue);
            }
            nbt.insert(name, TagType::Int(v as i32));
        }
        None => return Err(TagResolvingError::UnsupportedValue),
    }
    Ok(())
}

fn resolve_long<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &Number,
) -> Result<(), TagResolvingError> {
    let v = v.as_i64();
    match v {
        Some(v) => {
            nbt.insert(name, TagType::Long(LongNumber::from(v)));
        }
        None => return Err(TagResolvingError::UnsupportedValue),
    }
    Ok(())
}

fn resolve_float<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &Number,
) -> Result<(), TagResolvingError> {
    let v = v.as_f64();
    match v {
        Some(v) => {
            nbt.insert(name, TagType::Float(v as f32));
        }
        None => return Err(TagResolvingError::UnsupportedValue),
    }
    Ok(())
}

fn resolve_double<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &Number,
) -> Result<(), TagResolvingError> {
    let v = v.as_f64();
    match v {
        Some(v) => {
            nbt.insert(name, TagType::Double(v));
        }
        None => return Err(TagResolvingError::UnsupportedValue),
    }
    Ok(())
}

fn resolve_byte_array<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &[Value],
) -> Result<(), TagResolvingError> {
    let mut buf = vec![];
    for v in v.iter() {
        let mut buf1 = HashMap::new();
        match v {
            Value::Number(v) => resolve_byte(&mut buf1, cesu8::to_java_cesu8(""), v)?,
            _ => return Err(TagResolvingError::UnsupportedValue),
        }
        match buf1.values().collect::<Vec<_>>()[0] {
            TagType::Byte(v) => buf.push(*v),
            _ => return Err(TagResolvingError::UnsupportedValue),
        }
    }
    nbt.insert(name, TagType::ByteArray(buf));
    Ok(())
}

fn resolve_string<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &'a str,
) -> Result<(), TagResolvingError> {
    nbt.insert(name, TagType::String(cesu8::to_java_cesu8(v)));
    Ok(())
}

fn resolve_list<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &'a [Value],
    v_type: String,
) -> Result<(), TagResolvingError> {
    let mut buf = vec![];
    for v in v.iter() {
        let mut buf1 = HashMap::new();
        match v_type {
            ref v_type if v_type.starts_with("TAG_Byte") => match v {
                Value::Number(v) => resolve_byte(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Short") => match v {
                Value::Number(v) => resolve_short(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Int") => match v {
                Value::Number(v) => resolve_int(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Long") => match v {
                Value::Number(v) => resolve_long(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Float") => match v {
                Value::Number(v) => resolve_float(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Double") => match v {
                Value::Number(v) => resolve_double(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Byte_Array") => match v {
                Value::Array(v) => resolve_byte_array(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_String") => match v {
                Value::String(v) => resolve_string(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_List") => match v {
                Value::Array(v) => {
                    let v_type = v_type.split(";").collect::<Vec<_>>();
                    if v_type.len() == 1 {
                        return Err(TagResolvingError::UnsupportedValue);
                    } else {
                        let v_type = v_type[1].to_string();
                        resolve_list(&mut buf1, ANONYMOUS_KEY.clone(), v, v_type)?;
                    }
                }
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Compound") => match v {
                Value::Object(v) => resolve_compound(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Int_Array") => match v {
                Value::Array(v) => resolve_int_array(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            ref v_type if v_type.starts_with("TAG_Long_Array") => match v {
                Value::Array(v) => resolve_long_array(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
                _ => return Err(TagResolvingError::UnsupportedValue),
            },
            _ => return Err(TagResolvingError::UnsupportedValue),
        }
        buf.append(&mut buf1.into_values().collect());
    }
    nbt.insert(name, TagType::List(buf));
    Ok(())
}

pub fn resolve_compound<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &'a Map<String, Value>,
) -> Result<(), TagResolvingError> {
    let mut buf = HashMap::new();
    for (k, v) in v.iter() {
        match v {
            Value::Null => return Err(TagResolvingError::UnsupportedValue),
            Value::Bool(v) => {
                let name = cesu8::to_java_cesu8(k);
                resolve_byte(&mut buf, name, &Number::from(if *v { 1 } else { 0 }))?;
            }
            Value::Number(v) => {
                if !k.contains(";") {
                    let name = cesu8::to_java_cesu8(k);
                    if v.is_f64() {
                        resolve_double(&mut buf, name, v)?;
                    } else {
                        resolve_int(&mut buf, name, v)?;
                    }
                } else {
                    let name = cesu8::to_java_cesu8(&k[..k.len() - 2]);
                    match k {
                        k if k.ends_with(";B") => resolve_byte(&mut buf, name, v)?,
                        k if k.ends_with(";S") => resolve_short(&mut buf, name, v)?,
                        k if k.ends_with(";L") => resolve_long(&mut buf, name, v)?,
                        k if k.ends_with(";F") => resolve_float(&mut buf, name, v)?,
                        _ => return Err(TagResolvingError::UnsupportedValue),
                    }
                }
            }
            Value::String(v) => {
                let name = cesu8::to_java_cesu8(k);
                resolve_string(&mut buf, name, v)?;
            }
            Value::Array(v) => {
                let name = k.split(";").collect::<Vec<_>>();
                if name.len() == 1 {
                    return Err(TagResolvingError::UnsupportedValue);
                } else {
                    let v_type = name[1..].join(";");
                    let name = cesu8::to_java_cesu8(name[0]);
                    match v_type {
                        v_type if v_type.starts_with("B") => resolve_byte_array(&mut buf, name, v)?,
                        v_type if v_type.starts_with("I") => resolve_int_array(&mut buf, name, v)?,
                        v_type if v_type.starts_with("L") => resolve_long_array(&mut buf, name, v)?,
                        v_type if v_type.starts_with("TAG_") => {
                            resolve_list(&mut buf, name, v, v_type)?
                        }
                        _ => return Err(TagResolvingError::UnsupportedValue),
                    }
                }
            }
            Value::Object(v) => {
                let name = cesu8::to_java_cesu8(k);
                resolve_compound(&mut buf, name, v)?;
            }
        }
    }
    nbt.insert(name, TagType::Compound(buf));
    Ok(())
}

fn resolve_int_array<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &'a [Value],
) -> Result<(), TagResolvingError> {
    let mut buf = vec![];
    for v in v.iter() {
        let mut buf1 = HashMap::new();
        match v {
            Value::Number(v) => resolve_int(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
            _ => return Err(TagResolvingError::UnsupportedValue),
        }
        for v in buf1.values() {
            match v {
                TagType::Int(v) => buf.push(*v),
                _ => return Err(TagResolvingError::UnsupportedValue),
            }
        }
    }
    nbt.insert(name, TagType::IntArray(buf));
    Ok(())
}

fn resolve_long_array<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    name: Cow<'a, [u8]>,
    v: &'a [Value],
) -> Result<(), TagResolvingError> {
    let mut buf = vec![];
    for v in v.iter() {
        let mut buf1 = HashMap::new();
        match v {
            Value::Number(v) => resolve_long(&mut buf1, ANONYMOUS_KEY.clone(), v)?,
            _ => return Err(TagResolvingError::UnsupportedValue),
        }
        for v in buf1.values() {
            match v {
                TagType::Long(v) => buf.push(v.clone()),
                _ => return Err(TagResolvingError::UnsupportedValue),
            }
        }
    }
    nbt.insert(name, TagType::LongArray(buf));
    Ok(())
}
