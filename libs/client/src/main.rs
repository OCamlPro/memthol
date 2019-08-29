use yew::App;

use client::{Model, Msg};

fn main() {
    web_logger::init();
    yew::initialize();
    App::<Model>::new()
        .mount_to_body()
        .send_message(Msg::start());
    yew::run_loop();
}
