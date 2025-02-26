use std::env;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use libpulse_binding::context::{Context, FlagSet};
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::State;
use libpulse_binding::operation::{Operation,State as OpState};

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

fn main() {
    if env::args().find(|x| x == "--version") != None {
        println!("pacc version 1.0");
        return;
    }

    let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
    let mut context = Context::new(&mainloop, "Audio Output Switcher").expect("Failed to create context");

    context.connect(None, FlagSet::NOFLAGS, None).expect("Failed to connect context");

    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                eprintln!("Iterate state was not success, quitting...");
                return;
            },
            IterateResult::Success(_) => {},
        }
        match context.get_state() {
            State::Ready => { break; },
            State::Failed |
            State::Terminated => {
                eprintln!("Context state failed/terminated, quitting...");
                return;
            },
            _ => {},
        }
    }

    struct Source {
        sink_idx: u32,
        sink_name: String,
        sink_description: String,
        port_idx: u32,
        port_name: String,
        port_description: String
    }

    let sources = Arc::new(Mutex::new(Vec::new()));
    let mut introspect = context.introspect();

    let op = introspect.get_sink_info_list({
        let sources = Arc::clone(&sources);
        move |result| {
            match result {
                ListResult::Item(sink) => {
                    let ports = &sink.ports;

                    for idx in 0..ports.iter().count() {
                        let port = &ports[idx];

                        let source = Source {
                            sink_idx: sink.index,
                            sink_name: sink.name.as_deref().unwrap_or("N/A").to_string(),
                            sink_description: sink.description.as_deref().unwrap_or("N/A").to_string(),
                            port_idx: idx as u32,
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

    match wait_op(&mut mainloop, op) {
        Err(e) => { eprintln!("{}", e); return; },
        Ok(_) => {}
    }

    println!("Detected outputs:");
    let sources_res = sources.lock().unwrap();

    for src_idx in 0..sources_res.iter().count() {
        let source = &sources_res[src_idx];
        println!("Output #{} - Sink #{}: {}, Port #{} '{}'",
            src_idx,
            source.sink_idx,
            source.sink_description,
            source.port_idx,
            source.port_description
        );
    }

    print!("Choose output or 'x' to exit: ");
    io::stdout().flush().expect("flush failed");

    let mut inp = String::new();
    io::stdin().read_line(&mut inp).expect("failed to read line");
    if inp.trim() == "x" {
        return;
    }

    let inp: usize = match inp.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Error: enter number of Output");
            return;
        }
    };

    if inp >= sources_res.iter().count() {
        eprintln!("Error: enter correct output number");
        return;
    }

    let choosen_src = &sources_res[inp];
    println!("Set Output: {}, '{}'", choosen_src.sink_description, choosen_src.port_description);

    let op = context.set_default_sink(&choosen_src.sink_name, {
        let sink_name = choosen_src.sink_name.clone();
        move |result| {
            if !result {
                eprintln!("can't set sink '{}'", sink_name);
            }
        }
    });

    match wait_op(&mut mainloop, op) {
        Err(e) => { eprintln!("{}", e); return; },
        Ok(_) => {}
    }

    let op = introspect.set_sink_port_by_name(&choosen_src.sink_name, &choosen_src.port_name, {
        let sink_name = choosen_src.sink_name.clone();
        let port_name = choosen_src.port_name.clone();
        Some(Box::new(move |result| {
            if !result {
                eprintln!("can't set port '{}' to sink '{}'", port_name, sink_name);
            }
        }))
    });

    match wait_op(&mut mainloop, op) {
        Err(e) => { eprintln!("{}", e); return; },
        Ok(_) => {}
    }
}
