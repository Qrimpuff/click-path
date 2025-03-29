use crossbeam_channel::unbounded;
use enigo::Button;
use enigo::Coordinate;
use enigo::Direction;
use enigo::Enigo;
use enigo::Mouse;
use enigo::Settings;
use serde::Deserialize;
use serde::Serialize;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use win_hotkeys::HotkeyManager;
use win_hotkeys::VKey;

#[derive(Debug, PartialEq)]
enum AppCommand {
    StartRecording,
    RegisterClick,
    StopRecording,
    PlayClicks(PlayClicks),
    Exit,
}

#[derive(Debug, PartialEq)]
enum PlayClicks {
    Once,
    Loop,
}

#[derive(Debug, Serialize, Deserialize)]
enum ClickEvent {
    Click(i32, i32),
    Wait(i32),
}

#[derive(Debug, Serialize, Deserialize)]
struct ClickPath {
    clicks: Vec<ClickEvent>,
}

impl ClickPath {
    fn new() -> Self {
        ClickPath { clicks: vec![] }
    }

    fn add_click(&mut self, x: i32, y: i32) {
        self.clicks.push(ClickEvent::Click(x, y));
    }

    fn add_wait(&mut self, duration_ms: i32) {
        self.clicks.push(ClickEvent::Wait(duration_ms));
    }
}

fn main() {
    const KEY_MODIFIERS: &[VKey] = &[VKey::LWin];
    const KEY_MODIFIERS_DISPLAY: &str = "Win";

    println!("ClickPath: auto clicker");
    println!("Press {KEY_MODIFIERS_DISPLAY} + 1 to start recording");
    println!("Press {KEY_MODIFIERS_DISPLAY} + 2 to add a click");
    println!("Press {KEY_MODIFIERS_DISPLAY} + 3 to stop recording");
    println!("Press {KEY_MODIFIERS_DISPLAY} + 4 to play the recorded clicks");
    println!("Press {KEY_MODIFIERS_DISPLAY} + 5 to play the recorded clicks in a loop");
    println!("Press {KEY_MODIFIERS_DISPLAY} + Esc to exit");

    // The HotkeyManager is generic over the return type of the callback functions.
    let mut hkm = HotkeyManager::new();

    hkm.register_hotkey(VKey::Vk1, KEY_MODIFIERS, || {
        println!("Pressed {KEY_MODIFIERS_DISPLAY} + 1");
        AppCommand::StartRecording
    })
    .unwrap();

    hkm.register_hotkey(VKey::Vk2, KEY_MODIFIERS, || {
        println!("Pressed {KEY_MODIFIERS_DISPLAY} + 2");
        AppCommand::RegisterClick
    })
    .unwrap();

    hkm.register_hotkey(VKey::Vk3, KEY_MODIFIERS, || {
        println!("Pressed {KEY_MODIFIERS_DISPLAY} + 3");
        AppCommand::StopRecording
    })
    .unwrap();

    hkm.register_hotkey(VKey::Vk4, KEY_MODIFIERS, || {
        println!("Pressed {KEY_MODIFIERS_DISPLAY} + 4");
        AppCommand::PlayClicks(PlayClicks::Once)
    })
    .unwrap();

    hkm.register_hotkey(VKey::Vk5, KEY_MODIFIERS, || {
        println!("Pressed {KEY_MODIFIERS_DISPLAY} + 5");
        AppCommand::PlayClicks(PlayClicks::Loop)
    })
    .unwrap();

    hkm.register_hotkey(VKey::Escape, KEY_MODIFIERS, || {
        println!("Pressed {KEY_MODIFIERS_DISPLAY} + Esc");
        AppCommand::Exit
    })
    .unwrap();

    // Register channel to receive events from the hkm event loop
    let (tx, rx) = unbounded();
    hkm.register_channel(tx);

    // Run HotkeyManager in background thread
    let handle = hkm.interrupt_handle();
    thread::spawn(move || {
        hkm.event_loop();
    });

    // App Logic
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let mut is_recording = false;
    let mut path = ClickPath::new();
    let mut wait_time = Instant::now();
    loop {
        let command = rx.recv().unwrap();

        match command {
            AppCommand::StartRecording => {
                println!("Starting recording...");
                wait_time = Instant::now();
                path = ClickPath::new();
                is_recording = true;
            }
            AppCommand::RegisterClick => {
                if !is_recording {
                    println!(
                        "Not recording. Press {KEY_MODIFIERS_DISPLAY} + 1 to start recording."
                    );
                    continue;
                }

                // add wait time
                let elapsed = wait_time.elapsed();
                if elapsed.as_millis() > 0 {
                    path.add_wait(elapsed.as_millis() as i32);
                    println!("Wait for: {} ms", elapsed.as_millis());
                }
                wait_time = Instant::now();

                // get mouse position
                let (x, y) = enigo.location().unwrap();
                println!("Click at: ({}, {})", x, y);
                path.add_click(x, y);
                enigo.button(Button::Left, Direction::Click).unwrap();
            }
            AppCommand::StopRecording => {
                println!("Stopping recording...");
                is_recording = false;

                // Serialize the path to a file
                let serialized_path = serde_json::to_string(&path).unwrap();
                let file_path = "click_path.json";
                std::fs::write(file_path, serialized_path).unwrap();
                println!("Click path saved to {}", file_path);
            }
            AppCommand::PlayClicks(play_mode) => {
                if is_recording {
                    println!("Cannot play while recording. Stop recording first.");
                    continue;
                }

                match play_mode {
                    PlayClicks::Once => println!("Playing recorded clicks once..."),
                    PlayClicks::Loop => println!("Playing recorded clicks in a loop..."),
                }

                let file_path = "click_path.json";
                let data = std::fs::read_to_string(file_path).unwrap();
                let path: ClickPath = serde_json::from_str(&data).unwrap();

                if path.clicks.is_empty() {
                    println!("No clicks recorded. Please record some clicks first.");
                    continue;
                }
                if play_mode == PlayClicks::Loop {
                    println!("Press {KEY_MODIFIERS_DISPLAY} + Esc to stop playing.");
                }

                'outer: loop {
                    for event in &path.clicks {
                        // Check for exit command
                        if let Ok(command) = rx.try_recv() {
                            if command == AppCommand::Exit {
                                println!("Exiting loop...");
                                break 'outer;
                            }
                        }

                        match event {
                            ClickEvent::Click(x, y) => {
                                enigo.move_mouse(*x, *y, Coordinate::Abs).unwrap();
                                println!("Click at: ({}, {})", x, y);
                                enigo.button(Button::Left, Direction::Click).unwrap();
                            }
                            ClickEvent::Wait(duration_ms) => {
                                thread::sleep(Duration::from_millis(*duration_ms as u64));
                            }
                        }
                    }
                    if play_mode == PlayClicks::Once {
                        println!("Finished playing recorded clicks.");
                        break;
                    }
                }
            }
            AppCommand::Exit => {
                println!("Exiting...");
                handle.interrupt();
                break;
            }
        }
    }
}
