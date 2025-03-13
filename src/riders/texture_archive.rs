use crate::util::Alignment;

use super::gvr_texture::GVRTexture;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{
    fs::File,
    io::{BufRead, Cursor, Read, Seek, SeekFrom, Write},
};

#[derive(Default)]
pub struct TextureArchive {
    file_path: String,
    cursor: Cursor<Vec<u8>>,

    texture_num: u16,
    is_without_model: bool,

    gvr_offsets: Vec<u32>,
    pub textures: Vec<GVRTexture>,
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

        // Skip flags if necessary
        if self.is_without_model {
            let _ = self.cursor.seek_relative(self.texture_num.into()); // TODO: implement EOF check
        }

        // Read all texture names in the file
        for i in 0..self.texture_num {
            let mut buf: Vec<u8> = vec![];
            let _ = self.cursor.read_until(0x00, &mut buf); // TODO: implement EOF check

            // Pop delimiter byte
            buf.pop();

            let ascii_buf: Vec<char> = buf.into_iter().map(|e| e as char).collect();

            if !ascii_buf
                .iter()
                .all(|&e| e.is_ascii_graphic() || e.is_ascii_whitespace())
            {
                return Err("Can't read texture file names. This is most likely an invalid texture archive.");
            }

            let tex_name: String = ascii_buf.into_iter().collect();

            let last_pos = self.cursor.position();
            if self
                .cursor
                .seek(SeekFrom::Start(self.gvr_offsets[i as usize].into()))
                .is_err()
            {
                return Err("Something went wrong reading the texture archive.");
            }

            if let Ok(tex) = GVRTexture::new_from_cursor(tex_name, &mut self.cursor) {
                self.textures.push(tex);
            }

            let _ = self.cursor.seek(SeekFrom::Start(last_pos));
        }

        self.debug_print();

        if !self.validate_textures() {
            return Err("The textures in this archive are not valid.");
        }

        Ok(())
    }

    pub fn export(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        file.write_u16::<BigEndian>(self.textures.len().try_into().unwrap())?;
        file.write_u16::<BigEndian>(self.is_without_model.into())?;

        let offsets = self.calculate_offset_table();

        // Write offset table
        for offset in &offsets {
            file.write_u32::<BigEndian>(*offset)?;
        }

        // Write flags if needed
        if self.is_without_model {
            for _ in 0..self.textures.len() {
                file.write_u8(0x11)?;
            }
        }

        // Write texture names
        for tex in &self.textures {
            file.write_all(tex.name.as_bytes())?;
            file.write_u8(0)?; // null delimiter
        }

        // Padding
        file.set_len(offsets[0].into())?;
        file.seek(SeekFrom::Start(offsets[0].into()))?;

        // Write texture data
        for tex in &self.textures {
            file.write_all(tex.data.get_ref())?;
        }

        Ok(())
    }

    fn calculate_first_tex_offset(&self) -> usize {
        let mut result_offset = 4; // 4 bytes to account for start of file
        let offset_table_size = self.textures.len() * size_of::<u32>();

        result_offset += offset_table_size;

        if self.is_without_model {
            result_offset += self.textures.len();
        }

        // Calculate length of each texture name, add it to the offset
        for tex in &self.textures {
            result_offset += tex.name.len() + 1; // extra byte for null delimiter
        }

        let aligned = Alignment::A32(result_offset);
        aligned.unwrap()
    }

    fn calculate_offset_table(&self) -> Vec<u32> {
        let mut offsets = Vec::with_capacity(self.textures.len());
        let mut cur_offset = self.calculate_first_tex_offset() as u32;

        for tex in &self.textures {
            offsets.push(cur_offset);
            cur_offset += tex.size;
        }

        offsets
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

        for GVRTexture { name, .. } in &self.textures {
            println!("{name}");
        }
    }
}

#[derive(Default)]
pub struct EditableTextureArchive {
    textures: GVRTexture,
}
