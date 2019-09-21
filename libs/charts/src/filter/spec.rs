//! Filter specification.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSpec {
    /// Uid of the filter.
    ///
    /// None if the specification if for the catch-all filter.
    uid: Option<FilterUid>,
    /// Name of the filter.
    name: String,
    /// Color of the filter.
    color: Color,
    /// True if the filter has been edited.
    ///
    /// This is only used by the client to keep track of which filters have been edited in the UI.
    edited: bool,
}
impl FilterSpec {
    /// Constructor.
    pub fn new(color: Color) -> Self {
        let uid = FilterUid::fresh();
        let name = format!("filter {}", uid);
        Self {
            uid: Some(uid),
            name,
            color,
            edited: false,
        }
    }

    /// Constructs a specification for the catch-all filter.
    pub fn new_catch_all() -> Self {
        Self {
            uid: None,
            name: "catch all".into(),
            color: Color::new(0x01, 0x93, 0xff),
            edited: false,
        }
    }

    /// UID accessor.
    pub fn uid(&self) -> Option<FilterUid> {
        self.uid
    }

    /// Value of the `edited` flag.
    pub fn edited(&self) -> bool {
        self.edited
    }
    /// Sets the `edited` flag to true.
    pub fn set_edited(&mut self) {
        self.edited = true
    }
    /// Sets the `edited` flag to false.
    pub fn unset_edited(&mut self) {
        self.edited = false
    }

    /// Name accessor.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Name setter.
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = name.into()
    }

    /// Color accessor.
    pub fn color(&self) -> &Color {
        &self.color
    }
    /// Color setter.
    pub fn set_color(&mut self, color: Color) {
        self.color = color
    }
}
