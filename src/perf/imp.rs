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

const PRINT_CPU_LOAD: bool = false;
const BITRATE_WINDOW_SIZE: u32 = 0;
const BITRATE_INTERVAL: u32 = 100;

#[derive(Debug)]
struct Settings {
    print_cpu_load: bool,
    bitrate_interval: u32,
    bitrate_window_size: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            print_cpu_load: PRINT_CPU_LOAD,
            bitrate_interval: BITRATE_INTERVAL,
            bitrate_window_size: BITRATE_WINDOW_SIZE,
        }
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
            vec![
                glib::ParamSpecBoolean::builder("print-cpu-load")
                    .nick("Print CPU load")
                    .blurb("Print the CPU load info")
                    .default_value(PRINT_CPU_LOAD)
                    .build(),
                glib::ParamSpecUInt::builder("bitrate-interval")
                    .nick("Interval between bitrate calculation in ms")
                    .blurb("Interval between two calculations in ms, this will run even when no buffers are received")
                    .default_value(BITRATE_INTERVAL)
                    .build(),
                glib::ParamSpecUInt::builder("bitrate-window-size")
                    .nick("Bitrate moving average window size")
                    .blurb("Number of samples used for bitrate moving average window size, 0 is all samples")
                    .default_value(BITRATE_WINDOW_SIZE)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "print-cpu-load" => {
                let print_cpu_load = value.get().expect("type checked upstream");
                let mut settings = self.settings.lock().unwrap();
                gst::info!(CAT, imp: self, "Changing print-cpu-load to {}", print_cpu_load);
                settings.print_cpu_load = print_cpu_load;
            }
            "bitrate-window-size" => {
                let bitrate_window_size = value.get().expect("type checked upstream");
                let mut settings = self.settings.lock().unwrap();
                gst::info!(CAT, imp: self, "Changing bitrate-window-size to {}", bitrate_window_size);
                settings.bitrate_window_size = bitrate_window_size;
            }
            "bitrate-interval" => {
                let bitrate_interval = value.get().expect("type checked upstream");
                let mut settings = self.settings.lock().unwrap();
                gst::info!(CAT, imp: self, "Changing bitrate-interval to {}", bitrate_interval);
                settings.bitrate_interval = bitrate_interval;
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "print-cpu-load" => {
                let settings = self.settings.lock().unwrap();
                settings.print_cpu_load.to_value()
            }
            "bitrate-window-size" => {
                let settings = self.settings.lock().unwrap();
                settings.bitrate_window_size.to_value()
            }
            "bitrate-interval" => {
                let settings = self.settings.lock().unwrap();
                settings.bitrate_interval.to_value()
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
