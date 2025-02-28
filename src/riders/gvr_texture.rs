use std::io::{self, Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Default)]
pub struct GVRTexture {
    name: String,
}

impl GVRTexture {
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
