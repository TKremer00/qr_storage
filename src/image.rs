use crate::contracts::ImageStreamReader;
use anyhow::Result;

pub struct PngReader;

impl PngReader {
    pub fn new() -> Self {
        Self
    }
}

impl ImageStreamReader for PngReader {
    fn extract_image<R: std::io::BufRead>(
        &self,
        reader: &mut R,
        buffer: &mut Vec<u8>,
    ) -> Result<usize> {
        const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        const PNG_FOOTER: [u8; 8] = [0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]; // IEND + CRC

        let mut in_png = false;
        let mut bytes_read = 0;
        let mut temp_buffer = Vec::new();

        loop {
            let mut byte = [0u8; 1];
            if reader.read_exact(&mut byte).is_err() {
                break;
            }
            bytes_read = bytes_read + 1;
            temp_buffer.push(byte[0]);

            if !in_png && temp_buffer.ends_with(&PNG_HEADER) {
                buffer.clear(); // Start a new PNG
                buffer.extend_from_slice(&temp_buffer);
                temp_buffer.clear();
                in_png = true;
            } else if in_png {
                buffer.push(byte[0]);

                if buffer.ends_with(&PNG_FOOTER) {
                    break;
                }
            }
        }

        Ok(bytes_read)
    }
}
