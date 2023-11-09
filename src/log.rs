#![allow(dead_code)]

use anyhow;
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
};

// FIXME: Do I need to make the path to the logs static or relative?
const LOG_PATH: &'static str = "log/changelog.json";

type Category = BTreeMap<String, BTreeSet<String>>;
type Log = BTreeMap<String, Info>;

fn category_icons() -> BTreeMap<&'static str, &'static str> {
    #[rustfmt::skip]
    let icons = BTreeMap::from([
        ("3dsmax",   "\u{1fad6}"),   //:teapot:
        ("channel",  "\u{1f4c8}"),   //:chart_with_upwards_trend:
        ("char",     "\u{1f3c2}"),   //:snowboarder:
        ("character","\u{1f3c2}"),   //:snowboarder:
        ("chop",     "\u{1f4c8}"),   //:chart_with_upwards_trend:
        ("cop2",     "\u{1f3ab}"),   //:ticket:
        ("crowd",    "\u{1f38e}"),   //:dolls:
        ("doc",      "\u{1f4c4}"),   //:page_facing_up:
        ("dop",      "\u{1f30a}"),   //:ocean:
        ("expr",     "\u{1f4e7}"),   //:email:
        ("fbx",      "\u{1f381}"),   //:gift:
        ("fur",      "\u{1f98a}"),   //:fox_face:
        ("general",  "\u{1f365}"),   //:fish_cake:
        ("geo",      "\u{1f371}"),   //:bento:
        ("gl",       "\u{2699}"),    //:gear:
        ("gltf",     ""),            //unknown
        ("gplay",    ""),            //unknown
        ("grave",    "\u{1f4cc}"),   //:pushpin:
        ("handle",   "\u{1f579}"),   //:joystick:
        ("hapi",     "\u{1f529}"),   //:nut_and_bolt:
        ("hom",      "\u{1f40d}"),   //:snake:
        ("hdk",      "\u{1f9f0}"),   //:toolbox:
        ("hqueue",   "\u{1f39b}"),   //:control_knobs:
        ("image",    "\u{1f303}"),   //:night_with_stars:
        ("jive",     "\u{1f4c8}"),   //:chart_with_upwards_trend:
        ("karma",    "\u{1f341}"),   //:maple_leaf:
        ("launcher", "\u{1f680}"),   //:rocket:
        ("license",  "\u{1f511}"),   //:key:
        ("linux",    "\u{1f427}"),   //:penguin:
        ("lop",      "\u{1f4a1}"),   //:bulb:
        ("mantra",   "\u{1f4fd}"),   //:film_projector:
        ("maya",     "\u{1f5ff}"),   //:film_projector:
        ("mplay",    "\u{1f4fc}"),   //:vhs:
        ("op",       "\u{2699}"),    //:gear:
        ("opencl",   "\u{1f680}"),   //:rocket:
        ("osx",      "\u{1f34f}"),   //:green_apple:
        ("otl",      "\u{1f4e6}"),   //:package:
        ("pdg",      "\u{1f3a9}"),   //:tophat:
        ("pop",      "\u{1f4a7}"),   //:droplet:
        ("pyro",     "\u{1f525}"),   //:flame:
        ("python",   "\u{1f40d}"),   //:snake:
        ("render",   "\u{1f39e}"),   //:film_frames:
        ("rop",      "\u{1f39e}"),   //:film_frames:
        ("soho",     "\u{1f40d}"),   //:snake:
        ("solaris",  "\u{1f4a5}"),   //:boom:
        ("sop",      "\u{1f9e0}"),   //:brain:
        ("top",      "\u{1f3a9}"),   //:tophat:
        ("ui",       "\u{1f39a}"),   //:level_slider:
        ("unity",    "\u{1f30f}"),   //:earth_asia:
        ("unreal",   "\u{1f52e}"),   //:crystal_ball:
        ("vex",      "\u{1f393}"),   //:mortar_board:
        ("vop",      "\u{1f393}"),   //:mortar_board:
        ("windows",  "\u{1f4ce}"),   //:paperclip:
    ]);
    icons
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Info {
    /// The category structure stores the category of a context
    /// and a description of the fixes for that context.
    /// { {"Category": { Description } }
    /// Examples: {"sop": "Fix bugs"} }
    category: Category,
}

impl Info {
    /// Returns the categories of this [`Info`].
    pub fn categories(&self) -> Vec<String> {
        self.category.keys().cloned().collect()
    }

    /// Returns a description by category
    pub fn description_by(&self, category: &str) -> Option<&BTreeSet<String>> {
        self.category.get(category)
    }

    /// Returns True if the list of categories is not empty in [`Info`].
    pub fn is_empty(&self) -> bool {
        self.category.is_empty()
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icons = category_icons();
        let backticks = Regex::new(r"`(\w+(\w+|[ ])*)`").unwrap();
        for (key, values) in self.category.iter() {
            writeln!(
                f,
                "{icon}#<b>{category}</b>:",
                icon = icons.get(key.as_str()).unwrap_or(&""),
                category = key.to_uppercase()
            )?;
            for value in values.iter() {
                for str in value.split("\n") {
                    let str = backticks.replace_all(str, "<code>$1</code>");
                    writeln!(f, "- {}\n", str)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
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

        //TODO: Is it possible to avoid copying here?
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
        next_info.category.retain(|_, items| !items.is_empty());
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
