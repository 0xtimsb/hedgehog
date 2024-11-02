use download_item::{download, DownloadItem, DownloadMessage, DownloadStatus};
use iced::{
    clipboard,
    widget::{button, column, container},
    Element, Task,
};
use rusqlite::{Connection, Result};
use ui::modal::modal;
use ui::url_input::{UrlInput, UrlInputMessage};

mod db;
mod download_item;
mod ui;
mod utils;

#[derive(Default)]
struct AppState {
    download_items: Vec<DownloadItem>,
    url_input: UrlInput,
    show_modal: bool,
}

#[derive(Debug, Clone)]
enum AppMessage {
    UrlInput(UrlInputMessage),
    DownloadItem(usize, download_item::DownloadMessage),
    ShowModal,
    HideModal,
}

impl AppState {
    pub fn new(conn: &Connection) -> Result<Self> {
        let mut downloads = db::load_downloads(conn)?;
        downloads.iter_mut().for_each(|item| {
            if matches!(item.status, DownloadStatus::InProgress { .. }) {
                let _ = item.update(DownloadMessage::StartDownload);
            }
        });

        Ok(Self {
            download_items: downloads,
            url_input: UrlInput::default(),
            show_modal: false,
        })
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::ShowModal => {
                self.show_modal = true;
                clipboard::read()
                    .map(|content| AppMessage::UrlInput(UrlInputMessage::ClipboardContent(content)))
            }
            AppMessage::HideModal => {
                self.show_modal = false;
                self.url_input.value.clear();
                Task::none()
            }
            AppMessage::UrlInput(url_msg) => match url_msg {
                UrlInputMessage::Add => {
                    let mut new_item = DownloadItem::new(self.url_input.value.clone());
                    let _ = new_item.update(download_item::DownloadMessage::StartDownload);

                    // Save to database
                    if let Ok(conn) = Connection::open("downloads.db") {
                        let _ = db::save_download(&conn, &new_item);
                    }

                    self.download_items.push(new_item);
                    self.url_input.value.clear();
                    self.show_modal = false;
                    Task::none()
                }
                _ => self.url_input.update(url_msg).map(AppMessage::UrlInput),
            },
            AppMessage::DownloadItem(index, download_message) => {
                if let Some(item) = self.download_items.get_mut(index) {
                    let _ = item.update(download_message);
                    // Update database when download status changes
                    if let Ok(conn) = Connection::open("downloads.db") {
                        let _ = db::save_download(&conn, item);
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<AppMessage> {
        let body = column![
            button("Add Download").on_press(AppMessage::ShowModal),
            column(
                self.download_items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| item.view().map(move |msg| AppMessage::DownloadItem(i, msg)))
            )
            .spacing(10),
        ]
        .spacing(10);

        if self.show_modal {
            let url_input = container(self.url_input.view().map(AppMessage::UrlInput));
            modal(body, url_input, AppMessage::HideModal)
        } else {
            body.into()
        }
    }

    pub fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::batch(self.download_items.iter().enumerate().map(|(i, item)| {
            let download_sub = item.subscription();
            download_sub.with(i).map(|(i, progress)| {
                let msg = match progress.1 {
                    Ok(download::Progress::Advanced(progress, bytes)) => {
                        DownloadMessage::UpdateProgress(progress, bytes)
                    }
                    Ok(download::Progress::Finished) => DownloadMessage::CompleteDownload,
                    Err(e) => DownloadMessage::FailDownload(e.to_string()),
                    _ => DownloadMessage::UpdateProgress(0.0, 0),
                };
                AppMessage::DownloadItem(i, msg)
            })
        }))
    }
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let conn = db::init_db().expect("Failed to initialize database");

    iced::application("Hedgehog", AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .run_with(move || {
            let state = AppState::new(&conn).expect("Failed to initialize application state");
            (state, Task::none())
        })
        .unwrap();
}
