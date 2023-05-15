pub mod srws {
    enum Actions {
        SendAllMessage,
        SendMessage,
    }

    type SessionId = u16;
    type Session = ezsockets::Session<SessoinId, Actions>;

    struct SrTrafficMessageServer {}

    struct SrSession {
        handle: Session,
        id: SessionId,
    }
}
