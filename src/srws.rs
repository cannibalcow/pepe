pub mod srws {
    use async_trait::async_trait;
    use ezsockets::{Error, SessionExt};
    use pepe::sr::Message;
    use tokio::sync::broadcast::Receiver;

    #[derive(Debug)]
    pub enum Actions {
        SendMessage { msg: Message },
    }

    type SessionId = u16;
    type Session = ezsockets::Session<SessionId, Actions>;

    pub struct SrTrafficMessageServer {
        pub rx: Receiver<Message>,
    }

    pub struct SrSession {
        id: SessionId,
        handle: Session,
        event_task: tokio::task::JoinHandle<()>,
    }

    impl Drop for SrSession {
        fn drop(&mut self) {
            self.event_task.abort();
        }
    }

    #[async_trait]
    impl SessionExt for SrSession {
        type ID = SessionId;
        type Args = ();
        type Call = Actions;

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
                Actions::SendMessage { msg } => self.handle.text(format!("{}", msg)),
            };
            Ok(())
        }
    }

    #[async_trait]
    impl ezsockets::ServerExt for SrTrafficMessageServer {
        type Session = SrSession;
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
                    let task = tokio::spawn({
                        let mut rx2 = self.rx.resubscribe();
                        let ss = handle.clone();
                        async move {
                            loop {
                                let new_message = rx2.recv().await;
                                match new_message {
                                    Ok(m) => ss.call(Actions::SendMessage { msg: m }),
                                    Err(e) => println!("Webclient error: {:?}", e),
                                };
                            }
                        }
                    });
                    SrSession {
                        id,
                        handle,
                        event_task: task,
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
}
