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
        }
    }

    /// Constructs a specification for the catch-all filter.
    pub fn new_catch_all() -> Self {
        Self {
            uid: None,
            name: "catch all".into(),
            color: Color::new(0x00, 0xd8, 0xff),
        }
    }

    /// UID accessor.
    pub fn uid(&self) -> Option<FilterUid> {
        self.uid
    }

    /// Name accessor.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Name setter.
    pub fn set_name<S: Into<String>>(&mut self, name: String) {
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
