use std::collections::BTreeMap;

/// Flattened representation of a labgrid place for display.
#[derive(Debug, Clone)]
pub struct PlaceInfo {
    pub name: String,
    pub aliases: Vec<String>,
    pub comment: String,
    pub tags: BTreeMap<String, String>,
    pub matches: Vec<MatchInfo>,
    pub acquired: Option<String>,
    pub acquired_resources: Vec<String>,
    pub allowed: Vec<String>,
    pub created: f64,
    pub changed: f64,
    pub reservation: Option<String>,
}

/// A resource match pattern on a place.
#[derive(Debug, Clone)]
pub struct MatchInfo {
    pub exporter: String,
    pub group: String,
    pub cls: String,
    pub name: Option<String>,
    pub rename: Option<String>,
}

impl MatchInfo {
    pub fn pattern_display(&self) -> String {
        let mut s = format!("{}/{}/{}", self.exporter, self.group, self.cls);
        if let Some(ref name) = self.name {
            s.push('/');
            s.push_str(name);
        }
        if let Some(ref rename) = self.rename {
            s.push_str(" → ");
            s.push_str(rename);
        }
        s
    }
}

impl PlaceInfo {
    pub fn aliases_display(&self) -> String {
        self.aliases.join(", ")
    }

    pub fn acquired_display(&self) -> &str {
        match &self.acquired {
            Some(owner) if !owner.is_empty() => owner.as_str(),
            _ => "",
        }
    }

    pub fn tags_display(&self) -> String {
        self.tags
            .iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    k.clone()
                } else {
                    format!("{k}={v}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn matches_display(&self) -> String {
        self.matches
            .iter()
            .map(|m| m.pattern_display())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
