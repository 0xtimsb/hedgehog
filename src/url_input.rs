use iced::{
    widget::{button, row, text_input},
    Element, Task,
};
use log::debug;

use crate::ui::debounced_input::DebouncedInput;

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
    debouncer: DebouncedInput<UrlInputMessage>,
    is_validating: bool,
}

impl Default for UrlInput {
    fn default() -> Self {
        Self {
            value: String::new(),
            content_type: None,
            debouncer: DebouncedInput::new(500),
            is_validating: false,
        }
    }
}

impl UrlInput {
    pub fn update(&mut self, message: UrlInputMessage) -> Task<UrlInputMessage> {
        match message {
            UrlInputMessage::Edit(url) => {
                self.value = url.clone();
                self.content_type = None;
                self.is_validating = true;
                self.debouncer
                    .debounce(UrlInputMessage::CheckValidation(url), |msg| msg)
            }
            UrlInputMessage::Validated(content_type) => {
                debug!("Validated content type: {:?}", content_type);
                self.content_type = content_type;
                self.is_validating = false;
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
        row![
            text_input("Enter URL...", &self.value).on_input(UrlInputMessage::Edit),
            if self.is_validating {
                button("Validating...").into()
            } else {
                button("Add").on_press_maybe(
                    (self.content_type.is_some() && !self.value.is_empty())
                        .then_some(UrlInputMessage::Add)
                )
            }
        ]
        .into()
    }
}
