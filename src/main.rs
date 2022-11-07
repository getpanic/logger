use std::thread;
use std::sync::mpsc::channel;

use device_query::{DeviceState, DeviceQuery};
use encoding::{all::GBK, Encoding, DecoderTrap};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextA};

static mut CURRENT: String = String::new();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = channel();

    let join_handle = thread::spawn(move || {
        while let Ok(msg) = receiver.recv() {
            println!("rec: {:?}", msg);
        }
    });

    let sender1 = sender.clone();
    let key_task = std::thread::spawn(move||{
        let device_state = DeviceState::new();
        let mut prev_keys = vec![];
        loop {
            let keys = device_state.get_keys();
            if keys != prev_keys && !keys.is_empty() {
                sender1.send(format!("{:?}", keys)).unwrap();
            }
            prev_keys = keys;
        }
    });
    let sender2 = sender.clone();
    let title_task = std::thread::spawn(move ||{
        loop{
            unsafe{
                let mut title: [u8; 96] = [0; 96];
                let window = GetForegroundWindow();
                GetWindowTextA(window, &mut title);
                let vec: Vec<u8> = title.into_iter().filter(|v| *v != 0).collect();
                let title_text = GBK.decode(&vec, DecoderTrap::Strict).unwrap();
                if title_text.ne(&CURRENT){
                    CURRENT = title_text;
                    sender2.send(CURRENT.clone()).unwrap();
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    });
    key_task.join();
    title_task.join();
    join_handle.join();
    Ok(())
}