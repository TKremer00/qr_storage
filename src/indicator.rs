use kdam::{tqdm, Bar, BarExt};

use crate::contracts::Progressbar;

pub struct Indicator {
    progress_bar: Bar,
}

impl Indicator {
    pub fn new(total_number_of_bytes: usize) -> Self {
        Self {
            progress_bar: tqdm!(total = total_number_of_bytes, force_refresh = true),
        }
    }
}

impl Progressbar for Indicator {
    fn update(&mut self, position: usize) {
        self.progress_bar.update(position).unwrap();
    }
}
