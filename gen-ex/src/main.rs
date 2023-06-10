use bevy::prelude::{ResMut, Resource, World};
use bincode::Options;
use parking_lot::Mutex;
use serde::Deserialize;
use std::{collections::VecDeque, mem::size_of, net::UdpSocket, sync::Arc, thread::JoinHandle};

//struct Vec2;
//struct Id;
//struct Socket;
//struct Deque;

//struct FrontendService {
//    // Socket
//    socket: Socket,
//    // Command Queue
//    queue: Mutex<Deque>,
//}
//
//// User public interface
//impl FrontendService {
//    // Autogenerated function
//    pub fn move_node(&self, id: Id, x: u32, y: u32) {
//        // Note: Self is async, borrowed queue through mutex
//        // Queue up the command
//        // - Build the generated message struct
//        // - push_back command into queue
//    }
//
//    // TODO: What functions would I need to block on?
//}
//
//impl FrontendService {
//    fn tick(&mut self) {
//        // Dump the queued commands over the socket
//    }
//}

const MAX_MSG_SIZE: usize = 4096;

#[derive(Resource)]
struct BackendService {
    thread: JoinHandle<()>,
    pub queue: Arc<Mutex<VecDeque<BackendEvent>>>,
}

// Autogenerated
#[derive(Deserialize, Debug)]
struct MessageWireHeader {
    size: u32,
    id: String,
}

type MessageId = str;
// Autogenerated
const InputMessageId: &'static MessageId = "Input";
#[derive(Deserialize, Debug)]
struct Input;
#[derive(Deserialize, Debug)]
#[repr(transparent)]
struct InputEvent(Input);
enum BackendEvent {
    InputEvent(Input),
}
trait ToBackendEvent {
    fn to_backend_event(self) -> BackendEvent;
}
impl ToBackendEvent for InputEvent {
    fn to_backend_event(self) -> BackendEvent {
        BackendEvent::InputEvent(self.0)
    }
}

// Autogenerated, (consumers can subscribe to InputEvent)
impl BackendService {
    fn gen_input_event(input: Input, bevy_world: &mut World) {
        bevy_world.send_event(InputEvent(Input));
    }
    pub fn gen_bevy_event(event: BackendEvent, world: &mut World) {
        match event {
            BackendEvent::InputEvent(input) => Self::gen_input_event(input, world),
        }
    }

    pub fn parse_event_type<'a, T>(buf: &'a [u8]) -> Option<(usize, BackendEvent)>
    where
        T: ToBackendEvent + Deserialize<'a>,
    {
        let event_input_size = size_of::<T>();
        if buf.len() < event_input_size {
            return None;
        }
        let my_options = bincode::DefaultOptions::new()
            .with_big_endian() // Network byte order
            .with_fixint_encoding()
            .allow_trailing_bytes();
        let event: T = my_options.deserialize(buf).unwrap();
        return Some((event_input_size, ToBackendEvent::to_backend_event(event)));
    }

    pub fn parse_event(buf: &[u8]) -> Option<(usize, BackendEvent)> {
        let my_options = bincode::DefaultOptions::new()
            .with_big_endian() // Network byte order
            .with_fixint_encoding()
            .allow_trailing_bytes();
        // Events will be of format: MessageWireHeader + Message
        if let Ok(header) = my_options.deserialize::<MessageWireHeader>(buf) {
            // Parse based on id...
            match header.id.as_str() {
                InputMessageId => Self::parse_event_type::<InputEvent>(&buf[..]),
                // Unrecognized message type
                _ => unimplemented!(),
            }
        } else {
            None
        }
    }
}

impl BackendService {
    fn new() -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let thread_queue_handle = queue.clone();
        // Spawn worker_loop
        Self {
            thread: std::thread::spawn(move || {
                Self::worker_loop("127.0.0.1:11000".into(), thread_queue_handle)
            }),
            queue,
        }
    }
    // Runs in a separate thread
    fn worker_loop(sock_params: String, queue: Arc<Mutex<VecDeque<BackendEvent>>>) {
        // Note: Needs to maintain connection and handle shutdown.
        loop {
            // TODO: Handle some eventual failure?
            let _ = Self::worker_inner(&sock_params, queue.clone());
        }
    }
    fn worker_inner(
        sock_params: &String,
        queue: Arc<Mutex<VecDeque<BackendEvent>>>,
    ) -> std::io::Result<()> {
        let sock = UdpSocket::bind(sock_params)?;
        let mut buf = [0; MAX_MSG_SIZE];
        loop {
            let (amt, _src) = sock.recv_from(&mut buf)?;
            let mut rcv_buf = &buf[..amt];
            let mut queue = queue.lock();
            loop {
                // TODO/FIXME: Add support for storing partial messages into a temp buffer...
                let res: Option<(usize, BackendEvent)> = Self::parse_event(&rcv_buf);
                if let Some((amt, event)) = res {
                    rcv_buf = &rcv_buf[amt..];
                    queue.push_back(event);
                } else {
                    // Nothing remains to parse.
                    break;
                }
            }
        }
    }
}

fn sys_backend_service(world: &mut World, service: ResMut<BackendService>) {
    // Take from the backend service Queue
    //let service = world.resource_mut::<BackendService>();
    let mut queue = service.queue.lock();
    // Deque all received events, push all events into Bevy
    for event in queue.drain(0..) {
        BackendService::gen_bevy_event(event, world);
    }
}

fn main() {}
