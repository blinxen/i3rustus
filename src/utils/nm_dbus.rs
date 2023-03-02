use std::time::Duration;

use dbus::arg;
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use dbus::{blocking::Connection, Path};

// https://people.freedesktop.org/~lkundrak/nm-docs/spec.html

pub fn get_property<T: for<'b> arg::Get<'b> + 'static>(
    connection: &Connection,
    path: &Path,
    interface: &str,
    property_name: &str,
) -> Result<T, dbus::Error> {
    connection
        .with_proxy(
            "org.freedesktop.NetworkManager",
            path.clone(),
            Duration::new(5, 0),
        )
        .get(interface, property_name)
}

pub fn method_call<T: for<'z> dbus::arg::Get<'z> + dbus::arg::Arg, R: dbus::arg::AppendAll>(
    connection: &Connection,
    path: &Path,
    interface: &str,
    method_name: &str,
    args: R,
) -> Result<T, dbus::Error> {
    connection
        .with_proxy(
            "org.freedesktop.NetworkManager",
            path.clone(),
            Duration::new(5, 0),
        )
        .method_call(interface, method_name, args)
        .map(|r: (T,)| r.0)
}
