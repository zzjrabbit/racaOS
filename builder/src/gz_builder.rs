use std::{fs::File, io, path::PathBuf};

use flate2::{Compression, GzBuilder};

pub fn compress_file(src_file_path: String, compressed_file_path: String) {
    let src_file_path = PathBuf::from(src_file_path);
    let compressed_file_path = PathBuf::from(compressed_file_path);

    let mut compressed_file = GzBuilder::new().read(
        File::open(src_file_path).expect("Unable to open the file!"),
        Compression::fast(),
    );

    io::copy(
        &mut compressed_file,
        &mut std::fs::File::create(compressed_file_path.clone()).unwrap(),
    )
    .unwrap();
}
