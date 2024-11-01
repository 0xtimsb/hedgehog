use iced::{
    widget::{button, row, text_input},
    Element, Task,
};
use log::debug;

use crate::utils::debounce::DebouncedInput;

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
    validation_handle: Option<iced::task::Handle>,
}

impl Default for UrlInput {
    fn default() -> Self {
        Self {
            value: String::new(),
            content_type: None,
            debouncer: DebouncedInput::new(500),
            is_validating: false,
            validation_handle: None,
        }
    }
}

impl UrlInput {
    pub fn update(&mut self, message: UrlInputMessage) -> Task<UrlInputMessage> {
        match message {
            UrlInputMessage::Edit(url) => {
                // cancels previous validation request if it is still in progress (e.g. super slow network)
                if let Some(handle) = self.validation_handle.take() {
                    handle.abort();
                }
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
                let (task, handle) = Task::abortable(Task::future(async move {
                    crate::utils::http::get_downloadable_content_type(&url).await
                }));
                // store handle as we might need to cancel it later if user starts typing again before validation is complete
                self.validation_handle = Some(handle);
                task.map(UrlInputMessage::Validated)
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
                        .then_some(UrlInputMessage::Add),
                )
            }
        ]
        .into()
    }
}
