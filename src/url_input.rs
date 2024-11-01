use iced::{
    widget::{button, column, text_input},
    Element, Task,
};
use log::debug;

#[derive(Debug, Clone)]
pub enum UrlInputMessage {
    Edit(String),
    Add,
    Validated(Option<String>),
    CheckValidation(String),
}

pub struct UrlInput {
    pub value: String,
    pub content_type: Option<String>,
    pub validation_handle: Option<iced::task::Handle>,
}

impl Default for UrlInput {
    fn default() -> Self {
        Self {
            value: String::new(),
            content_type: None,
            validation_handle: None,
        }
    }
}

impl UrlInput {
    pub fn update(&mut self, message: UrlInputMessage) -> Task<UrlInputMessage> {
        match message {
            UrlInputMessage::Edit(url) => {
                self.value = url.clone();
                self.content_type = None;
                if let Some(handle) = &self.validation_handle {
                    handle.abort();
                }
                let (task, handle) = Task::abortable(Task::future(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    UrlInputMessage::CheckValidation(url)
                }));
                self.validation_handle = Some(handle);
                task
            }
            UrlInputMessage::Validated(content_type) => {
                debug!("Validated content type: {:?}", content_type);
                self.content_type = content_type;
                Task::none()
            }
            UrlInputMessage::CheckValidation(url) => {
                debug!("Checking validation for {}", url);
                Task::perform(
                    async move { crate::utils::get_downloadable_content_type(&url).await },
                    UrlInputMessage::Validated,
                )
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<UrlInputMessage> {
        column![
            text_input("Enter URL...", &self.value).on_input(UrlInputMessage::Edit),
            button("Add").on_press_maybe(
                (self.content_type.is_some() && !self.value.is_empty())
                    .then_some(UrlInputMessage::Add)
            ),
        ]
        .into()
    }
}
