use indexmap::IndexMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write, Seek, BufRead};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::str;

pub fn convert_to_tsv(input_file: &str, output_file: &str) -> io::Result<()> {
    let mut fs = BufReader::new(File::open(input_file)?);

    fs.seek(std::io::SeekFrom::Current(4))?;
    let column_count = fs.read_u16::<LittleEndian>()?;
    let row_count = fs.read_u32::<LittleEndian>()?;

    let mut type_dictionary = IndexMap::new();

    type_dictionary.insert("_RowID".to_string(), 3);

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
            if col_name.contains("_RowID") {
                continue;
            }
            let value = match type_byte {
                1 => {
                    let length = fs.read_i16::<LittleEndian>()?;
                    if length > 0 {
                        let mut string_value = vec![0; length as usize];
                        fs.read_exact(&mut string_value)?;
                        String::from_utf8_lossy(&string_value).replace(",", "^").to_string()
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
    let file = File::open(input_file)?;
    let mut reader = BufReader::new(file);

    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    let fields: Vec<&str> = first_line.trim().split('\t').collect();
    let field_count = (fields.len() - 1) as u16;
    let row_lines: Vec<String> = reader.lines()
    .filter_map(|line| {
        match line {
            Ok(l) if !l.trim().is_empty() => Some(l),
            _ => None,
        }
    })
    .collect();

    let row_count = row_lines.len() as u32;

    let mut fs = BufWriter::new(File::create(output_file)?);

    fs.write_all(&[0; 4])?;
    fs.write_u16::<LittleEndian>(field_count)?;
    fs.write_u32::<LittleEndian>(row_count as u32)?;

    let mut field_types = Vec::with_capacity(fields.len());
    for field in fields.iter() {
        let parts: Vec<&str> = field.split('|').collect();
        if parts.len() == 2 {
            let field_name = parts[0];
            let field_type = parts[1].parse::<u8>().unwrap_or(0);
            if field_name.contains("RowID") { 
                field_types.push(field_type);
                continue 
            }; 
            field_types.push(field_type);

            let field_name_length = field_name.len() as u16;
            fs.write_u16::<LittleEndian>(field_name_length)?;
            fs.write_all(field_name.as_bytes())?;
            fs.write_u8(field_type)?;
        }
    }

    for row in row_lines.iter() {
        let row_fields: Vec<&str> = row.trim().split('\t').collect();
        for (index, value) in row_fields.iter().enumerate() {
            let field_type = *field_types.get(index).unwrap_or(&0);
            match field_type {
                1 => {
                    let encoded_value = if value.is_empty() || *value == "0.0" {
                        Vec::new()
                    } else if value.ends_with(".0") {
                        value[..value.len() - 2]
                            .replace("^", ",")
                            .as_bytes()
                            .to_vec()
                    } else {
                        value.replace("^", ",")
                            .as_bytes()
                            .to_vec()
                    };

                    fs.write_u16::<LittleEndian>(encoded_value.len() as u16)?;
                    fs.write_all(&encoded_value)?;
                }
                2 | 3 => {
                    let int_value: i32 = value.parse().unwrap_or(0);
                    fs.write_i32::<LittleEndian>(int_value)?;
                }
                4 | 5 => {
                    let float_value: f32 = value.parse().unwrap_or(0.0);
                    fs.write_f32::<LittleEndian>(float_value)?;
                }
                6 => {
                    let double_value: f64 = value.parse().unwrap_or(0.0);
                    fs.write_f64::<LittleEndian>(double_value)?;
                }
                _ => {
                    fs.write_u8(0)?;
                }
            }
        }
    }

    fs.write_all(&[5])?;
    fs.write_all(b"THEND")?;

    Ok(())
}