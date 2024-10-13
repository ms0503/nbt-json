use bytes::Buf;
use clap::ValueEnum;
use flate2::read::GzDecoder;
use serde_json::Map;
use serde_json::Number;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::BufRead;
use std::io::Read;
use std::str::FromStr;
use std::sync::LazyLock;

mod resolver;

pub(crate) static ANONYMOUS_KEY: LazyLock<Cow<[u8]>> = LazyLock::new(|| cesu8::to_java_cesu8(""));

const JS_MAX_SAFE_INTEGER: i64 = 2i64.pow(53) - 1;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LongNumber {
    Number(i64),
    BigInt(String),
}

impl From<i64> for LongNumber {
    fn from(value: i64) -> Self {
        match value {
            ..=JS_MAX_SAFE_INTEGER => Self::Number(value),
            _ => Self::BigInt(value.to_string()),
        }
    }
}

impl From<LongNumber> for i64 {
    fn from(value: LongNumber) -> Self {
        match value {
            LongNumber::Number(v) => v,
            LongNumber::BigInt(v) => i64::from_str(&v).unwrap(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TagType<'a> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(LongNumber),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(Cow<'a, [u8]>),
    List(Vec<TagType<'a>>),
    Compound(HashMap<Cow<'a, [u8]>, TagType<'a>>),
    IntArray(Vec<i32>),
    LongArray(Vec<LongNumber>),
}

impl TagType<'_> {
    pub fn u8_to_str<'a>(tag_type: u8) -> Result<&'a str, &'a str> {
        match tag_type {
            0x00 => Ok("TAG_End"),
            0x01 => Ok("TAG_Byte"),
            0x02 => Ok("TAG_Short"),
            0x03 => Ok("TAG_Int"),
            0x04 => Ok("TAG_Long"),
            0x05 => Ok("TAG_Float"),
            0x06 => Ok("TAG_Double"),
            0x07 => Ok("TAG_Byte_Array"),
            0x08 => Ok("TAG_String"),
            0x09 => Ok("TAG_List"),
            0x0a => Ok("TAG_Compound"),
            0x0b => Ok("TAG_Int_Array"),
            0x0c => Ok("TAG_Long_Array"),
            _ => Err("Unknown tag type"),
        }
    }

    pub fn str_to_u8(tag_type: &str) -> Result<u8, &str> {
        match tag_type {
            "TAG_End" => Ok(0x00),
            "TAG_Byte" => Ok(0x01),
            "TAG_Short" => Ok(0x02),
            "TAG_Int" => Ok(0x03),
            "TAG_Long" => Ok(0x04),
            "TAG_Float" => Ok(0x05),
            "TAG_Double" => Ok(0x06),
            "TAG_Byte_Array" => Ok(0x07),
            "TAG_String" => Ok(0x08),
            "TAG_List" => Ok(0x09),
            "TAG_Compound" => Ok(0x0a),
            "TAG_Int_Array" => Ok(0x0b),
            "TAG_Long_Array" => Ok(0x0c),
            _ => Err("Unknown tag type"),
        }
    }
}

impl From<&TagType<'_>> for u8 {
    fn from(value: &TagType) -> Self {
        match value {
            TagType::End => 0x00,
            TagType::Byte(_) => 0x01,
            TagType::Short(_) => 0x02,
            TagType::Int(_) => 0x03,
            TagType::Long(_) => 0x04,
            TagType::Float(_) => 0x05,
            TagType::Double(_) => 0x06,
            TagType::ByteArray(_) => 0x07,
            TagType::String(_) => 0x08,
            TagType::List(_) => 0x09,
            TagType::Compound(_) => 0x0a,
            TagType::IntArray(_) => 0x0b,
            TagType::LongArray(_) => 0x0c,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TagResolvingError {
    UnsupportedValue,
}

impl Display for TagResolvingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TagResolvingError::UnsupportedValue => write!(f, "Unsupported value."),
        }
    }
}

impl Error for TagResolvingError {}

pub fn resolve_tag_types<'a>(
    nbt: &mut HashMap<Cow<'a, [u8]>, TagType<'a>>,
    json: &'a Map<String, Value>,
) -> Result<(), TagResolvingError> {
    resolver::resolve_compound(nbt, ANONYMOUS_KEY.clone(), json)?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum FileType {
    Raw,
    #[value(alias = "gz")]
    GZip,
}

pub fn get_file_type(nbt: &[u8]) -> FileType {
    let mut nbt = nbt;
    let mut first_word = vec![];
    let _ = nbt.read_until(2, &mut first_word);
    let first_word = (first_word[0] as u16) << 8 | first_word[1] as u16;
    match first_word {
        0x0a00 => FileType::Raw,
        0x1f8b => FileType::GZip,
        _ => unreachable!(),
    }
}

pub fn to_json(nbt: Vec<u8>, json: &mut HashMap<String, Value>) {
    let nbt = {
        let mut buf = vec![];
        match get_file_type(&nbt) {
            FileType::Raw => {
                buf = nbt;
            }
            FileType::GZip => {
                let compressed_nbt = &nbt[..];
                let mut d = GzDecoder::new(compressed_nbt);
                d.read_to_end(&mut buf).unwrap();
            }
        }
        buf
    };
    let mut nbt = &nbt[..];
    let _ = nbt.get_u8(); // 0x0a   - Tag Type: TAG_Compound
    let _ = nbt.get_u16(); // 0x0000 - Name Length: 0
    loop {
        let tag_type = nbt.get_u8();
        if walk(json, &mut nbt, tag_type, false) {
            break;
        }
    }
}

// Return: is_end
fn walk(
    json: &mut HashMap<String, Value>,
    nbt: &mut &[u8],
    tag_type: u8,
    is_anonymous: bool,
) -> bool {
    match tag_type {
        // TAG_End
        0x00 => return true,
        // TAG_Byte
        0x01 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                format!(
                    "{};B",
                    cesu8::from_java_cesu8(get_name(nbt).as_ref()).unwrap()
                )
            };
            let value = nbt.get_i8();
            json.insert(name, Value::Number(Number::from(value)));
        }
        // TAG_Short
        0x02 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                format!(
                    "{};S",
                    cesu8::from_java_cesu8(get_name(nbt).as_ref()).unwrap()
                )
            };
            let value = nbt.get_i16();
            json.insert(name, Value::Number(Number::from(value)));
        }
        // TAG_Int
        0x03 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                cesu8::from_java_cesu8(get_name(nbt).as_ref())
                    .unwrap()
                    .to_string()
            };
            let value = nbt.get_i32();
            json.insert(name, Value::Number(Number::from(value)));
        }
        // TAG_Long
        0x04 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                format!(
                    "{};L",
                    cesu8::from_java_cesu8(get_name(nbt).as_ref()).unwrap()
                )
            };
            let value = nbt.get_i64();
            let value = LongNumber::from(value);
            match value {
                LongNumber::Number(v) => {
                    json.insert(name, Value::Number(Number::from(v)));
                }
                LongNumber::BigInt(v) => {
                    json.insert(name, Value::String(v));
                }
            }
        }
        // TAG_Float
        0x05 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                format!(
                    "{};F",
                    cesu8::from_java_cesu8(get_name(nbt).as_ref()).unwrap()
                )
            };
            let value = nbt.get_f32();
            json.insert(name, Value::Number(Number::from_f64(value as f64).unwrap()));
        }
        // TAG_Double
        0x06 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                cesu8::from_java_cesu8(get_name(nbt).as_ref())
                    .unwrap()
                    .to_string()
            };
            let value = nbt.get_f64();
            json.insert(name, Value::Number(Number::from_f64(value).unwrap()));
        }
        // TAG_Byte_Array
        0x07 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                format!(
                    "{};B",
                    cesu8::from_java_cesu8(get_name(nbt).as_ref()).unwrap()
                )
            };
            let len = nbt.get_i32();
            let mut buf = vec![];
            for _ in 0..len {
                buf.push(Value::Number(Number::from(nbt.get_i8())));
            }
            json.insert(name, Value::Array(buf));
        }
        // TAG_String
        0x08 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                cesu8::from_java_cesu8(get_name(nbt).as_ref())
                    .unwrap()
                    .to_string()
            };
            let len = nbt.get_u16();
            let mut buf = vec![];
            for _ in 0..len {
                buf.push(nbt.get_u8());
            }
            let string = cesu8::from_java_cesu8(buf.as_ref()).unwrap().to_string();
            json.insert(name, Value::String(string));
        }
        // TAG_List
        0x09 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                cesu8::from_java_cesu8(get_name(nbt).as_ref())
                    .unwrap()
                    .to_string()
            };
            let item_type = nbt.get_u8();
            let name = if is_anonymous {
                name
            } else {
                let tag_type = TagType::u8_to_str(item_type).expect("Unknown tag type.");
                format!("{};{}", name, tag_type)
            };
            let len = nbt.get_i32();
            let mut buf = vec![];
            for _ in 0..len {
                let mut buf1 = HashMap::new();
                walk(&mut buf1, nbt, item_type, true);
                buf.extend(buf1.into_values());
            }
            json.insert(name, Value::Array(buf));
        }
        // TAG_Compound
        0x0a => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                cesu8::from_java_cesu8(get_name(nbt).as_ref())
                    .unwrap()
                    .to_string()
            };
            let mut buf = HashMap::new();
            loop {
                let tag_type = nbt.get_u8();
                if walk(&mut buf, nbt, tag_type, false) {
                    break;
                }
            }
            json.insert(name, Value::Object(Map::from_iter(buf)));
        }
        // TAG_Int_Array
        0x0b => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                format!(
                    "{};I",
                    cesu8::from_java_cesu8(get_name(nbt).as_ref()).unwrap()
                )
            };
            let len = nbt.get_i32();
            let mut buf = vec![];
            for _ in 0..len {
                buf.push(Value::Number(Number::from(nbt.get_i32())));
            }
            json.insert(name, Value::Array(buf));
        }
        // TAG_Long_Array
        0x0c => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                format!(
                    "{};L",
                    cesu8::from_java_cesu8(get_name(nbt).as_ref()).unwrap()
                )
            };
            let len = nbt.get_i32();
            let mut buf = vec![];
            for _ in 0..len {
                let v = LongNumber::from(nbt.get_i64());
                match v {
                    LongNumber::Number(v) => {
                        buf.push(Value::Number(Number::from(v)));
                    }
                    LongNumber::BigInt(v) => {
                        buf.push(Value::String(v));
                    }
                }
            }
            json.insert(name, Value::Array(buf));
        }
        _ => unreachable!(),
    }
    false
}

fn get_name(nbt: &mut &[u8]) -> String {
    let name_len = nbt.get_u16();
    let mut name: Vec<u8> = vec![];
    for _ in 0..name_len {
        name.push(nbt.get_u8());
    }
    String::from_utf8(name).unwrap()
}
