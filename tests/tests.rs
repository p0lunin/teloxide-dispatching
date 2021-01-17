use std::ops::Deref;
use std::sync::Arc;
use teloxide_dispatching::core::{Dispatcher, Guards, ParserHandler, Service};
use tokio::sync::Mutex;

#[tokio::test]
async fn test() {
    let char = Arc::new(Mutex::new(None));
    let dispatcher = Dispatcher::<String>::new().service(Service {
        guards: Guards::new().add(|s: &String| s.is_ascii()),
        handler: Arc::new(ParserHandler::new(|s: String| s.chars().next().unwrap(), {
            let char = char.clone();
            move |req: char| {
                let char = char.clone();
                async move {
                    *char.lock().await = Some(req);
                }
            }
        })) as _,
    });
    dispatcher.dispatch_one(String::from("Hello")).await;
    assert_eq!(char.lock().await.deref(), &Some('H'));
}
