//! Filter section of the footer.

use crate::{base::*, filter::*};

/// Filter section messages.
#[derive(Clone, Debug)]
pub enum FooterFilterMsg {
    Update { index: usize, filter: Filter },
    Add { filter: Filter },
    Rm { index: usize },
}
impl FooterFilterMsg {
    pub fn update(index: usize, filter: Filter) -> Msg {
        footer::FooterMsg::filter(Self::Update { index, filter })
    }
    pub fn add(filter: Filter) -> Msg {
        footer::FooterMsg::filter(Self::Add { filter })
    }
    pub fn rm(index: usize) -> Msg {
        footer::FooterMsg::filter(Self::Rm { index })
    }
}

/// Top-type for the filter section.
pub struct FilterFooter {
    /// Filters previously constructed.
    filters: Vec<Filter>,
}

impl FilterFooter {
    /// Constructor.
    pub fn new() -> Self {
        use filter::label::LabelSpec;
        let mut filters = vec![];

        let test_filter_1: LabelSpec = "label 1".into();
        let test_filter_2: LabelSpec = Regex::new("^set.*").unwrap().into();
        filters.push(LabelFilter::contain(vec![test_filter_1, test_filter_2]).into());

        let test_filter_1: LabelSpec = Regex::new("^list.*").unwrap().into();
        let test_filter_2: LabelSpec = "label 7".into();
        filters.push(LabelFilter::contain(vec![test_filter_1, test_filter_2]).into());

        filters.push(SizeFilter::between(17, 42).unwrap().into());

        Self {
            // filter_edit: FilterEdit::of(Filter::default()),
            filters,
        }
    }

    /// Handles a message.
    pub fn update(&mut self, msg: FooterFilterMsg) -> ShouldRender {
        use FooterFilterMsg::*;
        match msg {
            Update { index, filter } => {
                info!("updating filter #{} to {:?}", index, filter);
                self.filters[index] = filter;
                true
            }
            Add { filter } => {
                self.filters.push(filter);
                true
            }
            Rm { index } => {
                self.filters.remove(index);
                true
            }
        }
    }

    /// Renders itself.
    pub fn render(&self) -> Html {
        html! {
            <>
                // { self.filter_edit.render() }
                { for self.filters.iter().enumerate().map(
                    |(index, filter)| html! {
                        <ul class=style::class::filter::LINE>
                            <li class=style::class::filter::BUTTONS>
                                { buttons::close(move |_|
                                    FooterFilterMsg::rm(index)
                                ) }
                            </li>
                            { filter.render(
                                move |filter_opt| match filter_opt {
                                    Ok(filter) => FooterFilterMsg::update(index, filter),
                                    Err(e) => Msg::err(e),
                                }
                            ) }
                        </ul>
                    }
                ) }
                <ul class=style::class::filter::LINE>
                    <li class=style::class::filter::BUTTONS>
                        { buttons::add(|_|
                            FooterFilterMsg::add(Filter::default())
                        ) }
                    </li>
                </ul>
            </>
        }
    }
}
