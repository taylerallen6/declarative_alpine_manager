use crate::modules::declaration_trait::Declaration;
use crate::modules::error::DeclarativeAlpineError;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

const PACKAGES_LOCATION: &str = "/etc/apk/world";

#[derive(Debug)]
pub struct PackagesDeclaration;

#[derive(Debug)]
pub struct PackagesDiff {
    pub missing: Vec<String>,
    pub extras: Vec<String>,
}

impl Declaration for PackagesDeclaration {
    type Desired = Vec<String>;
    type Current = HashSet<String>;
    type Diff = PackagesDiff;

    fn get_current(&self) -> Result<Self::Current, DeclarativeAlpineError> {
        let path = Path::new(PACKAGES_LOCATION);
        if !path.exists() {
            return Err(DeclarativeAlpineError::ApkWorldFileError);
        }

        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        let items: HashSet<String> = reader
            .lines()
            .filter_map(Result::ok)
            .filter(|s| !s.is_empty())
            .collect();

        Ok(items)
    }

    fn compute_diff(&self, current: &Self::Current, desired: &Self::Desired) -> Self::Diff {
        let desired_set: HashSet<String> = desired.iter().cloned().collect();
        PackagesDiff {
            missing: desired_set.difference(current).cloned().collect(),
            extras: current.difference(&desired_set).cloned().collect(),
        }
    }

    fn apply(&self, diff: &Self::Diff, dry_run: bool) -> Result<(), DeclarativeAlpineError> {
        let path = PACKAGES_LOCATION;

        let mut new_set = self.get_current()?;
        for item in &diff.missing {
            new_set.insert(item.clone());
        }
        for item in &diff.extras {
            new_set.remove(item);
        }

        println!("Packages Diff: {:?}", diff);
        println!("New apk world content: {:?}", new_set);

        if dry_run {
            return Ok(());
        }

        // INFO: Backup existing apk world file.
        fs::copy(path, format!("{}.bak", path))?;

        // INFO: Create new apk world file.
        let mut file = File::create(path)?;
        for pkg in new_set.iter() {
            writeln!(file, "{}", pkg)?;
        }

        let output = Command::new("apk").arg("upgrade").output()?;
        if !output.status.success() {
            fs::rename(format!("{}.bak", path), path)?;

            return Err(DeclarativeAlpineError::ApkUpgradeError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }
}
