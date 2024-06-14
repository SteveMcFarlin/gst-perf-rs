use gst::glib;
use gst::prelude::*;
use gst_audio::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_video::subclass::prelude::*;

use anyhow::Result;
use std::f32::consts::E;
use std::sync::Mutex;
use std::time::Instant;

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
#[allow(dead_code)]
enum PerfError {
    UnsupportedPlatform, // This code ignores this error
    CpuLoad(String),
    UpdateBps(String),
    Start(String),
    Stop(String),
}

#[derive(Debug)]
struct State {
    // Note: This may need to be optional. In the algo from GstPerf
    // there is the use of GST_CLOCK_TIME_NONE
    prev_time: Instant,

    fps: f64,
    frame_count: u32,
    frame_count_total: u64,

    bps: f64,
    mean_bps: f64,
    bps_window_buffer: Vec<f64>,
    bps_window_size: u32,
    bps_window_buffer_current: u32,
    byte_count: u64,
    byte_count_total: u64,
    bps_interval: u32,
    bps_running_interval: u32,
    // Do we need mutexes?
    bps_source_id: u32,

    prev_cpu_total: u32,
    prev_cpu_idle: u32,

    print_cpu_load: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            prev_time: Instant::now(),
            fps: 0.0,
            frame_count: 0,
            frame_count_total: 0,
            bps: 0.0,
            mean_bps: 0.0,
            bps_window_buffer: Vec::with_capacity(BITRATE_WINDOW_SIZE as usize),
            bps_window_size: BITRATE_WINDOW_SIZE,
            bps_window_buffer_current: 0,
            byte_count: 0,
            byte_count_total: 0,
            bps_interval: BITRATE_INTERVAL,
            bps_running_interval: BITRATE_INTERVAL,
            bps_source_id: 0,

            prev_cpu_total: 0,
            prev_cpu_idle: 0,

            print_cpu_load: PRINT_CPU_LOAD,
        }
    }
}

#[derive(Debug, Default)]
pub struct Perf {
    settings: Mutex<State>,
}

impl Perf {
    pub fn start() -> Result<(), PerfError> {
        Err(PerfError::Start("Failed to start".to_string()))
    }

    pub fn stop() -> Result<(), PerfError> {
        Err(PerfError::Stop("Failed to stop".to_string()))
    }

    pub fn update_bps(settings: &mut State) -> Result<u32, PerfError> {
        let byte_count = settings.byte_count;
        settings.byte_count = 0;

        Err(PerfError::UpdateBps("Failed to update bps".to_string()))
    }

    pub fn compute_cpu(settings: &mut State, current_idle: u32, current_total: u32) -> u32 {
        let idle = current_idle - settings.prev_cpu_idle;
        let total = current_total - settings.prev_cpu_total;

        if total != 0 {
            let busy = total - idle;
            (1000 * busy / total + 5) / 10
        } else {
            0
        }
    }

    #[cfg(target_os = "linux")]
    pub fn get_load(settings: &State) -> Result<u32, PerfError> {
        Err(PerfError::CpuLoad("Failed to get CPU load".to_string()))
    }

    // TODO: Note this is only the algorithm. We need to pull in the
    // C calls to get the CPU load.
    #[cfg(target_os = "macos")]
    pub fn get_load(settings: &State) -> Result<u32, PerfError> {
        let idle = u32;
        let total = u32;

        // I have no idea why the C code returns false if the count is
        // 0. I'm going to assume the reason is that must be an error
        // given the pipeline is running. Again the C code ignores the
        // return
        if cpu_load == 0 {
            return false;
        }

        // This is the C code. Need to translate to Rust. Time to learn
        // how to interface with C code.

        // host_cpu_load_info_data_t cpuinfo = {0};
        // mach_msg_type_number_t count = HOST_CPU_LOAD_INFO_COUNT;

        // let host_stats = host_statistics(mach_host_self(), HOST_CPU_LOAD_INFO, &cpuinfo, &count);
        // if host_stats != KERN_SUCCESS {
        //     gst::error!(CAT, "Failed to get CPU load info");
        //     return false;
        // }

        // cpu_load =
    }

    #[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    // Add other supported operating systems here
)))]
    pub fn get_load(_settings: &Setting) -> Result<u32, PerfError> {
        // Simply return true as we don't know how to get the CPU load
        // on this platform. The C code never checks the return value.
        Err(PerfError::UnsupportedPlatform)
    }

    pub fn update_average(count: u64, current: f64, old: f64) -> f64 {
        if count != 0 {
            ((count - 1) as f64 * old + current) / count as f64
        } else {
            0.0
        }
    }

    pub fn update_moving_average(
        window_size: u32,
        old_average: f64,
        new_sample: f64,
        old_sample: f64,
    ) -> f64 {
        let mut new_average = 0.0;

        if window_size != 0 {
            new_average =
                (old_average * window_size as f64 - old_sample + new_sample) / window_size as f64;
        }

        new_average
    }

    pub fn reset(settings: &mut State) {
        settings.frame_count = 0;
    }

    pub fn clear(settings: &mut State) {
        settings.fps = 0.0;
        settings.frame_count_total = 0;

        settings.bps = 0.0;
        settings.byte_count = 0;
        settings.byte_count_total = 0;

        settings.prev_time = Instant::now();
        settings.prev_cpu_total = 0;
        settings.prev_cpu_idle = 0;
    }
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
                settings.bps_window_size = bitrate_window_size;
            }
            "bitrate-interval" => {
                let bitrate_interval = value.get().expect("type checked upstream");
                let mut settings = self.settings.lock().unwrap();
                gst::info!(CAT, imp: self, "Changing bitrate-interval to {}", bitrate_interval);
                settings.bps_interval = bitrate_interval;
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
                settings.bps_window_size.to_value()
            }
            "bitrate-interval" => {
                let settings = self.settings.lock().unwrap();
                settings.bps_interval.to_value()
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

    fn transform_ip(
        &self,
        buffer: &mut gst::BufferRef,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::debug!(CAT, "perf->transform_ip");

        Err(gst::FlowError::Error)
    }
}
