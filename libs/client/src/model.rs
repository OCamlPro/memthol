//! Model of the client.

use yew::{services::websocket, Component, ComponentLink, Renderable, ShouldRender};

use crate::{base::*, top_tabs::TopTabs};

/// Model of the client.
pub struct Model {
    /// The top tabs.
    pub top_tabs: TopTabs,
    /// Component link.
    pub link: ComponentLink<Self>,
    // /// TCP stream.
    // pub stream: TcpStream,
    pub socket: websocket::WebSocketService,
    pub socket_task: Option<websocket::WebSocketTask>,
    count: usize,
    memory: Vec<AllocUid>,

    charts: chart::Charts,
}

impl Model {
    fn activate_ws(&mut self) {
        info! { "activating websocket" }
        debug_assert! { self.socket_task.is_none() }
        let (addr, port) = get_server_addr();
        let addr = format!("ws://{}:{}", addr, port + 1);
        let callback = self.link.send_back(|diff| Msg::Diff(diff));
        let notification = self.link.send_back(|_| Msg::Nop);
        info! { "creating websocket" }
        let task = self.socket.connect(&addr, callback, notification);
        info! { "done with websocket" }
        self.socket_task = Some(task)
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut model = Model {
            top_tabs: TopTabs::new(),
            link,
            socket: websocket::WebSocketService::new(),
            socket_task: None,
            count: 0,
            memory: vec![],

            charts: chart::Charts::new(),
        };
        model.activate_ws();
        model
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::JsInit => {
                info! { "js init" }
                false
            }
            Msg::Start => {
                info! { "start" }
                let _should_render = self.top_tabs.activate_default();
                self.charts.init_setup();
                true
            }
            Msg::ChangeTab(tab) => {
                info!("changing to tab {:?}", tab);
                self.top_tabs.activate(tab)
            }
            Msg::Diff(diff) => {
                info!("receiving diff..");
                let diff_str = diff
                    .destroy()
                    .expect("failed to receive new diff from server");
                let is_last = &diff_str[0..1] == "1";
                let diff_str = &diff_str[1..];
                let diff = AllocDiff::of_str(diff_str).expect("could not parse ill-formed diff");
                self.charts.add_diff(diff);
                if is_last {
                    self.charts.update()
                }
                false
            }
            Msg::UpdateD3(_) => {
                let (toc_s, toc_n) = (self.count * 15, self.count * 3574 % 103);
                let (tod_s, tod_n) = (toc_s + (self.count * 21), self.count * 2574 % 103);
                let toc = format!("{}.{}", toc_s, toc_n);
                let tod = format!("{}.{}", tod_s, tod_n);
                let alloc = if self.count % 2 == 0 {
                    let alloc = Alloc::of_str(&format!(
                        "{}: {}, {}, {}, {}, {}",
                        self.count * self.count * self.count * self.count % 2_000_000_000,
                        "Major",
                        match self.count * 11 % 4 {
                            0 => 7,
                            1 => 8,
                            2 => 16,
                            3 => 32,
                            _ => 4,
                        },
                        "[ blah/stuff/file.ml:325:7-38#3 file.ml:754230:1-3#11 ]",
                        toc,
                        tod
                    ))
                    .unwrap();
                    alloc
                } else {
                    let alloc = Alloc::of_str(&format!(
                        "{}: {}, {}, {}, {}",
                        self.count * self.count * self.count * self.count % 2_000_000_000,
                        "Major",
                        match self.count * 11 % 4 {
                            0 => 7,
                            1 => 8,
                            2 => 16,
                            3 => 32,
                            _ => 4,
                        },
                        "[ blah/stuff/file.ml:325:7-38#3 file.ml:754230:1-3#11 ]",
                        toc
                    ))
                    .unwrap();
                    self.memory.push(alloc.uid().clone());
                    alloc
                };
                self.charts.add_alloc(alloc);
                self.count += 1;
                false
            }
            Msg::Nop => {
                info!("received nop message");
                false
            }
        }
    }
}

impl Model {
    /// Renders the header (tabs).
    pub fn render_header(&self) -> Html {
        html! {
            <header class="header_style">
                {self.top_tabs.render()}
            </header>
        }
    }

    /// Renders the content.
    fn render_content(&self) -> Html {
        html! {
            <div class="h1_brush">
                <h1 class="h1"> {"Memthol"} </h1>
                { self.charts.render() }
                // <br/>
                // <br/>
                // <br/>
                // { self.d3_graph.render() }
            </div>
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html {
        html! {
            <div>
                { self.render_header() }
                { self.render_content() }
            </div>
        }
    }
}
