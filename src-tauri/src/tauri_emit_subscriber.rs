use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    level: String,
    message: String,
    module_path: String,
    timestamp: String,
}

pub struct TauriEmitSubscriber {
    pub app_handle: AppHandle,
}

struct MessageExtractVisitor {
    message: String,
    module_path: String,
}

impl tracing::field::Visit for MessageExtractVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else if field.name() == "log.module_path" {
            self.module_path = format!("{:?}", value);
        }
    }
}

impl<S> Layer<S> for TauriEmitSubscriber
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {

        let mut visitor = MessageExtractVisitor {
            message: String::new(),
            module_path: String::new()
        };

        event.record(&mut visitor);

        let now = chrono::Local::now();
        let event = Event {
            level: event.metadata().level().to_string(),
            message: visitor.message,
            module_path: format!("{}{}", event.metadata().module_path().unwrap_or_default(), visitor.module_path),
            timestamp: format!("{}-{}-{} {}:{}:{}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second()),
        };

        if self
            .app_handle
            .emit(
                "log-event",
                event
            )
            .is_err()
        {
            println!("App handle add log failed");
        }
    }
}