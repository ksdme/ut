use crate::tool::{Output, Tool};
use clap::{Command, CommandFactory, Parser};
use rand::{Rng, rngs::OsRng};

#[derive(Parser, Debug)]
#[command(name = "lorem")]
pub struct Lorem {
    /// Number of paragraphs to generate
    #[arg(short = 'p', long = "paragraphs", default_value = "3")]
    paragraphs: usize,

    /// Minimum number of sentences per paragraph
    #[arg(long = "min-sentences", default_value = "3")]
    min_sentences: usize,

    /// Maximum number of sentences per paragraph
    #[arg(long = "max-sentences", default_value = "7")]
    max_sentences: usize,

    /// Minimum number of words per sentence
    #[arg(long = "min-words", default_value = "5")]
    min_words: usize,

    /// Maximum number of words per sentence
    #[arg(long = "max-words", default_value = "15")]
    max_words: usize,
}

impl Lorem {
    fn generate_sentence(&self, rng: &mut impl Rng) -> String {
        let sentence = (0..rng.gen_range(self.min_words..=self.max_words))
            .map(|_| LOREM_WORDS[rng.gen_range(0..LOREM_WORDS.len())])
            .collect::<Vec<&str>>()
            .join(" ");

        crate::tools::case::capitalize_first(&sentence) + "."
    }

    fn generate_paragraph(&self, rng: &mut impl Rng) -> String {
        (0..rng.gen_range(self.min_sentences..=self.max_sentences))
            .map(|_| self.generate_sentence(rng))
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn generate_lorem(&self) -> String {
        let mut rng = OsRng;

        (0..self.paragraphs)
            .map(|_| self.generate_paragraph(&mut rng))
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

impl Tool for Lorem {
    fn cli() -> Command {
        Lorem::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        Ok(Some(Output::JsonValue(serde_json::json!(
            self.generate_lorem()
        ))))
    }
}

const LOREM_WORDS: &[&str] = &[
    "lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "sed",
    "do",
    "eiusmod",
    "tempor",
    "incididunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magna",
    "aliqua",
    "enim",
    "ad",
    "minim",
    "veniam",
    "quis",
    "nostrud",
    "exercitation",
    "ullamco",
    "laboris",
    "nisi",
    "aliquip",
    "ex",
    "ea",
    "commodo",
    "consequat",
    "duis",
    "aute",
    "irure",
    "in",
    "reprehenderit",
    "voluptate",
    "velit",
    "esse",
    "cillum",
    "fugiat",
    "nulla",
    "pariatur",
    "excepteur",
    "sint",
    "occaecat",
    "cupidatat",
    "non",
    "proident",
    "sunt",
    "culpa",
    "qui",
    "officia",
    "deserunt",
    "mollit",
    "anim",
    "id",
    "est",
    "laborum",
    "at",
    "vero",
    "eos",
    "accusamus",
    "accusantium",
    "doloremque",
    "laudantium",
    "totam",
    "rem",
    "aperiam",
    "eaque",
    "ipsa",
    "quae",
    "ab",
    "illo",
    "inventore",
    "veritatis",
    "et",
    "quasi",
    "architecto",
    "beatae",
    "vitae",
    "dicta",
    "explicabo",
    "nemo",
    "ipsam",
    "quia",
    "voluptas",
    "aspernatur",
    "aut",
    "odit",
    "fugit",
    "sed",
    "quia",
    "consequuntur",
    "magni",
    "dolores",
    "ratione",
    "sequi",
    "nesciunt",
    "neque",
    "porro",
    "quisquam",
    "qui",
    "dolorem",
    "adipisci",
    "numquam",
    "eius",
    "modi",
    "tempora",
    "incidunt",
    "magnam",
    "quaerat",
    "voluptatem",
    "aliquam",
    "quam",
    "nihil",
    "molestiae",
    "et",
    "iusto",
    "odio",
    "dignissimos",
    "ducimus",
    "blanditiis",
    "praesentium",
    "voluptatum",
    "deleniti",
    "atque",
    "corrupti",
    "quos",
    "quas",
    "molestias",
    "excepturi",
    "occaecati",
    "cupiditate",
    "similique",
    "eleifend",
    "donec",
    "pretium",
    "vulputate",
    "sapien",
    "nec",
    "sagittis",
    "aliquam",
    "malesuada",
    "bibendum",
    "arcu",
    "vitae",
    "elementum",
    "curabitur",
    "gravida",
    "cursus",
    "risus",
    "facilisis",
    "magna",
    "etiam",
    "tempor",
    "orci",
    "dapibus",
    "ultrices",
    "in",
    "iaculis",
    "nunc",
    "faucibus",
    "a",
    "pellentesque",
    "habitant",
    "morbi",
    "tristique",
    "senectus",
    "netus",
    "malesuada",
    "fames",
    "ac",
    "turpis",
    "egestas",
    "integer",
    "feugiat",
    "scelerisque",
    "varius",
];
