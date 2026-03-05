use std::collections::BTreeMap;
use std::fmt;

/// Unique identifier for a resource: (exporter, group, resource_name).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourcePath {
    pub exporter: String,
    pub group: String,
    pub name: String,
}

impl fmt::Display for ResourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.exporter, self.group, self.name)
    }
}

/// Flattened representation of a labgrid resource for display.
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub path: ResourcePath,
    pub cls: String,
    pub params: BTreeMap<String, String>,
    pub extra: BTreeMap<String, String>,
    pub acquired: Option<String>,
    pub available: bool,
}

impl ResourceInfo {
    /// Human-readable acquired status.
    pub fn acquired_display(&self) -> &str {
        match &self.acquired {
            Some(owner) if !owner.is_empty() => owner.as_str(),
            _ => "",
        }
    }

    /// Human-readable availability.
    pub fn avail_display(&self) -> &str {
        if self.available {
            "yes"
        } else {
            "no"
        }
    }
}
