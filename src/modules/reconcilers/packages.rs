use std::collections::HashSet;
use std::process::Command;

pub fn reconcile_packages(desired: &[String], dry_run: bool) -> Result<(), String> {
    // Query current state
    let output = Command::new("apk")
        .arg("info")
        .output()
        .map_err(|e| e.to_string())?;
    let current: HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    let desired_set: HashSet<String> = desired.iter().cloned().collect();

    // Diff: missing = desired - current; extras = current - desired (optional removal)
    let missing: Vec<String> = desired_set.difference(&current).cloned().collect();
    let extras: Vec<String> = current.difference(&desired_set).cloned().collect(); // Decide policy on extras

    if !missing.is_empty() {
        println!("Missing packages: {:?}", missing);
        if !dry_run {
            let cmd_res = Command::new("apk").arg("add").args(&missing).output();
            let cmd = match cmd_res {
                Ok(cmd_output) => cmd_output,
                Err(err) => {
                    return Err(err.to_string());
                }
            };
            if !cmd.status.success() {
                return Err(String::from_utf8_lossy(&cmd.stderr).to_string());
            }
        }
    }

    // Similarly for extras: apk del if policy allows
    if !extras.is_empty() && /* your policy */ true {
        println!("Extra packages: {:?}", extras);
        // ...
    }

    Ok(())
}
