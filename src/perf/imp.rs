use gst::glib;
use gst::prelude::*;
use gst_audio::subclass::prelude::*;
use gst_video::subclass::prelude::*;

use std::sync::Mutex;

use once_cell::sync::Lazy;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "rsperf",
        gst::DebugColorFlags::empty(),
        Some("Pipeline Performance Monitor"),
    )
});

pub struct Perf {}

#[glib::object_subclass]
impl ObjectSubclass for Perf {
    const NAME: &'static str = "GstRsPerf";
    type Type = super::Perf;
    type ParentType = gst::Element;
}

impl Default for Perf {
    fn default() -> Self {
        Perf {}
    }
}

impl ObjectImpl for Perf {}
impl GstObjectImpl for Perf {}

impl ElementImpl for Perf {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "Pipeline Performance Monitor",
                "Filter/Audio/Video",
                "Measure framerate, bitrate and CPU usage",
                "RidgeRun <http://www.ridgerun.com>, Steve McFarlin <steve@stevemcfarlin.com>",
            )
        });
        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let src_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &gst::Caps::new_any(),
            )
            .unwrap();

            let sink_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &gst::Caps::new_any(),
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });
        PAD_TEMPLATES.as_ref()
    }
}

impl BaseTransformImpl for Perf {
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::NeverInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;
}
impl VideoFilterImpl for Perf {}
impl AudioFilterImpl for Perf {}
