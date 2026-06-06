use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct DownloadProgress {
    pub multi_progress: Arc<MultiProgress>,
    pub album_progress: Option<ProgressBar>,
    pub track_progress: Vec<ProgressBar>,
    pub show_progress: bool,
    pub colored_output: bool,
}

impl DownloadProgress {
    pub fn new(show_progress: bool, colored_output: bool) -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            album_progress: None,
            track_progress: Vec::new(),
            show_progress,
            colored_output,
        }
    }

    pub fn create_album_progress(&mut self, total_tracks: u32, album_title: &str) {
        if !self.show_progress {
            return;
        }

        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-");

        let pb = self.multi_progress.add(
            ProgressBar::new(total_tracks.into())
                .with_style(style)
                .with_message(format!("Downloading: {}", album_title)),
        );

        self.album_progress = Some(pb);
    }

    pub fn update_album_progress(&mut self, completed: u32, track_title: &str) {
        if let Some(pb) = &self.album_progress {
            pb.set_position(completed.into());
            pb.set_message(format!("Downloading: {}", track_title));
        }
    }

    pub fn create_track_progress(
        &mut self,
        track_title: &str,
        file_size: Option<u64>,
    ) -> std::io::Result<ProgressBar> {
        if !self.show_progress {
            return Ok(ProgressBar::hidden());
        }

        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-");

        let pb = if let Some(size) = file_size {
            ProgressBar::new(size)
        } else {
            ProgressBar::new_spinner()
        };

        pb.set_style(style);
        pb.set_message(track_title.to_string());

        let progress_bar = self.multi_progress.add(pb);
        self.track_progress.push(progress_bar.clone());

        Ok(progress_bar)
    }

    pub fn finish_track(&mut self, track_pb: &ProgressBar) {
        track_pb.finish_with_message("✓ Complete");
    }

    pub fn finish_album(&mut self) {
        if let Some(pb) = &self.album_progress {
            pb.finish_with_message("✓ Album complete");
        }
        self.album_progress = None;
        self.track_progress.clear();
    }

    pub fn print_info(&self, message: &str) {
        if self.colored_output {
            println!("{}", message.blue());
        } else {
            println!("{}", message);
        }
    }

    pub fn print_success(&self, message: &str) {
        if self.colored_output {
            println!("{}", message.green());
        } else {
            println!("{}", message);
        }
    }

    pub fn print_warning(&self, message: &str) {
        if self.colored_output {
            eprintln!("{}", message.yellow());
        } else {
            eprintln!("{}", message);
        }
    }

    pub fn print_error(&self, message: &str) {
        if self.colored_output {
            eprintln!("{}", message.red());
        } else {
            eprintln!("{}", message);
        }
    }
}

pub fn format_download_speed(bytes_per_sec: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s"];
    let mut size = bytes_per_sec;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn format_duration(seconds: f64) -> String {
    let seconds = seconds as u64;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}
