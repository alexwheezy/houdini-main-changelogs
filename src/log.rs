#![allow(dead_code)]

use anyhow;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
};

const LOG_PATH: &'static str = "log/changelog.json";

type Category = BTreeMap<String, BTreeSet<String>>;
type Log = BTreeMap<String, Info>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Info {
    /// The category structure stores the category of a context
    /// and a description of the fixes for that context.
    /// { {"Category": { Description } }
    /// Examples: {"sop": "Fix bugs"} }
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
    /// The structure will store a record in the form of the current build number
    /// and a description of the categories with changes between versions.
    /// { Build: {"Category": { Description } }
    /// Examples: {"19.5.501": {"sop": "Fix bugs"} }
    data: Log,
}

impl ChangeLog {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    /// Additional constructor so that you can create a log immediately of
    /// their build version number and [`Log`] object.
    pub fn with_data(build: &str, log: Info) -> Self {
        Self {
            data: Log::from([(build.to_owned(), log)]),
        }
    }

    /// We fill the structure with the necessary data obtained during page parsing.
    pub fn fill(&mut self, build: &str, category: &str, description: &str) {
        self.data
            .entry(build.to_owned())
            .or_insert(Info::default())
            .category
            .entry(category.to_owned())
            .or_insert(BTreeSet::default())
            .insert(description.to_owned());
    }

    /// Returns the load of this [`ChangeLog`].
    ///
    /// # Errors
    ///
    /// This function will return an error if file could not be read or does not exist.
    pub fn load(&self) -> anyhow::Result<ChangeLog> {
        let file = OpenOptions::new().read(true).open(LOG_PATH)?;
        let reader = BufReader::new(file);
        let changelog = serde_json::from_reader(reader)?;
        Ok(changelog)
    }

    /// Returns the store of this [`ChangeLog`].
    ///
    /// # Errors
    ///
    /// This function will return an error if file could not be created or written.
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

    /// The method will update the change log saved in the logs of the previous version
    /// of the build for the current day, so that later on the next update,
    /// restore the change log of the previous version and find the difference in the changes.
    pub fn update(&mut self) -> anyhow::Result<()> {
        // Restoring the changelog log of the previous version.
        let prev_changelog = self.load()?;
        // Since we always need the current version, we read the latest log entries.
        let (prev_build, prev_info) = prev_changelog.last_record().unwrap();
        let (next_build, next_info) = self.last_record().unwrap();

        // TODO: Is it possible to avoid copying here?
        let mut next_info = next_info.clone();
        if prev_build == next_build {
            for (category, description) in next_info.category.iter_mut() {
                if let Some(data) = prev_info.category.get(category) {
                    //NOTE: Not always, but sometimes it happens that the next version of
                    //the build does not change, but only the entries change.
                    //Therefore, we must remove duplicate entries so that
                    //the log always contains only up-to-date data.
                    let diff = description
                        .difference(data)
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    *description = diff;
                }
            }
        }
        // Delete all empty categories along with entries
        next_info.category.retain(|_, items| items.is_empty());
        // Recreate the structure with new records
        *self = Self::with_data(next_build, next_info);
        Ok(())
    }

    /// Pretty output to stdout of the [`ChangeLog`].
    pub fn pretty_print(&self) {
        println!("{:#?}", self.data);
    }

    /// Returns the first record of this [`ChangeLog`].
    pub fn first_record(&self) -> Option<(&String, &Info)> {
        self.data.first_key_value()
    }

    /// Returns the last record of this [`ChangeLog`].
    pub fn last_record(&self) -> Option<(&String, &Info)> {
        self.data.last_key_value()
    }
}
