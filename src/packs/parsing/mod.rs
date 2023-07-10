use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub(crate) mod ruby;
pub(crate) use ruby::experimental::parser::process_from_path as process_from_ruby_path_experimental;
pub(crate) use ruby::packwerk::parser::process_from_path as process_from_ruby_path;
pub(crate) mod erb;
pub(crate) use erb::experimental::parser::process_from_path as process_from_erb_path_experimental;
pub(crate) use erb::packwerk::parser::process_from_path as process_from_erb_path;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use super::{
    caching::{Cache, CacheResult},
    file_utils::{get_file_type, SupportedFileType},
    ProcessedFile,
};

pub fn process_file(path: &Path, experimental_parser: bool) -> ProcessedFile {
    let file_type_option = get_file_type(path);

    if let Some(file_type) = file_type_option {
        match file_type {
            SupportedFileType::Ruby => {
                if experimental_parser {
                    process_from_ruby_path_experimental(path)
                } else {
                    process_from_ruby_path(path)
                }
            }
            SupportedFileType::Erb => {
                if experimental_parser {
                    process_from_erb_path_experimental(path)
                } else {
                    process_from_erb_path(path)
                }
            }
        }
    } else {
        // Later, we can perhaps have this error, since in theory the Configuration.intersect
        // method should make sure we never get any files we can't handle.
        ProcessedFile {
            absolute_path: path.to_path_buf(),
            unresolved_references: vec![],
            definitions: vec![], // TODO
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct UnresolvedReference {
    pub name: String,
    pub namespace_path: Vec<String>,
    pub location: Range,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct Range {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq)]
pub struct ParsedDefinition {
    pub fully_qualified_name: String,
    pub location: Range,
}

pub fn process_files_with_cache(
    absolute_root: &Path,
    paths: &HashSet<PathBuf>,
    cache: Box<dyn Cache + Send + Sync>,
    experimental_parser: bool,
) -> Vec<ProcessedFile> {
    paths
        .par_iter()
        .map(|absolute_path| -> ProcessedFile {
            match cache.get(absolute_root, absolute_path) {
                CacheResult::Processed(processed_file) => processed_file,
                CacheResult::Miss(empty_cache_entry) => {
                    let processed_file =
                        process_file(absolute_path, experimental_parser);
                    cache.write(&empty_cache_entry, &processed_file);
                    processed_file
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::packs::file_utils::get_file_type;

    use super::*;

    fn assert_is_ruby(filename: &str) {
        assert_eq!(
            SupportedFileType::Ruby,
            get_file_type(Path::new(filename)).expect("Should be supported")
        )
    }

    fn assert_is_erb(filename: &str) {
        assert_eq!(
            SupportedFileType::Erb,
            get_file_type(Path::new(filename)).expect("Should be supported")
        )
    }

    #[test]
    fn identifies_ruby_files() {
        assert_is_ruby("foo.rb");
        assert_is_ruby("foo.rake");
        assert_is_ruby("Gemfile");
        assert_is_ruby("my_gem.gemspec");
    }

    #[test]
    fn identifies_erb_files() {
        assert_is_erb("foo.erb");
    }
}