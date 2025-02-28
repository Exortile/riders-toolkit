use super::gvr_texture::GVRTexture;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{BufRead, Cursor, Seek, SeekFrom};

#[derive(Default)]
pub struct TextureArchive {
    file_path: String,
    cursor: Cursor<Vec<u8>>,

    texture_num: u16,
    is_without_model: bool,

    gvr_offsets: Vec<u32>,
    texture_names: Vec<String>,
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

    pub fn read(&mut self) -> Result<(), &str> {
        self.texture_num = self.cursor.read_u16::<BigEndian>().unwrap();
        let is_without_model = self.cursor.read_u16::<BigEndian>().unwrap();

        if is_without_model > 1 {
            return Err("This is an invalid texture archive!");
        }

        self.is_without_model = is_without_model == 1;

        // Read all offsets to the textures in the file
        for _ in 0..self.texture_num {
            self.gvr_offsets
                .push(self.cursor.read_u32::<BigEndian>().unwrap());
        }

        // Read all texture names in the file
        for _ in 0..self.texture_num {
            let mut buf: Vec<u8> = vec![];
            let _ = self.cursor.read_until(0x00, &mut buf); // TODO: implement EOF check

            // Pop delimiter byte
            buf.pop();

            let ascii_buf: Vec<char> = buf.into_iter().map(|e| e as char).collect();

            if !ascii_buf
                .iter()
                .all(|&e| e.is_alphanumeric() || e.is_ascii_whitespace())
            {
                return Err("Can't read texture file names. This is most likely an invalid texture archive.");
            }

            let tex_name: String = ascii_buf.into_iter().collect();
            self.texture_names.push(tex_name);
        }

        self.debug_print();

        if !self.validate_textures() {
            return Err("The textures in this archive are not valid.");
        }

        Ok(())
    }

    fn validate_textures(&mut self) -> bool {
        for offset in &self.gvr_offsets {
            if self.cursor.seek(SeekFrom::Start(*offset as u64)).is_err() {
                return false;
            }

            if GVRTexture::validate(&mut self.cursor).is_err() {
                return false;
            }

            let tex_size = GVRTexture::read_texture_size(&mut self.cursor).unwrap();
            println!("texture size: {tex_size}");
        }

        true
    }

    #[cfg(debug_assertions)]
    fn debug_print(&self) {
        println!("File: {}", self.file_path);

        println!(
            "texture_num: {}, is_without_model: {}",
            self.texture_num, self.is_without_model
        );

        for offset in &self.gvr_offsets {
            println!("{offset}");
        }

        for name in &self.texture_names {
            println!("{name}");
        }
    }
}

#[derive(Default)]
pub struct EditableTextureArchive {
    textures: GVRTexture,
}
