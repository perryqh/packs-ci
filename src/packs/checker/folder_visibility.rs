use super::{get_referencing_pack, CheckerInterface, ViolationIdentifier};
use crate::packs::checker::reference::Reference;
use crate::packs::pack::Pack;
use crate::packs::{Configuration, Violation};
use anyhow::Result;

pub struct Checker {}

impl CheckerInterface for Checker {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
    ) -> Option<Violation> {
        let pack_set = &configuration.pack_set;
        let referencing_pack = &reference.referencing_pack(pack_set);
        let relative_defining_file = &reference.relative_defining_file;
        if relative_defining_file.is_none() {
            return None;
        }
        let defining_pack = &reference.defining_pack(pack_set);
        if defining_pack.is_none() {
            return None;
        }
        let defining_pack = defining_pack.unwrap();
        if !folder_visible(referencing_pack, defining_pack).unwrap() {
            let message = format!(
                "{}:{}:{}\nFolder Visibility violation: `{}` belongs to `{}`, which is not visible to `{}` as it is not a sibling pack or parent pack.",
                reference.relative_referencing_file,
                reference.source_location.line,
                reference.source_location.column,
                reference.constant_name,
                defining_pack.name,
                referencing_pack.name,
            );
            let identifier = ViolationIdentifier {
                violation_type: self.violation_type(),
                file: reference.relative_referencing_file.clone(),
                constant_name: reference.constant_name.clone(),
                referencing_pack_name: referencing_pack.name.clone(),
                defining_pack_name: defining_pack.name.clone(),
            };
            Some(Violation {
                message,
                identifier,
            })
        } else {
            None
        }
    }

    fn is_strict_mode_violation(
        &self,
        violation: &ViolationIdentifier,
        configuration: &Configuration,
    ) -> bool {
        let referencing_pack =
            get_referencing_pack(violation, &configuration.pack_set);

        referencing_pack.enforce_folder_visibility().is_strict()
    }

    fn violation_type(&self) -> String {
        "folder_visibility".to_owned()
    }
}

fn folder_visible(from_pack: &Pack, to_pack: &Pack) -> Result<bool> {
    if to_pack.enforce_folder_visibility().is_false() {
        return Ok(true);
    }

    if from_pack.relative_path.to_string_lossy() == "." {
        return Ok(true); // root pack is visible to all
    }

    if let (Some(from_pack_parent_path), Some(to_pack_parent_path)) = (
        from_pack.relative_path.parent(),
        to_pack.relative_path.parent(),
    ) {
        if from_pack_parent_path == to_pack_parent_path {
            return Ok(true); // siblings are visible to each other
        }
    }
    // visible if "to" is a descendant of "from"
    Ok(to_pack
        .relative_path
        .to_string_lossy()
        .starts_with(from_pack.relative_path.to_string_lossy().as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::pack::CheckerSetting;
    use std::path::PathBuf;

    fn assert_folder_visibility(
        from_pack_path: &str,
        to_pack_path: &str,
        to_pack_enforce_folder_visibility: Option<CheckerSetting>,
        expected: bool,
    ) {
        let from_pack = Pack {
            name: from_pack_path.to_string(),
            relative_path: PathBuf::from(&from_pack_path),
            ..Pack::default()
        };
        if from_pack_path == to_pack_path {
            assert_eq!(
                expected,
                folder_visible(&from_pack, &from_pack).unwrap()
            );
            return;
        }
        let to_pack = Pack {
            name: to_pack_path.to_string(),
            relative_path: PathBuf::from(&to_pack_path),
            enforce_folder_visibility: to_pack_enforce_folder_visibility,
            ..Pack::default()
        };

        assert_eq!(expected, folder_visible(&from_pack, &to_pack).unwrap());
    }

    #[test]
    fn test_folder_visibility_when_different_parent_invisible() {
        assert_folder_visibility(
            "packs/bars/bar",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_visibility_when_not_enforced() {
        assert_folder_visibility(
            "packs/bar",
            "packs/foos/zoo",
            Some(CheckerSetting::False),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_siblings() {
        assert_folder_visibility(
            "packs/bar",
            "packs/foos",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_same() {
        assert_folder_visibility(
            "packs/bar",
            "packs/bar",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_descendant() {
        assert_folder_visibility(
            "packs/foo",
            "packs/foo/bar",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_parent_invisible() {
        assert_folder_visibility(
            "packs/foo/bar",
            "packs/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_visibility_when_invisible() {
        assert_folder_visibility(
            "packs/baz",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_visibility_when_from_is_root() {
        assert_folder_visibility(
            ".",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            true,
        );
    }
}