mod app;
mod match_test;
mod message;
mod screens;
mod selection;
mod shell;
mod theme;
mod widgets;

use app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}
