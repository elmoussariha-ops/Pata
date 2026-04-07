use std::path::Path;

#[cfg(unix)]
pub fn set_secure_dir(path: &Path) -> Result<(), String> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    let perm = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, perm).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
pub fn set_secure_dir(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
pub fn set_secure_file(path: &Path) -> Result<(), String> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    let perm = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perm).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
pub fn set_secure_file(_path: &Path) -> Result<(), String> {
    Ok(())
}
