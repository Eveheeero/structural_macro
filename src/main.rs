mod hook;
pub mod winapi;
use iced::{
    widget::{button, text},
    Element,
};

pub fn main() -> iced::Result {
    let _ = unsafe { hook::Hook::new() };
    iced::application("Structural Macro", update, view).run()
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}
#[derive(Default)]
struct Counter {
    value: u64,
}

fn update(counter: &mut Counter, message: Message) {
    match message {
        Message::Increment => counter.value += 1,
    }
}

fn view(counter: &Counter) -> Element<Message> {
    button(text(counter.value))
        .on_press(Message::Increment)
        .into()
}
