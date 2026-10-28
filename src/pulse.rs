use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use libpulse_binding::context::{Context, FlagSet};
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::State;
use libpulse_binding::operation::{Operation,State as OpState};
use libpulse_binding::volume::ChannelVolumes;

use crate::source::PaOutput;

fn wait_op<T>(m: &mut Mainloop, op: Operation<T>) -> Result<(), &'static str>
where T: ?Sized
{
    while op.get_state() == OpState::Running {
        match m.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                return Err("Iterate state was not success, quitting mainloop...");
            },
            IterateResult::Success(_) => {},
        }
    }

    Ok(())
}

pub struct PaContext {
    context: Box<Context>,
    mainloop: Box<Mainloop>
}

impl PaContext {
    pub fn new() -> Self {
        let mainloop = Box::new(Mainloop::new().expect("Failed to create mainloop"));
        let context = Box::new(Context::new(mainloop.as_ref(), "Audio Output Switcher").expect("Failed to create context"));
        Self{ mainloop, context }
    }

    pub fn connect_context(&mut self) -> Result<(), &'static str> {
        self.context.connect(None, FlagSet::NOFLAGS, None).expect("Failed to connect context");

        loop {
            match self.mainloop.iterate(true) {
                IterateResult::Quit(_) |
                IterateResult::Err(_) => {
                    return Err("Iterate state was not success, quitting...");
                },
                IterateResult::Success(_) => {},
            }
            match self.context.get_state() {
                State::Ready => { break; },
                State::Failed |
                State::Terminated => {
                    return Err("Context state failed/terminated, quitting...");
                },
                _ => {},
            }
        }

        Ok(())
    }

    pub fn get_pa_outputs_list(&mut self, sources: Arc<Mutex<Vec<PaOutput>>>) -> Result<(), &'static str> {
        let introspect = self.context.introspect();

        let op = introspect.get_sink_info_list({
            let sources = Arc::clone(&sources);
            move |result| {
                match result {
                    ListResult::Item(sink) => {
                        let ports = &sink.ports;
                        let active_port_name = &sink.active_port.as_deref().unwrap().name;

                        for port in ports {

                            let is_active_port = port.name.as_deref().eq(&active_port_name.as_deref());
                            let source = PaOutput {
                                is_active_port,
                                sink_name: sink.name.as_deref().unwrap_or("N/A").to_string(),
                                sink_description: sink.description.as_deref().unwrap_or("N/A").to_string(),
                                port_name: port.name.as_deref().unwrap_or("N/A").to_string(),
                                port_description: port.description.as_deref().unwrap_or("N/A").to_string(),
                                volume: sink.volume,
                                volume_known: is_active_port,
                                pending_volume: false,
                            };
                            sources.lock().unwrap().push(source);
                        }
                    }
                    ListResult::End => {}
                    ListResult::Error => {
                        eprintln!("Error occurred while fetching sink list");
                    }
                }
            }
        });

        wait_op(&mut self.mainloop, op)?;

        Ok(())
    }

    pub fn set_sink_volume(&mut self, sink_name: &str, volume: &ChannelVolumes) -> Result<(), &'static str> {
        let succeeded = Rc::new(RefCell::new(false));
        let callback_result = Rc::clone(&succeeded);
        let mut introspect = self.context.introspect();
        let callback: Box<dyn FnMut(bool)> = Box::new(move |result| {
            *callback_result.borrow_mut() = result;
        });
        let op = introspect.set_sink_volume_by_name(sink_name, volume, Some(callback));

        wait_op(&mut self.mainloop, op)?;
        if *succeeded.borrow() { Ok(()) } else { Err("can't set sink volume") }
    }

    pub fn set_pa_sink_and_port(&mut self, pa_out: &PaOutput) -> Result<(), &'static str> {
        let sink_succeeded = Rc::new(RefCell::new(false));
        let sink_result = Rc::clone(&sink_succeeded);
        let op = self.context.set_default_sink(&pa_out.sink_name, {
            move |result| {
                *sink_result.borrow_mut() = result;
            }
        });

        wait_op(&mut self.mainloop, op)?;
        if !*sink_succeeded.borrow() {
            return Err("can't set default sink");
        }

        let mut introspect = self.context.introspect();
        let port_succeeded = Rc::new(RefCell::new(false));
        let port_result = Rc::clone(&port_succeeded);

        let op = introspect.set_sink_port_by_name(&pa_out.sink_name, &pa_out.port_name, {
            let callback: Box<dyn FnMut(bool)> = Box::new(move |result| {
                *port_result.borrow_mut() = result;
            });
            Some(callback)
        });

        wait_op(&mut self.mainloop, op)?;
        if *port_succeeded.borrow() { Ok(()) } else { Err("can't set sink port") }
    }

    pub fn get_default_sink_name(&mut self) -> Result<String, &'static str> {
        let sn = Rc::new(RefCell::new(None));

        let op = self.context.introspect().get_server_info({
            let sn_clone = sn.clone();
            move |info| {
            if let Some(name) =  info.default_sink_name.clone() {
                sn_clone.replace(Some(name.to_string()));
            }
        }});

        wait_op(&mut self.mainloop, op)?;

        sn.take().ok_or("can't get default sink name")
    }
}

impl Default for PaContext {
    fn default() -> Self {
        Self::new()
    }
}
