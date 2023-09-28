use crate::prelude::*;

pub mod chunks {
    use std::cmp::Ordering;

    use super::*;

    pub type VariantId = String;

    #[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash)]
    pub struct Chunk {
        pub prompt: String,
        pub pos: Option<i32>,
        pub sub_pos: Option<i32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Chunks {
        pub defaults: Option<Vec<Chunk>>,
        pub variants: HashMap<String, Vec<Chunk>>,
    }

    impl PartialEq for Chunk {
        fn eq(&self, other: &Self) -> bool {
            self.pos == other.pos
        }
    }

    impl PartialOrd for Chunk {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.pos
                .unwrap_or(0)
                .partial_cmp(&other.pos.unwrap_or(0))
                .and_then(|e| match e {
                    std::cmp::Ordering::Equal => self.sub_pos.partial_cmp(&other.sub_pos),
                    e => Some(e.reverse()),
                })
        }
    }

    impl Ord for Chunk {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap().reverse()
        }
    }

    impl Chunks {
        pub fn from_yaml_file(path: &Path) -> Result<Chunks> {
            Ok(serde_yaml::from_str(&std::fs::read_to_string(path)?)?)
        }
    }
}

pub mod template {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Element {
        Include(IncludeElement),
        IncludeVaried(IncludeVariedElement),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IncludeElement {
        pub uri: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IncludeVariedElement {
        pub name: String,
        pub choices: HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct YamlTemplate(pub Vec<Element>);

    impl YamlTemplate {
        pub fn from_yaml_file(path: &Path) -> Result<YamlTemplate> {
            Ok(serde_yaml::from_str(&std::fs::read_to_string(path)?)?)
        }
    }
}
