use std::convert::Infallible;
use std::ops::Deref;
use std::sync::Arc;
use teloxide_dispatching::core::{DispatcherBuilder, ParserHandler, ParserOut, RecombineFrom};
use tokio::sync::Mutex;

struct Nums(u32, u32, u32);

impl<Parser> RecombineFrom<Parser> for Nums {
    type From = u32;
    type Rest = (u32, u32);

    fn recombine(data: ParserOut<u32, (u32, u32)>) -> Nums {
        let (a, (b, c)) = data.into_inner();
        Nums(a, b, c)
    }
}

#[tokio::test]
async fn test() {
    let char = Arc::new(Mutex::new(None));
    let dispatcher = DispatcherBuilder::<Nums, Infallible, _, _>::new()
        .handle(ParserHandler::new(
            |nums: Nums| Ok(ParserOut::new(nums.0, (nums.1, nums.2))),
            {
                let char = char.clone();
                move |req: u32| {
                    let char = char.clone();
                    async move {
                        *char.lock().await = Some(req);
                    }
                }
            },
        ))
        .error_handler(|_| async { unreachable!() })
        .build();
    dispatcher.dispatch_one(Nums(1, 2, 3)).await;
    assert_eq!(char.lock().await.deref(), &Some(1));
}
