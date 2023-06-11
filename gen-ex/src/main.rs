use bevy::prelude::*;
use bincode::Options;
use parking_lot::Mutex;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    mem::size_of,
    net::UdpSocket,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

pub mod frontend;

const MAX_MSG_SIZE: usize = 4096;

//pub trait Proto
//where
//    Self: Sized,
//{
//    fn serialize_into(&self, buf: &mut [u8]) -> Result<(), usize>;
//    fn serialized_size(&self);
//    fn try_deserialize(buf: &[u8]) -> Option<Self>;
//    fn deserialize(buf: &[u8]) -> Self {
//        Self::try_deserialize(buf).unwrap()
//    }
//}

#[derive(Resource)]
struct BackendService {
    thread: JoinHandle<()>,
    pub queue: Arc<Mutex<VecDeque<BackendEvent>>>,
}

// Autogenerated
#[repr(C)]
struct MessageWireHeader {
    _size: u64,
    // Bincode deserialize of string is a network byte order
    id: [u8],
}

type MessageId = str;
// Autogenerated
const InputMessageId: &'static MessageId = "Input";
#[derive(Deserialize, Debug)]
struct Input {
    key_code: KeyCode,
}

// Autogenerated
#[derive(Deserialize, Debug)]
#[serde(into = "u16")]
enum KeyCode {
    Spacebar = 0,
    W,
}

#[derive(Deserialize, Debug)]
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
        bevy_world.send_event(InputEvent(input));
    }
    pub fn gen_bevy_event(event: BackendEvent, world: &mut World) {
        match event {
            BackendEvent::InputEvent(input) => Self::gen_input_event(input, world),
        }
    }
    pub fn insert_bevy_events(app: &mut App) {
        app.add_event::<InputEvent>();
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
        if let Ok(header) = my_options.deserialize::<String>(buf) {
            let header_size = my_options.serialized_size(&header).unwrap() as usize;
            // Parse based on id...
            match header.as_str() {
                InputMessageId => Self::parse_event_type::<InputEvent>(&buf[header_size..]),
                // Unrecognized message type
                _ => unimplemented!(),
            }
            .and_then(|(size, e)| Some((size + header_size, e)))
        } else {
            None
        }
    }
}

impl BackendService {
    fn new(socket: String) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let thread_queue_handle = queue.clone();
        // Spawn worker_loop
        Self {
            thread: std::thread::spawn(move || Self::worker_loop(socket, thread_queue_handle)),
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

fn sys_backend_service(world: &mut World) {
    // Take from the backend service Queue
    world.resource_scope::<BackendService, ()>(|world, service| {
        let mut queue = service.queue.lock();
        // Deque all received events, push all events into Bevy
        for event in queue.drain(0..) {
            BackendService::gen_bevy_event(event, world);
        }
        ()
    });
}

fn main() {
    let mut app = App::new();
    app.insert_resource(bevy::app::ScheduleRunnerSettings::run_loop(
        Duration::from_secs_f64(1.0 / 60.0),
    ))
    .add_plugins(MinimalPlugins)
    .insert_resource(BackendService::new("127.0.0.1:11000".into()))
    .add_system(sys_backend_service)
    .add_system(sys_log_event);
    BackendService::insert_bevy_events(&mut app);
    app.run();
}

fn sys_log_event(mut ev: EventReader<InputEvent>) {
    for e in ev.iter() {
        println!("Event received!");
    }
}

#[test]
fn test_socket() {
    let sock = UdpSocket::bind("127.0.0.1:11002").unwrap();
    let sock2 = UdpSocket::bind("127.0.0.1:11003").unwrap();
    //sock2.connect("127.0.0.1:11000").unwrap();
    sock2.send_to(&[8u8], "127.0.0.1:11002").unwrap();
    let mut buf = [0u8; 64];
    let res = sock.recv(&mut buf).unwrap();
}

#[test]
fn test_main() {
    let handle = std::thread::spawn(|| main());
    // Conenct to socket, send message
    let sock = UdpSocket::bind("127.0.0.1:11001").unwrap();
    //sock.connect("127.0.0.1:11000").unwrap();

    let big_endian = bincode::DefaultOptions::new()
        .with_big_endian() // Network byte order
        .with_fixint_encoding() // Use a full 8 bytes to encode
        .allow_trailing_bytes();
    thread::sleep(Duration::from_secs_f32(1.0));
    let _ = sock.send_to(
        &big_endian.serialize(InputMessageId).unwrap(),
        "127.0.0.1:11000",
    );
    thread::sleep(Duration::from_secs_f32(1.0));
    sock.send_to(
        &big_endian.serialize(InputMessageId).unwrap(),
        "127.0.0.1:11000",
    )
    .unwrap();
}

#[test]
fn bincode_test() {
    let mut target = String::new();
    for i in 0..256 {
        target.push('a');
    }
    let my_options = bincode::DefaultOptions::new()
        .with_big_endian() // Network byte order
        .with_fixint_encoding() // Use a full 8 bytes to encode
        .allow_trailing_bytes();

    // Big endian, should place 256 at the 1<<8 bit
    let mut expected = vec![0, 0, 0, 0, 0, 0, 1, 0];
    expected.extend_from_slice(&['a' as u8; 256]);
    let encode = my_options.serialize(&target).unwrap();
    assert_eq!(encode, expected);

    // Little endian, should place 256 at the MAX+1>>8 bit
    let mut expected = vec![0, 1, 0, 0, 0, 0, 0, 0];
    expected.extend_from_slice(&['a' as u8; 256]);
    let my_options = bincode::DefaultOptions::new()
        .with_little_endian()
        .with_fixint_encoding()
        .allow_trailing_bytes();
    let encode = my_options.serialize(&target).unwrap();
    assert_eq!(encode, expected);
}
