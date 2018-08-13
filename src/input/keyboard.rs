use evdev::raw::input_event;
use input::ev::Device;
use input::{InputDeviceState, InputEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct KeyboardState {
    shift: AtomicBool,
    ctrl: AtomicBool,
    alt: AtomicBool,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum KeyboardEvent {
    Char(char),
    Combo {
        shift: bool,
        ctrl: bool,
        alt: bool,
        // TODO: Should be limited to actual keys avail on US or ISO keyboards
        // _or_ ASCII.
        key: u8,
    },
}

fn decode_(
    &input_event {
        time,
        _type,
        code,
        value,
    }: &input_event,
    // TODO: Use this arg
    keyboard_state: &KeyboardState,
) -> Option<KeyboardEvent> {
    match _type {
        // ??? things ???
        0 => None,
        // ??? things ???
        4 => None,

        1 => keyboard_state.char_map(
            code,
            match value {
                0 => false,
                1 => true,
                _ => {
                    error!("Unexpected value from keyboard input: {}", value);
                    error!(
                        "Unknown keyboard event: `input_event {{ _type: {}, code: {}, value: {} }}",
                        _type, code, value
                    );
                    return None;
                }
            },
        ),

        // Unknown
        other => {
            error!(
                "Unknown keyboard event: `input_event {{ _type: {}, code: {}, value: {} }}",
                _type, code, value
            );
            None
        }
    }
}

pub fn decode(
    event: &input_event,
    // TODO: Use this arg
    outer_state: &InputDeviceState,
) -> Option<InputEvent> {
    decode_(
        event,
        &*match outer_state {
            &InputDeviceState::KeyboardState(ref keyboard_state) => keyboard_state,
            _ => unreachable!(),
        },
    ).map(|event| InputEvent::Keyboard { event })
}

impl KeyboardState {
    fn char_map(&self, code: u16, pressed: bool) -> Option<KeyboardEvent> {
        if !pressed {
            // TODO Check if code maps to other modifier characters
            match code {
                // Left Shift
                | 42
                // Right Shift
                | 54 => &self.shift,

                _ => return None,
            }.store(false, Ordering::Relaxed);

            return None;
        }

        // TODO Check if self.ctrl is true
        // TODO Check if self.alt is true
        let shift = self.shift.load(Ordering::Relaxed);

        // TODO Pull in `Key` from evdev
        Some(KeyboardEvent::Char(match (code, shift) {
            (30, true)  => 'A',
            (30, false) => 'a',

            (48, true)  => 'B',
            (48, false) => 'b',

            (46, true)  => 'C',
            (46, false) => 'c',

            (32, true)  => 'D',
            (32, false) => 'd',

            (18, true)  => 'E',
            (18, false) => 'e',

            (33, true)  => 'F',
            (33, false) => 'f',

            (34, true)  => 'G',
            (34, false) => 'g',

            (35, true)  => 'H',
            (35, false) => 'h',

            (23, true)  => 'I',
            (23, false) => 'i',

            (36, true)  => 'J',
            (36, false) => 'j',

            (37, true)  => 'K',
            (37, false) => 'k',

            (38, true)  => 'L',
            (38, false) => 'l',

            (50, true)  => 'M',
            (50, false) => 'm',

            (49, true)  => 'N',
            (49, false) => 'n',

            (24, true)  => 'O',
            (24, false) => 'o',

            (25, true)  => 'P',
            (25, false) => 'p',

            (16, true)  => 'Q',
            (16, false) => 'q',

            (19, true)  => 'R',
            (19, false) => 'r',

            (31, true)  => 'S',
            (31, false) => 's',

            (20, true)  => 'T',
            (20, false) => 't',

            (22, true)  => 'U',
            (22, false) => 'u',

            (47, true)  => 'V',
            (47, false) => 'v',

            (17, true)  => 'W',
            (17, false) => 'w',

            (45, true)  => 'X',
            (45, false) => 'x',

            (21, true)  => 'Y',
            (21, false) => 'y',

            (44, true)  => 'Z',
            (44, false) => 'z',

            (2, false) => '1',
            (2, true)  => '!',

            (3, false) => '2',
            (3, true)  => '@',

            (4, false) => '3',
            (4, true)  => '#',

            (5, false) => '4',
            (5, true)  => '$',

            (6, false) => '5',
            (6, true)  => '%',

            (7, false) => '6',
            (7, true)  => '^',

            (8, false) => '7',
            (8, true)  => '&',

            (9, false) => '8',
            (9, true)  => '*',

            (10, false) => '9',
            (10, true)  => '(',

            (11, false) => '0',
            (11, true)  => ')',

            (41, false) => '`',
            (41, true)  => '~',

            (12, false) => '-',
            (12, true)  => '_',

            (13, false) => '=',
            (13, true)  => '+',

            (40, false) => '\'',
            (40, true)  => '"',

            // Left tab
            (15, false) => '\t',
            // TODO Ctrl char rather than pure tab
            (15, true)  => '\t',

            // Right tab
            (43, false) => '\t',
            // TODO Ctrl char rather than pure tab
            (43, true)  => '\t',

            // TODO Ctrl char rather than pure escape
            (1,  false) => '\x1b',
            // TODO Ctrl char rather than pure escape
            (1,  true)  => '\x1b',

            // Left Shift
            | (42, _)
            // Right Shift
            | (54, _) => {
                self.shift.store(true, Ordering::Relaxed);
                return None;
            }

            _ => {
                error!(
                    "Uncategorized keyboard code: {} (with{} shift pressed)",
                    code,
                    if shift { "" } else { "out" }
                );
                return None;
            }
        }))
    }
}

impl From<KeyboardEvent> for InputEvent {
    fn from(event: KeyboardEvent) -> Self {
        InputEvent::Keyboard { event }
    }
}

impl Device for KeyboardState {
    const INPUT_PATH: &'static str = "/dev/input/event3";
    const LABEL: &'static str = "KeyboardState";
    type Event = KeyboardEvent;

    fn decode(&self, event: input_event) -> Option<Self::Event> {
        decode_(&event, self)
    }
}

#[test]
fn abc() {
    let (tx, rx) = ::std::sync::mpsc::channel();
    let evdevctx: ::input::ev::EvDevContext0<KeyboardState, InputEvent> =
        ::input::ev::EvDevContext0::new(tx);
    assert_eq!(evdevctx.started(), false);
}
