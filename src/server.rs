use crate::{Command, KvEngine, Result, CyKvError};
use serde::{Deserialize, Serialize,Serializer,Deserializer};
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
                if let Err(e) = serve(engine,stream.unwrap()) {
                    eprintln!("{:?}",e);
                }
            });
        }

        Ok(())
    }
}


fn serve<E: KvEngine>(engine: E,stream: TcpStream) -> Result<()> {
    let reader = serde_json::Deserializer::from_reader(&stream);

    for req in reader.into_iter::<Request>() {
        if let Err(e) = req {
            let res = Response::err("error when read request".to_owned());
            serde_json::to_writer(&stream,&res)?;
            return Err(CyKvError::SerdeJson(e));
        }

        let res = match req.unwrap() {
            Request::get{key} => {
                match engine.get(key) {
                    Ok(value) => Response::ok(value),
                    Err(e) => Response::err("".to_owned())
                }
            }
            Request::set {key,value} => {
                match engine.set(key,value) {
                    Ok(_) => Response::ok(None),
                    Err(e) => Response::err("".to_owned()),
                }
            }
            Request::remove {key} => {
                match engine.remove(key) {
                    Ok(_) => Response::ok(None),
                    Err(e) => Response::err("".to_owned()),
                }
            }
        };

        serde_json::to_writer(&stream,&res)?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
#[serde(tag = "type")]
pub enum Request {
    get { key: String },
    set { key: String, value: String },
    remove { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
#[serde(tag = "type")]
pub enum Response {
	ok(Option<String>),
	err(String),
}
