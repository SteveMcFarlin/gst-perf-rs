use gst::glib;
use gst::prelude::*;

mod imp;

glib::wrapper! {
    pub struct Perf(ObjectSubclass<imp::Perf>)
        @extends gst::Element, gst_base::BaseTransform, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(Some(plugin), "rsperf", gst::Rank::NONE, Perf::static_type())
}
