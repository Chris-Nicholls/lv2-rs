#[cfg(feature = "plugin")]
mod effect;
use effect::*;


#[cfg(feature = "plugin")]
use lv2::prelude::*;


#[cfg(feature = "plugin")]
#[derive(PortCollection)]
pub struct Ports {
    left_in: InputPort<Audio>,
    right_in: InputPort<Audio>,

    left_output: OutputPort<Audio>,
    right_output: OutputPort<Audio>,

    mix: InputPort<Control>,
}

#[cfg(feature = "plugin")]
unsafe impl UriBound for Effect {
    const URI: &'static [u8] = b"https://github.com/Chris-Nicholls/lv2-rs\0";
}

#[cfg(feature = "plugin")]
impl Plugin for Effect {
    type Ports = Ports;

    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        Some(Effect::new())
    }

    fn activate(&mut self, _features: &mut ()) {}

    fn run<'a>(&'a mut self, ports: &'a mut Ports, _features: &mut ()) {
        let mut i = 0;
        while i < ports.left_in.len() {
            let left = ports.left_in[i];
            let right = ports.left_in[i];
            let (l, r) = self.add_sample(left + right);
            ports.left_output[i] = l;
            ports.right_output[i] = r;
            i += 1;
        }
    }
}

#[cfg(feature = "plugin")]
lv2_descriptors!(Effect);
