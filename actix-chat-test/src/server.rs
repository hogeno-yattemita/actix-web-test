use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
    pub room: String,
}

pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize,
    pub name: String,
}


#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer{
        let mut rooms = HashMap::new();
        rooms.insert("main".to_string(), HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count
        }
    }
}

impl ChatServer{
    fn send_message(&self, room:&str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions{
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_string()));
                    }
                }
            }
        }
    }
}


impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer{
    type Result = usize;
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        self.send_message("main", "Someone joined", 0);

        let id = self.sessions.len();
        self.sessions.insert(id, msg.addr);

        self.rooms
            .entry("main".to_owned())
            .or_insert(HashSet::new())
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::Relaxed);
        self.send_message("main", &format!("{} visitors online", count), 0);

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");
        
        let mut rooms: Vec<String> = Vec::new();

        if self.sessions.remove(&msg.id).is_some() {
            for  (name, sessions)in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.clone());
                }
            }

        }
        
        for room in rooms {
            self.send_message(&room, &format!("{} left", msg.id), 0);
        }
    }
}


impl Handler<ClientMessage> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        println!("{} sent message: {}", msg.id, msg.msg);
        self.send_message(&msg.room, msg.msg.as_str(), msg.id);
    }
}

impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned());
        }
        
        MessageResult(rooms)
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join {id, name} = msg;
        let mut rooms = Vec::new();

        for(n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        
        for room in rooms {
            self.send_message(&room, &format!("{} left", id), 0);
        }

        self.rooms
            .entry(name.clone())
            .or_insert(HashSet::new())
            .insert(id);

        self.send_message(&name, &format!("{} joined", id), id);
    }
}