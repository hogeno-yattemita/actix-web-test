use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsChatSession {
    pub id: usize,
    
    pub hb:Instant,

    pub room: String,

    pub name: Option<String>,

    pub addr: Addr<server::ChatServer>,
}

impl WsChatSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                fut::ready(())
            }).wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => {
                ctx.stop();
                return;
            }
        };
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            },
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }, // ignore pong responses
            ws::Message::Text(text) => {
                let m = text.trim();
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0]{
                        "/list" => {
                            println!("Listing rooms");
                            self.addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, act, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                }).wait(ctx);
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.room = v[1].to_string();
                                self.addr
                                    .do_send(server::Join {
                                        id: self.id,
                                        name: self.room.clone(),
                                    });

                                    ctx.text("joined");
                            }else{
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_string());
                                ctx.text("name set");
                            }else{
                                ctx.text("!!! name is required");
                            }
                        }
                        _ => ctx.text("!!! unknown command"),
                    }
                } else{
                    let msg = if let Some(ref name) = self.name {
                        format!("{}: {}", name, m)
                    } else {
                        m.to_string()
                    };
                    self.addr.do_send(server::ClientMessage {
                        id: self.id,
                        msg: msg,
                        room: self.room.clone(),
                    });
                }
            }
            ws::Message::Binary(_) => {
                ctx.text("only text messages supported");
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        };
    }
}