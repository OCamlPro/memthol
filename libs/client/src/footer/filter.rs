//! Filter section of the footer.

use crate::{base::*, filter::*};

/// Filter section messages.
#[derive(Clone, Debug)]
pub enum FooterFilterMsg {
    /// Adds a new filter.
    AddFilter(FilterKind),
}

/// Top-type for the filter section.
pub struct FilterFooter {
    /// The filters under construction.
    filters: Vec<Filter>,
}

impl FilterFooter {
    /// Constructor.
    pub fn new() -> Self {
        use filter::label::LabelSpec;
        let test_filter_1: LabelSpec = "label 1".into();
        let test_filter_2: LabelSpec = Regex::new("^set.*").unwrap().into();
        let label_filter_1 = LabelFilter::contains(vec![test_filter_1, test_filter_2]);
        let test_filter_1: LabelSpec = Regex::new("^list.*").unwrap().into();
        let test_filter_2: LabelSpec = "label 7".into();
        let label_filter_2 = LabelFilter::contains(vec![test_filter_1, test_filter_2]);
        Self {
            filters: vec![label_filter_1.into(), label_filter_2.into()],
        }
    }

    /// Handles a message.
    pub fn update(&mut self, msg: FooterFilterMsg) -> ShouldRender {
        use FooterFilterMsg::*;
        match msg {
            AddFilter(kind) => {
                self.filters.insert(0, kind.into());
                true
            }
        }
    }

    /// Renders itself.
    pub fn render(&self) -> Html {
        html! {
            <>
                { for self.filters.iter().map(
                    |filter| filter.render()
                ) }
            </>
        }
    }
}
