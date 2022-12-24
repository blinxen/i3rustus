use std::ffi::CString;
use std::mem;
use std::path::Path;

use libc::statvfs;
use serde::Serialize;
use serde_json::Value;

use crate::config::NEUTRAL;
use crate::config::RED;
use crate::utils::macros::cast_to_u64;
use crate::widgets::Widget;
use crate::widgets::WidgetError;

const DISK_THRESHOLD: f64 = 20.0;

#[derive(Serialize)]
// A struct that holds a Map of all paths that we want to watch over
pub struct Disk<'a> {
    // Name of the widget
    name: &'a str,
    // Text that will be shown in the status bar
    full_text: Option<String>,
    // Color of the text
    color: &'a str,
    // Paths to watch
    #[serde(skip_serializing)]
    path_to_watch: (String, String),
}

impl<'a> Disk<'a> {
    pub fn new(display_name: String, path: String) -> Self {
        Disk {
            name: "disk",
            full_text: None,
            color: RED,
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

impl<'a> Widget for Disk<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn update(&mut self) {
        // We need to borrow here because "String" does not implement the copy trait
        // and self is already borrowed. That means that we cannot move the "path_to_watch" variable
        // out of the shared reference because we don't own the reference.
        let (name, path) = &self.path_to_watch;
        let available_space = self.calulcate_available_disk_storage(Path::new(path));
        let total_space = self.get_total_disk_storage(Path::new(path));
        self.color = if (available_space as f64 / total_space as f64 * 100.0) < DISK_THRESHOLD {
            RED
        } else {
            NEUTRAL
        };

        self.full_text = Some(format!("{name}: {available_space} GiB"));
    }

    fn display_text(&self) -> Result<Value, WidgetError> {
        Ok(serde_json::to_value(self)?)
    }
}
