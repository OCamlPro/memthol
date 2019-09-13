//! Filter section of the footer.

use crate::{base::*, filter::*};

/// Filter section messages.
#[derive(Clone, Debug)]
pub enum FooterFilterMsg {
    /// Changes the filter at some index.
    Update { index: usize, filter: Filter },
    /// Adds a filter at the end of the filter list.
    Add { filter: Filter },
    /// Removes a filter at some index.
    Rm { index: usize },
    /// Cancels all filter edits.
    Revert,
    /// Activates all filters.
    Apply,
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
    pub fn revert() -> Msg {
        footer::FooterMsg::filter(Self::Revert)
    }
    pub fn apply() -> Msg {
        footer::FooterMsg::filter(Self::Apply)
    }
}

/// Top-type for the filter section.
pub struct FilterFooter {
    /// Filters, with a boolean indicating whether the filter have been changed.
    filters: Vec<(Filter, bool)>,
    /// Flag indicating Whether some filter has been removed.
    deleted: bool,
    /// Link to the model.
    model_callback: Callback<Msg>,
}

impl FilterFooter {
    /// Constructor.
    pub fn new(model_callback: Callback<Msg>) -> Self {
        // use filter::label::LabelSpec;
        let filters = vec![];
        let deleted = false;

        // let test_filter_1: LabelSpec = "label 1".into();
        // let test_filter_2: LabelSpec = Regex::new("^set.*").unwrap().into();
        // filters.push(LabelFilter::contain(vec![test_filter_1, test_filter_2]).into());

        // let test_filter_1: LabelSpec = Regex::new("^list.*").unwrap().into();
        // let test_filter_2: LabelSpec = "label 7".into();
        // filters.push(LabelFilter::contain(vec![test_filter_1, test_filter_2]).into());

        // filters.push(SizeFilter::between(17, 42).unwrap().into());

        Self {
            // filter_edit: FilterEdit::of(Filter::default()),
            filters,
            deleted,
            model_callback,
        }
    }

    /// True if the filters are edited in any way.
    pub fn is_edited(&self) -> bool {
        self.deleted || self.filters.iter().any(|(_, edited)| *edited)
    }

    /// Filter accessor.
    pub fn get_and_set_unedited(&mut self) -> Vec<Filter> {
        self.deleted = false;
        let mut res = Vec::with_capacity(self.filters.len());
        for (filter, edited) in &mut self.filters {
            res.push(filter.clone());
            *edited = false
        }
        res
    }

    /// Handles a message.
    pub fn update(&mut self, data: Option<&mut Storage>, msg: FooterFilterMsg) -> ShouldRender {
        use FooterFilterMsg::*;
        match msg {
            Update { index, filter } => {
                self.filters[index] = (filter, true);
                true
            }
            Add { filter } => {
                self.filters.push((filter, true));
                true
            }
            Rm { index } => {
                self.filters.remove(index);
                self.deleted = true;
                true
            }
            Revert => match data {
                Some(data) => {
                    self.filters = data
                        .filters()
                        .iter()
                        .map(|filter| (filter.clone(), false))
                        .collect();
                    self.deleted = false;
                    true
                }
                None => {
                    warn!("ignoring filter `Revert` message, there is no allocation data");
                    false
                }
            },
            Apply => match data {
                Some(data) => {
                    let filters = self.get_and_set_unedited();
                    data.set_filters(filters);
                    self.model_callback.emit(msg::ChartsMsg::reload());
                    true
                }
                None => {
                    warn!("ignoring filter `Apply` message, there is no allocation data");
                    false
                }
            },
        }
    }

    /// Renders itself.
    pub fn render(&self, _data: Option<&Storage>) -> Html {
        html! {
            <>
                // { self.filter_edit.render() }
                { for self.filters.iter().enumerate().map(
                    |(index, (filter, edited))| html! {
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
                    <li
                        class=style::class::filter::BUTTONS
                    >
                        { buttons::add(|_|
                            FooterFilterMsg::add(Filter::default())
                        ) }
                    </li>
                </ul>
            </>
        }
    }

    /// Renders the top buttons of the filter footer.
    pub fn render_top_buttons(&self) -> Html {
        html! {
            <li class=style::class::tabs::li::RIGHT>{
                if self.is_edited() {
                    buttons::inactive_tickbox(|_| FooterFilterMsg::apply())
                } else {
                    buttons::active_tickbox(
                        |_| Msg::Nop
                    )
                }
            }</li>
        }
    }
}
