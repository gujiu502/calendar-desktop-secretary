use crate::database::Result;
use std::{fs, path::Path};

pub fn publish(source: &Path, destination: &Path) -> Result<()> {
    match fs::rename(source, destination) {
        Ok(()) => return Ok(()),
        #[cfg(windows)]
        Err(e) if e.raw_os_error() == Some(17) => {}
        Err(e) => return Err(e.to_string()),
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_COPY_ALLOWED, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        };
        // Redirected/encrypted AppData can reject same-directory renames as cross-volume.
        // Preserve an existing export before the native copy/delete fallback.
        let recovery =
            destination.with_file_name(format!(".cds-recovery-{}.tmp", uuid::Uuid::new_v4()));
        let previous = destination.exists();
        if previous {
            fs::copy(destination, &recovery).map_err(|e| e.to_string())?;
        }
        let from = source
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let to = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            let error = std::io::Error::last_os_error();
            if previous {
                let _ = fs::copy(&recovery, destination);
                return Err(format!("{error}；原文件恢复副本：{}", recovery.display()));
            }
            // A failed new-file copy is not a completed backup.
            let _ = fs::remove_file(destination);
            return Err(error.to_string());
        }
        if previous {
            let _ = fs::remove_file(recovery);
        }
        Ok(())
    }
    #[cfg(not(windows))]
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publish_over_existing_export_in_appdata() {
        let folder = std::env::var_os("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!("cds-publish-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&folder).unwrap();
        let source = folder.join("export.tmp");
        let dest = folder.join("export.json");
        fs::write(&source, "new data").unwrap();
        fs::write(&dest, "old data").unwrap();
        publish(&source, &dest).unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "new data");
        assert!(!source.exists());
        fs::remove_dir_all(folder).unwrap();
    }
}
