pub mod poller {
    use rand::Rng;
    use std::time::{Duration, Instant};
    use tokio::sync::broadcast::Sender;
    use tracing::{event, Level};

    use crate::sr::{fetch_page, Message};

    pub(crate) struct SrPoller {
        messages: Vec<Message>,
        last_message_id: i32,
        tx: Sender<Message>,
        options: SrPollOptions,
    }

    pub struct SrPollOptions {
        sleep_duration: usize,
    }

    impl SrPollOptions {
        pub fn new(sleep_duration: usize) -> Self {
            Self { sleep_duration }
        }
    }

    impl SrPoller {
        pub fn new(tx: Sender<Message>, options: SrPollOptions) -> Self {
            Self {
                messages: vec![],
                last_message_id: 0,
                tx,
                options,
            }
        }

        // Todo: Fix panichandling ok. thanks. bye
        pub async fn poll(&mut self) {
            if self.messages.is_empty() {
                let start = Instant::now();
                let mut msgs = match fetch_page(1).await {
                    Ok(sr) => sr.messages,
                    Err(e) => {
                        event!(Level::ERROR, "Could not fetch message: {}", e);
                        vec![]
                    }
                };
                event!(Level::INFO, "Loading messages took: {:?}", start.elapsed());
                self.messages.append(&mut msgs);
            }

            event!(Level::INFO, "Loaded {} messages", self.messages.len());

            loop {
                let start = Instant::now();
                let poll_messages = match fetch_page(1).await {
                    Ok(sr) => sr.messages,
                    Err(e) => {
                        event!(Level::ERROR, "Could not fetch message: {}", e);
                        vec![]
                    }
                };

                event!(Level::INFO, "Polling messages took: {:?}", start.elapsed());

                let new_messages = self.get_new_messages(poll_messages);

                event!(Level::INFO, "Found {} new messsages.", new_messages.len());

                // TODO remove dummy send
                {
                    if self.messages.len() > 0 {
                        event!(Level::INFO, "Sending random message");
                        let mut rng = rand::thread_rng();
                        let rindex = rng.gen_range(0..(self.messages.len() - 1));

                        match self.tx.send(self.messages[rindex].clone()) {
                            Ok(x) => event!(Level::INFO, "Sent to number of subs: {}", x),
                            Err(e) => event!(Level::ERROR, ": {}", e),
                        };
                    }
                }
                // TODO remove dummy send

                for message in &new_messages {
                    self.tx.send(message.clone()).unwrap();
                    self.last_message_id = message.id;
                    self.messages.push(message.clone());
                }

                std::thread::sleep(Duration::from_secs(self.options.sleep_duration as u64));
            }
        }

        fn get_new_messages(&self, new_messages: Vec<Message>) -> Vec<Message> {
            new_messages
                .into_iter()
                .filter(|new_msg| {
                    self.messages
                        .iter()
                        .find(|old_msg| old_msg.id == new_msg.id)
                        .is_none()
                })
                .collect::<Vec<Message>>()
        }
    }
}
