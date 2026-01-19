use crate::modules::error::DeclarativeAlpineError;
use std::fmt::Debug;

pub trait Declaration: Debug {
    type Desired: Debug; // From config.toml
    type Current: Debug; // Parsed from files
    type Diff: Debug; // Changes to apply

    fn get_current(&self) -> Result<Self::Current, DeclarativeAlpineError>; // Read files
    fn compute_diff(&self, current: &Self::Current, desired: &Self::Desired) -> Self::Diff; // Memory diff
    fn apply(&self, diff: &Self::Diff, dry_run: bool) -> Result<(), DeclarativeAlpineError>; // Write files atomically + minimal activation
}

// Helper to run (same as before)
pub fn reconcile<D: Declaration>(
    declaration: &D,
    desired: &D::Desired,
    dry_run: bool,
) -> Result<(), DeclarativeAlpineError> {
    let current = declaration.get_current()?;
    let diff = declaration.compute_diff(&current, desired);
    println!("Diff: {:?}", diff);
    declaration.apply(&diff, dry_run)
}
