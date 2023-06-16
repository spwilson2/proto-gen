use std::{
    net::UdpSocket,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread::JoinHandle,
};

use hello::GameFrontendMessage;

pub mod hello;

fn main() {
    // TODO FrontendClient
    // TODO BackendServer
    todo!()
}

struct FrontendClient {
    handle: JoinHandle<()>,
    chan: FrontendClientHandle,
}
struct FrontendClientHandle(SyncSender<hello::GameFrontendMessage>);
struct FrontendClientThread {}
impl FrontendClient {
    fn new(bind: String) -> Self {
        let (client_chan, thread_chan) = sync_channel(1000);
        let handle = std::thread::spawn(|| {
            (FrontendClientThread::run(bind, thread_chan));
            ()
        });
        Self {
            handle,
            chan: FrontendClientHandle(client_chan),
        }
    }
}
impl FrontendClientThread {
    fn run(bind: String, mut queue: Receiver<GameFrontendMessage>) {
        loop {
            let _ = Self::run_loop(&bind, &mut queue);
        }
    }

    fn run_loop(bind: &String, queue: &mut Receiver<GameFrontendMessage>) -> std::io::Result<()> {
        let srv = UdpSocket::bind(bind).unwrap();
        let mut buf = vec![0; 4096];
        loop {
            let res = queue.recv().unwrap();
            // TODO: Needed impl.
            res.serialize_into(buf);
        }
    }
}

struct BackendServer {
    handle: Option<JoinHandle<()>>,
    bind: String,
}
struct BackendServerThread {}
impl BackendServer {
    fn new(bind: String) -> Self {
        Self { handle: None, bind }
    }

    fn start(&mut self) {
        let bind = self.bind.clone();
        self.handle = Some(std::thread::spawn(|| (BackendServerThread::run(&bind))));
    }
}

impl BackendServerThread {
    fn run(bind: &String) {
        loop {
            let _ = Self::run_loop(bind);
        }
    }

    fn run_loop(bind: &String) -> std::io::Result<()> {
        let srv = UdpSocket::bind(bind).unwrap();
        let mut buf = vec![0; 4096];
        loop {
            let cnt = srv.recv(&mut buf)?;
            let msg = hello::GameBackendMessage::try_deserialize_msg(&buf[..cnt])
                .expect("Failed to parse message");
            // TODO receive updates, print them. Then trigger a move commnd send
            match msg {
                hello::GameBackendMessage::NotifyInputEventArg(input) => {
                    println!("Input Keycode: {:?}", input.keycode)
                } //,
                hello::GameBackendMessage::NotifyInputEventRet(_) => todo!(),
            }
        }
    }
}
