#![allow(dead_code)]
use std::{collections::BTreeMap, fmt::Display};

type LogCategory = BTreeMap<String, Vec<String>>;

#[derive(Debug, Default)]
pub struct ChangeLog {
    category: LogCategory,
}

impl Display for ChangeLog {
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

#[derive(Debug, Default)]
pub struct ChangeLogList {
    id: i32,
    data: BTreeMap<String, ChangeLog>,
}

impl ChangeLogList {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            data: BTreeMap::default(),
        }
    }

    pub fn add_log(&mut self, build: &str, category: &str, description: &str) {
        self.data
            .entry(build.to_owned())
            .or_insert(ChangeLog::default())
            .category
            .entry(category.to_owned())
            .or_insert(Vec::default())
            .push(description.to_owned())
    }

    pub fn last_log(&self) -> (&String, &ChangeLog) {
        self.data.last_key_value().unwrap()
    }
}

#[test]
fn test_key() {
    let change_log_list = ChangeLogList::new(42);
    assert!(!change_log_list.data.contains_key("19.5.501"));
}

#[test]
fn test_add_log() {
    let mut change_log_list = ChangeLogList::new(42);
    change_log_list.add_log("19.5.501", "lop", "sphere");
    let log = change_log_list.data.get(&"19.5.501".to_owned()).unwrap();
    assert_eq!(
        log.category,
        BTreeMap::from([("lop".to_owned(), vec!["sphere".to_owned()])])
    )
}
