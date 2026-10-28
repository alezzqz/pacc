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
        println!("pacc version 0.3.0");
        return;
    }

    let mut pulse_ctx = PaContext::new();
    check_res!(pulse_ctx.connect_context());

    let outputs = Arc::new(Mutex::new(Vec::new()));
    check_res!(pulse_ctx.get_pa_outputs_list(outputs.clone()));

    let def_sink_name = pulse_ctx.get_default_sink_name().unwrap_or("".to_string());

    let active_out_idx = outputs.lock().unwrap().iter().position(|x| x.is_active_port && x.sink_name == def_sink_name );

    let mut state = ListState::default();
    state.select(active_out_idx.or_else(|| (!outputs.lock().unwrap().is_empty()).then_some(0)));
    check_res!(ui::show_ui(&mut state, &mut outputs.lock().unwrap(), &mut pulse_ctx));

    if state.selected().is_none() {
        return;
    }

    let selected_out = &outputs.lock().unwrap()[state.selected().unwrap()];
    check_res!(pulse_ctx.set_pa_sink_and_port(&selected_out));
    if selected_out.pending_volume {
        check_res!(pulse_ctx.set_sink_volume(&selected_out.sink_name, &selected_out.volume));
    }

    println!("Output switched to '{}'", selected_out.to_list_line());
}
