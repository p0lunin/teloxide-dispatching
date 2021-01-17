use crate::handlers::parser::{NotInitialized, UpdateParser};
use impls::parser;
use teloxide_core::types::Update;

pub fn any() -> UpdateParser<Update, parser::Update, NotInitialized> {
    UpdateParser::new()
}

pub fn message() -> UpdateParser<Update, parser::Message, NotInitialized> {
    UpdateParser::new()
}

pub fn edited_message() -> UpdateParser<Update, parser::EditedMessage, NotInitialized> {
    UpdateParser::new()
}

pub fn channel_post() -> UpdateParser<Update, parser::ChannelPost, NotInitialized> {
    UpdateParser::new()
}

pub fn edited_channel_post() -> UpdateParser<Update, parser::EditedChannelPost, NotInitialized> {
    UpdateParser::new()
}

pub fn inline_query() -> UpdateParser<Update, parser::InlineQuery, NotInitialized> {
    UpdateParser::new()
}

pub fn chosen_inline_result() -> UpdateParser<Update, parser::ChosenInlineResult, NotInitialized> {
    UpdateParser::new()
}

pub fn callback_query() -> UpdateParser<Update, parser::CallbackQuery, NotInitialized> {
    UpdateParser::new()
}

pub fn shipping_query() -> UpdateParser<Update, parser::ShippingQuery, NotInitialized> {
    UpdateParser::new()
}

pub fn pre_checkout_query() -> UpdateParser<Update, parser::PreCheckoutQuery, NotInitialized> {
    UpdateParser::new()
}

pub fn poll() -> UpdateParser<Update, parser::Poll, NotInitialized> {
    UpdateParser::new()
}

pub fn poll_answer() -> UpdateParser<Update, parser::PollAnswer, NotInitialized> {
    UpdateParser::new()
}

mod impls {
    use crate::handlers::parser::ParseUpdate;
    use teloxide_core::types::Update;

    pub(crate) mod parser {
        pub struct Update;
        pub struct Message;
        pub struct EditedMessage;
        pub struct ChannelPost;
        pub struct EditedChannelPost;
        pub struct InlineQuery;
        pub struct ChosenInlineResult;
        pub struct CallbackQuery;
        pub struct ShippingQuery;
        pub struct PreCheckoutQuery;
        pub struct Poll;
        pub struct PollAnswer;
    }

    macro_rules! impl_parser {
        ($(($ty:ident, $teloxide_ty:ident),)*) => {
            $(
                impl ParseUpdate<teloxide_core::types::Update> for parser::$ty {
                    type Upd = teloxide_core::types::$teloxide_ty;

                    fn check(update: &Update) -> bool {
                        matches!(update.kind, teloxide_core::types::UpdateKind::$ty(_))
                    }

                    fn parse(update: Update) -> Self::Upd {
                        match update.kind {
                            teloxide_core::types::UpdateKind::$ty(message) => message,
                            _ => unreachable!("We already checks that update is {}", stringify!($ty)),
                        }
                    }
                }
            )*
        }
    }

    impl ParseUpdate<Update> for parser::Update {
        type Upd = teloxide_core::types::Update;

        fn check(_: &Update) -> bool {
            true
        }

        fn parse(update: Update) -> Self::Upd {
            update
        }
    }

    impl_parser!(
        (Message, Message),
        (EditedMessage, Message),
        (ChannelPost, Message),
        (EditedChannelPost, Message),
        (InlineQuery, InlineQuery),
        (ChosenInlineResult, ChosenInlineResult),
        (CallbackQuery, CallbackQuery),
        (ShippingQuery, ShippingQuery),
        (PreCheckoutQuery, PreCheckoutQuery),
        (Poll, Poll),
        (PollAnswer, PollAnswer),
    );
}
