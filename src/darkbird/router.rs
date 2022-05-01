use crate::darkbird::{SessionResult, TIMEOUT, Status};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendTimeoutError;

use crate::darkbird::WorkerState;


/// In some cases it is useful to distribute messages of the same type over a set of channels, 
/// so that messages can be processed in parallel - a single task will only process one message at a time.
/// 
/// 
/// ```rust 
/// 
///  #[tokio::main]
///  async fn main() {
///  
///      let w1 = worker();
///      let w2 = worker();
///      let w3 = worker();
///  
///  
///  
///      // Create new `Router` and pass `Vec<Sender>`, and `RouterType` (`RoundRobin` || `Broadcast`)
///      let router = Router::new(vec![w1, w2, w3], RouterType::RoundRobin).unwrap();
///      
///  
///      // # Session can do
///      //      1. `dispatch` msg
///      //      2. `register` new worker (if not exist)
///      //      3.  `clone`   (it just internally call mpsc::sender::clone(&self))
///  
///      // run_service 
///      let session = router.run_service();
///  
///  
///      // dispatch message by router
///      session.dispatch("Message-1".to_owned()).await;
///  
///  
///      // Create and register new channel, can register after `run_service`
///      let w5 = worker();
///      session.register(w5).await;
///  
///  
///      // dispatch message by router
///      session.dispatch("Message-2".to_owned()).await;
///  
///  }
///  
///  
///  fn worker() -> mpsc::Sender<String> {
///      
///      // Create channel
///      let (sx, mut rx) = mpsc::channel(10);
///      
///      // Spawn task
///      tokio::spawn(async move {
///          while let Some(msg) = rx.recv().await {
///              println!("==> {}", msg);
///          }
///  
///      });
///  
///      sx
///  }
/// 
/// 
/// ```
/// 




/// Msg is msg produced but not exit any channel to consume it 
pub struct DestinationDown<Msg>(Msg);


pub enum Request<Msg> {
    Register(Sender<Msg>),
    Dispatch(Msg)
}




pub enum RouterType {
    Broadcast
}


pub struct Router<Msg> {
    c: usize,
    channels: Vec<Sender<Msg>>,
    router_type: RouterType
}

impl<Msg> Router<Msg> 
where
    Msg: Clone + Send + 'static
{
    

    pub fn new(channels: Vec<Sender<Msg>>) -> Result<Self, Status> {

        // Check channels to not be repetive
        if let Err(_) = Router::list_check(&channels) {
            return Err(Status::SendersRepetive);
        }

        Ok(Router { 
            c: 0, 
            channels,
            router_type: RouterType::Broadcast
        })
    }


    pub fn run_service(mut self) -> Session<Msg> {

        let (sx, mut rx) = mpsc::channel(30);
        
        let session = Session::new(sx);

        tokio::spawn(async move {
            loop {
                let res = rx.recv().await;
                if let WorkerState::Disconnected = self.handle_recv(res).await {
                    return ();
                }
            }
        });

        return session
    }

    async fn handle_recv(&mut self, res: Option<Request<Msg>>) -> WorkerState {
        match res {
            Some(req) => {
                match req  {
                    Request::Register(sender) => {
                        match self.check(&sender) {
                            Ok(_) => {
                                self.channels.push(sender);
                                WorkerState::Continue
                            }
                            Err(_) => {
                                return WorkerState::Continue
                            }
                        }
                        
                    }
                    Request::Dispatch(msg) => {
                        let _ = self.dispatch(msg).await;
                        WorkerState::Continue
                    }
                }
            }
            None => WorkerState::Disconnected
        }
    }
    

    #[inline]
    async fn dispatch(&mut self, msg: Msg) -> Result<(), DestinationDown<Msg>> {

        if self.channels.len() == 0 {
            return Ok(())
        }

        self.broadcast(msg).await;
        Ok(()) 
    }

    
    #[inline]
    async fn broadcast(&mut self, msg: Msg) {        
        for index in 0..=(self.channels.len() - 2) {
            let msg = msg.clone();
            let _ = self.channels[index].send(msg).await;
        }

        let _ = self.channels[self.channels.len() - 1].send(msg).await;

    }




    fn next_index(&mut self) -> usize {
        let mut index = self.c;

        self.c += 1;

        if index >= self.channels.len() {
            self.c = 0;
            index = 0;
        }

        return index
    }

    
    /// Check channels to not be repetive
    fn list_check(channels: &Vec<Sender<Msg>>) -> Result<(), ()> {
        for (oindex, outer_dst) in channels.iter().enumerate() {
            for (iindex, inner_dst) in channels.iter().enumerate() {
            
                // if not was itself && channel was same
                if oindex != iindex && outer_dst.same_channel(inner_dst) {
                    return Err(())
                }
            
            }
        }

        Ok(())
    }

    /// Check channel to registered before
    fn check(&self, chan: &Sender<Msg>) -> Result<(), ()> {
        for (_index, dst) in self.channels.iter().enumerate() {
            
            // if channel was same
            if chan.same_channel(dst) {
                return Err(())
            }

        }
        Ok(())
    }
}




// --------------------- Client Code --------------------------


pub struct Session<Msg> {
    sender: mpsc::Sender<Request<Msg>>
}

impl<Msg> Session<Msg> 
where
    Msg: Send + 'static
{
    fn new(sender: mpsc::Sender<Request<Msg>>) -> Self {
        Session { 
            sender 
        }
    }


    /// register new channel to router
    pub async fn register(&self, sender: Sender<Msg>) -> Result<(), SessionResult> {
        let res = self.sender.send_timeout(Request::Register(sender), TIMEOUT).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    SendTimeoutError::Timeout(_) => Err(SessionResult::Timeout),
                    SendTimeoutError::Closed(_) => Err(SessionResult::Closed),
                }
            }
        }
    }   


    /// dispatch msg by router
    pub async fn dispatch(&self, msg: Msg) -> Result<(), SessionResult> {
        let res = self.sender.send_timeout(Request::Dispatch(msg), TIMEOUT).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    SendTimeoutError::Timeout(_) => Err(SessionResult::Timeout),
                    SendTimeoutError::Closed(_) => Err(SessionResult::Closed),
                }
            }
        }
    }   

}