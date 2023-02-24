use iced::executor;
use iced::{Application, Command, Element, Settings, Theme};

fn main() -> iced::Result {
    Hello::run(Settings::default())
}

struct Hello;

impl Application for Hello {
    type Executor = executor::Default;
    type Flags = ();
    type Message = ();
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Hello, Command::none())
    }

    fn title(&self) -> String {
        String::from("my first gui app")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        "Hello World!".into()
    }
}




