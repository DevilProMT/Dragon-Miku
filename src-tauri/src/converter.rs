use indexmap::IndexMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write, Seek};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::str;
use csv::ReaderBuilder;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn convert_to_tsv(input_file: &str, output_file: &str) -> io::Result<()> {
    let mut fs = BufReader::new(File::open(input_file)?);

    fs.seek(std::io::SeekFrom::Current(4))?;
    let column_count = fs.read_u16::<LittleEndian>()?;
    let row_count = fs.read_u32::<LittleEndian>()?;

    let mut type_dictionary = IndexMap::new();

    type_dictionary.insert("_RowID|3".to_string(), 3);

    for _ in 0..column_count {
        let length = fs.read_u16::<LittleEndian>()?;
        let mut name_bytes = vec![0; length as usize];
        fs.read_exact(&mut name_bytes)?;
        let name = str::from_utf8(&name_bytes).unwrap();
        let type_byte = fs.read_u8()?;
        type_dictionary.insert(name.to_string(), type_byte);
    }

    let mut output = BufWriter::new(File::create(output_file)?);

    let headers: Vec<String> = type_dictionary.iter()
    .map(|(name, type_byte)| format!("{}|{}", name, type_byte))
    .collect();
    writeln!(output, "{}", headers.join("\t"))?;

    for _ in 0..row_count {
        let num4 = fs.read_u32::<LittleEndian>()?;
        let mut row_data = Vec::with_capacity(type_dictionary.len());
        row_data.push(num4.to_string());

        for (col_name, type_byte) in &type_dictionary {
            if col_name == "_RowID|3" {
                continue;
            }
            let value = match type_byte {
                1 => {
                    let length = fs.read_i16::<LittleEndian>()?;
                    if length > 0 {
                        let mut string_value = vec![0; length as usize];
                        fs.read_exact(&mut string_value)?;
                        String::from_utf8_lossy(&string_value).to_string()
                    } else {
                        String::new()
                    }
                },
                2 | 3 => fs.read_i32::<LittleEndian>()?.to_string(),
                4 | 5 => fs.read_f32::<LittleEndian>()?.to_string(),
                6 => fs.read_f64::<LittleEndian>()?.to_string(),
                _ => String::new(),
            };
            row_data.push(value);
        }
        writeln!(output, "{}", row_data.join("\t"))?;
    }

    Ok(())
}

pub fn convert_to_dnt(input_file: &str, output_file: &str) -> io::Result<()> {
    let mut rdr = ReaderBuilder::new().delimiter(b'\t').from_path(input_file)?;
    let headers = rdr.headers()?.clone();
    let column_count = headers.len() - 1;
    let records: Vec<_> = rdr.records().collect::<Result<_, _>>()?;
    let row_count = records.len();

    let mut fs = BufWriter::new(File::create(output_file)?);
    fs.write_all(&[0; 4])?;

    let mut actual_column_count = column_count;
    for header in headers.iter() {
        if header.contains("64") {
            actual_column_count -= 1;
        }
    }

    fs.write_u16::<LittleEndian>(actual_column_count as u16)?;
    fs.write_u32::<LittleEndian>(row_count as u32)?;

    let mut type_dictionary = IndexMap::new();

    for header in headers.iter() {
        if header.contains("_RowID") || header.contains("64") {
            continue;
        }
        let parts: Vec<&str> = header.split('|').collect();
        let col_name = parts[0];
        let type_byte: u8 = parts[1].parse().unwrap();
        let name_length = col_name.len() as u16;
        fs.write_u16::<LittleEndian>(name_length)?;
        fs.write_all(col_name.as_bytes())?;
        fs.write_u8(type_byte)?;
        type_dictionary.insert(col_name.to_string(), type_byte);
    }

    let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::with_capacity(row_count * 1000)));

    records.par_iter().for_each(|record| {
        let row_id: i32 = record.get(0).unwrap().parse().unwrap();
        let mut local_buffer = Vec::with_capacity(1000);
        local_buffer.write_i32::<LittleEndian>(row_id).unwrap();
        for (col_name, type_byte) in &type_dictionary {
            let value = record.get(headers.iter().position(|h| h == format!("{}|{}", col_name, type_byte)).unwrap()).unwrap();
            match type_byte {
                1 => {
                    let encoded_value = if value.is_empty() || value == "0.0" {
                        Vec::new()
                    } else if value.ends_with(".0") {
                        value[..value.len() - 2].as_bytes().to_vec()
                    } else {
                        value.as_bytes().to_vec()
                    };
                    local_buffer.write_u16::<LittleEndian>(encoded_value.len() as u16).unwrap();
                    local_buffer.write_all(&encoded_value).unwrap();
                }
                2 | 3 => {
                    let int_value: i32 = value.parse().unwrap();
                    local_buffer.write_i32::<LittleEndian>(int_value).unwrap();
                }
                4..=6 => {
                    let float_value: f32 = value.parse().unwrap();
                    local_buffer.write_f32::<LittleEndian>(float_value).unwrap();
                }
                _ => {
                    local_buffer.write_u8(0).unwrap();
                }
            }
        }
        let mut buffer = buffer.lock().unwrap();
        buffer.extend_from_slice(&local_buffer);
    });

    let mut fs = fs;
    let buffer = buffer.lock().unwrap();
    fs.write_all(&buffer)?;
    fs.write_all(&[5])?;
    fs.write_all(b"THEND")?;
    Ok(())
}
