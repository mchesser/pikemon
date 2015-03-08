use std::io;
use std::io::prelude::*;
use std::fs::{self, File};

use gb_emu::cart::SaveFile;

pub struct LocalSaveWrapper {
    pub path: Path,
}

impl SaveFile for LocalSaveWrapper {
    fn load(&mut self, data: &mut [u8]) {
        if let Ok(_) = File::open(&self.path).map(|mut f| f.read(data)) {
            println!("Loaded {}", self.path.display());
        }
    }

    fn save(&mut self, data: &[u8]) {
        // First create a temporary file and write to that, to ensure that if an error occurs, the
        // old file is not lost.
        let tmp_path = self.path.with_extension("sav.tmp");
        if let Err(e) = File::create(&tmp_path).map(|mut f| f.write_all(data)) {
            println!("An error occured when writing the save file: {}", e);
            return;
        }

        // At this stage the new save file has been successfully written, so we can safely remove
        // the old file if it exists.
        match fs::remove_file(&self.path) {
            Ok(_) => {},
            Err(ref e) if e.kind() == io::ErrorKind::FileNotFound => {},
            Err(e) => {
                println!("Error removing old save file ({}), current save has been written to: {}",
                    e, tmp_path.display());
                return;
            },
        }

        // Now rename the temporary file to the correct name
        if let Err(e) = fs::rename(&tmp_path, &self.path) {
            println!("Error renaming temporary save file: {}", e);
        }
    }
}
