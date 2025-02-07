use clap::{command, Parser, Subcommand};
use contracts::{Progressbar, QrCreater, QrReader, QrSettings, VideoReader, VideoWriter};
use image::PngReader;
use indicator::Indicator;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use qr::{QrcodeCreater, QrcodeReader};
use video::{CommandlineReader, CommandlineWriter};

mod contracts;
mod image;
mod indicator;
mod qr;
mod video;

const DATA_HEADER: [u8; 4] = 0xABADBABEu32.to_be_bytes();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The version of the qr code
    #[arg(short, long)]
    qr_version: u8,

    /// The error correction level of the qr code
    #[arg(short, long)]
    ec_level: u8,

    /// The framerate of the video
    #[arg(short, long)]
    frame_rate: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Read from the file
    Read {
        /// the video with qr codes
        qr_video: PathBuf,

        /// the file to create
        output_file: PathBuf,
    },
    /// Write to the file
    Write {
        /// the file to convert to qr video
        input_file: PathBuf,

        /// The path to the qr video
        output_file: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let qr_settings = QrSettings {
        qr_version: args.qr_version,
        error_correction_level: args.ec_level,
    };

    match args.command {
        Commands::Read {
            qr_video,
            output_file,
        } => {
            qr_to_file(qr_video, &output_file, args.frame_rate, qr_settings)?;
        }
        Commands::Write {
            input_file,
            output_file,
        } => {
            let file = File::open(input_file)?;
            file_to_qr(file, &output_file, args.frame_rate, qr_settings)?;
        }
    };

    Ok(())
}

fn file_to_qr(file: File, output_name: &Path, frame_rate: u8, settings: QrSettings) -> Result<()> {
    let file_size = file.metadata()?.len();
    let reader = BufReader::new(file);
    let writer = CommandlineWriter::new(output_name, frame_rate)?;
    let qr_creater = QrcodeCreater::new(settings);
    let progressbar = Indicator::new(file_size as usize);

    file_to_qr_internal(reader, writer, qr_creater, progressbar)?;

    Ok(())
}

fn file_to_qr_internal<R, W, C, P>(
    mut reader: R,
    mut writer: W,
    mut qr_creater: C,
    mut progressbar: P,
) -> Result<()>
where
    R: BufRead,
    W: VideoWriter,
    C: QrCreater,
    P: Progressbar,
{
    let mut buffer = vec![0; qr_creater.max_buffer_len() - (DATA_HEADER.len() * 4)];
    while let Ok(bytes_read) = reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        progressbar.update(bytes_read);
        let qr_code = qr_creater.create(&DATA_HEADER, &buffer[..bytes_read], &DATA_HEADER)?;
        writer.write(&qr_code)?;
    }

    writer.finish()?;

    Ok(())
}

fn qr_to_file(
    qr_video: PathBuf,
    output_name: &Path,
    frame_rate: u8,
    settings: QrSettings,
) -> Result<()> {
    let image_stream_reader = PngReader::new();
    let reader = CommandlineReader::new(qr_video, frame_rate, image_stream_reader)?;
    let progressbar = Indicator::new(reader.get_frame_count());
    let qr_reader = QrcodeReader::new(settings);
    let output_file = File::create(output_name)?;

    qr_to_file_internal(reader, qr_reader, BufWriter::new(output_file), progressbar)?;

    Ok(())
}

fn qr_to_file_internal<VR, QR, W, P>(
    mut video_reader: VR,
    mut qr_reader: QR,
    mut writer: W,
    mut progressbar: P,
) -> Result<()>
where
    VR: VideoReader,
    QR: QrReader,
    W: Write,
    P: Progressbar,
{
    let mut buffer = vec![0; qr_reader.max_buffer_len()];
    while let Ok(bytes_read) = video_reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        let qr_buffer = qr_reader.read(&DATA_HEADER, &mut buffer[..bytes_read], &DATA_HEADER)?;

        progressbar.update(1);

        writer.write(&qr_buffer)?;
    }

    video_reader.finish()?;
    writer.flush()?;

    Ok(())
}
