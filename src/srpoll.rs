pub mod srpoll {

    use std::time::{Duration, Instant};

    use pepe::sr::Message;
    use tokio::sync::broadcast::Sender;
    use tracing::{event, Level};

    use crate::srclient::srclient::fetch_page;

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
                let mut msgs = fetch_page(1).await.unwrap();
                event!(Level::INFO, "Loading messages took: {:?}", start.elapsed());
                self.messages.append(&mut msgs.messages);
            }

            event!(Level::INFO, "Loaded {} messages", self.messages.len());

            loop {
                let page = fetch_page(1).await.unwrap();

                let new_messages = self.get_new_messages(page.messages);

                event!(Level::INFO, "Found {} new messsages.", new_messages.len());

                // TODO remove retard
                self.tx.send(self.messages[0].clone()).unwrap();

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
