use std::fmt::Display;
use std::os::unix;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug, Default)]
pub struct Config {
    pub difference: i32,
    pub recursive: bool,
    pub verbose: bool,
    pub no_permissions: bool,
    pub with_xattr: bool,
    pub dry_run: bool,
}

impl Config {
    fn log_verbose(&self, msg: impl Display) {
        if self.verbose {
            println!("{}", msg)
        }
    }
}

pub fn apply(c: Config, path: PathBuf) -> io::Result<()> {
    for simulate in [true, false] {
        let label = if simulate { "Simulating..." } else { "Applying..." };
        c.log_verbose(label);

        if c.recursive {
            apply_recursive(&c, &path, simulate)?;
        } else {
            let metadata = fs::symlink_metadata(&path)?;
            let mut known_inodes = Vec::new();
            apply_to_item(&c, metadata, &path, &mut known_inodes, simulate)?;
        }

        if c.dry_run {
            break;
        }
    }

    Ok(())
}

fn apply_recursive(c: &Config, path: &Path, simulate: bool) -> io::Result<()> {
    let mut stack = vec![path.to_path_buf()];
    let mut known_inodes: Vec<u64> = Vec::new();

    while let Some(dir_path) = stack.pop() {
        let entries = fs::read_dir(&dir_path).map_err(|e| {
            io::Error::new(e.kind(), format!("reading dir '{}': {}", dir_path.display(), e))
        })?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            // entry.metadata is fs::symlink_metadata
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                stack.push(entry_path.clone());
            }
            apply_to_item(c, metadata, &entry_path, &mut known_inodes, simulate).map_err(|e| {
                io::Error::new(e.kind(), format!("file '{}': {}", path.display(), e))
            })?
        }
    }

    Ok(())
}


fn apply_to_item(c: &Config, meta :fs::Metadata, path: &PathBuf, known_inodes: &mut Vec<u64>, simulate: bool) -> io::Result<()> {
    let nlink = meta.nlink();
    let inode = meta.ino();

    if nlink > 1 {
        if known_inodes.contains(&inode) {
            c.log_verbose(format!("[Simulating] {}: Skip known hard link ", path.display()));
            return Ok(());
        }
        known_inodes.push(inode);
    }

    let start_gid = meta.gid();
    let start_uid = meta.uid();

    let (target_gid, target_uid) = match (
        start_gid.checked_add_signed(c.difference),
        start_uid.checked_add_signed(c.difference),
    ) {
        (Some(g), Some(u)) => (g, u),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("UID/GID overflow for {} (uid:{}, gid:{})", path.display(), start_uid, start_gid),
            ));
        }
    };

    let action_label = if c.dry_run { "[Simulating]" } else { "[Changing]" };
    c.log_verbose(format!(
        "{} {}: {},{} -> {},{}",
        action_label, path.display(), start_uid, start_gid, target_uid, target_gid
    ));

    // Xattr key that holds file capability sets
    let cap_xattr = "security.capability";
    let mut saved_caps = None;
    if c.with_xattr {
        saved_caps = xattr::get(path, cap_xattr)?;
    }

    if c.dry_run || simulate {
        return Ok(());
    }

    let perms = meta.permissions();

    // lchown does not dereference symbolic links
    unix::fs::lchown(path, Some(target_uid), Some(target_gid)).map_err(|e| {
        io::Error::new(e.kind(), format!("chown: {}", e))
    })?;

    if !c.no_permissions {
        // Don't restore permissions for symlinks as that would affect the target
        // (unix does not support permissions on symlinks)
        if !meta.file_type().is_symlink() {
            // Restore permissions (setuid, setgid get cleared after chown)
            if let Err(e) = fs::set_permissions(path, perms) {
                return Err(io::Error::new(
                    e.kind(),
                    format!("chmod: {}", e)
                ));
            }
        }
    }

    // Restore xattr
    if let Some(val) = saved_caps {
        c.log_verbose(format!("Restoring capability set for file '{}'", path.display() ));
        xattr::set(path, cap_xattr, &val)?;
    }

    Ok(())
}
