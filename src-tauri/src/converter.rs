use indexmap::IndexMap;
use std::fs::{File};
use std::io::{self, BufReader, BufWriter, Read, Write, Seek};
use byteorder::{LittleEndian, ReadBytesExt};
use std::str;

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
