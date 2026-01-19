// src/reconcilers/users.rs
use crate::reconcilers::Declaration;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command; // For minimal shell if hashing password

// User: Struct for user details (from config.toml)
// Note: Password can be plain (will hash in apply) or pre-hashed (starts with $).
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub groups: Vec<String>,
    pub password: Option<String>, // Plain or hashed
    pub shell: String,
    pub home: String,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

// Custom struct for users (empty; could add flags later)
#[derive(Debug)]
pub struct UsersDeclaration;

// Diff for users
#[derive(Debug)]
pub struct UsersDiff {
    pub to_add: Vec<User>,
    pub to_update: Vec<(String, User)>, // (username, new attrs)
    pub to_remove: Vec<String>,
}

impl Declaration for UsersDeclaration {
    const PATH: &'static str = "/etc/passwd"; // Main file; also shadow/group

    type Desired = Vec<User>;
    type Current = HashMap<String, User>;
    type Diff = UsersDiff;

    fn get_current(&self) -> Result<Self::Current, String> {
        let passwd_path = Path::new(Self::PATH);
        let passwd_content = fs::read_to_string(passwd_path).map_err(|e| e.to_string())?;
        let mut current = HashMap::new();

        for line in passwd_content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 7 {
                let username = parts[0].to_string();
                let uid = parts[2].parse::<u32>().ok();
                let gid = parts[3].parse::<u32>().ok();
                let home = parts[5].to_string();
                let shell = parts[6].to_string();
                let groups = get_groups_for_user(&username)?; // Fetch from /etc/group

                current.insert(
                    username.clone(),
                    User {
                        username,
                        groups,
                        password: None, // Not fetching from shadow for security; assume managed separately
                        shell,
                        home,
                        uid,
                        gid,
                    },
                );
            }
        }
        Ok(current)
    }

    fn compute_diff(&self, current: &Self::Current, desired: &Self::Desired) -> Self::Diff {
        let desired_map: HashMap<String, User> = desired
            .iter()
            .cloned()
            .map(|u| (u.username.clone(), u))
            .collect();

        let mut to_add = Vec::new();
        let mut to_update = Vec::new();
        let mut to_remove = Vec::new();

        for username in desired_map.keys() {
            if !current.contains_key(username) {
                if let Some(user) = desired_map.get(username) {
                    to_add.push(user.clone());
                }
            }
        }

        for (username, current_user) in current {
            match desired_map.get(username) {
                Some(desired_user) => {
                    if current_user != desired_user {
                        to_update.push((username.clone(), desired_user.clone()));
                    }
                }
                None => to_remove.push(username.clone()),
            }
        }

        UsersDiff {
            to_add,
            to_update,
            to_remove,
        }
    }

    fn apply(&self, diff: &Self::Diff, dry_run: bool) -> Result<(), String> {
        if dry_run {
            return Ok(());
        }

        // Backup files
        backup_user_files()?;

        // For passwd: Read current lines, modify, write new
        let mut new_passwd_lines = read_current_lines(Self::PATH)?;

        for user in &diff.to_add {
            let uid = user.uid.unwrap_or(get_next_uid(&new_passwd_lines)?);
            let gid = user.gid.unwrap_or(get_next_gid()?);
            let new_line = format!(
                "{}:x:{}:{}::{}:{}",
                user.username, uid, gid, user.home, user.shell
            );
            new_passwd_lines.push(new_line);
            // Update shadow and groups
            update_shadow(&user.username, &user.password)?;
            update_groups(&user.username, &user.groups)?;
            // Minimal shell: Create home if not exists
            if !Path::new(&user.home).exists() {
                Command::new("mkdir")
                    .arg("-p")
                    .arg(&user.home)
                    .output()
                    .map_err(|e| e.to_string())?;
            }
        }

        for (username, user) in &diff.to_update {
            // Find and replace line in new_passwd_lines
            if let Some(index) = new_passwd_lines
                .iter()
                .position(|line| line.starts_with(&format!("{}:", username)))
            {
                let uid = user
                    .uid
                    .unwrap_or_else(|| parse_uid_from_line(&new_passwd_lines[index]));
                let gid = user
                    .gid
                    .unwrap_or_else(|| parse_gid_from_line(&new_passwd_lines[index]));
                new_passwd_lines[index] = format!(
                    "{}:x:{}:{}::{}:{}",
                    username, uid, gid, user.home, user.shell
                );
            }
            update_shadow(username, &user.password)?;
            update_groups(username, &user.groups)?;
        }

        for username in &diff.to_remove {
            // Remove line from new_passwd_lines
            new_passwd_lines.retain(|line| !line.starts_with(&format!("{}:", username)));
            // Clean shadow and groups
            remove_from_shadow(username)?;
            remove_from_groups(username)?;
        }

        // Write new passwd
        let mut file = File::create(Self::PATH).map_err(|e| e.to_string())?;
        for line in &new_passwd_lines {
            writeln!(file, "{}", line).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

// Helper functions
fn backup_user_files() -> Result<(), String> {
    fs::copy("/etc/passwd", "/etc/passwd.bak").map_err(|e| e.to_string())?;
    fs::copy("/etc/shadow", "/etc/shadow.bak").map_err(|e| e.to_string())?;
    fs::copy("/etc/group", "/etc/group.bak").map_err(|e| e.to_string())?;
    Ok(())
}

fn read_current_lines(path: &str) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

fn get_groups_for_user(username: &str) -> Result<Vec<String>, String> {
    let content = fs::read_to_string("/etc/group").map_err(|e| e.to_string())?;
    let mut groups = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() >= 4 {
            let members: Vec<&str> = parts[3].split(',').collect();
            if members.contains(&username) {
                groups.push(parts[0].to_string());
            }
        }
    }
    Ok(groups)
}

fn update_shadow(username: &str, password: &Option<String>) -> Result<(), String> {
    if let Some(pw) = password {
        // Use shell for hashing (minimal); alternative: Use rust-crypt crate
        let output = Command::new("mkpasswd")
            .arg(pw)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        let hashed = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Append/update shadow line
        let mut new_shadow_content = read_current_lines("/etc/shadow")?;

        if let Some(index) = new_shadow_content
            .iter()
            .position(|line| line.starts_with(&format!("{}:", username)))
        {
            let parts: Vec<&str> = new_shadow_content[index].split(':').collect();
            new_shadow_content[index] = format!(
                "{}:{}:{}:{}:{}:{}:::",
                username,
                hashed,
                parts.get(2).unwrap_or(&"19345"), // Preserve dates or defaults
                parts.get(3).unwrap_or(&"0"),
                parts.get(4).unwrap_or(&"99999"),
                parts.get(5).unwrap_or(&"7")
            );
        } else {
            new_shadow_content.push(format!("{}:{}:19345:0:99999:7:::", username, hashed));
        }

        let username_start_text = format!("{}:", username);
        match new_shadow_content
            .iter()
            .position(|line| line.starts_with(&username_start_text))
        {
            Some(index) => {
                let line_items: Vec<&str> = new_shadow_content[index].split(':').collect();

                new_shadow_content[index] = format!(
                    "{}:{}:{}:{}:{}:{}:::",
                    username,
                    hashed,
                    line_items.get(2).unwrap_or(&"19345"), // Preserve dates or defaults
                    line_items.get(3).unwrap_or(&"0"),
                    line_items.get(4).unwrap_or(&"99999"),
                    line_items.get(5).unwrap_or(&"7")
                );
            }
            None => new_shadow_content.push(format!("{}:{}:19345:0:99999:7:::", username, hashed)),
        }

        let mut file = File::create("/etc/shadow").map_err(|e| e.to_string())?;
        for line in &new_shadow_content {
            writeln!(file, "{}", line).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn remove_from_shadow(username: &str) -> Result<(), String> {
    let mut new_shadow_content = read_current_lines("/etc/shadow")?;
    new_shadow_content.retain(|line| !line.starts_with(&format!("{}:", username)));

    let mut file = File::create("/etc/shadow").map_err(|e| e.to_string())?;

    for line in &new_shadow_content {
        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn update_groups(username: &str, groups: &Vec<String>) -> Result<(), String> {
    let mut new_group = read_current_lines("/etc/group")?;
    for group in groups {
        if let Some(index) = new_group
            .iter()
            .position(|line| line.starts_with(&format!("{}:", group)))
        {
            let mut parts: Vec<&str> = new_group[index].split(':').collect();
            let mut members = if parts.len() >= 4 {
                parts[3].split(',').collect::<Vec<&str>>()
            } else {
                Vec::new()
            };
            if !members.contains(&username.as_str()) {
                members.push(username);
                parts[3] = &members.join(",");
                new_group[index] = parts.join(":");
            }
        } else {
            // Add new group if not exists
            let gid = get_next_gid()?;
            new_group.push(format!("{}:x:{}:{}", group, gid, username));
        }
    }
    let mut file = File::create("/etc/group").map_err(|e| e.to_string())?;
    for line in &new_group {
        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn remove_from_groups(username: &str) -> Result<(), String> {
    let mut new_group = read_current_lines("/etc/group")?;
    for i in 0..new_group.len() {
        let mut parts: Vec<&str> = new_group[i].split(':').collect();
        if parts.len() >= 4 {
            let members: Vec<&str> = parts[3].split(',').filter(|&m| m != username).collect();
            parts[3] = &members.join(",");
            new_group[i] = parts.join(":");
        }
    }
    let mut file = File::create("/etc/group").map_err(|e| e.to_string())?;
    for line in &new_group {
        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn get_next_uid(passwd_lines: &Vec<String>) -> Result<u32, String> {
    let max = passwd_lines
        .iter()
        .filter_map(|line| line.split(':').nth(2).and_then(|s| s.parse::<u32>().ok()))
        .max()
        .unwrap_or(1000);
    Ok(max + 1)
}

fn get_next_gid() -> Result<u32, String> {
    let content = fs::read_to_string("/etc/group").map_err(|e| e.to_string())?;
    let max_gid = content
        .lines()
        .filter_map(|line| line.split(':').nth(2).and_then(|s| s.parse::<u32>().ok()))
        .max()
        .unwrap_or(1000);
    Ok(max_gid + 1)
}

fn parse_uid_from_line(line: &str) -> u32 {
    line.split(':')
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
}

fn parse_gid_from_line(line: &str) -> u32 {
    line.split(':')
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
}
