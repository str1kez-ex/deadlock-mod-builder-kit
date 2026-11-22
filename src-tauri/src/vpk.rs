use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const VPK_SIGNATURE: u32 = 0x55aa1234;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VpkEntryInfo {
    pub full_path: String,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size: u32,
    pub crc32: u32,
    pub category: String, // "models", "sounds", "materials", "particles", "scripts", "other"
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VpkManifest {
    pub filename: String,
    pub total_files: usize,
    pub total_size: u64,
    pub has_models: bool,
    pub has_sounds: bool,
    pub has_materials: bool,
    pub files: Vec<VpkEntryInfo>,
}

fn detect_category(path: &str, ext: &str) -> String {
    let lower_p = path.to_lowercase();
    let lower_e = ext.to_lowercase();
    if lower_p.starts_with("sound") || lower_e.contains("snd") || lower_e == "wav" || lower_e == "mp3" {
        "sounds".into()
    } else if lower_p.starts_with("model") || lower_e.contains("mdl") || lower_e.contains("mesh") {
        "models".into()
    } else if lower_p.starts_with("material") || lower_e.contains("mat") || lower_e.contains("tex") {
        "materials".into()
    } else if lower_p.starts_with("particle") || lower_e.contains("pcf") {
        "particles".into()
    } else if lower_p.starts_with("script") || lower_p.starts_with("panorama") || lower_e.contains("vxml") || lower_e.contains("vcss") {
        "scripts".into()
    } else {
        "other".into()
    }
}

fn read_null_string<R: Read>(reader: &mut R) -> Result<String> {
    let mut bytes = Vec::new();
    let mut buf = [0u8; 1];
    loop {
        if reader.read(&mut buf)? == 0 {
            break;
        }
        if buf[0] == 0 {
            break;
        }
        bytes.push(buf[0]);
    }
    String::from_utf8(bytes).map_err(|e| anyhow!("Invalid UTF-8 in VPK string: {}", e))
}

pub struct VpkReader<R: Read + Seek> {
    reader: R,
    pub header_size: u64,
    pub tree_size: u64,
    pub version: u32,
    pub entries: HashMap<String, (VpkEntryInfo, u64, u32)>, // key -> (info, data_offset, data_length)
}

impl<R: Read + Seek> VpkReader<R> {
    pub fn new(mut reader: R) -> Result<Self> {
        reader.seek(SeekFrom::Start(0))?;

        let mut sig_buf = [0u8; 4];
        reader.read_exact(&mut sig_buf)?;
        let sig = u32::from_le_bytes(sig_buf);
        if sig != VPK_SIGNATURE {
            return Err(anyhow!("Not a valid VPK file (invalid signature 0x{:08x})", sig));
        }

        let mut ver_buf = [0u8; 4];
        reader.read_exact(&mut ver_buf)?;
        let version = u32::from_le_bytes(ver_buf);
        if version != 1 && version != 2 {
            return Err(anyhow!("Unsupported VPK version: {}", version));
        }

        let mut tree_sz_buf = [0u8; 4];
        reader.read_exact(&mut tree_sz_buf)?;
        let tree_size = u32::from_le_bytes(tree_sz_buf) as u64;

        let header_size = if version == 2 {
            // Read 16 additional bytes: file_data_size, archive_md5_size, other_md5_size, sig_size
            let mut v2_extra = [0u8; 16];
            reader.read_exact(&mut v2_extra)?;
            28u64
        } else {
            12u64
        };

        let mut entries = HashMap::new();

        loop {
            let ext = read_null_string(&mut reader)?;
            if ext.is_empty() {
                break;
            }

            loop {
                let mut path = read_null_string(&mut reader)?;
                if path.is_empty() {
                    break;
                }
                if path == " " {
                    path = String::new();
                }

                loop {
                    let filename = read_null_string(&mut reader)?;
                    if filename.is_empty() {
                        break;
                    }

                    let mut meta = [0u8; 18];
                    reader.read_exact(&mut meta)?;

                    let crc32 = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]);
                    let preload_bytes = u16::from_le_bytes([meta[4], meta[5]]);
                    let archive_index = u16::from_le_bytes([meta[6], meta[7]]);
                    let entry_offset = u32::from_le_bytes([meta[8], meta[9], meta[10], meta[11]]);
                    let entry_length = u32::from_le_bytes([meta[12], meta[13], meta[14], meta[15]]);
                    let _terminator = u16::from_le_bytes([meta[16], meta[17]]);

                    // Skip preload bytes if any
                    if preload_bytes > 0 {
                        reader.seek(SeekFrom::Current(preload_bytes as i64))?;
                    }

                    let full_path = if path.is_empty() {
                        format!("{}.{}", filename, ext)
                    } else {
                        format!("{}/{}.{}", path, filename, ext)
                    };

                    let category = detect_category(&path, &ext);
                    let info = VpkEntryInfo {
                        full_path: full_path.clone(),
                        path: path.clone(),
                        filename: filename.clone(),
                        extension: ext.clone(),
                        size: entry_length,
                        crc32,
                        category,
                    };

                    // In single-file VPKs, archive_index is 0x7fff
                    if archive_index == 0x7fff {
                        let abs_offset = header_size + tree_size + entry_offset as u64;
                        entries.insert(full_path, (info, abs_offset, entry_length));
                    }
                }
            }
        }

        Ok(Self {
            reader,
            header_size,
            tree_size,
            version,
            entries,
        })
    }

    pub fn read_file(&mut self, full_path: &str) -> Result<Vec<u8>> {
        let entry = self
            .entries
            .get(full_path)
            .ok_or_else(|| anyhow!("File '{}' not found in VPK", full_path))?;

        let (_, offset, length) = *entry;
        self.reader.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; length as usize];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn manifest(&self, filename: String) -> VpkManifest {
        let mut files: Vec<VpkEntryInfo> = self.entries.values().map(|(info, _, _)| info.clone()).collect();
        files.sort_by(|a, b| a.full_path.cmp(&b.full_path));

        let total_size = files.iter().map(|f| f.size as u64).sum();
        let has_models = files.iter().any(|f| f.category == "models");
        let has_sounds = files.iter().any(|f| f.category == "sounds");
        let has_materials = files.iter().any(|f| f.category == "materials");

        VpkManifest {
            filename,
            total_files: files.len(),
            total_size,
            has_models,
            has_sounds,
            has_materials,
            files,
        }
    }
}

pub struct VpkWriter {
    files: HashMap<String, Vec<u8>>, // full_path -> data
}

impl VpkWriter {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, full_path: &str, data: Vec<u8>) {
        let clean_path = full_path.replace('\\', "/").trim_start_matches('/').to_string();
        self.files.insert(clean_path, data);
    }

    pub fn write_to<W: Write + Seek>(&self, mut writer: W) -> Result<()> {
        // Group by extension -> path -> filename
        // Map: ext -> path -> Vec<(filename, data_len, offset, crc)>
        let mut ext_map: HashMap<String, HashMap<String, Vec<(String, usize, u32)>>> = HashMap::new();

        let mut sorted_keys: Vec<&String> = self.files.keys().collect();
        sorted_keys.sort();

        for key in &sorted_keys {
            let parts: Vec<&str> = key.rsplitn(2, '.').collect();
            let (ext, without_ext) = if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                ("", key.as_str())
            };

            let path_parts: Vec<&str> = without_ext.rsplitn(2, '/').collect();
            let (filename, path) = if path_parts.len() == 2 {
                (path_parts[0], path_parts[1])
            } else {
                (path_parts[0], " ")
            };

            let data = &self.files[*key];
            let crc = calculate_crc32(data);

            ext_map
                .entry(ext.to_string())
                .or_default()
                .entry(path.to_string())
                .or_default()
                .push((filename.to_string(), data.len(), crc));
        }

        // Calculate tree bytes
        let mut tree_buf = Vec::new();
        let mut cur_offset = 0u32;
        let mut file_order: Vec<&String> = Vec::new();

        for (ext, paths) in &ext_map {
            tree_buf.extend_from_slice(ext.as_bytes());
            tree_buf.push(0);

            for (path, file_list) in paths {
                tree_buf.extend_from_slice(path.as_bytes());
                tree_buf.push(0);

                for (filename, len, crc) in file_list {
                    tree_buf.extend_from_slice(filename.as_bytes());
                    tree_buf.push(0);

                    // Entry struct (18 bytes)
                    tree_buf.extend_from_slice(&crc.to_le_bytes()); // crc32
                    tree_buf.extend_from_slice(&0u16.to_le_bytes()); // preload_bytes = 0
                    tree_buf.extend_from_slice(&0x7fffu16.to_le_bytes()); // archive_index = 0x7fff (embedded)
                    tree_buf.extend_from_slice(&cur_offset.to_le_bytes()); // entry_offset
                    tree_buf.extend_from_slice(&(*len as u32).to_le_bytes()); // entry_length
                    tree_buf.extend_from_slice(&0xffffu16.to_le_bytes()); // terminator

                    cur_offset += *len as u32;

                    let full_key = if path == " " {
                        format!("{}.{}", filename, ext)
                    } else {
                        format!("{}/{}.{}", path, filename, ext)
                    };
                    file_order.push(self.files.keys().find(|k| **k == full_key).unwrap());
                }
                tree_buf.push(0); // end of files for this path
            }
            tree_buf.push(0); // end of paths for this ext
        }
        tree_buf.push(0); // end of extensions

        let tree_size = tree_buf.len() as u32;
        let file_data_size = cur_offset;

        // Write VPK v2 Header (28 bytes)
        writer.write_all(&VPK_SIGNATURE.to_le_bytes())?;
        writer.write_all(&2u32.to_le_bytes())?; // Version 2
        writer.write_all(&tree_size.to_le_bytes())?;
        writer.write_all(&file_data_size.to_le_bytes())?;
        writer.write_all(&0u32.to_le_bytes())?; // archive_md5_size = 0
        writer.write_all(&0u32.to_le_bytes())?; // other_md5_size = 0
        writer.write_all(&0u32.to_le_bytes())?; // signature_size = 0

        // Write Directory Tree
        writer.write_all(&tree_buf)?;

        // Write File Payloads in exact tree order
        for key in file_order {
            let data = &self.files[key];
            writer.write_all(data)?;
        }

        Ok(())
    }
}

pub fn calculate_crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        let mut c = (crc ^ (byte as u32)) & 0xFF;
        for _ in 0..8 {
            c = if (c & 1) != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        crc = c ^ (crc >> 8);
    }
    !crc
}

/// Inspects a VPK file on disk and returns its manifest and tree
pub fn inspect_vpk_file<P: AsRef<Path>>(path: P) -> Result<VpkManifest> {
    let file = std::fs::File::open(path.as_ref())
        .with_context(|| format!("Failed to open VPK file: {:?}", path.as_ref()))?;
    let reader = VpkReader::new(file)?;
    let filename = path
        .as_ref()
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.vpk")
        .to_string();
    Ok(reader.manifest(filename))
}

/// Merges multiple VPKs based on selected categories or file filters.
/// e.g. VPK A (models, materials) + VPK B (sounds) -> Merged VPK C
pub fn merge_vpks(
    source_vpks: &[PathBuf],
    category_filters: &HashMap<PathBuf, Vec<String>>, // VpkPath -> allowed categories ("models", "sounds", etc.)
    output_path: &Path,
) -> Result<usize> {
    let mut writer = VpkWriter::new();
    let mut total_merged = 0;

    for vpk_path in source_vpks {
        if !vpk_path.exists() {
            continue;
        }
        let file = std::fs::File::open(vpk_path)
            .with_context(|| format!("Failed to open source VPK: {:?}", vpk_path))?;
        let mut reader = VpkReader::new(file)?;

        let allowed_cats = category_filters.get(vpk_path);

        let entries_to_take: Vec<(String, String)> = reader
            .entries
            .iter()
            .map(|(k, (info, _, _))| (k.clone(), info.category.clone()))
            .collect();

        for (key, cat) in entries_to_take {
            let include = match allowed_cats {
                Some(cats) => cats.is_empty() || cats.contains(&cat) || cats.contains(&"all".to_string()),
                None => true,
            };

            if include {
                if let Ok(bytes) = reader.read_file(&key) {
                    writer.add_file(&key, bytes);
                    total_merged += 1;
                }
            }
        }
    }

    if total_merged == 0 {
        return Err(anyhow!("No files were matched for merging from selected VPKs"));
    }

    let out_file = std::fs::File::create(output_path)
        .with_context(|| format!("Failed to create merged VPK output: {:?}", output_path))?;
    let mut buf_writer = std::io::BufWriter::new(out_file);
    writer.write_to(&mut buf_writer)?;

    Ok(total_merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_vpk_pack_and_unpack() {
        let mut writer = VpkWriter::new();
        writer.add_file("sounds/heroes/ratking/ult_cast.vsnd_c", b"RATKING_SOUND_BYTES".to_vec());
        writer.add_file("models/heroes/ratking/model.vmdl_c", b"RATKING_MODEL_BYTES_XYZ".to_vec());
        writer.add_file("scripts/npc/ratking_config.vxml_c", b"<config></config>".to_vec());

        let mut cursor = Cursor::new(Vec::new());
        writer.write_to(&mut cursor).expect("Failed to write VPK");

        let mut reader = VpkReader::new(cursor).expect("Failed to read created VPK");
        assert_eq!(reader.entries.len(), 3);

        let sound_data = reader.read_file("sounds/heroes/ratking/ult_cast.vsnd_c").unwrap();
        assert_eq!(sound_data, b"RATKING_SOUND_BYTES");

        let model_data = reader.read_file("models/heroes/ratking/model.vmdl_c").unwrap();
        assert_eq!(model_data, b"RATKING_MODEL_BYTES_XYZ");

        let manifest = reader.manifest("test.vpk".into());
        assert!(manifest.has_sounds);
        assert!(manifest.has_models);
        assert_eq!(manifest.total_files, 3);
    }
}
