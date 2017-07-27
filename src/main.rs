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
use std::sync::mpsc;

//Handler

use ws::{connect, listen, Factory, CloseCode, Sender, Handler, Message, Result};
use parking_lot::Mutex;
use std::sync::Arc;
use threadpool::ThreadPool;


struct Client {
    connection: Sender,
    contian: Arc<Mutex<HashMap<String, (String, Sender)>>>,
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
            .insert(id.to_string(), (msg.into_text().unwrap(), sendr));
        self.thread_pool
            .lock()
            .execute(move || {
                let thread_id = thread::current().id();
                println!("----mq thread_id = {:?}--", thread_id);
                let mut count = 100;
                while count > 0 {
                    count = count - 1;
                    if count == 1 {
                        println!("--处理请求的业务逻辑--loop-{:?}---{:?}-", id_clone, count);
                    }
                }
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
    sender_contian: Arc<Mutex<HashMap<String, (String, Sender)>>>,
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

struct MqHandler {
    contian: Arc<Mutex<HashMap<String, (String, Sender)>>>,
    //every client call thread pool
    thread_pool: Arc<Mutex<ThreadPool>>,
}

use std::ops::Deref;

fn main() {
    // Setup logging
    env_logger::init().unwrap();

    let cpu_num = num_cpus::get();
    let sender_contian = Arc::new(Mutex::new(HashMap::new()));
    let thread_pool = Arc::new(Mutex::new(ThreadPool::new(2 * cpu_num)));

    let mq_handler = MqHandler {
        contian: sender_contian.clone(),
        thread_pool: thread_pool.clone(),
    };
    println!("--------ws");
    let factory = MyFactory {
        sender_contian: sender_contian.clone(),
        thread_pool: thread_pool.clone(),
    };

    let (tx, rx) = mpsc::channel::<String>();
    let mq_sumlare = thread::spawn(move || {
        //模拟mq,
        while true {
            //接收到数据以后发给线程池处理任务.两个线程池分开吧,有锁,
            //但是容器不能分开!
            thread::sleep(Duration::new(0, 10000));
            let contain = mq_handler.contian.clone();
            mq_handler.thread_pool.lock().execute(move || {
                //用于处理应答的问题.

                for (k, v) in contain.lock().deref() {
                    println!("--处理应答的业务逻辑--key = {:?}----", k);
                    v.1.send("已经处理完了,这是应答");
                    let thread_id = thread::current().id();
                    println!("----mq thread_id = {:?}--", thread_id);
                }
            });
            //            tx.send("你好".to_string());
        }
    });


    let ws_server = WebSocket::new(factory).unwrap();
    ws_server.listen("127.0.0.1:1337");
    mq_sumlare.join();
    println!("All done.")
}

