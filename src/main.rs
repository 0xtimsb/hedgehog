use download_item::DownloadItem;
use iced::{
    widget::{button, column, row, text_input},
    Element,
};

mod download_item;

#[derive(Default)]
struct AppState {
    download_items: Vec<DownloadItem>,
    url_input: String,
}

#[derive(Debug, Clone)]
enum AppMessage {
    AddUrl,
    EditUrl(String),
    DownloadItem(usize, download_item::DownloadMessage),
}

impl AppState {
    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::AddUrl => {
                if !self.url_input.is_empty() {
                    let new_item = DownloadItem::new(self.url_input.clone());
                    self.download_items.push(new_item);
                    self.url_input.clear();
                }
            }
            AppMessage::EditUrl(url) => {
                self.url_input = url;
            }
            AppMessage::DownloadItem(index, download_message) => {
                if let Some(item) = self.download_items.get_mut(index) {
                    item.update(download_message);
                }
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
            column![
                text_input("Enter URL...", &self.url_input).on_input(AppMessage::EditUrl),
                button("Add").on_press(AppMessage::AddUrl),
            ],
            column(downloads).spacing(10),
        ]
        .spacing(10)
        .into()
    }
}

pub fn main() -> iced::Result {
    iced::run("Hedgehog", AppState::update, AppState::view)
}
