//! Progress-bar rendering.
//!
//! This is used when the server is still parsing the dumps.

prelude! {}

/// Renders the progress bar.
///
/// This is used when the server is still parsing the dumps.
pub fn render(info: &LoadInfo) -> Html {
    define_style! {
        BIG = {
            font_size(180%),
        };
        PROGRESS = {
            width(70%),
        };
    }

    let percent = info.percent();

    html! {
        <center
            style = BIG
        >
            <br/>
            <div>
                {"Please wait, memthol is loading..."}
            </div>
            <br/>
            <div>
                {format!(
                    "{} / {}",
                    info.loaded, info.total,
                )}
            </div>
            <br/>
            <progress
                value = percent
                max = 100
                style = PROGRESS
            >
                { format!("{}%", percent) }
            </progress>
        </center>
    }
}
