use anyhow::{bail, Context, Result};
use core::str;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::contracts::{ImageStreamReader, VideoReader, VideoWriter};

pub struct CommandlineWriter {
    ffmpeg: Child,
    stdin: ChildStdin,
}

impl CommandlineWriter {
    pub fn new(video_path: &Path, frame_rate: u8) -> Result<Self> {
        if Path::new(&video_path).exists() {
            bail!("File already exists!");
        }

        let mut ffmpeg = Command::new("ffmpeg")
            .args(&[
                "-f",
                "image2pipe",
                "-vcodec",
                "png",
                "-r",
                &frame_rate.to_string(),
                "-i",
                "-",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                video_path.as_os_str().to_str().unwrap(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start FFmpeg")?;
        let stdin = ffmpeg.stdin.take().context("Failed to get stdin")?;

        Ok(Self { ffmpeg, stdin })
    }
}

impl VideoWriter for CommandlineWriter {
    fn write(&mut self, buffer: &[u8]) -> Result<()> {
        self.stdin
            .write_all(&buffer)
            .context("Failed to send data to FFmpeg")?;

        Ok(())
    }

    fn finish(mut self) -> Result<()> {
        drop(self.stdin);

        let _ = self.ffmpeg.wait()?;
        Ok(())
    }
}

pub struct CommandlineReader<I>
where
    I: ImageStreamReader,
{
    qr_file_path: PathBuf,
    ffmpeg: Child,
    stdout: BufReader<ChildStdout>,
    img_reader: I,
}

impl<I> CommandlineReader<I>
where
    I: ImageStreamReader,
{
    pub fn new(qr_file_path: PathBuf, frame_rate: u8, image_stream_reader: I) -> Result<Self> {
        if !Path::new(&qr_file_path).exists() {
            bail!("Qr video file doesn't exist '{:?}'", qr_file_path);
        }

        let mut ffmpeg = Command::new("ffmpeg")
            .args(&[
                "-i",
                qr_file_path.to_str().unwrap(),
                "-vf",
                &format!("fps={}", frame_rate),
                "-f",
                "image2pipe",
                "-pix_fmt",
                "gray",
                "-vcodec",
                "png",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start ffmpeg")?;

        let stdout = ffmpeg.stdout.take().context("Failed to get stdout")?;

        Ok(Self {
            qr_file_path,
            ffmpeg,
            stdout: BufReader::new(stdout),
            img_reader: image_stream_reader,
        })
    }
}

impl<I> VideoReader for CommandlineReader<I>
where
    I: ImageStreamReader,
{
    fn get_frame_count(&self) -> usize {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-count_frames",
                "-show_entries",
                "stream=nb_read_frames",
                "-of",
                "default=nokey=1:noprint_wrappers=1",
                self.qr_file_path.to_str().unwrap(),
            ])
            .output()
            .expect("File should exists, because it is already checked");

        let stdout = str::from_utf8(&output.stdout)
            .expect("ffprobe should always give an output")
            .trim();
        let frame_count = stdout
            .parse::<usize>()
            .expect("The frame count should always be a number");

        frame_count
    }

    fn read(&mut self, buffer: &mut Vec<u8>) -> Result<usize> {
        let bytes_read = self.img_reader.extract_image(&mut self.stdout, buffer)?;
        Ok(bytes_read)
    }

    fn finish(mut self) -> Result<()> {
        drop(self.stdout);
        let _ = self.ffmpeg.wait()?;
        Ok(())
    }
}
