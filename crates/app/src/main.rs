mod accelerator;
mod app;
mod match_test;
mod message;
mod palette_commands;
mod screens;
mod selection;
mod shell;
mod theme;
mod widgets;

use app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update_and_dispatch, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}
