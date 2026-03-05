use super::resource::ResourceInfo;

/// Aggregated exporter view — derived from resources grouped by exporter name.
#[derive(Debug, Clone)]
pub struct Exporter {
    pub name: String,
    pub total_resources: usize,
    pub available: usize,
    pub unavailable: usize,
    pub acquired: usize,
    pub resource_classes: Vec<String>,
}

impl Exporter {
    /// Build an exporter summary from a set of resources sharing the same exporter name.
    pub fn from_resources(name: &str, resources: &[&ResourceInfo]) -> Self {
        let total_resources = resources.len();
        let available = resources.iter().filter(|r| r.available).count();
        let acquired = resources
            .iter()
            .filter(|r| r.acquired.as_ref().is_some_and(|a| !a.is_empty()))
            .count();
        let unavailable = total_resources - available;

        let mut classes: Vec<String> = resources.iter().map(|r| r.cls.clone()).collect();
        classes.sort();
        classes.dedup();

        Exporter {
            name: name.to_string(),
            total_resources,
            available,
            unavailable,
            acquired,
            resource_classes: classes,
        }
    }

    pub fn classes_display(&self) -> String {
        self.resource_classes.join(", ")
    }
}
