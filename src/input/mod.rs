/// Contains the epoll code to read from the device when the worker thread is woken up by the
/// kernel upon new data to consume
pub mod ev;

/// Contains the code to decode Wacom events
pub mod wacom;

/// Contains the code to decode physical button events
pub mod gpio;

/// Contains the code to decode multitouch events
pub mod multitouch;

pub mod keyboard;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum InputDevice {
    Wacom,
    Multitouch,
    GPIO,
    Keyboard,
    Unknown,
}

use std;
use std::sync::Arc;

pub enum InputDeviceState {
    WacomState(Arc<wacom::WacomState>),
    MultitouchState(Arc<multitouch::MultitouchState>),
    GPIOState(Arc<gpio::GPIOState>),
    KeyboardState(Arc<keyboard::KeyboardState>),
}

impl Clone for InputDeviceState {
    fn clone(&self) -> InputDeviceState {
        match self {
            InputDeviceState::WacomState(ref state) => {
                InputDeviceState::WacomState(Arc::clone(state))
            }
            InputDeviceState::MultitouchState(ref state) => {
                InputDeviceState::MultitouchState(Arc::clone(state))
            }
            InputDeviceState::GPIOState(ref state) => {
                InputDeviceState::GPIOState(Arc::clone(state))
            }
            InputDeviceState::KeyboardState(ref state) => {
                InputDeviceState::KeyboardState(Arc::clone(state))
            }
        }
    }
}

impl InputDeviceState {
    pub fn new(dev: InputDevice) -> InputDeviceState {
        match dev {
            InputDevice::GPIO => InputDeviceState::GPIOState(Arc::new(gpio::GPIOState::default())),
            InputDevice::Wacom => {
                InputDeviceState::WacomState(Arc::new(wacom::WacomState::default()))
            }
            InputDevice::Multitouch => {
                InputDeviceState::MultitouchState(Arc::new(multitouch::MultitouchState::default()))
            }
            InputDevice::Keyboard => {
                InputDeviceState::KeyboardState(Arc::new(keyboard::KeyboardState::default()))
            },
            InputDevice::Unknown => unreachable!(),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum InputEvent {
    WacomEvent { event: wacom::WacomEvent },
    MultitouchEvent { event: multitouch::MultitouchEvent },
    GPIO { event: gpio::GPIOEvent },
    Keyboard { event: keyboard::KeyboardEvent },
    Unknown {},
}

impl Default for InputEvent {
    fn default() -> InputEvent {
        InputEvent::Unknown {}
    }
}
