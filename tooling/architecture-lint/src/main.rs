mod lint_source;

use std::{fs, path::Path, process::ExitCode};

fn check_directory(directory: &Path, errors: &mut Vec<String>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();

        if path.is_dir() {
            check_directory(&path, errors)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path)?;

            match lint_source::lint_source(&path, &source) {
                Ok(violations) => errors.extend(violations),
                Err(error) => errors.push(format!("{}: {error}", path.display())),
            }
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut errors = Vec::new();

    for relative in ["features", "apps", "tooling/architecture-lint/src"] {
        if let Err(error) = check_directory(&root.join(relative), &mut errors) {
            errors.push(format!("{relative}: {error}"));
        }
    }

    if errors.is_empty() {
        println!("Rust structure OK: one type per file; no universal business dispatcher");
        ExitCode::SUCCESS
    } else {
        for error in errors {
            eprintln!("{error}");
        }

        ExitCode::FAILURE
    }
}
