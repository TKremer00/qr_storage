use std::io::BufRead;

use anyhow::Result;

pub trait VideoWriter {
    fn write(&mut self, buffer: &[u8]) -> Result<()>;

    fn finish(self) -> Result<()>;
}

pub trait VideoReader {
    fn get_frame_count(&self) -> usize;

    fn read(&mut self, buffer: &mut Vec<u8>) -> Result<usize>;

    fn finish(self) -> Result<()>;
}

pub trait ImageStreamReader {
    fn extract_image<R: BufRead>(&self, reader: &mut R, buffer: &mut Vec<u8>) -> Result<usize>;
}

pub trait Progressbar {
    fn update(&mut self, position: usize);
}

pub trait QrCreater {
    fn max_buffer_len(&self) -> usize;
    fn create(&mut self, header: &[u8], buffer: &[u8], footer: &[u8]) -> Result<Vec<u8>>;
}

pub trait QrReader {
    fn max_buffer_len(&self) -> usize;
    fn read(&mut self, header: &[u8], buffer: &[u8], footer: &[u8]) -> Result<Vec<u8>>;
}

pub struct QrSettings {
    pub qr_version: u8,
    pub error_correction_level: u8,
}

impl QrSettings {
    pub fn max_len(&self) -> usize {
        let capacities: [[u16; 4]; 40] = [
            [17, 14, 11, 7],
            [32, 26, 20, 14],
            [53, 42, 32, 24],
            [78, 62, 46, 34],
            [106, 84, 60, 44],
            [134, 106, 74, 58],
            [154, 122, 86, 64],
            [192, 152, 108, 84],
            [230, 180, 130, 98],
            [271, 213, 151, 119],
            [321, 251, 177, 137],
            [367, 287, 203, 155],
            [425, 331, 241, 177],
            [458, 362, 258, 194],
            [520, 412, 292, 220],
            [586, 450, 322, 250],
            [644, 504, 364, 280],
            [718, 560, 394, 310],
            [792, 624, 442, 338],
            [858, 666, 482, 382],
            [929, 711, 509, 403],
            [1003, 779, 565, 439],
            [1091, 857, 611, 461],
            [1171, 911, 661, 511],
            [1273, 997, 715, 535],
            [1367, 1059, 751, 593],
            [1465, 1125, 805, 625],
            [1528, 1190, 868, 658],
            [1628, 1264, 908, 698],
            [1732, 1370, 982, 742],
            [1840, 1452, 1030, 790],
            [1952, 1538, 1112, 842],
            [2068, 1628, 1168, 898],
            [2188, 1722, 1228, 958],
            [2303, 1809, 1283, 983],
            [2431, 1911, 1351, 1051],
            [2563, 1989, 1423, 1093],
            [2699, 2099, 1499, 1139],
            [2809, 2213, 1579, 1219],
            [2953, 2331, 1663, 1273],
        ];

        capacities[(self.qr_version - 1) as usize][self.error_correction_level as usize] as usize
    }
}
