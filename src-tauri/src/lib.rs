// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod commands;
pub mod events;

use commands::window::force_close_window;
use std::sync::Mutex;
use tauri::Manager;

use crate::commands::sample::greet;
use crate::commands::simulation::{
    generate_map, load_map, reset_simulation, save_map, set_motor_command, set_simulation_running,
    set_visualization_speed, simulation_snapshot, step_simulation, AppSimulation,
};

fn create_main_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let title = app.config().product_name.as_deref().unwrap_or("Tauri App");
    let mut builder =
        tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App(Default::default()))
            .title(title)
            .maximized(true);

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .decorations(true)
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true)
            .traffic_light_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: 23,
                y: 35,
            }));
    }

    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.decorations(false);
    }

    builder.build()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            create_main_window(app)?;
            let simulation = AppSimulation::create_default().map_err(std::io::Error::other)?;
            app.manage(Mutex::new(simulation));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Sample commands
            greet,
            // Window commands
            force_close_window,
            // Simulation commands
            simulation_snapshot,
            generate_map,
            load_map,
            save_map,
            reset_simulation,
            set_simulation_running,
            step_simulation,
            set_visualization_speed,
            set_motor_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
