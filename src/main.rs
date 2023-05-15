use std::time::Duration;

use async_trait::async_trait;
use ezsockets::Error;
use pepe::sr;
use tokio;

type SessionId = u16;
type Session = ezsockets::Session<SessionId, Message>;

struct EchoServer {}

struct EchoSession {
    handle: Session,
    id: SessionId,
    counter: usize,
    echo_task: tokio::task::JoinHandle<()>,
}

impl Drop for EchoSession {
    fn drop(&mut self) {
        self.echo_task.abort();
    }
}

#[derive(Debug)]
enum Message {
    Increment,
    Share,
}

#[async_trait]
impl ezsockets::ServerExt for EchoServer {
    type Session = EchoSession;
    type Call = ();

    async fn on_connect(
        &mut self,
        socket: ezsockets::Socket,
        address: std::net::SocketAddr,
        _: <Self::Session as ezsockets::SessionExt>::Args,
    ) -> Result<Session, Error> {
        let id = address.port();
        let session = Session::create(
            |handle| {
                let echo_task = tokio::spawn({
                    let ss = handle.clone();
                    ss.call(Message::Increment);
                    async move {
                        loop {
                            ss.call(Message::Increment);
                            ss.call(Message::Share);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                });
                EchoSession {
                    id,
                    handle,
                    counter: 0,
                    echo_task,
                }
            },
            id,
            socket,
        );
        Ok(session)
    }

    async fn on_disconnect(
        &mut self,
        id: <Self::Session as ezsockets::SessionExt>::ID,
    ) -> Result<(), Error> {
        println!("Disconnection {}", id);
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        let () = call;
        Ok(())
    }
}

#[async_trait]
impl ezsockets::SessionExt for EchoSession {
    type ID = SessionId;
    type Args = ();
    type Call = Message;

    fn id(&self) -> &Self::ID {
        &self.id
    }

    async fn on_text(&mut self, text: String) -> Result<(), Error> {
        self.handle.text(text);
        Ok(())
    }

    async fn on_binary(&mut self, _: Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        match call {
            Message::Increment => self.counter += 1,
            Message::Share => self.handle.text(format!("counter: {}", self.counter)),
        };
        Ok(())
    }
}

#[tokio::main()]
async fn main() {
    match sr::load_all_messages().await {
        Ok(messages) => {
            for m in &messages {
                println!("=====================================");
                println!("= Location:    {}", m.exactlocation);
                println!("= Description: {}", m.description);
                println!("= Category:    {}", m.subcategory);
                println!("=====================================")
            }
            println!("Number of message: {}", messages.len())
        }
        Err(e) => println!("error: {:?}", e),
    };

    /*
    let req = SrRequest {
        format: String::from("json"),
        indent: true,
        page: 1,
    };
    let sr_response = sr::fetch_messages(req).await.unwrap();
    println!("response: {}", sr_response);
    let sr = Sr::from_json(&sr_response).unwrap();

    println!("SR: {:?}", sr);

    tracing_subscriber::fmt::init();
    let (server, _) = Server::create(|_server| EchoServer {});
    ezsockets::tungstenite::run(server, "127.0.0.1:8080", |_socket| async move { Ok(()) })
        .await
        .unwrap();
        */
}
