use std::{
    fs::File,
    io::{Cursor, Read, Seek, Write},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::util::Alignment;

#[derive(Default)]
pub struct PackManFile {
    pub data: Vec<u8>,
    pub exported_offset: u32,
}

impl PackManFile {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct PackManFolder {
    pub is_id_valid: bool,
    pub id: u16,
    pub file_count: u8,
    pub files: Vec<PackManFile>,
}

impl PackManFolder {
    pub fn new(file_count: u8) -> Self {
        Self {
            file_count,
            files: Vec::with_capacity(file_count as usize),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct PackManArchive {
    cursor: Cursor<Vec<u8>>,

    pub folders: Vec<PackManFolder>,
}

impl PackManArchive {
    pub fn new(file_path: &str) -> std::io::Result<Self> {
        Ok(Self {
            cursor: Cursor::new(std::fs::read(file_path)?),
            ..Default::default()
        })
    }

    pub fn new_empty() -> Self {
        Default::default()
    }

    pub fn read(&mut self) -> std::io::Result<()> {
        let folder_count = self.cursor.read_u32::<BigEndian>()?;

        for _ in 0..folder_count {
            let file_count = self.cursor.read_u8()?;
            self.folders.push(PackManFolder::new(file_count));
        }

        let aligned_next_pos = Alignment::A4(self.cursor.position()).unwrap();
        self.cursor
            .seek(std::io::SeekFrom::Start(aligned_next_pos))?;

        // Skip the starting file indices for each folder (unnecessary info)
        self.cursor.seek_relative(
            (size_of::<u16>() * folder_count as usize)
                .try_into()
                .unwrap(),
        )?;

        for i in 0..folder_count {
            let folder_id = self.cursor.read_u16::<BigEndian>()?;
            let folder = &mut self.folders[i as usize];
            folder.id = folder_id;
            folder.is_id_valid = true;
        }

        let file_count = self.get_all_file_count();
        let mut cur_file_count = 0;
        for folder in &mut self.folders {
            for _ in 0..folder.file_count {
                let offset = self.cursor.read_u32::<BigEndian>()?;
                cur_file_count += 1;

                if offset == 0 {
                    // Empty file
                    folder.files.push(PackManFile::default());
                    continue;
                }

                let next_file_offset = self.cursor.position();
                let mut next_nonzero_offset = None;
                let mut cur_count_copy = cur_file_count;

                // Find the next non-zero offset to calculate file size
                while cur_count_copy < file_count && next_nonzero_offset.is_none() {
                    let next_offset = self.cursor.read_u32::<BigEndian>()?;
                    cur_count_copy += 1;

                    if next_offset != 0 {
                        next_nonzero_offset = Some(next_offset);
                    }
                }

                if next_nonzero_offset.is_none() {
                    next_nonzero_offset = Some(self.cursor.get_ref().len().try_into().unwrap());
                }

                let file_size = next_nonzero_offset.unwrap() - offset;

                // Read file
                let mut buf = vec![0; file_size.try_into().unwrap()];
                self.cursor.seek(std::io::SeekFrom::Start(offset.into()))?;
                self.cursor.read_exact(&mut buf)?;
                folder.files.push(PackManFile::new(buf));

                self.cursor
                    .seek(std::io::SeekFrom::Start(next_file_offset))?;
            }
        }

        Ok(())
    }

    /// Gets the count of all the files from each folder in the archive.
    /// Only used when reading in an archive via read().
    fn get_all_file_count(&self) -> usize {
        let mut file_count: usize = 0;
        for folder in &self.folders {
            file_count += folder.file_count as usize;
        }
        file_count
    }

    pub fn export(&mut self, output_path: &str) -> std::io::Result<()> {
        let mut file = File::create(output_path)?;

        // Folders
        file.write_u32::<BigEndian>(self.folders.len() as u32)?;

        for folder in &self.folders {
            file.write_u8(folder.files.len() as u8)?;
        }

        // Padding
        let aligned_next_pos = Alignment::A4(file.stream_position()?).unwrap();
        file.set_len(aligned_next_pos)?;
        file.seek(std::io::SeekFrom::Start(aligned_next_pos))?;

        // First file in each folder
        let mut cur_file_idx = 0; // Will have total file count in archive at the end of loop

        for folder in &self.folders {
            file.write_u16::<BigEndian>(cur_file_idx)?;
            cur_file_idx += folder.files.len() as u16;
        }

        // Folder IDs
        for folder in &self.folders {
            file.write_u16::<BigEndian>(folder.id)?;
        }

        let first_file_offset = self.get_first_file_offset(&mut file, cur_file_idx)?;
        let mut cur_file_offset = first_file_offset;

        // Offset table
        for folder in &mut self.folders {
            for f in &mut folder.files {
                if f.data.is_empty() {
                    file.write_u32::<BigEndian>(0)?;
                    continue;
                }

                file.write_u32::<BigEndian>(cur_file_offset)?;
                f.exported_offset = cur_file_offset;
                cur_file_offset = Alignment::A32(cur_file_offset + f.data.len() as u32).unwrap();
            }
        }

        file.set_len(first_file_offset as u64)?;
        file.seek(std::io::SeekFrom::Start(first_file_offset as u64))?;

        // File data
        for folder in &self.folders {
            for f in &folder.files {
                if f.data.is_empty() {
                    continue;
                }

                debug_assert!(f.exported_offset as u64 == file.stream_position()?);
                file.write_all(&f.data)?;

                // Padding
                let aligned_next_pos = Alignment::A32(file.stream_position()?).unwrap();
                file.set_len(aligned_next_pos)?;
                file.seek(std::io::SeekFrom::Start(aligned_next_pos))?;
            }
        }

        Ok(())
    }

    /// Gets the offset of where the first file in the archive will be written to.
    /// Only used during exporting right before writing offset table.
    fn get_first_file_offset(&self, file: &mut File, file_count: u16) -> std::io::Result<u32> {
        Ok(Alignment::A32(
            (file.stream_position()? as usize) + size_of::<u32>() * file_count as usize,
        )
        .unwrap()
        .try_into()
        .unwrap())
    }
}
