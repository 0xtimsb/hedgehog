use download_item::DownloadItem;
use iced::{
    clipboard,
    widget::{button, column, container},
    Element, Task,
};
use ui::modal::modal;
use ui::url_input::{UrlInput, UrlInputMessage};

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
                    let new_item = DownloadItem::new(self.url_input.value.clone());
                    self.download_items.push(new_item);
                    self.url_input.value.clear();
                    self.show_modal = false;
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
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    iced::application("Hedgehog", AppState::update, AppState::view)
        .run()
        .unwrap();
}
