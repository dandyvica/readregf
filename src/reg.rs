// Structures defining the hive file format
// see: https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md
//
// a visualisation of the REGF format
//
// +--------------------+------------------------------+
// |     Base block     |        Hive bins data        |
// +--------------------+------------------------------+
//                                    |
//                                    v
//             +-----------+  +-----------+  +-----------+      +-----------+
//             |  Hive bin |  |  Hive bin |  |  Hive bin |  ... |  Hive bin |
//             +-----------+  +-----------+  +-----------+      +-----------+
//                     |
//                     v
//         +-------------------+---------+---------+-----+---------+
//         | Hive bin header   |  Cell   |  Cell   | ... |  Cell   |
//         |     (32 bytes)    |         |         |     |         |
//         +-------------------+---------+---------+-----+---------+
//
// keys structure:
//
// REGF Header
// └── Root Key (nk)
//     ├── Subkey 1 (nk)
//     │   ├── Subkey 1a (nk)
//     │   │   └── Values (vk)
//     │   └── Values (vk)
//     ├── Subkey 2 (nk)
//     └── Values (vk)
//
// - Each nk cell can have subkeys (nk) and values (vk).
// - Values may store data inline or in separate db cells.
// - Subkeys may be organized in lists (lf, lh, ri) to optimize lookups.
//
use std::{
    fmt,
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use anyhow::Ok;
use bincode::{Decode, error::DecodeError};

// an overall structure keeping reader and current number of hbins read
#[derive(Debug)]
pub struct RegistryFile {
    reader: BufReader<File>,

    // a regf could contain left over data, need this to correctly read hbins
    total_hbins_size: u32,
    current_hbins_size: u32,
}

impl TryFrom<&Path> for RegistryFile {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);

        Ok(Self {
            reader,
            total_hbins_size: 0,
            current_hbins_size: 0,
        })
    }
}

impl RegistryFile {
    // read base block
    pub fn read_header(&mut self) -> anyhow::Result<BaseBlock> {
        // base block is 4096 bytes
        let mut buffer = [0u8; 4096];

        // use bincode
        let config = bincode::config::standard()
            .with_little_endian()
            .with_fixed_int_encoding();
        let header: BaseBlock = bincode::decode_from_reader(&mut self.reader, config)?;

        // header is read: we have the theoretical total hbins size
        self.total_hbins_size = header.hive_bins_data_size;

        Ok(header)
    }
}

// we can loop through hbins
impl Iterator for RegistryFile {
    type Item = HiveBin;

    fn next(&mut self) -> Option<Self::Item> {
        // not at the end
        if self.current_hbins_size < self.total_hbins_size {
            let hive_bin = HiveBin::try_from(&mut self.reader).ok()?;
            self.current_hbins_size += hive_bin.header.size;
            Some(hive_bin)
        } else {
            None
        }
    }
}

#[derive(Debug, Decode)]
pub struct BaseBlock {
    // ASCII string
    signature: [u8; 4],

    // This number is incremented by 1 in the beginning of a write operation on the primary file
    primary_sequence_number: u32,

    // This number is incremented by 1 at the end of a write operation on the primary file, a *primary sequence number* and a *secondary sequence number* should be equal after a successful write operation
    secondary_sequence_number: u32,

    // FILETIME (UTC)
    last_written_timestamp: u64,

    // Major version of a hive writer
    major_version: u32,

    // Minor version of a hive writer
    minor_version: u32,

    // 0 means *primary file*
    file_type: u32,

    // 1 means *direct memory load*
    file_format: u32,

    // Offset of a root cell in bytes, relative from the start of the hive bins data
    root_cell_offset: u32,

    // Size of the hive bins data in bytes
    pub hive_bins_data_size: u32,

    // Logical sector size of the underlying disk in bytes divided by 512
    clustering_factor: u32,

    // UTF-16LE string (contains a partial file path to the primary file, or a file name of the primary file), used for debugging purposes
    file_name: [u16; 32],

    //
    reserved1: [u8; 396],

    // XOR-32 checksum of the previous 508 bytes
    checksum: u32,

    //
    reserved2: [u8; 3576],

    // This field has no meaning on a disk
    boot_type: u32,

    // This field has no meaning on a disk
    boot_recover: u32,
}

impl fmt::Display for BaseBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "signature: {}",
            String::from_utf8(self.signature.to_vec()).unwrap()
        )?;
        writeln!(f, "major version: {}", self.major_version)?;
        writeln!(f, "minor version: {}", self.minor_version)?;
        writeln!(f, "hive bins data size: {}", self.hive_bins_data_size)?;
        writeln!(
            f,
            "file_name: {}",
            String::from_utf16(&self.file_name).unwrap()
        )
    }
}

// Hive bin header
#[derive(Debug, Decode)]
pub struct HiveBinHeader {
    // ASCII string
    signature: [u8; 4],

    // Offset of a current hive bin in bytes, relative from the start of the hive bins data
    offset: u32,

    // Size of a current hive bin in bytes
    pub size: u32,

    //
    reserved: u64,

    // FILETIME (UTC), defined for the first hive bin only (see below)
    timestamp: u64,

    // This field has no meaning on a disk (see below)
    spare: u32,
}

impl fmt::Display for HiveBinHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "signature: {} ",
            String::from_utf8(self.signature.to_vec()).unwrap()
        )?;
        write!(f, "offset: 0x{:X?} ", self.offset)?;
        write!(f, "size: 0x{:X?}", self.size)
    }
}

impl TryFrom<&mut BufReader<File>> for HiveBinHeader {
    type Error = DecodeError;

    fn try_from(reader: &mut BufReader<File>) -> Result<Self, Self::Error> {
        let config = bincode::config::standard()
            .with_little_endian()
            .with_fixed_int_encoding();
        bincode::decode_from_reader(reader, config)
    }
}

// A hive bin has header and a list of cells
//         +-------------------+---------+---------+-----+---------+
//         | Hive bin header   |  Cell   |  Cell   | ... |  Cell   |
//         |     (32 bytes)    |         |         |     |         |
//         +-------------------+---------+---------+-----+---------+
#[derive(Debug)]
pub struct HiveBin {
    pub header: HiveBinHeader,
    pub cells_data: Cursor<Vec<u8>>,
    // pub cells: Vec<u8>,
    // this will keep current cell size when reading cells
    current_cells_size: u32,
}

impl Iterator for HiveBin {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        // not at the end
        if self.current_cells_size < self.header.size - 32 {
            let cell = Cell::try_from(&mut self.cells_data).ok()?;

            // need to take absolute value because cell size is negative for allocated cells
            self.current_cells_size += cell.size.unsigned_abs();

            Some(cell)
        } else {
            None
        }
    }
}

impl fmt::Display for HiveBin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "header: {} ", self.header)?;
        let len = self.cells_data.get_ref().len();
        write!(
            f,
            "data: {:X?}, len = {} last bytes: {:X?}",
            &self.cells_data.get_ref()[0..16],
            len,
            &self.cells_data.get_ref()[len - 16..len]
        )
    }
}

impl TryFrom<&mut BufReader<File>> for HiveBin {
    type Error = anyhow::Error;

    fn try_from(reader: &mut BufReader<File>) -> Result<Self, Self::Error> {
        // let pos = reader.seek(SeekFrom::Current(0))?;
        // println!("pos index try_from before={:x?}", pos);
        let header = HiveBinHeader::try_from(&mut *reader)?;
        let mut data = vec![0u8; header.size as usize - size_of::<HiveBinHeader>()];

        reader.read_exact(&mut data)?;

        Ok(Self {
            header,
            cells_data: Cursor::new(data),
            current_cells_size: 0,
        })
    }
}

#[derive(Debug)]
pub struct Cell {
    pub size: i32,
    pub r#type: CellType,
    pub data: Vec<u8>,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "size: {} type: {:X?} data: {:X?}",
            self.size, self.r#type, &self.data
        )?;
        if self.r#type == CellType::NamedKey {
            let s = String::from_utf8_lossy(&self.data);
            write!(f, "found s = {s}")?;
        }
        write!(f, "")
    }
}

impl TryFrom<&mut Cursor<Vec<u8>>> for Cell {
    type Error = anyhow::Error;

    fn try_from(c: &mut Cursor<Vec<u8>>) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 4];
        let _ = c.read_exact(&mut buf);
        let cell_size = i32::from_le_bytes(buf);
        let cell_type = CellType::try_from(&mut *c)?;

        let mut cell_data = vec![0u8; cell_size.unsigned_abs() as usize - 6];
        let _ = c.read_exact(&mut cell_data);

        Ok(Self {
            size: cell_size,
            r#type: cell_type,
            data: cell_data,
        })
    }
}

// each cell can only be this enum
#[derive(Debug, PartialEq)]
pub enum CellType {
    LeafIndex,   // Subkeys list
    LeafFast,    // Subkeys list with name hints
    LeafHash,    // Subkeys list with name hashes
    RootIndex,   // List of subkeys lists (used to subdivide subkeys lists)
    NamedKey,    // Registry key node
    ValueKey,    // Registry key value
    SecurityKey, // Security descriptor
    DataBlock,   // List of data segments
    Unknown([u8; 2]),
}

impl TryFrom<&mut Cursor<Vec<u8>>> for CellType {
    type Error = anyhow::Error;

    fn try_from(c: &mut Cursor<Vec<u8>>) -> Result<Self, Self::Error> {
        let mut key = [0u8; 2];
        c.read_exact(&mut key)?;
        match &key {
            b"li" => Ok(CellType::LeafIndex),
            b"lf" => Ok(CellType::LeafFast),
            b"lh" => Ok(CellType::LeafHash),
            b"ri" => Ok(CellType::RootIndex),
            b"nk" => Ok(CellType::NamedKey),
            b"vk" => Ok(CellType::ValueKey),
            b"sk" => Ok(CellType::SecurityKey),
            b"db" => Ok(CellType::DataBlock),
            _ => Ok(CellType::Unknown(key)),
        }
    }
}
