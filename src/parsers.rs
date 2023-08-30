use failure::Fail;
use itertools::Itertools;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use std::path::Path;

use crate::log::ChangeLog;

#[derive(Debug, Fail)]
pub enum ParseChangeLogError {
    #[fail(display = "source not found")]
    SourceNotFound,
    #[fail(display = "description not found")]
    DescriptionNotFound,
}

pub fn parse_change_log(doc: &Document) -> Result<ChangeLog, ParseChangeLogError> {
    let mut logs = ChangeLog::new();
    for table in doc.find(Class("table-striped").descendant(Name("tr"))) {
        if let Some(path) = table.find(Name("img")).next() {
            let source = path.attr("src").ok_or(ParseChangeLogError::SourceNotFound);
            let category = Path::new(source.unwrap())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_prefix("icon_")
                .unwrap();

            let data = table
                .find(Name("td"))
                .filter(|data| !data.text().is_empty())
                .map(|data| {
                    data.text()
                        .trim()
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .join(" ")
                })
                .collect::<Vec<String>>();

            match (data.get(0), data.get(1)) {
                (Some(build), Some(description)) => logs.fill(build, category, description),
                _ => return Err(ParseChangeLogError::DescriptionNotFound),
            }
        }
    }
    Ok(logs)
}
