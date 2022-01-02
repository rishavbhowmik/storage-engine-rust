use std::sync::{
    mpsc::{channel, Receiver, Sender, sync_channel, SyncSender},
};

use se1::{IORequest, Engine};
use se1::storage::Storage;
type IORequestChan = (SyncSender<IORequest>, Receiver<IORequest>);

fn main() {
    let (chan_sender, chan_reciver): IORequestChan = sync_channel(100);
    let chan_sender_clone = chan_sender.clone();
    let thread_joiner = std::thread::spawn(move || {
        // send something in every 1s
        loop {
            chan_sender_clone.send((None, None, None)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
    let mut engine = Engine::new(Storage::new(String::from("tmp/temp.hex"), 16).unwrap());
    loop {
        let request = chan_reciver.recv_timeout(std::time::Duration::from_millis(900));
        match request {
            Ok(request) => {
                println!("{:?}", request);
            }
            Err(_) => {
                println!("Continue");
            }
        }
        
        // engine.append_request(request);
    }
    // thread_joiner.join();
}