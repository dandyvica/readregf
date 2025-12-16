// main refs:
// https://googleprojectzero.blogspot.com/2024/12/the-windows-registry-adventure-5-regf.html
//
use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
    path::PathBuf,
};

use crate::reg::{BaseBlock, Cell, CellType, HiveBin, HiveBinHeader, RegistryFile};

mod reg;

fn main() -> anyhow::Result<()> {
    let path = PathBuf::from("./data/SYSTEM");
    let mut regf = RegistryFile::try_from(path.as_path())?;

    let base_block = regf.read_header()?;
    println!("{:?}", base_block);

    for mut hbin in &mut regf {
        println!("{hbin}");

        for cell in &mut hbin {
            println!("{cell}");
        }
    }

    // let file = File::open("./data/SYSTEM")?;
    // let mut reader = BufReader::new(file);

    // let mut buffer = [0u8; 4096];

    // let config = bincode::config::standard()
    //     .with_little_endian()
    //     .with_fixed_int_encoding();

    // let header: BaseBlock = bincode::decode_from_reader(&mut reader, config).unwrap();
    // let total_hbins_size = header.hive_bins_data_size;
    // let mut current_hbins_size = 0u32;
    // // let pos = reader.seek(SeekFrom::Current(0))?;
    // println!("{}", header);
    // // println!("pos after base block = 0x{:X?}", pos);

    // while current_hbins_size < total_hbins_size {
    //     let hive_bin = HiveBin::try_from(&mut reader)?;

    //     current_hbins_size += hive_bin.header.size;
    //     // let pos = reader.seek(SeekFrom::Current(0))?;
    //     // println!("pos: {:X?} {}", pos, hive_bin);

    //     let total_cells_size = hive_bin.header.size - 32;
    //     let mut current_cells_size = 0u32;

    //     while current_cells_size <= total_cells_size {
    //         let mut cursor = Cursor::new(hive_bin.cells.as_slice());

    //         let cell = Cell::try_from(&mut cursor)?;
    //         assert!(cell.size % 8 == 0);

    //         current_cells_size += cell.size.unsigned_abs();

    //         println!("cell={}", cell);
    //     }
    //     // println!("pos after hive_bin = 0x{:X?}", pos);
    //     // read hive bin header
    //     // println!("{:X?} len={}", hive_bin.header, hive_bin.cells.len());

    //     // // read all cells which follow the hive bin header
    //     // let all_cells_size = hive_bin_header.size as usize - 32;
    //     // println!("all_cells_size={all_cells_size}");
    //     // let mut current_size = 0usize;

    //     // // read each individual cell including its data
    //     // // A free (unused) cell is indicated by a positive size, and an allocated cell is indicated by a negative one.
    //     // // For example, a free cell of 32 bytes has a length marker of 0x00000020, while an active cell of 128 bytes
    //     // // has its size encoded as 0xFFFFFF80
    //     // while current_size < all_cells_size {
    //     //     let mut cell_size = [0u8; 4];
    //     //     let _ = reader.read_exact(&mut cell_size);
    //     //     let cell_size = i32::from_le_bytes(cell_size) as isize;
    //     //     println!("cell_size={cell_size}");

    //     //     if cell_size % 8 != 0 {
    //     //         panic!("error");
    //     //     }

    //     //     // now we know cell data size, so read cell data
    //     //     // read cell type
    //     //     let mut cell_type = [0u8; 2];
    //     //     let _ = reader.read_exact(&mut cell_type);
    //     //     let cell_type = CellType::try_from(cell_type).unwrap();
    //     //     println!("cell type={:?}", cell_type);

    //     //     let mut buf = vec![0u8; cell_size.unsigned_abs() - 6];
    //     //     let _ = reader.read_exact(&mut buf);

    //     //     current_size += cell_size.unsigned_abs();
    //     //     println!("current_size={current_size}");
    //     // }
    //     /*         let mut buf = vec![0u8; hive_bin_header.size as usize - 32];
    //     reader.read_exact(&mut buf);  */
    // }

    Ok(())

    /*     let hive_bin_header: HiveBinHeader = bincode::decode_from_reader(&mut reader, config).unwrap();
    println!("{}", hive_bin_header); */
}
