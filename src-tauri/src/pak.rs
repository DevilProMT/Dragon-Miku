use std::fs::{self, File};
use std::io::{self, Read, Write, Seek, SeekFrom, Cursor, BufRead, BufReader, BufWriter};
use std::path::PathBuf;
use aes::Aes256;
use cipher::{KeyInit, BlockDecryptMut, generic_array::GenericArray};
use cipher::block_padding::Pkcs7;
use ecb::Decryptor;
use flate2::read::ZlibDecoder;
use rayon::prelude::*;

type Aes256Ecb = Decryptor<Aes256>;

fn load_keys(filename: &str) -> io::Result<Vec<Vec<u8>>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    
    let mut keys = Vec::new();
    for line in reader.lines() {
        if let Ok(mut key) = line {
            key = key.trim().to_string();
            if key.len() == 31 {
                key = key.trim_end().to_string();
                let mut key_bytes = key.into_bytes();
                key_bytes.push(0);
                keys.push(key_bytes);
            }
        }
    }
    Ok(keys)
}

fn decrypt_with_keys(compressed_data: &[u8], keys: &[Vec<u8>]) -> Option<Vec<u8>> {
    for key_bytes in keys {
        let key = GenericArray::clone_from_slice(key_bytes);
        let cipher = Aes256Ecb::new(&key);

        let mut decrypted_data = compressed_data.to_vec();
        if cipher.decrypt_padded_mut::<Pkcs7>(&mut decrypted_data).is_ok() {
            let mut decoder = ZlibDecoder::new(Cursor::new(&decrypted_data));
            let mut decompressed_data = Vec::new();
            if decoder.read_to_end(&mut decompressed_data).is_ok() && !decompressed_data.is_empty() {
                return Some(decompressed_data);
            }
        }
    }
    None
}

pub fn pak_extract(input_file: &str, output_file: &str, encryption: bool) -> io::Result<()> {
    let mut fs = BufReader::new(File::open(&input_file)?);
    fs.seek(SeekFrom::Start(256))?;

    let mut buffer = [0u8; 16];
    fs.read_exact(&mut buffer)?;
    let _version = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
    let file_count = u32::from_le_bytes(buffer[4..8].try_into().unwrap());
    let file_info_offset = u32::from_le_bytes(buffer[8..12].try_into().unwrap());

    let keys = if encryption {
        let keys = load_keys("keylist.txt")?;
        if keys.is_empty() {
            eprintln!("No valid keys found in keylist.txt");
            return Ok(());
        }
        Some(keys)
    } else {
        None
    };

    let mut file_infos = Vec::with_capacity(file_count as usize);
    for _ in 0..file_count {
        fs.seek(SeekFrom::Start(file_info_offset as u64 + file_infos.len() as u64 * 316))?;
        let mut file_path_buffer = [0u8; 256];
        fs.read_exact(&mut file_path_buffer)?;
        let file_path = String::from_utf8_lossy(&file_path_buffer)
            .trim_end_matches('\0')
            .trim_start_matches('\\')
            .to_string();

        let mut file_info_buffer = [0u8; 24];
        fs.read_exact(&mut file_info_buffer)?;
        let compressed_size = u32::from_le_bytes(file_info_buffer[8..12].try_into().unwrap());
        let offset_value = u32::from_le_bytes(file_info_buffer[12..16].try_into().unwrap());

        file_infos.push((file_path, compressed_size, offset_value));
    }

    file_infos.into_par_iter().for_each(|(file_path, compressed_size, offset_value)| {
        let mut fs = File::open(input_file).unwrap();
        fs.seek(SeekFrom::Start(offset_value as u64)).unwrap();
        let mut compressed_data = vec![0u8; compressed_size as usize];
        fs.read_exact(&mut compressed_data).unwrap();

        let file_path = file_path.split('\0').next().unwrap_or("").to_string();

        let mut write_data = None;
        
        if !file_path.ends_with(".exe") && !file_path.ends_with(".dll") && !file_path.contains("xigncode") && !file_path.contains("testbranch") && encryption {
            if compressed_data.len() > 16 {
                let compressed_data = &compressed_data[16..];
                if let Some(keys) = &keys {
                    write_data = decrypt_with_keys(compressed_data, &keys);
                }
            }
        } else {
            let mut decoder = ZlibDecoder::new(Cursor::new(&compressed_data));
            let mut decompressed_data = Vec::new();
            if decoder.read_to_end(&mut decompressed_data).is_ok() && !decompressed_data.is_empty() {
                write_data = Some(decompressed_data);
            }
        }
        if let Some(decompressed_data) = write_data {
            let mut output_path = PathBuf::from(output_file);
            let cleaned_path: PathBuf = output_path
                .components()
                .filter(|comp| !comp.as_os_str().to_string_lossy().contains(".pak"))
                .collect();
        
            let final_output_path = cleaned_path.join("Export").join(&file_path);
        
            if let Some(parent) = final_output_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
        
            let mut output_file = BufWriter::new(File::create(&final_output_path).unwrap());
            output_file.write_all(&decompressed_data).unwrap();
        }
    });

    Ok(())
}
