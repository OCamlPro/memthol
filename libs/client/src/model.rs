//! Model of the client.

use yew::{services::websocket, Component, ComponentLink, Renderable, ShouldRender};

use crate::{base::*, footer::Footer, top_tabs::TopTabs};

/// Model of the client.
pub struct Model {
    /// The top tabs.
    pub top_tabs: TopTabs,
    /// The footer.
    pub footer: Footer,
    /// Component link.
    pub link: ComponentLink<Self>,
    // /// TCP stream.
    // pub stream: TcpStream,
    pub socket: websocket::WebSocketService,
    pub socket_task: Option<websocket::WebSocketTask>,
    data: Option<Storage>,
    charts: chart::Charts,
}

impl Model {
    /// Activates the websocket to receive data from the server.
    pub fn activate_ws(&mut self) {
        debug_assert! { self.socket_task.is_none() }
        let (addr, port) = get_server_addr();
        let addr = format!("ws://{}:{}", addr, port + 1);
        let callback = self.link.send_back(|msg| Msg::FromServer(msg));
        let notification = self.link.send_back(|_| Msg::Nop);
        let task = self.socket.connect(&addr, callback, notification);
        self.socket_task = Some(task)
    }

    /// Retrieves the current data (mutable).
    ///
    /// Panics if `self.data` is not set.
    pub fn data_mut(&mut self) -> &mut Storage {
        if let Some(data) = &mut self.data {
            data
        } else {
            fail!("trying to access the allocation data while none is available")
        }
    }

    /// Retrieves the current data (immutable).
    ///
    /// Panics if `self.data` is not set.
    pub fn data(&self) -> &Storage {
        if let Some(data) = &self.data {
            data
        } else {
            fail!("trying to access the allocation data while none is available")
        }
    }

    /// Registers an initialization message.
    pub fn init(&mut self, init: alloc_data::Init) {
        let data = Storage::new(init, self.footer.get_filters_and_set_unedited());
        self.charts.init(&data);
        self.data = Some(data)
    }

    /// Registers a diff.
    pub fn add_diff(&mut self, diff: AllocDiff) {
        let _new_stuff = self.data_mut().add_diff(diff);
        // if new_stuff {
        let data = if let Some(data) = self.data.as_ref() {
            data
        } else {
            fail!("received allocation data before init message")
        };
        self.charts.update_data(data)
        // }
    }

    /// Handles a message from the server.
    pub fn handle_server_msg(&mut self, msg: Res<msg::from_server::Msg>) -> ShouldRender {
        let msg = msg.unwrap();
        info!(
            "received a message from the server:\n{}",
            msg.as_json().unwrap()
        );
        unimplemented!()
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|msg: Msg| msg);
        let mut model = Model {
            top_tabs: TopTabs::new(),
            footer: Footer::new(callback),
            link,
            socket: websocket::WebSocketService::new(),
            socket_task: None,
            data: None,
            charts: chart::Charts::new(),
        };
        model.activate_ws();
        model
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::Start => {
                let _should_render = self.top_tabs.activate_default();
                true
            }
            Msg::ChartsAction(msg) => {
                let render = self.charts.update(self.data.as_ref(), msg);
                if render {
                    self.link.send_self(msg::ChartsMsg::refresh())
                }
                render
            }
            Msg::FooterAction(msg) => self.footer.update(self.data.as_mut(), msg),
            Msg::ChangeTab(tab) => {
                warn!("[unimplemented] changing to tab {:?}", tab);
                self.top_tabs.activate(tab)
            }
            Msg::Blah(blah) => {
                info!("[message] {}", blah);
                false
            }

            Msg::Error(err) => {
                alert!("{}", err.pretty());
                true
            }

            Msg::Alarm(blah) => {
                alert!("{}", blah);
                true
            }

            Msg::FromServer(msg) => {
                let msg: Res<charts::msg::to_client::Msg> = msg.into();
                self.handle_server_msg(msg)
            }

            // Msg::Diff(diff) => {
            //     let txt = diff
            //         .destroy()
            //         .expect("failed to receive new diff from server");
            //     if txt.len() > "start".len() && &txt[0.."start".len()] == "start" {
            //         info!("receiving init...");
            //         let init = match alloc_data::Init::from_str(&txt) {
            //             Ok(init) => init,
            //             Err(e) => {
            //                 error!("Error:");
            //                 for line in e.pretty().lines() {
            //                     error!("{}", line)
            //                 }
            //                 fail!("could not parse ill-formed init")
            //             }
            //         };
            //         self.init(init);
            //         true
            //     } else {
            //         info!("receiving diff...");
            //         // let is_last = &txt[0..1] == "1";
            //         let diff_str = &txt[1..];
            //         let diff =
            //             AllocDiff::from_str(diff_str).expect("could not parse ill-formed diff");
            //         self.add_diff(diff);
            //         false
            //     }
            // }
            Msg::Nop => false,
        }
    }
}

impl Model {
    /// Renders the header (tabs).
    pub fn render_header(&self) -> Html {
        html! {
            <header>
                { self.top_tabs.render() }
            </header>
        }
    }

    /// Renders the content.
    pub fn render_content(&self) -> Html {
        html! {
            <div class=style::class::BODY>
                { self.charts.render() }
            </div>
        }
    }

    /// Renders the footer.
    pub fn render_footer(&self) -> Html {
        html! {
            <footer>
                { self.footer.render(self.data.as_ref()) }
            </footer>
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html {
        html! {
            <div class=style::class::FULL_BODY>
                { self.render_header() }
                { self.render_content() }
                { self.render_footer() }
            </div>
        }
    }
}
