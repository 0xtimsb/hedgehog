use iced::Subscription;
use iced::{
    widget::{button, column, text},
    Element, Task,
};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum DownloadStatus {
    #[default]
    Pending,
    InProgress {
        progress: f32,
        downloaded_bytes: u64,
    },
    Completed,
    Cancelled,
    Failed(String),
}

impl Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadStatus::Pending => write!(f, "Pending"),
            DownloadStatus::InProgress {
                progress,
                downloaded_bytes,
            } => {
                let size = format_bytes(*downloaded_bytes);
                write!(f, "InProgress:{}%:{}", progress, size)
            }
            DownloadStatus::Completed => write!(f, "Completed"),
            DownloadStatus::Failed(msg) => write!(f, "Failed: {}", msg),
            DownloadStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Default, Clone)]
pub struct DownloadItem {
    pub id: i64,
    pub url: String,
    pub file_path: String,
    pub total_size: Option<i64>,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    StartDownload,
    UpdateProgress(f32, u64),
    CompleteDownload,
    CancelDownload,
    FailDownload(String),
}

pub mod download {
    use futures::StreamExt;
    use std::fmt;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    use super::DownloadItem;

    #[derive(Debug, Clone)]
    pub enum Progress {
        Started,
        Advanced(f32, u64),
        Finished,
    }

    #[derive(Debug, Clone)]
    pub enum Error {
        DownloadError(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::DownloadError(msg) => write!(f, "Download error: {}", msg),
            }
        }
    }

    pub fn file(
        id: i64,
        url: String,
        downloaded_bytes: u64,
    ) -> iced::Subscription<(i64, Result<Progress, Error>)> {
        iced::Subscription::run_with_id(
            (std::any::TypeId::of::<DownloadItem>(), id, url.clone()),
            create_download_stream(id, url, downloaded_bytes),
        )
    }

    enum State {
        Ready(i64, String, u64),
        InitDownload {
            id: i64,
            url: String,
            file_path: String,
            total_size: u64,
        },
        Downloading {
            id: i64,
            url: String,
            file_path: String,
            total_size: u64,
            downloaded: u64,
            file: File,
            response_stream: reqwest::Response,
        },
        Finished,
    }

    fn create_download_stream(
        id: i64,
        url: String,
        resume_from: u64,
    ) -> impl futures::Stream<Item = (i64, Result<Progress, Error>)> {
        futures::stream::unfold(
            State::Ready(id, url, resume_from),
            move |state| async move {
                match state {
                    State::Ready(id, url, resume_from) => {
                        if let Err(e) = tokio::fs::create_dir_all("downloads").await {
                            return Some((
                                (id, Err(Error::DownloadError(e.to_string()))),
                                State::Finished,
                            ));
                        }

                        let client = reqwest::Client::new();
                        let mut request = client.get(&url);
                        if resume_from > 0 {
                            request = request.header("Range", format!("bytes={}-", resume_from));
                        }

                        match request.send().await {
                            Ok(response) => {
                                if resume_from > 0
                                    && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
                                {
                                    return Some((
                                        (
                                            id,
                                            Err(Error::DownloadError(
                                                "Server doesn't support resume".to_string(),
                                            )),
                                        ),
                                        State::Finished,
                                    ));
                                }

                                let total_size = if resume_from > 0 {
                                    response
                                        .headers()
                                        .get(reqwest::header::CONTENT_RANGE)
                                        .and_then(|ct_range| ct_range.to_str().ok())
                                        .and_then(|ct_range| {
                                            ct_range
                                                .split('/')
                                                .last()
                                                .and_then(|size| size.parse::<u64>().ok())
                                        })
                                        .unwrap_or(0)
                                } else {
                                    response
                                        .headers()
                                        .get(reqwest::header::CONTENT_LENGTH)
                                        .and_then(|ct_len| ct_len.to_str().ok())
                                        .and_then(|ct_len| ct_len.parse::<u64>().ok())
                                        .unwrap_or(0)
                                };

                                let file_name = url.split('/').last().unwrap_or("download");
                                let file_path = format!("downloads/{}", file_name);

                                Some((
                                    (id, Ok(Progress::Started)),
                                    State::InitDownload {
                                        id,
                                        url,
                                        file_path,
                                        total_size,
                                    },
                                ))
                            }
                            Err(e) => Some((
                                (id, Err(Error::DownloadError(e.to_string()))),
                                State::Finished,
                            )),
                        }
                    }
                    State::InitDownload {
                        id,
                        url,
                        file_path,
                        total_size,
                    } => {
                        let client = reqwest::Client::new();
                        match client.get(&url).send().await {
                            Ok(response_stream) => match File::options()
                                .write(true)
                                .create(true)
                                .append(resume_from > 0)
                                .open(&file_path)
                                .await
                            {
                                Ok(file) => Some((
                                    (id, Ok(Progress::Advanced(0.0, resume_from))),
                                    State::Downloading {
                                        id,
                                        url,
                                        file_path,
                                        total_size,
                                        downloaded: resume_from,
                                        file,
                                        response_stream,
                                    },
                                )),
                                Err(e) => Some((
                                    (id, Err(Error::DownloadError(e.to_string()))),
                                    State::Finished,
                                )),
                            },
                            Err(e) => Some((
                                (id, Err(Error::DownloadError(e.to_string()))),
                                State::Finished,
                            )),
                        }
                    }
                    State::Downloading {
                        id,
                        url,
                        file_path,
                        total_size,
                        downloaded,
                        mut file,
                        mut response_stream,
                    } => {
                        let mut bytes = Vec::new();
                        match response_stream.chunk().await {
                            Ok(Some(chunk)) => {
                                bytes.extend_from_slice(&chunk);
                                if let Err(e) = file.write_all(&bytes).await {
                                    return Some((
                                        (id, Err(Error::DownloadError(e.to_string()))),
                                        State::Finished,
                                    ));
                                }

                                let new_downloaded = downloaded + bytes.len() as u64;
                                let progress = (new_downloaded as f32 / total_size as f32) * 100.0;

                                Some((
                                    (id, Ok(Progress::Advanced(progress, new_downloaded))),
                                    State::Downloading {
                                        id,
                                        url,
                                        file_path,
                                        total_size,
                                        downloaded: new_downloaded,
                                        file,
                                        response_stream,
                                    },
                                ))
                            }
                            Ok(None) => Some(((id, Ok(Progress::Finished)), State::Finished)),
                            Err(e) => Some((
                                (id, Err(Error::DownloadError(e.to_string()))),
                                State::Finished,
                            )),
                        }
                    }
                    State::Finished => None,
                }
            },
        )
    }
}

impl DownloadItem {
    pub fn new(url: String) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self {
            id,
            status: DownloadStatus::default(),
            url,
            file_path: String::new(),
            total_size: None,
        }
    }

    pub fn update(&mut self, message: DownloadMessage) -> Task<DownloadMessage> {
        match message {
            DownloadMessage::StartDownload => {
                let downloaded_bytes = match self.status {
                    DownloadStatus::InProgress {
                        downloaded_bytes, ..
                    } => downloaded_bytes,
                    _ => 0,
                };

                self.status = DownloadStatus::InProgress {
                    progress: 0.0,
                    downloaded_bytes,
                };
                Task::none()
            }
            DownloadMessage::UpdateProgress(progress, bytes) => {
                if let DownloadStatus::InProgress {
                    progress: _,
                    downloaded_bytes: _,
                } = self.status
                {
                    self.status = DownloadStatus::InProgress {
                        progress,
                        downloaded_bytes: bytes,
                    };
                } else {
                    self.status = DownloadStatus::InProgress {
                        progress,
                        downloaded_bytes: bytes,
                    };
                }
                Task::none()
            }
            DownloadMessage::CompleteDownload => {
                self.status = DownloadStatus::Completed;
                Task::none()
            }
            DownloadMessage::CancelDownload => {
                self.status = DownloadStatus::Cancelled;
                Task::none()
            }
            DownloadMessage::FailDownload(msg) => {
                self.status = DownloadStatus::Failed(msg);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<DownloadMessage> {
        let status_text = match &self.status {
            DownloadStatus::InProgress {
                progress,
                downloaded_bytes,
            } => {
                let size = format_bytes(*downloaded_bytes);
                format!("Downloading: {:.1}% ({})", progress, size)
            }
            _ => self.status.to_string(),
        };

        column![
            text(&self.url),
            button("start download").on_press(DownloadMessage::StartDownload),
            text(status_text),
        ]
        .into()
    }

    pub fn subscription(&self) -> Subscription<(i64, Result<download::Progress, download::Error>)> {
        match self.status {
            DownloadStatus::InProgress {
                downloaded_bytes, ..
            } => download::file(self.id, self.url.clone(), downloaded_bytes),
            _ => Subscription::none(),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
