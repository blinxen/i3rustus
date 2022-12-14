use std::ffi::CString;
use std::mem;
use std::path::Path;

use libc::statvfs;

use crate::config::TextColor;
use crate::utils::macros::cast_to_u64;
use crate::widgets::Widget;
use crate::widgets::WidgetError;

const DISK_THRESHOLD: u64 = 20;

// A struct that holds a Map of all paths that we want to watch over
pub struct Disk {
    path_to_watch: (String, String),
}

impl Disk {
    pub fn new(display_name: String, path: String) -> Self {
        Disk {
            path_to_watch: (display_name, path),
        }
    }

    fn calulcate_available_disk_storage(&self, path: &Path) -> u64 {
        let mut directory_size = 0;

        unsafe {
            let mut stat: statvfs = mem::zeroed();
            let path_in_c = CString::new(path.to_string_lossy().as_bytes())
                .expect("Error trying to convert disk path to a C string");
            // Look at the statvfs implementation to understand why it is way faster than to
            // calculate the value ourselfs
            if statvfs(path_in_c.as_ptr() as *const _, &mut stat) == 0 {
                directory_size =
                    cast_to_u64!(cast_to_u64!(stat.f_bsize) * cast_to_u64!(stat.f_bavail));
            }

            directory_size / 1024 / 1024 / 1024
        }
    }

    fn get_total_disk_storage(&self, path: &Path) -> u64 {
        let mut directory_size = 0;

        unsafe {
            let mut stat: statvfs = mem::zeroed();
            let path_in_c = CString::new(path.to_string_lossy().as_bytes())
                .expect("Error trying to convert disk path to a C string");
            // Look at the statvfs implementation to understand why it is way faster than to
            // calculate the value ourselfs
            if statvfs(path_in_c.as_ptr() as *const _, &mut stat) == 0 {
                directory_size =
                    cast_to_u64!(cast_to_u64!(stat.f_blocks) * cast_to_u64!(stat.f_frsize));
            }

            directory_size / 1024 / 1024 / 1024
        }
    }
}

impl Widget for Disk {
    fn name(&self) -> &str {
        "disk"
    }

    fn display_text(&self) -> Result<(String, TextColor), WidgetError> {
        // We need to borrow here because "String" does not implement the copy trait
        // and self is already borrowed. That means that we cannot move the "path_to_watch" variable
        // out of the shared reference because we don't own the reference.
        let (name, path) = &self.path_to_watch;
        let available_space = self.calulcate_available_disk_storage(Path::new(path));
        let total_space = self.get_total_disk_storage(Path::new(path));
        let color = if (available_space / total_space * 100) > DISK_THRESHOLD {
            TextColor::Critical
        } else {
            TextColor::Neutral
        };

        Ok((format!("{name}: {available_space} GiB"), color))
    }
}
