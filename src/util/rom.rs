use std::fs;
use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct ROM {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
    pub prg_rom_mirroring: bool
}

impl ROM {
    const NES_SIGNATURE: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
    const CHR_ROM_PAGE_SIZE: usize = 0x2000; // 8kB
    const PRG_ROM_PAGE_SIZE: usize = 0x4000; // 16kB

    pub fn from_filepath(filepath: &str) -> Result<ROM, String> {
        let mut f = File::open(filepath).expect("no file found");
        let metadata = fs::metadata(filepath).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        ROM::from_buffer(&buffer)
    }

    pub fn from_buffer(raw: &Vec<u8>) -> Result<ROM, String> {
        if &raw[0..4] != ROM::NES_SIGNATURE {
            return Err("File is not in iNES file format".to_string());
        }

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * ROM::PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * ROM::CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let prg_rom = raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();
        let chr_rom = raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec();
        let prg_rom_mirroring = prg_rom.len() == ROM::PRG_ROM_PAGE_SIZE;
        Ok(ROM { prg_rom, chr_rom, mapper, screen_mirroring, prg_rom_mirroring })
    }
}
