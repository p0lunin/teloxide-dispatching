use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use teloxide_core::types::{CallbackQuery, Message, Update, UpdateKind};
use teloxide_dispatching::core::Dispatcher;
use teloxide_dispatching::updates;

#[tokio::test]
async fn test() {
    let handled = Arc::new(AtomicBool::new(false));
    let handled2 = handled.clone();

    let dispatcher = Dispatcher::<Update>::new()
        .service(updates::message().common().to(move |message: Message| {
            assert_eq!(message.text().unwrap(), "text");
            handled2.store(true, Ordering::SeqCst);
            async move {}
        }))
        .service(
            updates::callback_query().to(move |_: CallbackQuery| async move { unreachable!() }),
        );

    let message = Update::new(0, UpdateKind::Message(text_message("text")));

    dispatcher.dispatch_one(message).await;

    assert!(handled.load(Ordering::SeqCst));
}

fn text_message<T: Into<String>>(text: T) -> Message {
    use teloxide_core::types::ChatKind::Private;
    use teloxide_core::types::ForwardKind::Origin;
    use teloxide_core::types::MediaKind::Text;
    use teloxide_core::types::MessageKind::Common;
    use teloxide_core::types::*;

    Message {
        id: 199785,
        date: 1568289890,
        chat: Chat {
            id: 250918540,
            kind: Private(ChatPrivate {
                type_: (),
                username: Some("aka_dude".into()),
                first_name: Some("Андрей".into()),
                last_name: Some("Власов".into()),
            }),
            photo: None,
        },
        via_bot: None,
        kind: Common(MessageCommon {
            from: Some(User {
                id: 250918540,
                is_bot: false,
                first_name: "Андрей".into(),
                last_name: Some("Власов".into()),
                username: Some("aka_dude".into()),
                language_code: Some("en".into()),
            }),
            forward_kind: Origin(ForwardOrigin {
                reply_to_message: None,
            }),
            edit_date: None,
            media_kind: Text(MediaText {
                text: text.into(),
                entities: vec![],
            }),
            reply_markup: None,
        }),
    }
}
