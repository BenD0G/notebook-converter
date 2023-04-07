use glob::glob;
use std::{fs::create_dir_all, path::PathBuf};

use clap::{self, Parser};

const IPYNB_ENDING: &str = ".ipynb";
const PY_ENDING: &str = ".py";

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    reverse: bool,
}

#[derive(PartialEq, Eq, Debug)]
struct FileRename {
    old_path: PathBuf,
    new_path: PathBuf,
    notebook_to_script: bool,
}

fn find_all_files(notebook_to_script: bool) -> Vec<FileRename> {
    let old_ending = if notebook_to_script {
        IPYNB_ENDING
    } else {
        PY_ENDING
    };
    let new_ending = if notebook_to_script {
        PY_ENDING
    } else {
        IPYNB_ENDING
    };

    let old_pattern = if notebook_to_script {
        "**/*.ipynb"
    } else {
        "**/.nb/*.py"
    };

    glob(&old_pattern)
        .unwrap()
        .map(|path| {
            let old_path = path.unwrap();
            let old_dir = old_path.parent().unwrap();
            let old_fname = old_path.file_name().unwrap();

            let new_fname = old_fname.to_str().unwrap().replace(old_ending, new_ending);
            let new_dir = if notebook_to_script {
                old_dir.join(".nb/")
            } else {
                old_dir.parent().unwrap().into()
            };
            let new_path = new_dir.join(new_fname);
            FileRename {
                old_path,
                new_path,
                notebook_to_script,
            }
        })
        .collect()
}

fn convert(replacement: &FileRename) {
    let new_extension = if replacement.notebook_to_script {
        "py"
    } else {
        "ipynb"
    };

    let parent_dir = replacement.new_path.parent().unwrap();
    if !parent_dir.exists() {
        create_dir_all(parent_dir).unwrap();
    }

    let args = vec![
        format!("--to={}", new_extension),
        format!("--output={}", replacement.new_path.as_path().display()),
        format!("{}", replacement.old_path.as_path().display()),
    ];

    std::process::Command::new("jupytext")
        .args(args)
        .output()
        .unwrap();
}

fn main() {
    let notebook_to_script = !Args::parse().reverse;
    let replacements = find_all_files(notebook_to_script);

    for replacement in replacements {
        convert(&replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs::File;
    use tempdir::TempDir;

    #[test]
    #[serial]
    fn test_find_all_files() {
        let parent_dir = TempDir::new("test_find_all_files").unwrap();

        std::env::set_current_dir(&parent_dir).unwrap();

        std::fs::create_dir(parent_dir.path().join("bar")).unwrap();

        let _ = File::create(parent_dir.path().join("baz.ipynb")).unwrap();
        let _ = File::create(parent_dir.path().join("bar/baz.ipynb")).unwrap();

        let renames = find_all_files(true);

        let expected_renames = vec![
            FileRename {
                old_path: "bar/baz.ipynb".into(),
                new_path: "bar/.nb/baz.py".into(),
                notebook_to_script: true,
            },
            FileRename {
                old_path: "baz.ipynb".into(),
                new_path: ".nb/baz.py".into(),
                notebook_to_script: true,
            },
        ];

        assert_eq!(renames, expected_renames);
    }

    #[test]
    #[serial]
    fn test_find_all_files_reverse() {
        let parent_dir = TempDir::new("test_find_all_files_reverse").unwrap();

        std::env::set_current_dir(&parent_dir).unwrap();

        std::fs::create_dir_all(parent_dir.path().join(".nb")).unwrap();
        std::fs::create_dir_all(parent_dir.path().join("bar/.nb")).unwrap();

        let _ = File::create(parent_dir.path().join(".nb/baz.py")).unwrap();
        let _ = File::create(parent_dir.path().join("bar/.nb/baz.py")).unwrap();

        let renames = find_all_files(false);

        let expected_renames = vec![
            FileRename {
                old_path: ".nb/baz.py".into(),
                new_path: "baz.ipynb".into(),
                notebook_to_script: false,
            },
            FileRename {
                old_path: "bar/.nb/baz.py".into(),
                new_path: "bar/baz.ipynb".into(),
                notebook_to_script: false,
            },
        ];

        assert_eq!(renames, expected_renames);
    }
}
