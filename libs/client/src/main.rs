use yew::App;

use client::{Model, Msg};

fn main() {
    web_logger::init();
    yew::initialize();
    App::<Model>::new().mount_to_body().send_message(Msg::Start);
    yew::run_loop();
}
