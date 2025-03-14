pub mod ui;
pub mod source;
pub mod pulse;

use std::{env, sync::{Arc, Mutex}};

use pulse::PaContext;
use tui::widgets::ListState;

macro_rules! check_res {
    ($x:expr) => {
        if let Err(e) = $x {
            eprintln!("{e}");
            return;
        }
    };
}

fn main() {
    if env::args().find(|x| x == "--version" || x == "-v") != None {
        println!("pacc version 0.2.3");
        return;
    }

    let mut pulse_ctx = PaContext::new();
    check_res!(pulse_ctx.connect_context());

    let outputs = Arc::new(Mutex::new(Vec::new()));
    check_res!(pulse_ctx.get_pa_outputs_list(outputs.clone()));

    let active_out_idx = outputs.lock().unwrap().iter().position(|x| x.active );
    let mut state = ListState::default();
    state.select(active_out_idx);
    check_res!(ui::show_ui(&mut state, &outputs.lock().unwrap()));

    if let None = state.selected() {
        return;
    }

    let selected_out = &outputs.lock().unwrap()[state.selected().unwrap()];
    check_res!(pulse_ctx.set_pa_sink_and_port(&selected_out));

    println!("Output switched to '{}'", selected_out.to_list_line());
}