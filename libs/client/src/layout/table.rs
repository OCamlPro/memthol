//! Factors code for layout tables.

prelude! {}

pub const LEFT_COL_WIDTH: usize = 15;
pub const RIGHT_COL_WIDTH: usize = 100 - LEFT_COL_WIDTH - 1;

define_style! {
    row_style! = {
        width(100%),
        block,
    };
    FIRST_ROW_STYLE = {
        extends(row_style),
    };
    NON_FIRST_ROW_STYLE = {
        extends(row_style),
        border(top, 2 px, white),
    };
    NON_FIRST_ROW_BLACK_STYLE = {
        extends(row_style),
        border(top, 2 px, black),
    };

    left_col! = {
        block,
        height(100%),
        width({LEFT_COL_WIDTH}%),
        text_align(center),
        float(left),
        padding(none),
        margin(none),
    };

    LEFT_COL = {
        extends(left_col),
        border(right, 2 px, white),
    };
    LEFT_COL_BLACK = {
        extends(left_col),
        border(right, 2 px, black),
    };
    RIGHT_COL = {
        block,
        height(100%),
        width({RIGHT_COL_WIDTH}%),
        float(left),
        padding(none),
        margin(none),
        overflow(auto),
    };
    table_cell_style! = {
        table cell,
        vertical_align(middle),
    };
}

define_style! {
    cell_style! = {
        height(100%),
        display(table),
        text_align(center),
    };
    VALUE_CONTAINER_STYLE = {
        extends(cell_style),
        width(min 10%),
        margin(0, auto),
    };
    TINY_VALUE_CONTAINER_STYLE = {
        extends(cell_style),
        width(3%),
    };
    SINGLE_VALUE_CONTAINER_STYLE = {
        extends(cell_style),
        width(100%),
    };
    SEP_CONTAINER_STYLE = {
        extends(cell_style),
        width(2%),
    };
    SELECTOR_CONTAINER_STYLE = {
        extends(cell_style),
        width(10%),
    };
    SINGLE_VALUE_WITH_SELECTOR_CONTAINER_STYLE = {
        extends(cell_style),
        width(90%)
    };

    value_cell_style! = {
        extends(table_cell_style),
        height(100%),
        width(auto),
        // width(min 15%),
    };
    CELL_STYLE = {
        extends(value_cell_style),
    };
    VALUE_STYLE = {
        extends(value_cell_style),
        // margin(0%, 1%),
    };
    SEP_STYLE = {
        extends(value_cell_style),
        // padding(0%, 1%),
        // width(5%),
        font(code),
    };
    ADD_STYLE = {
        // extends(value_cell_style),
        // width(5%),
        font(code),
        pointer,
    };

    SELECTOR_STYLE = {
        extends(value_cell_style),
        // margin(0%, 1%),
    };

    BUTTON_STYLE = {
        font(code),
        pointer,
    };
}

pub struct TableRow {
    lft: Html,
    rgt: SVec<Html>,
    height_px: usize,
    is_first: bool,
    white_sep: bool,
}

impl TableRow {
    fn new(is_first: bool, lft: Html) -> Self {
        let lft = html! {
            <div
                style = SINGLE_VALUE_CONTAINER_STYLE
            >
                {Self::new_cell(lft)}
            </div>
        };

        Self {
            lft,
            rgt: SVec::new(),
            height_px: 40,
            is_first,
            white_sep: true,
        }
    }

    pub fn new_menu(is_first: bool, lft: Html) -> Self {
        Self::new(is_first, lft)
    }

    pub fn black_sep(mut self) -> Self {
        self.white_sep = false;
        self
    }
    pub fn white_sep(mut self) -> Self {
        self.white_sep = true;
        self
    }
    pub fn height_px(mut self, height: usize) -> Self {
        self.height_px = height;
        self
    }

    pub fn render(self) -> Html {
        let style = if self.is_first {
            &*FIRST_ROW_STYLE
        } else {
            if self.white_sep {
                &*NON_FIRST_ROW_STYLE
            } else {
                &*NON_FIRST_ROW_BLACK_STYLE
            }
        };
        let style = inline_css! {
            extends_style(style),
            height({self.height_px} px),
        };

        let lft_style = if self.white_sep {
            &*LEFT_COL
        } else {
            &*LEFT_COL_BLACK
        };
        let rgt_style = &*RIGHT_COL;

        html! {
            <div
                style = style
            >
                <div
                    style = lft_style
                >
                    {self.lft}
                </div>
                <div
                    style = rgt_style
                >
                    { for self.rgt.into_iter() }
                </div>
            </div>
        }
    }

    fn new_cell(inner: Html) -> Html {
        html! {
            <div
                style = CELL_STYLE
            >
                {inner}
            </div>
        }
    }

    pub fn push_selector(&mut self, selector: Html) {
        self.rgt.push(html! {
            <div
                style = SELECTOR_CONTAINER_STYLE
            >
                {Self::new_cell(selector)}
            </div>
        })
    }
    pub fn push_sep(&mut self, sep: Html) {
        self.rgt.push(html! {
            <div
                style = SEP_CONTAINER_STYLE
            >
                {Self::new_cell(sep)}
            </div>
        })
    }
    pub fn push_value(&mut self, value: Html) {
        self.rgt.push(html! {
            <div
                style = VALUE_CONTAINER_STYLE
            >
                {Self::new_cell(value)}
            </div>
        })
    }
    pub fn push_tiny_value(&mut self, value: Html) {
        self.rgt.push(html! {
            <div
                style = TINY_VALUE_CONTAINER_STYLE
            >
                {Self::new_cell(value)}
            </div>
        })
    }
    pub fn push_single_value(&mut self, value: Html) {
        self.rgt.push(html! {
            <div
                style = SINGLE_VALUE_CONTAINER_STYLE
            >
                {Self::new_cell(value)}
            </div>
        })
    }

    pub fn push_single_selector_and_value(&mut self, selector: Html, value: Html) {
        self.rgt.push(html! {
            <div
                style = SELECTOR_CONTAINER_STYLE
            >
                {Self::new_cell(selector)}
            </div>
        });
        self.rgt.push(html! {
            <div
                style = VALUE_CONTAINER_STYLE
            >
                {Self::new_cell(value)}
            </div>
        })
    }

    pub fn push_button(&mut self, txt: &str, action: OnClickAction) {
        self.rgt.push(html! {
            <div
                style = SEP_CONTAINER_STYLE
            >
                {Self::new_cell(html! {
                    <div
                        style = BUTTON_STYLE
                        onclick = action
                    >
                        {txt}
                    </div>
                })}
            </div>
        })
    }
}
