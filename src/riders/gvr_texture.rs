//! This module contains all the functionality to work with buffers of data that are GVR textures.

use std::io::{Cursor, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

/// Represents a buffer of data that is a GVR texture.
///
/// It's possible that when first constructed, it may not be a GVR texture.
/// This should be double-checked via [`GVRTexture::validate()`].
#[derive(Default, Clone)]
pub struct GVRTexture {
    /// Name of the texture file. This only contains the name and not the file extension.
    pub name: String,
    /// The full size of the texture in bytes.
    pub size: u32,
    /// The texture data.
    pub data: Cursor<Vec<u8>>,
}

impl GVRTexture {
    /// Constructs a new [`GVRTexture`] in a simple manner from the given `data` with a predefined `size`, and a
    /// `name` to represent the name of the texture file.
    ///
    /// This already assumes that this is a valid GVR texture, as it doesn't perform any checks,
    /// and simply makes a new [`GVRTexture`].
    pub fn new(name: String, size: u32, data: Cursor<Vec<u8>>) -> Self {
        Self { name, size, data }
    }

    /// Constructs a new [`GVRTexture`] from the given `cursor` and a `name` to represent the name
    /// of the texture file.
    ///
    /// This function should be used for when you're trying to attempt to make a valid [`GVRTexture`]
    /// for the first time, as it performs checks to see if it's a valid GVR texture file, and also
    /// calculates the size of the texture file in full.
    ///
    /// This assumes that the `cursor` is at the very start of the file!
    /// If it's a valid GVR texture, the `cursor` position is returned back to the start.
    /// Otherwise the `cursor` position will be altered when this function returns.
    pub fn new_from_cursor(name: String, cursor: &mut Cursor<Vec<u8>>) -> Result<Self, ()> {
        GVRTexture::validate(cursor)?;
        let tex_size = GVRTexture::read_texture_size(cursor)?;
        let mut buf = vec![0; tex_size.try_into().unwrap()];

        // Read whole texture into buffer
        if cursor.read_exact(&mut buf).is_err() {
            return Err(());
        }

        // Return texture with a cursor containing just the texture
        Ok(GVRTexture::new(name, tex_size, Cursor::new(buf)))
    }

    /// Checks if the given buffer in `cursor` is a valid GVR texture.
    ///
    /// This assumes that the `cursor` is at the very start of the file!
    /// If it's a valid GVR texture, the `cursor` position is returned back to the start.
    /// Otherwise the `cursor` position will be altered when this function returns.
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

    /// Calculates the size of the given GVR texture from the buffer in `cursor`.
    ///
    /// This assumes that the buffer in `cursor` is a valid GVR texture!
    /// This means a call to [`GVRTexture::validate()`] should be performed beforehand.
    ///
    /// This also assumes that the `cursor` is at the very start of the file!
    /// If it's a valid GVR texture, the `cursor` position is returned back to the start.
    /// Otherwise the `cursor` position will be altered when this function returns.
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
