use crate::{Command, CyKvError, KvEngine, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

pub struct Server<E: KvEngine> {
    engine: E,
    listener: TcpListener,
}

impl<E: KvEngine> Server<E> {
    pub fn new(engine: E, addr: SocketAddr) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self { engine, listener })
    }

    pub fn run(&self) -> Result<()> {
        for stream in self.listener.incoming() {
            let engine = self.engine.clone();
            thread::spawn(move || {
                if let Err(e) = serve(engine, stream.unwrap()) {
                    eprintln!("{:?}", e);
                }
            });
        }

        Ok(())
    }
}

fn serve<E: KvEngine>(engine: E, stream: TcpStream) -> Result<()> {
    let reader = serde_json::Deserializer::from_reader(&stream);

    for req in reader.into_iter::<Request>() {
        if let Err(e) = req {
            // Client close the connection
            if e.line() == 0 && e.column() == 0 {
                return Ok(());
            }
            let res = Response::Err("error when read request".to_owned());
            serde_json::to_writer(&stream, &res)?;
            return Err(CyKvError::SerdeJson(e));
        }

        let res = match req.unwrap() {
            Request::Get { Key: key } => match engine.get(key) {
                Ok(value) => Response::Ok(value),
                Err(e) => Response::Err("".to_owned()),
            },
            Request::Set {
                Key: key,
                Value: value,
            } => match engine.set(key, value) {
                Ok(_) => Response::Ok(None),
                Err(e) => Response::Err("".to_owned()),
            },
            Request::Remove { Key: key } => match engine.remove(key) {
                Ok(_) => Response::Ok(None),
                Err(e) => Response::Err("".to_owned()),
            },
        };

        serde_json::to_writer(&stream, &res)?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "type")]
pub enum Request {
    Get { Key: String },
    Set { Key: String, Value: String },
    Remove { Key: String },
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "type")]
pub enum Response {
    Ok(Option<String>),
    Err(String),
}
