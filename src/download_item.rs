use iced::{
    widget::{button, column, text},
    Element,
};

use std::fmt::Display;

#[derive(Default)]
enum DownloadStatus {
    #[default]
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

impl Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadStatus::Pending => write!(f, "Pending"),
            DownloadStatus::Downloading => write!(f, "Downloading"),
            DownloadStatus::Completed => write!(f, "Completed"),
            DownloadStatus::Failed => write!(f, "Failed"),
            DownloadStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Default)]
pub struct DownloadItem {
    status: DownloadStatus,
    url: String,
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    ResumeDownload,
    PauseDownload,
    CancelDownload,
    CompleteDownload,
    FailDownload,
}

impl DownloadItem {
    pub fn new(url: String) -> Self {
        Self {
            status: DownloadStatus::default(),
            url,
        }
    }

    pub fn update(&mut self, message: DownloadMessage) {
        match message {
            DownloadMessage::ResumeDownload => {
                self.status = DownloadStatus::Downloading;
            }
            DownloadMessage::PauseDownload => {
                self.status = DownloadStatus::Pending;
            }
            DownloadMessage::CancelDownload => {
                self.status = DownloadStatus::Cancelled;
            }
            DownloadMessage::CompleteDownload => {
                self.status = DownloadStatus::Completed;
            }
            DownloadMessage::FailDownload => {
                self.status = DownloadStatus::Failed;
            }
        }
    }

    pub fn view(&self) -> Element<DownloadMessage> {
        column![
            text(&self.url),
            button("start download").on_press(DownloadMessage::ResumeDownload),
            text(self.status.to_string()),
        ]
        .into()
    }
}
