mod srws {
    use tokio::sync::mpsc::Receiver;

    use crate::Message;

    #[derive(Debug)]
    enum Actions {
        SendMessage,
    }

    type SessionId = u16;
    type Session = ezsockets::Session<SessionId, Actions>;

    struct SrTrafficMessageServer {}

    struct SrSession {
        handle: Session,
        id: SessionId,
        last_sent_message_id: i32,
        message_channel: Receiver<Message>,
    }
}
