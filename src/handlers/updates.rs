use crate::handlers::parser::UpdateParser;
use teloxide_core::{types, types::Update};

pub(crate) use impls::{parser, UpdateRest};

pub fn any<Err>() -> UpdateParser<Update, Update, (), Err, parser::Update> {
    UpdateParser::new(parser::Update)
}

pub fn message<Err>() -> UpdateParser<Update, types::Message, UpdateRest, Err, parser::Message> {
    UpdateParser::new(parser::Message)
}

pub fn edited_message<Err>(
) -> UpdateParser<Update, types::Message, UpdateRest, Err, parser::EditedMessage> {
    UpdateParser::new(parser::EditedMessage)
}

pub fn channel_post<Err>(
) -> UpdateParser<Update, types::Message, UpdateRest, Err, parser::ChannelPost> {
    UpdateParser::new(parser::ChannelPost)
}

pub fn edited_channel_post<Err>(
) -> UpdateParser<Update, types::Message, UpdateRest, Err, parser::EditedChannelPost> {
    UpdateParser::new(parser::EditedChannelPost)
}

pub fn inline_query<Err>(
) -> UpdateParser<Update, types::InlineQuery, UpdateRest, Err, parser::InlineQuery> {
    UpdateParser::new(parser::InlineQuery)
}

pub fn chosen_inline_result<Err>(
) -> UpdateParser<Update, types::ChosenInlineResult, UpdateRest, Err, parser::ChosenInlineResult> {
    UpdateParser::new(parser::ChosenInlineResult)
}

pub fn callback_query<Err>(
) -> UpdateParser<Update, types::CallbackQuery, UpdateRest, Err, parser::CallbackQuery> {
    UpdateParser::new(parser::CallbackQuery)
}

pub fn shipping_query<Err>(
) -> UpdateParser<Update, types::ShippingQuery, UpdateRest, Err, parser::ShippingQuery> {
    UpdateParser::new(parser::ShippingQuery)
}

pub fn pre_checkout_query<Err>(
) -> UpdateParser<Update, types::PreCheckoutQuery, UpdateRest, Err, parser::PreCheckoutQuery> {
    UpdateParser::new(parser::PreCheckoutQuery)
}

pub fn poll<Err>() -> UpdateParser<Update, types::Poll, UpdateRest, Err, parser::Poll> {
    UpdateParser::new(parser::Poll)
}

pub fn poll_answer<Err>(
) -> UpdateParser<Update, types::PollAnswer, UpdateRest, Err, parser::PollAnswer> {
    UpdateParser::new(parser::PollAnswer)
}

mod impls {
    use crate::core::{Parser, ParserOut, RecombineFrom};
    use teloxide_core::types::{Update, UpdateKind};

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

    pub struct UpdateRest(i32);

    macro_rules! impl_parser {
        ($(($ty:ident, $teloxide_ty:ident),)*) => {
            $(
                impl RecombineFrom<parser::$ty> for Update {
                    type From = teloxide_core::types::$teloxide_ty;
                    type Rest = UpdateRest;

                    fn recombine(data: ParserOut<teloxide_core::types::$teloxide_ty, UpdateRest>) -> Update {
                        let (kind, UpdateRest(id)) = data.into_inner();
                        Update {
                            id,
                            kind: UpdateKind::$ty(kind),
                        }
                    }
                }
                impl Parser<teloxide_core::types::Update, teloxide_core::types::$teloxide_ty, UpdateRest> for parser::$ty {
                    fn parse(&self, update: Update) -> Result<ParserOut<teloxide_core::types::$teloxide_ty, UpdateRest>, Update> {
                        let Update { id, kind } = update;
                        let rest = UpdateRest(id);
                        match kind {
                            UpdateKind::$ty(message) => Ok(ParserOut::new(message, rest)),
                            _ => Err(<Update as RecombineFrom<UpdateKind>>::recombine(ParserOut::new(kind, rest))),
                        }
                    }
                }
            )*
        }
    }

    impl RecombineFrom<UpdateKind> for Update {
        type From = UpdateKind;
        type Rest = UpdateRest;

        fn recombine(data: ParserOut<UpdateKind, UpdateRest>) -> Update {
            let (kind, UpdateRest(id)) = data.into_inner();
            Update { id, kind }
        }
    }

    impl RecombineFrom<parser::Update> for Update {
        type From = Update;
        type Rest = ();

        fn recombine(data: ParserOut<Update, ()>) -> Update {
            let (update, _) = data.into_inner();
            update
        }
    }

    impl Parser<Update, Update, ()> for parser::Update {
        fn parse(&self, update: Update) -> Result<ParserOut<Update, ()>, Update> {
            Ok(ParserOut::new(update, ()))
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
