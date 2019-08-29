//! Basic types and stuffs.

pub use std::{collections::BTreeMap as Map, collections::BTreeSet as Set, fmt};

use lazy_static::lazy_static;

pub use log::{error, info, warn};

pub use stdweb::{js, Value as JsVal};

pub use yew::{html, services::websocket, Component, ComponentLink, Renderable, ShouldRender};

pub use alloc_data::{Alloc, Date as AllocDate, Diff as AllocDiff, SinceStart, Uid as AllocUid};

pub use crate::{chart, chart::ChartUid, cst, data::Storage, model::Model, msg, msg::Msg};

/// Retrieves the address and port of the server.
pub fn get_server_addr() -> (String, usize) {
    use stdweb::unstable::TryInto;
    let addr: String = js! {
        return serverAddr.get_addr();
    }
    .try_into()
    .expect("addr");
    let port: usize = js! {
        return serverAddr.get_port();
    }
    .try_into()
    .expect("port");
    (addr, port)
}

/// Type of HTML elements in the client.
pub type Html = yew::Html<Model>;

#[derive(Debug)]
pub struct DiffMsg {
    inner: yew::format::Text,
}
impl DiffMsg {
    pub fn destroy(self) -> yew::format::Text {
        self.inner
    }
}
impl From<yew::format::Binary> for DiffMsg {
    fn from(bin: yew::format::Binary) -> Self {
        let inner = match bin {
            Ok(bin) => String::from_utf8(bin).map_err(|e| e.into()),
            Err(e) => Err(e),
        };
        Self { inner }
    }
}
impl From<yew::format::Text> for DiffMsg {
    fn from(inner: yew::format::Text) -> Self {
        Self { inner }
    }
}

lazy_static! {
    /// Some lorem ipsum.
    pub static ref LOREM: Vec<&'static str> = vec![
        "\
In aliquam non metus non ullamcorper. Donec sit amet quam iaculis ex porta consequat pretium id
turpis. In pharetra eu lectus sed ultricies. Aenean eget dolor ante. Nam iaculis velit vitae est
posuere lobortis. Praesent quis risus a arcu malesuada tincidunt. Vestibulum non nunc mollis,
fringilla sem eu, pellentesque nulla. Vivamus orci purus, congue ut porta ac, sollicitudin eget
lectus. Aenean tempus, metus a volutpat lobortis, libero nibh rutrum justo, ut varius odio sem eu
leo. Fusce dui eros, tristique ac ex at, pharetra sagittis sapien.\
        ",
        "\
Phasellus quis erat eget tortor dapibus tristique finibus id elit. Maecenas gravida tortor at risus
convallis, ut aliquam purus viverra. Nulla iaculis efficitur eros, vel mollis velit. Maecenas a quam
a velit semper gravida lobortis vel tellus. Aenean porttitor magna vel fringilla faucibus.
Pellentesque tincidunt justo quis ligula aliquet, id lobortis dui tempus. Vestibulum at dolor ut
lacus tincidunt sagittis non ac massa. Cras sed dolor sagittis, luctus purus sed, faucibus metus.
Vestibulum sed velit nec felis ultrices vestibulum. Pellentesque feugiat volutpat nulla id accumsan.
Ut tristique pharetra ornare. Donec eget nisl eget ex consequat eleifend. Aliquam varius, metus sed
imperdiet pulvinar, turpis tellus vehicula nibh, quis aliquet libero mi vitae justo. Aenean ac diam
porttitor velit gravida posuere. Mauris tincidunt eget velit ut posuere. Fusce vel libero fringilla,
accumsan magna sit amet, interdum tortor.\
        ",
        "\
Proin eu justo eleifend, vehicula odio eu, consectetur purus. Aenean malesuada leo sed nisl
convallis dapibus. Pellentesque molestie quis velit nec commodo. Morbi non dapibus nulla. Nunc at
efficitur erat, sit amet suscipit mauris. Aliquam quis posuere ante. Fusce placerat nibh non ipsum
consectetur finibus. Proin id purus at eros hendrerit sagittis. Nullam sit amet tristique turpis.
Integer diam augue, interdum non elit non, dapibus convallis elit. Orci varius natoque penatibus et
magnis dis parturient montes, nascetur ridiculus mus. Donec mi mauris, pretium quis pellentesque
vel, cursus ac magna.\
        ",
        "\
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus ut dictum quam. Maecenas nec neque
vitae nulla finibus iaculis. Integer pretium dolor lectus. Sed fermentum purus at ligula cursus, sed
aliquam enim posuere. Integer a nulla augue. Duis elit nisl, porta ut lorem ac, consequat pharetra
risus. Phasellus turpis lacus, vestibulum eu auctor rhoncus, lobortis vel lectus. Nunc sed
pellentesque leo. Morbi nibh neque, rutrum eu leo quis, feugiat eleifend diam. Integer lacinia,
libero non sodales convallis, magna orci pellentesque lacus, ut pellentesque ligula enim in nunc.
Vestibulum accumsan imperdiet bibendum. Mauris tincidunt massa lacus, vel vulputate leo condimentum
consectetur. Pellentesque posuere convallis feugiat. Morbi fermentum et justo vel blandit. Nam neque
diam, auctor quis laoreet a, fringilla sed nulla.\
        ",
        "\
Nunc commodo mi quam. Morbi eu pretium nisi. Nullam non lorem non ipsum rhoncus pellentesque. Donec
tempor tristique dignissim. Mauris tempor, ligula ut malesuada fermentum, nulla tellus eleifend
libero, et dictum mauris arcu at nibh. Nulla condimentum porttitor odio, eu lobortis lorem. Donec
ipsum est, volutpat ac elementum quis, porttitor tempor ipsum. Morbi at felis est. Vestibulum
sagittis, ligula in scelerisque blandit, urna elit tincidunt est, at efficitur nulla justo at nunc.\
        ",
        "\
Nulla eleifend turpis sed ipsum aliquam pharetra. In nec sapien nec diam dapibus interdum non
rhoncus metus. Maecenas vestibulum vitae elit nec malesuada. Donec molestie auctor ligula eget
congue. Etiam feugiat eros vel ante blandit, non sollicitudin nisi consectetur. Nam sit amet dui
quis sem egestas posuere. Suspendisse justo enim, rutrum nec pellentesque ut, facilisis ac purus.
Nunc condimentum bibendum placerat. Sed vestibulum finibus felis quis commodo. Aliquam erat
volutpat. Fusce lectus elit, dictum quis risus in, ultrices elementum ligula. Nullam lacinia orci
eget nisi feugiat mattis. Vivamus malesuada et enim ac condimentum.\
        ",
    ];
}
