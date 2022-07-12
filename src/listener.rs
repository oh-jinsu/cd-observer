use std::str;
use std::io::prelude::*;
use std::error::Error;
use std::net::{ TcpListener };

pub type Handler = Box<dyn Fn(&str) -> Result<String, Box<dyn Error>>>;

pub trait Listener {
  fn add(&mut self, handler: Handler);
  fn listen(&self, address: &str) -> Result<(), Box<dyn Error>>;
}

struct ListenerImpl
{
  handlers: Vec<Handler>
}

impl Listener for ListenerImpl  {
  fn add(&mut self, handler: Handler) {
    self.handlers.push(handler)
  }

  fn listen(&self, address: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
      let mut stream = match stream {
          Ok(x) => x,
          Err(e) => {
              println!("{}", e);

              continue;
          }
      };

      for handler in self.handlers.iter() {
        let mut buffer = [0; 512];

        stream.read(&mut buffer)?;
    
        let request = str::from_utf8(&buffer)?;

        match handler(request) {
          Ok(response) => {
            stream.write(response.as_bytes())?;

            stream.flush()?;
          }
          Err(e) => {
            println!("{}", e);
          }
        }
      }
    }

    Ok(())
  }
}

pub fn new() -> Box<dyn Listener> {
  return Box::new(ListenerImpl {
    handlers: vec![]
  })
}