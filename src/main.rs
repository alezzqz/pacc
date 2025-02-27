pub mod ui;
pub mod source;
pub mod pulse;

use std::{env, sync::{Arc, Mutex}};

use libpulse_binding::context::Context;
use libpulse_binding::mainloop::standard::Mainloop;
use pulse::{connect_context, get_pa_outputs_list, set_pa_sink_and_port};
use tui::widgets::ListState;

fn main() {
    if env::args().find(|x| x == "--version") != None {
        println!("paccu version 1.1");
        return;
    }

    let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
    let mut context = Context::new(&mainloop, "Audio Output Switcher").expect("Failed to create context");

    if let Err(e) = connect_context(&mut mainloop, &mut context) {
        eprintln!("{}", e);
        return;
    }

    let sources = Arc::new(Mutex::new(Vec::new()));

    let introspect = context.introspect();

    if let Err(e) = get_pa_outputs_list(&mut mainloop, &introspect, sources.clone()) {
        eprintln!("{}", e);
        return;
    }

    let mut state = ListState::default();
    state.select(Some(0));

    if let Err(e) = ui::show_ui(&mut state, &sources.lock().unwrap()) {
        eprintln!("{e}");
        return;
    };

    if let None = state.selected() {
        return;
    }

    let choosen_src = &sources.lock().unwrap()[state.selected().unwrap()];

    if let Err(e) = set_pa_sink_and_port(&mut mainloop, &mut context, &choosen_src) {
        eprintln!("{}", e);
        return;
    }

    println!("Output switched to '{}'", choosen_src.to_list_line());
}