use std::os::fd::AsFd;
use calloop::channel as calloop_channel;
use calloop_wayland_source::WaylandSource;
use wayland_client::{Connection, Dispatch, QueueHandle, protocol::{wl_output, wl_registry}};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
    zwlr_gamma_control_v1::{self, ZwlrGammaControlV1},
};

#[derive(Debug, Clone)]
pub enum Input {
    SetBrightness(f32), // 0.0..=1.0
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready(calloop_channel::Sender<Input>),
    Failed,
    Error(String),
}

struct State {
    manager: Option<ZwlrGammaControlManagerV1>,
    output: Option<wl_output::WlOutput>,
    control: Option<ZwlrGammaControlV1>,
    gamma_size: Option<u32>,
    brightness: f32,
}

impl State {
    fn apply_brightness(&mut self, brightness: f32) {
        let Some(size) = self.gamma_size else { return };
        let brightness = brightness.clamp(0.0, 1.0);
        let mut ramp = Vec::with_capacity(size as usize * 3);
        for _ in 0..3 {
            for i in 0..size {
                let v = (i as f32 / (size - 1) as f32) * brightness * u16::MAX as f32;
                ramp.push(v.round() as u16);
            }
        }
        let fd = write_ramp_to_memfd(&ramp).expect("memfd");
        if let Some(control) = &self.control {
            control.set_gamma(fd);
        }
        self.brightness = brightness;
    }
}

pub fn connect() -> impl iced::futures::Stream<Item = Event> {
    iced::stream::channel(32, move |mut output| async move {
        let (tx, rx) = calloop_channel::channel::<Input>();
        // Own the Wayland connection on a dedicated OS thread for the
        // lifetime of the app; gamma reverts the instant this thread dies.
        std::thread::spawn(move || {
            if let Err(e) = run_wayland_thread(rx) {
                let _ = output.try_send(Event::Error(e.to_string()));
            }
        });
        let _ = output.send(Event::Ready(tx)).await;
        std::future::pending::<()>().await; // keep the stream alive forever
    })
}

fn run_wayland_thread(
    rx: calloop_channel::Channel<Input>,
) -> anyhow::Result<()> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut queue) = wayland_client::globals::registry_queue_init::<State>(&conn)?;
    let qh = queue.handle();
    let manager: ZwlrGammaControlManagerV1 =
        globals.bind(&qh, 1..=1, ())?;
    let output: wl_output::WlOutput = globals.bind(&qh, 1..=4, ())?;
    let control = manager.get_gamma_control(&output, &qh, ());
    let mut state = State {
        manager: Some(manager), output: Some(output),
        control: Some(control), gamma_size: None,
        brightness: 1.0,
    };
    queue.roundtrip(&mut state)?; // pick up the gamma_size event
    let mut event_loop = calloop::EventLoop::<State>::try_new()?;
    WaylandSource::new(conn, queue).insert(event_loop.handle())?;
    event_loop.handle().insert_source(rx, |ev, _, state| {
        if let calloop_channel::Event::Msg(Input::SetBrightness(b)) = ev {
            state.apply_brightness(b);
        }
    })?;
    event_loop.run(None, &mut state, |_| {})?;
    Ok(())
}
