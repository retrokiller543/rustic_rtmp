// This file will handle encoding and decoding AMF data. It should define functions for encoding and decoding AMF data to and from bytes.

// Path: src/server/connection/message/amf.rs
use {
    super::{amf0_markers, errors::Amf0ReadErrorValue},
    crate::server::connection::message::amf0::{
        errors::Amf0ReadError, define::Amf0ValueType,
    },
    byteorder::BigEndian,
    bytesio::bytes_reader::BytesReader,
    indexmap::IndexMap,
    log,
};


pub struct Amf0Reader {
    reader: BytesReader,
}

impl Amf0Reader {
    pub fn new(reader: BytesReader) -> Self {
        Self { reader }
    }

    // Read all and call read_any for each
    pub fn read_all(&mut self) -> Result<Vec<Amf0ValueType>, Amf0ReadError> {
        let mut results = vec![];
        loop {
            let result = self.read_any()?;

            match result {
                Amf0ValueType::END => break,
                _ => results.push(result),
            }
        }
        Ok(results)
    }

    // Read any type of AMF0 value by calling correct method
    pub fn read_any(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        if self.reader.is_empty() {
            return Ok(Amf0ValueType::END);
        }

        let markers = self.reader.read_u8()?;

        if markers == amf0_markers::OBJECT_END {
            return Ok(Amf0ValueType::END);
        }

        match markers {
            amf0_markers::NUMBER => self.read_number(),
            amf0_markers::BOOLEAN => self.read_bool(),
            amf0_markers::STRING => self.read_string(),
            amf0_markers::OBJECT => self.read_object(),
            amf0_markers::NULL => self.read_null(),
            amf0_markers::ECMA_ARRAY => self.read_ecma_array(),
            amf0_markers::LONG_STRING => self.read_long_string(),
            _ => Err(Amf0ReadError {
                value: Amf0ReadErrorValue::UnknownMarker { marker: markers },
            }),
        }
    }

    pub fn read_with_type(&mut self, specified_marker: u8) -> Result<Amf0ValueType, Amf0ReadError> {
        let marker = self.reader.advance_u8()?;

        if marker != specified_marker {
            return Err(Amf0ReadError {
                value: Amf0ReadErrorValue::WrongType,
            });
        }

        self.read_any()
    }

    pub fn read_number(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        let number = self.reader.read_f64::<BigEndian>()?;
        let value = Amf0ValueType::Number(number);
        Ok(value)
    }

    pub fn read_bool(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        let value = self.reader.read_u8()?;

        match value {
            1 => Ok(Amf0ValueType::Boolean(true)),
            _ => Ok(Amf0ValueType::Boolean(false)),
        }
    }

    pub fn read_raw_string(&mut self) -> Result<String, Amf0ReadError> {
        let l = self.reader.read_u16::<BigEndian>()?;

        let bytes = self.reader.read_bytes(l as usize)?;
        let val = String::from_utf8(bytes.to_vec())?;

        Ok(val)
    }

    pub fn read_string(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        let raw_string = self.read_raw_string()?;
        Ok(Amf0ValueType::UTF8String(raw_string))
    }

    pub fn read_null(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        Ok(Amf0ValueType::Null)
    }

    pub fn is_read_object_eof(&mut self) -> Result<bool, Amf0ReadError> {
        let marker = self.reader.advance_u24::<BigEndian>()?;
        if marker == amf0_markers::OBJECT_END as u32 {
            self.reader.read_u24::<BigEndian>()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn read_object(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        let mut properties = IndexMap::new();

        loop {
            let is_eof = self.is_read_object_eof()?;

            if is_eof {
                break;
            }

            let key = self.read_raw_string()?;
            let val = self.read_any()?;

            properties.insert(key, val);
        }

        Ok(Amf0ValueType::Object(properties))
    }

    pub fn read_ecma_array(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        let len = self.reader.read_u32::<BigEndian>()?;

        let mut properties = IndexMap::new();

        //here we do not use length to traverse the map, because in some
        //other media server, the length is 0 which is not correct.
        while !self.is_read_object_eof()? {
            let key = self.read_raw_string()?;
            let val = self.read_any()?;
            properties.insert(key, val);
        }

        if len != properties.len() as u32 {
            log::warn!("the ecma array length is not correct!");
        }

        Ok(Amf0ValueType::Object(properties))
    }

    pub fn read_long_string(&mut self) -> Result<Amf0ValueType, Amf0ReadError> {
        let l = self.reader.read_u32::<BigEndian>()?;

        let buff = self.reader.read_bytes(l as usize)?;

        let val = String::from_utf8(buff.to_vec())?;
        Ok(Amf0ValueType::LongUTF8String(val))
    }
}