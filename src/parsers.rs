use failure::Fail;
use itertools::Itertools;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use std::path::Path;

use crate::log::ChangeLog;

#[derive(Debug, Fail)]
pub enum ParseChangeLogError {
    #[fail(display = "build not found")]
    Build,
    #[fail(display = "source not found")]
    Source,
    #[fail(display = "description not found")]
    Description,
}

/// The function parses an HTML page and returns a new object of type [`ChangeLog`]
///
/// # Errors
///
/// This function will return an error if category source tags or category description not found.
pub fn parse_change_log(doc: &Document) -> Result<ChangeLog, ParseChangeLogError> {
    let mut logs = ChangeLog::new();
    for table in doc.find(Class("table-striped").descendant(Name("tr"))) {
        if let Some(path) = table.find(Name("img")).next() {
            let source = path.attr("src").ok_or(ParseChangeLogError::Source);

            let (category, version, description) = (
                build_category(source),
                build_version(&table)?,
                build_description(&table),
            );

            if description.is_empty() {
                return Err(ParseChangeLogError::Description);
            }

            logs.fill(&version, category, &description);
        }
    }
    Ok(logs)
}

fn build_category(source: Result<&str, ParseChangeLogError>) -> &str {
    Path::new(source.unwrap())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("icon_")
        .unwrap()
}

fn build_version(table: &Node) -> Result<String, ParseChangeLogError> {
    let version = table
        .find(Name("td"))
        .skip(1)
        .take(1)
        .next()
        .ok_or(ParseChangeLogError::Build)?
        .text();
    Ok(version)
}

fn build_description(table: &Node) -> String {
    table
        .find(Name("td").descendant(Name("p").or(Name("li"))))
        .map(|node| {
            node.text()
                .trim()
                .split(' ')
                .filter(|s| !s.is_empty())
                .join(" ")
        })
        .collect::<Vec<String>>()
        .join("\n")
}
