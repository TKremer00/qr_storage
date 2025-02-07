use anyhow::{bail, Context, Result};
use image::{GrayImage, ImageReader, Luma};
use qrcode::{bits::Bits, EcLevel, QrCode, Version};

use crate::contracts::{QrCreater, QrReader, QrSettings};

fn get_qr_version(qr_version: u8) -> Version {
    if qr_version < 4 {
        return Version::Micro(qr_version as i16);
    }

    Version::Normal(qr_version as i16)
}

fn get_ec_level(error_correction_level: u8) -> EcLevel {
    match error_correction_level {
        0 => EcLevel::L,
        1 => EcLevel::M,
        2 => EcLevel::Q,
        3 => EcLevel::H,
        _ => unreachable!("A error correction level should be between 0 and 3"),
    }
}

pub struct QrcodeCreater {
    settings: QrSettings,
}

impl QrcodeCreater {
    pub fn new(settings: QrSettings) -> Self {
        Self { settings }
    }
}

impl QrCreater for QrcodeCreater {
    fn max_buffer_len(&self) -> usize {
        self.settings.max_len()
    }

    fn create(&mut self, header: &[u8], buffer: &[u8], footer: &[u8]) -> Result<Vec<u8>> {
        let mut bits = Bits::new(get_qr_version(self.settings.qr_version));
        bits.push_byte_data(header)
            .context("The qr header buffer is to long")?;
        bits.push_byte_data(buffer)
            .context("The qr data buffer is to long")?;
        bits.push_byte_data(footer)
            .context("The qr footer buffer is to long")?;

        bits.push_terminator(get_ec_level(self.settings.error_correction_level))
            .context("Could not terminate the qr code")?;
        let qr_code = QrCode::with_bits(bits, get_ec_level(self.settings.error_correction_level))?;

        let image: GrayImage = qr_code.render::<Luma<u8>>().build();

        let mut qr_buffer = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut qr_buffer),
                image::ImageFormat::Png,
            )
            .expect("Failed to write image");
        Ok(qr_buffer)
    }
}

pub struct QrcodeReader {
    settings: QrSettings,
}

impl QrcodeReader {
    pub fn new(settings: QrSettings) -> Self {
        Self { settings }
    }
}

impl QrReader for QrcodeReader {
    fn max_buffer_len(&self) -> usize {
        self.settings.max_len()
    }

    fn read(&mut self, header: &[u8], buffer: &[u8], footer: &[u8]) -> Result<Vec<u8>> {
        let mut res = ImageReader::new(std::io::Cursor::new(&buffer));
        res.set_format(image::ImageFormat::Png);

        let res = res.decode();
        let img = res.context("decoding qr image to buffer failed")?;

        let gray_image = img.to_luma8();
        let mut qr_code = rqrr::PreparedImage::prepare(gray_image);
        let mut qr_buffer = vec![0; self.settings.max_len()];

        for code in qr_code.detect_grids() {
            if let Ok(_meta) = code.decode_to(&mut qr_buffer) {
                let trimmed_qr_buffer = qr_buffer
                    .windows(header.len())
                    .position(|w| w == header)
                    .map(|p| &qr_buffer[p + header.len()..])
                    .unwrap();

                let trimmed_qr_buffer = trimmed_qr_buffer
                    .windows(footer.len())
                    .position(|w| w == footer)
                    .map(|p| &trimmed_qr_buffer[..p])
                    .unwrap();

                return Ok(trimmed_qr_buffer.to_vec());
            }
        }

        bail!("Failed to decode the qr code")
    }
}
