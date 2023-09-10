use failure::Fail;
use itertools::Itertools;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use std::path::Path;

use crate::log::ChangeLog;

#[derive(Debug, Fail)]
pub enum ParseChangeLogError {
    #[fail(display = "build not found")]
    BuildNotFound,
    #[fail(display = "source not found")]
    SourceNotFound,
    #[fail(display = "description not found")]
    DescriptionNotFound,
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
            let source = path.attr("src").ok_or(ParseChangeLogError::SourceNotFound);
            // Context category
            let category = Path::new(source.unwrap())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_prefix("icon_")
                .unwrap();

            // Version number of the current program build
            let build = table
                .find(Name("td"))
                .skip(1)
                .take(1)
                .next()
                .ok_or(ParseChangeLogError::BuildNotFound)?
                .text();

            // Description of the fix in the category
            let description = table
                .find(Name("td").descendant(Name("p").or(Name("li"))))
                .map(|node| {
                    node.text()
                        .trim()
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .join(" ")
                })
                .collect::<Vec<String>>()
                .join("\n");

            if description.is_empty() {
                return Err(ParseChangeLogError::DescriptionNotFound);
            }
            logs.fill(&build, category, &description);
        }
    }
    Ok(logs)
}
