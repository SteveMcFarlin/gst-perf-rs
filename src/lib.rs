// Take a look at the license at the top of the repository in the LICENSE file.
#![allow(unused_doc_comments)]

//!  GStreamer element to measure framerate, bitrate and CPU usage

use gst::glib;
mod perf;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    perf::register(plugin)?;
    Ok(())
}

gst::plugin_define!(
    rsperf,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "LGPL2.1+",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);
