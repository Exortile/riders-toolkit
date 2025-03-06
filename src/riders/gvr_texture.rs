use std::io::{self, Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Default)]
pub struct GVRTexture {
    pub name: String,
    data: Cursor<Vec<u8>>,
}

impl GVRTexture {
    pub fn new(name: String, data: Cursor<Vec<u8>>) -> Self {
        Self { name, data }
    }

    pub fn new_from_cursor(name: String, cursor: &mut Cursor<Vec<u8>>) -> Result<Self, ()> {
        GVRTexture::validate(cursor)?;
        let tex_size = GVRTexture::read_texture_size(cursor)?;
        let mut buf = Vec::with_capacity(tex_size.try_into().unwrap());

        // Read whole texture into buffer
        if cursor.read_exact(&mut buf).is_err() {
            return Err(());
        }

        // Return texture with a cursor containing just the texture
        Ok(GVRTexture::new(name, Cursor::new(buf)))
    }

    pub fn validate(cursor: &mut Cursor<Vec<u8>>) -> Result<(), ()> {
        let start_pos = cursor.position();
        let mut buf = [0; 4];

        // Read "GCIX" magic into buffer
        if cursor.read_exact(&mut buf).is_err() {
            return Err(());
        }

        // Check if "GCIX" magic matches
        let gcix_buf: Vec<char> = buf.iter().map(|&e| e as char).collect();
        let gcix_magic: String = gcix_buf.into_iter().collect();
        if gcix_magic != "GCIX" {
            return Err(());
        }

        // Seek to next magic location
        if cursor.seek(SeekFrom::Current(0xC)).is_err() {
            return Err(());
        }

        // Read "GVRT" magic into buffer
        if cursor.read_exact(&mut buf).is_err() {
            return Err(());
        }

        // Check if "GVRT" magic matches
        let gvrt_buf: Vec<char> = buf.iter().map(|&e| e as char).collect();
        let gvrt_magic: String = gvrt_buf.into_iter().collect();
        if gvrt_magic != "GVRT" {
            return Err(());
        }

        // Return cursor back to original position
        let _ = cursor.seek(SeekFrom::Start(start_pos));
        Ok(())
    }

    pub fn read_texture_size(cursor: &mut Cursor<Vec<u8>>) -> Result<u32, ()> {
        let start_pos = cursor.position();

        // Seek to texture size value
        if cursor.seek(SeekFrom::Current(0x14)).is_err() {
            return Err(());
        }

        let tex_size = cursor.read_u32::<LittleEndian>();
        if tex_size.is_err() {
            return Err(());
        }

        // Return cursor back to original position
        let _ = cursor.seek(SeekFrom::Start(start_pos));
        Ok(tex_size.unwrap() + 0x18)
    }
}
