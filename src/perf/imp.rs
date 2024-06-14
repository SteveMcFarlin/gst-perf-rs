use gst::glib;
use gst::prelude::*;
use gst_audio::subclass::prelude::*;
use gst_base::subclass::prelude::*;
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

#[derive(Debug)]
struct Settings {
    collect: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { collect: false }
    }
}

#[derive(Debug, Default)]
pub struct Perf {
    settings: Mutex<Settings>,
}

#[glib::object_subclass]
impl ObjectSubclass for Perf {
    const NAME: &'static str = "GstRsPerf";
    type Type = super::Perf;
    type ParentType = gst_base::BaseTransform;
}

impl ObjectImpl for Perf {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecBoolean::builder("collect")
                .nick("Collect")
                .blurb("Collect performance data")
                .default_value(false)
                .build()]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "collect" => {
                let collect = value.get().expect("type checked upstream");
                let mut settings = self.settings.lock().unwrap();
                gst::info!(CAT, imp: self, "Changing collect to {}", collect);
                settings.collect = collect;
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "collect" => {
                let settings = self.settings.lock().unwrap();
                settings.collect.to_value()
            }
            _ => unimplemented!(),
        }
    }
}

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
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &gst::Caps::new_any(),
            )
            .unwrap();

            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
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
// impl VideoFilterImpl for Perf {}
// impl AudioFilterImpl for Perf {}
