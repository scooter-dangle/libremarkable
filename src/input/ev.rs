use epoll;
use evdev;
use input;
use std;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

// TODO: Setup inotify for tracking whether /dev/input/* are present

pub struct EvDevContext {
    device: input::InputDevice,
    pub state: input::InputDeviceState,
    pub tx: mpsc::Sender<input::InputEvent>,
    exit_requested: Arc<AtomicBool>,
    exited: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
}

use evdev::raw::input_event;
pub trait Device: Default + Sync + Send {
    const INPUT_PATH: &'static str;
    const LABEL: &'static str;
    type Event: Send;
    fn decode(&self, event: input_event) -> Option<Self::Event>;
}

// TODO: Future version
pub struct EvDevContext0<D, EV>
where
    D: Device,
    EV: 'static + From<<D as Device>::Event> + Send,
{
    pub state: Arc<D>,
    pub tx: mpsc::Sender<EV>,
    exit_requested: Arc<AtomicBool>,
    exited: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
}

impl EvDevContext {
    pub fn started(&self) -> bool {
        self.started.load(Ordering::Relaxed)
    }

    pub fn exited(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }

    /// After exit is requested, there will be one more event read from the device before
    /// it is closed.
    pub fn exit_requested(&self) -> bool {
        self.exit_requested.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        self.exit_requested.store(true, Ordering::Relaxed);
    }

    pub fn new(device: input::InputDevice, tx: mpsc::Sender<input::InputEvent>) -> EvDevContext {
        EvDevContext {
            device,
            tx,
            state: input::InputDeviceState::new(device),
            started: Arc::new(AtomicBool::new(false)),
            exit_requested: Arc::new(AtomicBool::new(false)),
            exited: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Non-blocking function that will open the provided path and wait for more data with epoll
    pub fn start(&mut self) -> bool {
        self.started.store(true, Ordering::Relaxed);
        self.exited.store(false, Ordering::Relaxed);
        self.exit_requested.store(false, Ordering::Relaxed);

        let path = match self.device {
            input::InputDevice::Wacom => "/dev/input/event0",
            input::InputDevice::Multitouch => "/dev/input/event1",
            input::InputDevice::GPIO => "/dev/input/event2",
            input::InputDevice::Keyboard => "/dev/input/event3",
            input::InputDevice::Unknown => unreachable!(),
        };

        match evdev::Device::open(&path) {
            Err(e) => {
                error!("Error while reading events from epoll fd: {0}", e);
                self.exited.store(true, Ordering::Relaxed);
                self.state = input::InputDeviceState::new(self.device);
                return false;
            }
            Ok(mut dev) => {
                let mut buf = vec![epoll::Event {
                    events: (epoll::Events::EPOLLET
                        | epoll::Events::EPOLLIN
                        | epoll::Events::EPOLLPRI)
                        .bits(),
                    data: 0,
                }];
                let epfd = epoll::create(false).unwrap();
                epoll::ctl(epfd, epoll::ControlOptions::EPOLL_CTL_ADD, dev.fd(), buf[0]).unwrap();

                // init callback
                info!("Init complete for {0}", String::from(path));

                let exit_req = Arc::clone(&self.exit_requested);
                let exited = Arc::clone(&self.exited);
                let device_type = self.device;
                let state = self.state.clone();
                let tx = self.tx.clone();
                let _ = std::thread::Builder::new()
                    .name(format!("EvDevContext:{:#?}", device_type))
                    .spawn(move || {
                        while !exit_req.load(Ordering::Relaxed) {
                            // -1 indefinite wait but it is okay because our EPOLL FD
                            // is watching on ALL input devices at once.
                            let res = epoll::wait(epfd, -1, &mut buf[0..1]).unwrap();
                            if res != 1 {
                                warn!("epoll_wait returned {0}", res);
                            }

                            for ev in match dev.events_no_sync() {
                                Ok(events) => events,
                                Err(err) => {
                                    error!(
                                        "Error in EvDevContext:{:#?} after epoll::wait: {:#?}",
                                        device_type, err
                                    );
                                    break;
                                }
                            } {
                                // event callback
                                let decoded_event = match device_type {
                                    input::InputDevice::Multitouch => input::multitouch::decode,
                                    input::InputDevice::Wacom => input::wacom::decode,
                                    input::InputDevice::GPIO => input::gpio::decode,
                                    input::InputDevice::Keyboard => input::keyboard::decode,
                                    input::InputDevice::Unknown => unreachable!(),
                                }(&ev, &state);
                                if let Some(event) = decoded_event {
                                    match tx.send(event) {
                                        Ok(_) => {}
                                        Err(e) => error!(
                                            "Failed to write InputEvent into the channel: {0}",
                                            e
                                        ),
                                    };
                                }
                            }
                        }
                        exited.store(true, Ordering::Relaxed);
                    })
                    .unwrap();
                true
            }
        }
    }
}

impl<D, EV> EvDevContext0<D, EV>
where
    D: 'static + Device + Sync + Send,
    EV: 'static + From<<D as Device>::Event> + Send,
{
    pub fn started(&self) -> bool {
        self.started.load(Ordering::Relaxed)
    }

    pub fn exited(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }

    /// After exit is requested, there will be one more event read from the device before
    /// it is closed.
    pub fn exit_requested(&self) -> bool {
        self.exit_requested.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        self.exit_requested.store(true, Ordering::Relaxed);
    }

    pub fn new(tx: mpsc::Sender<EV>) -> Self {
        Self {
            tx,
            state: Arc::new(D::default()),
            started: Arc::new(AtomicBool::new(false)),
            exit_requested: Arc::new(AtomicBool::new(false)),
            exited: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Non-blocking function that will open the provided path and wait for more data with epoll
    pub fn start(&mut self) -> bool {
        self.started.store(true, Ordering::Relaxed);
        self.exited.store(false, Ordering::Relaxed);
        self.exit_requested.store(false, Ordering::Relaxed);

        match evdev::Device::open(&D::INPUT_PATH) {
            Err(e) => {
                error!("Error while reading events from epoll fd: {0}", e);
                self.exited.store(true, Ordering::Relaxed);
                // Clear state before returning
                self.state = Arc::new(D::default());
                return false;
            }
            Ok(mut dev) => {
                let mut buf = vec![epoll::Event {
                    events: (epoll::Events::EPOLLET
                        | epoll::Events::EPOLLIN
                        | epoll::Events::EPOLLPRI)
                        .bits(),
                    data: 0,
                }];
                let epfd = epoll::create(false).unwrap();
                epoll::ctl(epfd, epoll::ControlOptions::EPOLL_CTL_ADD, dev.fd(), buf[0]).unwrap();

                // init callback
                info!("Init complete for {0}", &D::INPUT_PATH);

                let exit_req = Arc::clone(&self.exit_requested);
                let exited = Arc::clone(&self.exited);
                let state: Arc<D> = Arc::clone(&self.state);
                let tx = self.tx.clone();
                let _ = std::thread::Builder::new()
                    .name(format!("EvDevContext:{:#?}", &D::LABEL))
                    .spawn(move || {
                        while !exit_req.load(Ordering::Relaxed) {
                            // -1 indefinite wait but it is okay because our EPOLL FD
                            // is watching on ALL input devices at once.
                            let res = epoll::wait(epfd, -1, &mut buf[0..1]).unwrap();
                            if res != 1 {
                                warn!("epoll_wait returned {0}", res);
                            }

                            let events = match dev.events_no_sync() {
                                Ok(events) => events,
                                Err(err) => {
                                    error!(
                                        "Error in EvDevContext:{:#?} after epoll::wait: {:#?}",
                                        &D::LABEL,
                                        err
                                    );
                                    break;
                                }
                            };

                            for ev in events {
                                // event callback
                                if let Some(event) = state.decode(ev).map(EV::from) {
                                    if let Err(e) = tx.send(event) {
                                        error!(
                                            "Failed to write InputEvent into the channel: {0}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        exited.store(true, Ordering::Relaxed);
                    })
                    .unwrap();
                true
            }
        }
    }
}
