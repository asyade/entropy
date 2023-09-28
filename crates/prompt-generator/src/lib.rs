use builder::Prompt;
use rand::{rngs::ThreadRng, Rng};
use template::Template;

use crate::{builder::PromptBuilder, prelude::*};

pub mod error;
pub mod prelude;
pub mod schemas;

pub mod builder {
    use super::*;

    pub struct PromptBuilder {
        chunks: BinaryHeap<Chunk>,
    }

    #[derive(Debug)]
    pub struct Prompt {
        pub render: String,
    }

    impl PromptBuilder {
        pub fn new() -> Self {
            PromptBuilder {
                chunks: BinaryHeap::new(),
            }
        }

        pub fn add(&mut self, chunk: &Chunk) -> &mut Self {
            self.chunks.push(chunk.clone());
            self
        }

        pub fn append(&mut self, variation: &Chunks, variants: &[(&str, usize)]) -> &mut Self {
            for chunk in variation.defaults.iter().flat_map(|e| e.iter()) {
                self.add(chunk);
            }
            for (name, index) in variants {
                if let Some(chunk) = variation.variants.get(*name).unwrap().get(*index) {
                    self.add(chunk);
                } else {
                    eprintln!("wrong chunk");
                }
            }
            self
        }

        pub fn build(mut self) -> Prompt {
            let mut render = String::new();
            let mut current_group = None;
            let mut index = 0;
            while let Some(chunk) = self.chunks.pop() {
                let ponct = match current_group {
                    Some(group) if group == chunk.pos.unwrap_or_default() => " ",
                    _ => {
                        current_group = Some(chunk.pos.unwrap_or_default());
                        ", "
                    }
                };

                if index > 0 {
                    render += ponct;
                }
                render += &chunk.prompt;
                index += 1;
            }
            Prompt { render }
        }
    }
}

pub mod template {
    use super::*;

    #[derive(Debug)]
    pub struct Template {
        pub include: Vec<Chunks>,
        pub include_varied: HashMap<String, HashMap<String, Chunks>>,
    }

    impl Template {
        pub fn from_yaml_template(root: &Path, template: YamlTemplate) -> Result<Template> {
            let mut include = Vec::new();
            let mut include_varied = HashMap::new();

            for element in template.0 {
                match element {
                    Element::Include(IncludeElement { uri }) => {
                        let path = root.join(Path::new(&uri));
                        include.push(Chunks::from_yaml_file(&path)?);
                    }
                    Element::IncludeVaried(varied) => {
                        let mut choices = HashMap::with_capacity(varied.choices.len());
                        for (name, uri) in varied.choices {
                            let path = root.join(Path::new(&uri));
                            choices.insert(name, Chunks::from_yaml_file(&path)?);
                        }
                        include_varied.insert(varied.name, choices);
                    }
                }
            }
            Ok(Self {
                include,
                include_varied,
            })
        }
    }
}

#[derive(Debug)]
pub struct PromptGenerator {
    template: Template,
}

impl PromptGenerator {
    pub fn new(template_path: &Path, chumks_path: &Path) -> Result<PromptGenerator> {
        let template = Template::from_yaml_template(
            chumks_path,
            YamlTemplate::from_yaml_file(template_path)?,
        )?;
        Ok(PromptGenerator { template })
    }

    pub fn randomize(&self) -> Prompt {
        fn randomize_chunks(rng: &mut ThreadRng, chunks: &Chunks, builder: &mut PromptBuilder) {
            for default in chunks.defaults.as_ref().iter().flat_map(|e| e.iter()) {
                builder.add(default);
            }
            for (_name, variation) in chunks.variants.iter() {
                let num = rng.gen_range(0..variation.len());
                builder.add(&variation[num]);
            }
        }

        let mut prompt = PromptBuilder::new();
        let mut rng = rand::thread_rng();

        for include in self.template.include.iter() {
            randomize_chunks(&mut rng, include, &mut prompt);
        }

        for choices in self.template.include_varied.values() {
            let num = rng.gen_range(0..choices.len());
            let include = choices.values().nth(num).unwrap();
            randomize_chunks(&mut rng, include, &mut prompt);
        }
        prompt.build()
    }
}

// Tests
#[cfg(test)]
mod tests {

}
