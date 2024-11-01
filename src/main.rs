use download_item::DownloadItem;
use iced::{
    widget::{column, row},
    Element, Task,
};
use ui::url_input::{UrlInput, UrlInputMessage};

mod download_item;
mod ui;
mod utils;

#[derive(Default)]
struct AppState {
    download_items: Vec<DownloadItem>,
    url_input: UrlInput,
}

#[derive(Debug, Clone)]
enum AppMessage {
    UrlInput(UrlInputMessage),
    DownloadItem(usize, download_item::DownloadMessage),
}

impl AppState {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::UrlInput(url_msg) => match url_msg {
                UrlInputMessage::Add => {
                    if !self.url_input.value.is_empty() {
                        let new_item = DownloadItem::new(self.url_input.value.clone());
                        self.download_items.push(new_item);
                        self.url_input.value.clear();
                    }
                    Task::none()
                }
                _ => self.url_input.update(url_msg).map(AppMessage::UrlInput),
            },
            AppMessage::DownloadItem(index, download_message) => {
                if let Some(item) = self.download_items.get_mut(index) {
                    item.update(download_message);
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<AppMessage> {
        let downloads = self
            .download_items
            .iter()
            .enumerate()
            .map(|(i, item)| item.view().map(move |msg| AppMessage::DownloadItem(i, msg)));
        row![
            self.url_input.view().map(AppMessage::UrlInput),
            column(downloads).spacing(10),
        ]
        .spacing(10)
        .into()
    }
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    iced::application("Hedgehog", AppState::update, AppState::view)
        .run()
        .unwrap();
}
