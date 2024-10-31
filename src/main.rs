use download_item::DownloadItem;
use iced::{
    widget::{button, column, row, text_input},
    Element, Task,
};
use log::debug;

mod download_item;
mod utils;

#[derive(Default)]
struct AppState {
    download_items: Vec<DownloadItem>,
    url_input: String,
    content_type: Option<String>,
}

#[derive(Debug, Clone)]
enum AppMessage {
    AddUrl,
    EditUrl(String),
    DownloadItem(usize, download_item::DownloadMessage),
    UrlValidated(Option<String>),
}

impl AppState {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::AddUrl => {
                if !self.url_input.is_empty() {
                    let new_item = DownloadItem::new(self.url_input.clone());
                    self.download_items.push(new_item);
                    self.url_input.clear();
                }
                Task::none()
            }
            AppMessage::EditUrl(url) => {
                self.url_input = url.clone();
                let url_to_validate = url.clone();
                Task::perform(
                    async move { utils::get_downloadable_content_type(&url_to_validate).await },
                    AppMessage::UrlValidated,
                )
            }
            AppMessage::UrlValidated(content_type) => {
                debug!("Content type: {:?}", content_type);
                self.content_type = content_type;
                Task::none()
            }
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
            column![
                text_input("Enter URL...", &self.url_input).on_input(AppMessage::EditUrl),
                button("Add").on_press_maybe(
                    (self.content_type.is_some() && !self.url_input.is_empty())
                        .then_some(AppMessage::AddUrl)
                ),
            ],
            column(downloads).spacing(10),
        ]
        .spacing(10)
        .into()
    }
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    iced::run("Hedgehog", AppState::update, AppState::view).unwrap();
}
