#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

use clack_extensions::params::info::ParamInfoFlags;
use clack_extensions::params::{implementation::*, info::ParamInfo, PluginParams};
use clack_plugin::{
    entry::SinglePluginEntry,
    entry::{PluginEntry, PluginEntryDescriptor},
    events::event_types::NoteEvent,
    events::list::EventList,
    events::{Event, EventType},
    extension::ExtensionDeclarations,
    host::HostHandle,
    plugin::{Plugin, Result},
    plugin::{PluginMainThread, SampleConfig},
    process::audio::Audio,
    process::events::ProcessEvents,
    process::Process,
    process::ProcessStatus,
};

pub struct GainPlugin;

impl<'a> Plugin<'a> for GainPlugin {
    type Shared = ();
    type MainThread = GainPluginMainThread;

    const ID: &'static [u8] = b"gain\0";

    fn new(
        _host: HostHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        _shared: &(),
        _sample_config: SampleConfig,
    ) -> Result<Self> {
        Ok(Self)
    }

    fn process(
        &mut self,
        _process: &Process,
        mut audio: Audio,
        events: ProcessEvents,
    ) -> Result<ProcessStatus> {
        // Only handle f32 samples for simplicity
        let io = audio.zip(0, 0).unwrap().into_f32().unwrap();

        // Supports safe in_place processing
        for (input, output) in io {
            output.set(input.get() * 2.0)
        }

        events
            .output
            .extend(events.input.iter().map(|e| match e.event() {
                Some(EventType::NoteOn(ne)) => Event::new(
                    e.time(),
                    EventType::NoteOn(NoteEvent::new(
                        ne.port_index(),
                        ne.key(),
                        ne.channel(),
                        ne.velocity() * 2.0,
                    )),
                ),
                _ => *e,
            }));

        self.flush(events.input, events.output);

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }

    fn declare_extensions(builder: &mut ExtensionDeclarations<Self>, _shared: &()) {
        builder.register::<PluginParams>();
    }
}

impl<'a> PluginParamsImpl<'a> for GainPlugin {
    fn flush(
        &mut self,
        _input_parameter_changes: &EventList,
        _output_parameter_changes: &EventList,
    ) {
    }
}

pub struct GainPluginMainThread {
    rusting: u32,
}

impl<'a> PluginMainThread<'a, ()> for GainPluginMainThread {
    fn new(_host: HostHandle<'a>, _shared: &()) -> Result<Self> {
        Ok(Self { rusting: 0 })
    }
}

impl<'a> PluginMainThreadParams<'a> for GainPluginMainThread {
    fn count(&self) -> u32 {
        1
    }

    fn get_info(&self, param_index: i32, info: &mut ParamInfoWriter) {
        if param_index > 0 {
            return;
        }

        info.set(
            ParamInfo::new(0)
                .with_name("Rusting")
                .with_module("gain/rusting")
                .with_default_value(0.0)
                .with_value_bounds(0.0, 1000.0)
                .with_flags(ParamInfoFlags::IS_STEPPED),
        )
    }

    fn get_value(&self, param_id: u32) -> Option<f64> {
        if param_id == 0 {
            Some(self.rusting as f64)
        } else {
            None
        }
    }

    fn value_to_text(
        &self,
        param_id: u32,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> ::core::fmt::Result {
        use ::core::fmt::Write;
        println!("Format param {}, value {}", param_id, value);

        if param_id == 0 {
            write!(writer, "{} crabz", value as u32)
        } else {
            Ok(())
        }
    }

    fn text_to_value(&self, _param_id: u32, _text: &str) -> Option<f64> {
        None
    }

    fn flush(&mut self, input_events: &EventList, _output_events: &EventList) {
        let value_events = input_events.iter().filter_map(|e| match e.event()? {
            EventType::ParamValue(v) => Some(v),
            _ => None,
        });

        for value in value_events {
            if value.param_id() == 0 {
                self.rusting = value.value() as u32;
            }
        }
    }
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static clap_plugin_entry: PluginEntryDescriptor = SinglePluginEntry::<GainPlugin>::DESCRIPTOR;