use iced::Task;
use std::time::Duration;

pub struct DebouncedInput<T> {
    handle: Option<iced::task::Handle>,
    delay: Duration,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> DebouncedInput<T> {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            handle: None,
            delay: Duration::from_millis(delay_ms),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn debounce<F>(&mut self, value: T, f: F) -> Task<T>
    where
        T: Clone + Send,
        F: FnOnce(T) -> T + Send + 'static,
    {
        // cancels previous task if delay time is not over yet
        if let Some(handle) = &self.handle {
            handle.abort();
        }
        let delay = self.delay;
        let (task, handle) = Task::abortable(Task::future(async move {
            tokio::time::sleep(delay).await;
            f(value)
        }));
        // store handle as we might need to cancel it later if user starts typing again before delay is over
        self.handle = Some(handle);
        task
    }
}
