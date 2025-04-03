use std::io::{Cursor, Read, Seek};

use byteorder::{BigEndian, ReadBytesExt};

use crate::util::Alignment;

#[derive(Default)]
pub struct PackManFolder {
    pub is_id_valid: bool,
    pub id: u16,
    pub file_count: u8,
    pub files: Vec<Vec<u8>>,
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
                    folder.files.push(Vec::new());
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
                folder.files.push(buf);

                self.cursor
                    .seek(std::io::SeekFrom::Start(next_file_offset))?;
            }
        }

        Ok(())
    }

    fn get_all_file_count(&self) -> usize {
        let mut file_count: usize = 0;
        for folder in &self.folders {
            file_count += folder.file_count as usize;
        }
        file_count
    }
}
