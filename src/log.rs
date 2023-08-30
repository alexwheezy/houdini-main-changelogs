#![allow(dead_code)]

use anyhow;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
};

const LOG_PATH: &'static str = "changelog.json";

type Category = BTreeMap<String, BTreeSet<String>>;
type Log = BTreeMap<String, Info>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Info {
    category: Category,
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, values) in self.category.iter() {
            writeln!(f, "#<b>{}</b>:", key.to_uppercase())?;
            for value in values.iter() {
                writeln!(f, "- {}\n", value)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeLog {
    data: Log,
}

impl ChangeLog {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn with_data(build: &str, log: Info) -> Self {
        Self {
            data: Log::from([(build.to_owned(), log)]),
        }
    }

    pub fn fill(&mut self, build: &str, category: &str, description: &str) {
        self.data
            .entry(build.to_owned())
            .or_insert(Info::default())
            .category
            .entry(category.to_owned())
            .or_insert(BTreeSet::default())
            .insert(description.to_owned());
    }

    pub fn load(&self) -> anyhow::Result<ChangeLog> {
        let file = OpenOptions::new().read(true).open(LOG_PATH)?;
        let reader = BufReader::new(file);
        let changelog = serde_json::from_reader(reader)?;
        Ok(changelog)
    }

    pub fn store(&self) -> anyhow::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(LOG_PATH)?;
        let mut writter = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writter, &self)?;
        writter.flush()?;
        Ok(())
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        let prev_changelog = self.load()?;
        let (prev_build, prev_info) = prev_changelog.last_record().unwrap();
        let (next_build, next_info) = self.last_record().unwrap();

        let mut next_info = next_info.clone();
        if prev_build == next_build {
            for (category, description) in next_info.category.iter_mut() {
                if let Some(data) = prev_info.category.get(category) {
                    let diff = description
                        .difference(data)
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    *description = diff;
                }
            }
        }
        *self = Self::with_data(next_build, next_info);
        Ok(())
    }

    pub fn pretty_print(&self) {
        println!("{:#?}", self.data);
    }

    pub fn first_record(&self) -> Option<(&String, &Info)> {
        self.data.first_key_value()
    }

    pub fn last_record(&self) -> Option<(&String, &Info)> {
        self.data.last_key_value()
    }
}
