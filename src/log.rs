#![allow(dead_code)]

use anyhow;
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
        ("channel", ":chart_with_upwards_trend:"),
        ("charater",":snowboarder:"),
        ("chop",    ":chart_with_upwards_trend:"),
        ("cop2",    ":ticket:"),
        ("crowd",   ":dolls:"),
        ("doc",     ":page_facing_up:"),
        ("dop",     ":ocean:"),
        ("expr",    ":email:"),
        ("fbx",     ":gift:"),
        ("fur",     ":fox_face:"),
        ("general", ":fish_cake:"),
        ("geo",     ":bento:"),
        ("gl",      ":gear:"),
        ("gltf",    ""),
        ("gplay",   ""),
        ("grave",   ":pushpin:"),
        ("handle",  ":joystick:"),
        ("hapi",    ":nut_and_bolt:"),
        ("hdk",     ":toolbox:"),
        ("hqueue",  ":control_knobs:"),
        ("image",   ":night_with_stars:"),
        ("jive",    ":chart_with_upwards_trend:"),
        ("karma",   ":maple_leaf:"),
        ("license", ":key:"),
        ("lop",     ":bulb:"),
        ("mantra",  ":film_projector:"),
        ("mplay",   ":vhs:"),
        ("op",      ":gear:"),
        ("opencl",  ":rocket:"),
        ("osx",     ":green_apple:"),
        ("otl",     ":package:"),
        ("pdg",     ":tophat:"),
        ("pop",     ":droplet:"),
        ("pyro",    ":flame:"),
        ("python",  ":snake:"),
        ("render",  ":film_frames:"),
        ("rop",     ":film_frames:"),
        ("soho",    ":snake:"),
        ("solaris", ":boom:"),
        ("sop",     ":brain:"),
        ("top",     ":tophat:"),
        ("ui",      ":level_slider:"),
        ("unreal",  ":crystal_ball:"),
        ("vex",     ":mortar_board:"),
        ("vop",     ":mortar_board:"),
        ("windows", ":paperclip:"),
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
        for (key, values) in self.category.iter() {
            writeln!(
                f,
                "{icon}#<b>{category}</b>:",
                icon = icons.get(key.as_str()).unwrap_or(&""),
                category = key.to_uppercase()
            )?;
            for value in values.iter() {
                for str in value.split("\n") {
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
