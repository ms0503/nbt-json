use crate::nbt::FileType;
use crate::nbt::TagType;
use crate::nbt::ANONYMOUS_KEY;
use bytes::BufMut;
use flate2::read::GzEncoder;
use flate2::Compression;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Read;

pub fn to_nbt<'a>(
    json: HashMap<Cow<'a, [u8]>, TagType<'a>>,
    nbt: &mut Vec<u8>,
    file_type: FileType,
) {
    let mut buf: Vec<u8> = vec![];
    for elem in json.iter() {
        walk(&mut buf, elem, false);
    }
    nbt.clear();
    match file_type {
        FileType::Raw => nbt.append(&mut buf),
        FileType::GZip => {
            let mut e = GzEncoder::new(&*buf, Compression::fast());
            e.read_to_end(nbt).unwrap();
        }
    }
}

fn walk<'a>(nbt: &mut Vec<u8>, elem: (&Cow<'a, [u8]>, &TagType<'a>), is_anonymous: bool) {
    let (k, v) = elem;
    match v {
        TagType::End => nbt.put_u8(0x00),
        TagType::Byte(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x01);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            nbt.put_i8(*v)
        }
        TagType::Short(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x02);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            nbt.put_i16(*v)
        }
        TagType::Int(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x03);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            nbt.put_i32(*v)
        }
        TagType::Long(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x04);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            nbt.put_i64(v.clone().into())
        }
        TagType::Float(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x05);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            nbt.put_f32(*v);
        }
        TagType::Double(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x06);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            nbt.put_f64(*v);
        }
        TagType::ByteArray(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x07);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            let len = v.len() as i32;
            nbt.put_i32(len);
            for v in v {
                nbt.put_i8(*v);
            }
        }
        TagType::String(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x08);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            let len = v.len() as u16;
            nbt.put_u16(len);
            for c in v.iter() {
                nbt.put_u8(*c);
            }
        }
        TagType::List(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x09);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            let tag_type = u8::from(&v[0]);
            let len = v.len() as i32;
            nbt.put_u8(tag_type);
            nbt.put_i32(len);
            for i in v.iter() {
                walk(nbt, (&ANONYMOUS_KEY, i), true);
            }
        }
        TagType::Compound(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x0a);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            for elem in v.iter() {
                walk(nbt, elem, false);
            }
            walk(nbt, (&ANONYMOUS_KEY, &TagType::End), false);
        }
        TagType::IntArray(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x0b);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            let len = v.len() as i32;
            nbt.put_i32(len);
            for i in v.iter() {
                walk(nbt, (&ANONYMOUS_KEY, &TagType::Int(*i)), true);
            }
        }
        TagType::LongArray(v) => {
            if !is_anonymous {
                let name_len = k.len() as u16;
                nbt.put_u8(0x0c);
                nbt.put_u16(name_len);
                nbt.put_slice(k);
            }
            let len = v.len() as i32;
            nbt.put_i32(len);
            for i in v.iter() {
                walk(nbt, (&ANONYMOUS_KEY, &TagType::Long(i.clone())), true);
            }
        }
    }
}
