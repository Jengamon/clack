use clack_plugin::plugin::descriptor::{PluginDescriptor, StaticPluginDescriptor};
use clack_plugin::prelude::*;
use std::ffi::CStr;

use clack_host::prelude::*;

pub struct DivaPluginStub;
pub struct DivaPluginStubMainThread;

impl<'a> PluginMainThread<'a, ()> for DivaPluginStubMainThread {
    fn new(_host: HostMainThreadHandle<'a>, _shared: &'a ()) -> Result<Self, PluginError> {
        Err(PluginError::AlreadyActivated)
    }
}

impl<'a> Plugin<'a> for DivaPluginStub {
    type Shared = ();
    type MainThread = DivaPluginStubMainThread;

    fn get_descriptor() -> Box<dyn PluginDescriptor> {
        use clack_plugin::plugin::descriptor::features::*;

        Box::new(StaticPluginDescriptor {
            id: CStr::from_bytes_with_nul(b"com.u-he.diva\0").unwrap(),
            name: CStr::from_bytes_with_nul(b"Diva\0").unwrap(),
            features: Some(&[SYNTHESIZER, STEREO]),
            ..Default::default()
        })
    }

    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut Self::MainThread,
        _shared: &'a Self::Shared,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        unreachable!()
    }

    fn process(
        &mut self,
        _process: &Process,
        _audio: Audio,
        _events: ProcessEvents,
    ) -> Result<ProcessStatus, PluginError> {
        unreachable!()
    }
}

static DIVA_STUB_ENTRY: PluginEntryDescriptor = SinglePluginEntry::<DivaPluginStub>::DESCRIPTOR;

struct MyHostShared;
impl<'a> HostShared<'a> for MyHostShared {
    fn request_restart(&self) {
        unreachable!()
    }
    fn request_process(&self) {
        unreachable!()
    }
    fn request_callback(&self) {
        unreachable!()
    }
}

struct MyHost;
impl<'a> Host<'a> for MyHost {
    type Shared = MyHostShared;

    type MainThread = ();
    type AudioProcessor = ();
}

#[test]
pub fn handles_instanciation_errors() {
    let bundle = unsafe {
        PluginBundle::load_from_raw(&DIVA_STUB_ENTRY, "/home/user/.clap/u-he/libdiva.so").unwrap()
    };
    let host_info =
        HostInfo::new("Legit Studio", "Legit Ltd.", "https://example.com", "4.3.2").unwrap();

    let plugin_instance = PluginInstance::<MyHost>::new(
        |_| MyHostShared,
        |_| (),
        &bundle,
        CStr::from_bytes_with_nul(b"com.u-he.diva\0").unwrap(),
        &host_info,
    );

    if plugin_instance.is_ok() {
        panic!("Instanciation should have failed")
    }
}