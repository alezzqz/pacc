use std::sync::{Arc, Mutex};

use libpulse_binding::context::introspect::Introspector;
use libpulse_binding::context::{Context, FlagSet};
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::State;
use libpulse_binding::operation::{Operation,State as OpState};

use crate::source::PaSource;

fn wait_op<T>(m: &mut Mainloop, op: Operation<T>) -> Result<(), &'static str>
where T: ?Sized
{
    while op.get_state() == OpState::Running {
        match m.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                eprintln!("");
                return Err("Iterate state was not success, quitting mainloop...");
            },
            IterateResult::Success(_) => {},
        }
    }

    Ok(())
}

pub fn connect_context(mainloop: &mut Mainloop, context: &mut Context) -> Result<(), &'static str> {
    context.connect(None, FlagSet::NOFLAGS, None).expect("Failed to connect context");

    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                return Err("Iterate state was not success, quitting...");
            },
            IterateResult::Success(_) => {},
        }
        match context.get_state() {
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

pub fn get_pa_outputs_list(mut mainloop: &mut Mainloop, introspect: &Introspector, sources: Arc<Mutex<Vec<PaSource>>>) -> Result<(), &'static str> {
    let op = introspect.get_sink_info_list({
        let sources = Arc::clone(&sources);
        move |result| {
            match result {
                ListResult::Item(sink) => {
                    let ports = &sink.ports;

                    for idx in 0..ports.iter().count() {
                        let port = &ports[idx];

                        let source = PaSource {
                            sink_name: sink.name.as_deref().unwrap_or("N/A").to_string(),
                            sink_description: sink.description.as_deref().unwrap_or("N/A").to_string(),
                            port_name: port.name.as_deref().unwrap_or("N/A").to_string(),
                            port_description: port.description.as_deref().unwrap_or("N/A").to_string(),
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

    wait_op(&mut mainloop, op)?;

    Ok(())
}

pub fn set_pa_sink_and_port(mut mainloop: &mut Mainloop, context: &mut Context, pa_out: &PaSource) -> Result<(), &'static str> {
    let op = context.set_default_sink(&pa_out.sink_name, {
        let sink_name = pa_out.sink_name.clone();
        move |result| {
            if !result {
                eprintln!("can't set sink '{}'", sink_name);
            }
        }
    });

    wait_op(&mut mainloop, op)?;

    let mut introspect = context.introspect();

    let op = introspect.set_sink_port_by_name(&pa_out.sink_name, &pa_out.port_name, {
        let sink_name = pa_out.sink_name.clone();
        let port_name = pa_out.port_name.clone();
        Some(Box::new(move |result| {
            if !result {
                eprintln!("can't set port '{}' to sink '{}'", port_name, sink_name);
            }
        }))
    });

    wait_op(&mut mainloop, op)?;

    Ok(())
}
