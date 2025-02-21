use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

#[derive(Default)]
pub struct TextureArchive {
    file_path: String,
    cursor: Cursor<Vec<u8>>,

    texture_num: u16,
    is_without_model: bool,
}

impl TextureArchive {
    pub fn new(file_path: String) -> std::io::Result<Self> {
        let cursor = Cursor::new(std::fs::read(&file_path)?);

        Ok(Self {
            file_path,
            cursor,
            ..Default::default()
        })
    }

    pub fn read(&mut self) {
        self.texture_num = self.cursor.read_u16::<BigEndian>().unwrap();
        self.is_without_model = self.cursor.read_u16::<BigEndian>().unwrap() == 1;
    }
}
