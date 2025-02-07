use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub fn convert_act_v6_to_v5(input_file: &str, output_base: &str) -> io::Result<bool> {
    let input_path = Path::new(input_file);
    let file_name = input_path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?;

    let file = File::open(input_file)?;
    let mut fs = BufReader::new(file);

    let mut header = [0u8; 32];
    fs.read_exact(&mut header)?;
    let version = fs.read_u32::<LittleEndian>()?;
    if version < 6 {
        return Ok(false);
    }

    let parent = input_path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid parent path"))?;
    let parent_str = parent.to_string_lossy();

    let mut output_path = PathBuf::from(output_base);

    if output_path.extension().is_some() {
        output_path.pop();
    };

    if let Some(pos) = parent_str.find("mapdata\\resource\\prop") {
        output_path.push(&parent_str[pos..]);
    } else if let Some(pos) = parent_str.find("resource") {
        if !parent_str[pos..].starts_with("resource\\prop") {
            output_path.push(&parent_str[pos..]);
        }
    }

    fs::create_dir_all(&output_path)?;

    output_path.push(file_name);
    output_path.set_extension("act");

    let action_count = fs.read_u32::<LittleEndian>()?;

    let mut output = BufWriter::new(File::create(&output_path)?);

    output.write_all(&header)?;
    output.write_u32::<LittleEndian>(5)?;
    output.write_u32::<LittleEndian>(action_count)?;

    for _ in 0..action_count {
        let _name = read_string(&mut fs, &mut output)?;
        let _link_ani_name = read_string(&mut fs, &mut output)?;
        let dw_length = fs.read_u32::<LittleEndian>()?;
        output.write_u32::<LittleEndian>(dw_length)?;
        let _next_action_name = read_string(&mut fs, &mut output)?;
        let dw_blend_frame = fs.read_u32::<LittleEndian>()?;
        output.write_u32::<LittleEndian>(dw_blend_frame)?;
        let dw_next_action_frame = fs.read_u32::<LittleEndian>()?;
        output.write_u32::<LittleEndian>(dw_next_action_frame)?;

        if version >= 2 {
            let unk_v2 = fs.read_u32::<LittleEndian>()?;
            output.write_u32::<LittleEndian>(unk_v2)?;
        }
        if version >= 3 {
            let unk_v3 = fs.read_u32::<LittleEndian>()?;
            output.write_u32::<LittleEndian>(unk_v3)?;
        }
        if version >= 4 {
            let unk_v4 = fs.read_u8()?;
            output.write_u8(unk_v4)?;
        }
        if version >= 5 {
            let unk_v5 = fs.read_u32::<LittleEndian>()?;
            output.write_u32::<LittleEndian>(unk_v5)?;
        }
        if version >= 6 {
            let _unk_v6 = fs.read_u8()?;
        }

        let signal_count = fs.read_u32::<LittleEndian>()?;
        output.write_u32::<LittleEndian>(signal_count)?;
        
        for _ in 0..signal_count {
            read_multiple_u32(&mut fs, &mut output, 4)?;
            let mut m_p_data = [0u8; 256];
            fs.read_exact(&mut m_p_data)?;
            output.write_all(&m_p_data)?;

            read_table(&mut fs, &mut output, 2)?;
            read_table(&mut fs, &mut output, 3)?;
            read_table(&mut fs, &mut output, 4)?;
            read_variable_size_table(&mut fs, &mut output)?;

            if version >= 6 {
                let _act6_signal = fs.read_u8()?; 
            }
        }
    }
    
    Ok(true)
}

fn read_string<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<String> {
    let len = reader.read_u32::<LittleEndian>()?;
    writer.write_u32::<LittleEndian>(len)?;
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    writer.write_all(&buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn read_multiple_u32<R: Read, W: Write>(reader: &mut R, writer: &mut W, count: usize) -> io::Result<()> {
    let mut buf = vec![0u8; count * 4];
    reader.read_exact(&mut buf)?;
    writer.write_all(&buf)?;
    Ok(())
}

fn read_table<R: Read, W: Write>(reader: &mut R, writer: &mut W, float_count: usize) -> io::Result<()> {
    let n_table_count = reader.read_u32::<LittleEndian>()?;
    writer.write_u32::<LittleEndian>(n_table_count)?;
    if n_table_count > 0 {
        read_multiple_u32(reader, writer, n_table_count as usize)?;
        for _ in 0..n_table_count {
            for _ in 0..float_count {
                let val = reader.read_f32::<LittleEndian>()?;
                writer.write_f32::<LittleEndian>(val)?;
            }
        }
    }
    Ok(())
}

fn read_variable_size_table<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let n_table_count = reader.read_u32::<LittleEndian>()?;
    writer.write_u32::<LittleEndian>(n_table_count)?;
    if n_table_count > 0 {
        read_multiple_u32(reader, writer, n_table_count as usize)?;
        for _ in 0..n_table_count {
            let size = reader.read_u32::<LittleEndian>()?;
            writer.write_u32::<LittleEndian>(size)?;
            if let Ok(size_usize) = usize::try_from(size) {
                let mut data = vec![0; size_usize];
                reader.read_exact(&mut data)?;
                writer.write_all(&data)?;
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Size too large"));
            }
        }
    }
    Ok(())
}
