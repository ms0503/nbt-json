use bytes::Buf;
use flate2::read::GzDecoder;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::BufRead;
use std::io::Read;

#[derive(Debug)]
pub enum FileType {
    Raw,
    GZip,
}

pub fn get_file_type(nbt: &[u8]) -> FileType {
    let mut nbt = &nbt[..];
    let mut first_word: Vec<u8> = vec![];
    let _ = nbt.read_until(2, &mut first_word);
    let first_word = (first_word[0] as u16) << 8 | first_word[1] as u16;
    match first_word {
        0x0a00 => FileType::Raw,
        0x1f8b => FileType::GZip,
        _ => unreachable!(),
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum TagType {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<TagType>),
    Compound(HashMap<String, TagType>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

pub fn nbt_to_json(nbt_file: &str, json: &mut HashMap<String, TagType>) {
    let nbt = {
        let file = fs::read(nbt_file).unwrap();
        let mut buf: Vec<u8> = vec![];
        match get_file_type(&file) {
            FileType::Raw => {
                buf = file.to_vec();
            }
            FileType::GZip => {
                let compressed_nbt = &file[..];
                let mut d = GzDecoder::new(compressed_nbt);
                d.read_to_end(&mut buf).unwrap();
            }
        }
        buf
    };
    let mut nbt = &nbt[..];
    let _ = nbt.get_u8();
    let _ = nbt.get_u16();
    loop {
        let tag_type = nbt.get_u8();
        let (name, value) = walk(&mut nbt, tag_type, false);
        if value == TagType::End {
            break;
        }
        json.insert(name, value);
    }
}

pub fn walk(nbt: &mut &[u8], tag_type: u8, is_anonymous: bool) -> (String, TagType) {
    match tag_type {
        // TAG_End
        0 => ("".to_string(), TagType::End),
        // TAG_Byte
        1 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let value = nbt.get_i8();
            (name, TagType::Byte(value))
        }
        // TAG_Short
        2 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let value = nbt.get_i16();
            (name, TagType::Short(value))
        }
        // TAG_Int
        3 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let value = nbt.get_i32();
            (name, TagType::Int(value))
        }
        // TAG_Long
        4 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let value = nbt.get_i64();
            (name, TagType::Long(value))
        }
        // TAG_Float
        5 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let value = nbt.get_f32();
            (name, TagType::Float(value))
        }
        // TAG_Double
        6 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let value = nbt.get_f64();
            (name, TagType::Double(value))
        }
        // TAG_Byte_Array
        7 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let len = nbt.get_i32();
            let mut buf: Vec<i8> = vec![];
            for _ in 0..len {
                buf.push(nbt.get_i8());
            }
            (name, TagType::ByteArray(buf))
        }
        // TAG_String
        8 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let len = nbt.get_u16();
            let mut buf: Vec<u8> = vec![];
            for _ in 0..len {
                buf.push(nbt.get_u8());
            }
            let string = String::from_utf8(buf).unwrap();
            (name, TagType::String(string))
        }
        // TAG_List
        9 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let item_type = nbt.get_u8();
            let len = nbt.get_i32();
            let mut buf: Vec<TagType> = vec![];
            for _ in 0..len {
                buf.push(walk(nbt, item_type, true).1);
            }
            (name, TagType::List(buf))
        }
        // TAG_Compound
        10 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let mut compound = HashMap::<String, TagType>::new();
            loop {
                let tag_type = nbt.get_u8();
                let (name, value) = walk(nbt, tag_type, false);
                if value == TagType::End {
                    break;
                }
                compound.insert(name, value);
            }
            (name, TagType::Compound(compound))
        }
        // TAG_Int_Array
        11 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let len = nbt.get_i32();
            let mut buf: Vec<i32> = vec![];
            for _ in 0..len {
                buf.push(nbt.get_i32());
            }
            (name, TagType::IntArray(buf))
        }
        // TAG_Long_Array
        12 => {
            let name = if is_anonymous {
                "".to_string()
            } else {
                get_name(nbt)
            };
            let len = nbt.get_i32();
            let mut buf: Vec<i64> = vec![];
            for _ in 0..len {
                buf.push(nbt.get_i64());
            }
            (name, TagType::LongArray(buf))
        }
        _ => unreachable!(),
    }
}

fn get_name(nbt: &mut &[u8]) -> String {
    let name_len = nbt.get_u16();
    let mut name: Vec<u8> = vec![];
    for _ in 0..name_len {
        name.push(nbt.get_u8());
    }
    String::from_utf8(name).unwrap()
}
