// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::time::Duration;
use slint::{Timer, TimerMode};
use slint_workshop_common::weather::{CityData, DummyWeatherController, OpenWeatherController, WeatherControllerPointer, WeatherControllerSharedPointer, WeatherData};

slint::include_modules!();


/// Our App struct that holds the UI
struct App {
    ui: MainWindow,
    timer: Timer,
    weather_controller: WeatherControllerSharedPointer
}


impl App {
    /// Create a new App struct.
    /// 
    /// The App struct initializes the UI and the weather controller.
    fn new() -> anyhow::Result<Self> {        
        // Make a new AppWindow
        let ui = MainWindow::new()?;

        let weather_controller = if let Some(api_key) = option_env!("OPEN_WEATHER_API") {
            Arc::new(Mutex::new(Box::new(OpenWeatherController::new(CityData {lat: 43.77, lon: 11.25, city_name: "Florence".into()}, api_key.into())) as WeatherControllerPointer))
        } else { Arc::new(Mutex::new(Box::new(DummyWeatherController::new()?) as WeatherControllerPointer))};

        ui.global::<ViewModel>().set_city_name(slint::SharedString::from(weather_controller.lock().unwrap().city_data().unwrap().city_name));

        // Return the App struct
        Ok(Self {
            ui,
            timer: Timer::default(),
            weather_controller,
        })
    }

    /// Run the App
    fn run(&mut self) -> anyhow::Result<()> {
        let ui_handle = self.ui.as_weak();

        let weather_controller = self.weather_controller.clone();

        self.timer.start(TimerMode::Repeated, Duration::from_secs(5), move || {
            let ui = ui_handle.unwrap();
            let model = ViewModel::get(&ui);

            let current_data = weather_controller.lock().unwrap().current_data().unwrap();

            model.set_current(WeatherRecord::from(current_data));
        });

        // Run the UI (and map an error to an anyhow::Error).
        self.ui.run().map_err(|e| e.into())
    }
}

impl From<WeatherData> for WeatherRecord {
    fn from(value: WeatherData) -> Self {
        Self {
            humidity_percent: value.current_humidity as f32,
            temperature_celsius: value.current_temperature as f32,
            timestamp: slint::SharedString::from(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}


/// A minimal main function that initializes the App and runs it.
fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let mut app = App::new()?;

    app.run()
}

