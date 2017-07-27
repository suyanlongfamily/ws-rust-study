extern crate ws;
extern crate url;
extern crate env_logger;
extern crate threadpool;
extern crate num_cpus;
extern crate parking_lot;
extern crate spmc;
extern crate uuid;


use std::collections::HashMap;
use ws::WebSocket;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

//Handler

use ws::{connect, listen, Factory, CloseCode, Sender, Handler, Message, Result};
use parking_lot::Mutex;
use std::sync::Arc;
use threadpool::ThreadPool;


struct Client {
    connection: Sender,
    contian: Arc<Mutex<HashMap<String, String>>>,
    //every client call thread pool
    thread_pool: Arc<Mutex<ThreadPool>>,
}

//Client handler

impl Handler for Client {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Server got message '{}'.  post thread_pool deal task ", msg);
        let id = self.connection.token().0;
        let id_clone = id.clone();
        let id_coone = id_clone.clone();
        let sendr = self.connection.clone();
        self.contian
            .lock()
            .insert(id.to_string(), msg.into_text().unwrap());
        self.thread_pool
            .lock()
            .execute(move || {
                let mut count = 1000;
                while count > 0 {
                    count = count - 1;
                    println!("----loop-{:?}---{:?}-", id_clone, count);
                }
                sendr.send("已经处理完了");
            });
        println!("----on_message-----token = {:?}-------", id_coone);
        Ok(())
    }
    
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("WebSocket closing for ({:?}) {}", code, reason);
        println!("Shutting down server after first connection closes.");
        let id = self.connection.token().0;
        println!("----on_close-----token = {:?}-------", id);
        self.contian.lock().remove(&id.to_string());
    }
}


struct MyFactory {
    sender_contian: Arc<Mutex<HashMap<String, String>>>,
    //every client call thread pool
    thread_pool: Arc<Mutex<ThreadPool>>,
}


impl Factory for MyFactory {
    type Handler = Client;
    
    fn connection_made(&mut self, ws: Sender) -> Client {
        Client {
            connection: ws,
            contian: self.sender_contian.clone(),
            thread_pool: self.thread_pool.clone(),
        }
    }
}


fn main() {
    // Setup logging
    env_logger::init().unwrap();
    println!("--------ws");
    let factory = MyFactory {
        sender_contian: Arc::new(Mutex::new(HashMap::new())),
        thread_pool: Arc::new(Mutex::new(ThreadPool::new(10))),
        
    };
    let ws_server = WebSocket::new(factory).unwrap();
    ws_server.listen("127.0.0.1:1337");
    
    
    println!("All done.")
}

