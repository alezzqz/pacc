pub mod ui;
pub mod source;
pub mod pulse;

use std::{env, sync::{Arc, Mutex}};

use pulse::PaContext;
use tui::widgets::ListState;

fn main() {
    if env::args().find(|x| x == "--version") != None {
        println!("paccu version 0.2.1");
        return;
    }

    let mut pulse_ctx = PaContext::new();
    if let Err(e) = pulse_ctx.connect_context() {
        eprintln!("{e}");
        return;
    }

    let outputs = Arc::new(Mutex::new(Vec::new()));
    if let Err(e) = pulse_ctx.get_pa_outputs_list(outputs.clone()) {
        eprintln!("{e}");
        return;
    }

    let mut state = ListState::default();
    state.select(Some(0));
    if let Err(e) = ui::show_ui(&mut state, &outputs.lock().unwrap()) {
        eprintln!("{e}");
        return;
    };

    if let None = state.selected() {
        return;
    }

    let selected_out = &outputs.lock().unwrap()[state.selected().unwrap()];
    if let Err(e) = pulse_ctx.set_pa_sink_and_port(&selected_out) {
        eprintln!("{e}");
        return;
    }

    println!("Output switched to '{}'", selected_out.to_list_line());
}