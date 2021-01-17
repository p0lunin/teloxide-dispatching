mod impls {
    use crate::core::{Guards, Handler, HandlerInto, Service};
    use crate::handlers::parser::{Initialized, NotInitialized, ParseUpdate, UpdateParser};
    use std::future::Future;
    use std::marker::PhantomData;
    use teloxide_core::types::{Message, Update};

    pub(crate) mod parser {
        pub struct Common;
        pub struct NewChatMembers;
        pub struct LeftChatMember;
        pub struct NewChatTitle;
        pub struct NewChatPhoto;
        pub struct DeleteChatPhoto;
        pub struct GroupChatCreated;
        pub struct SupergroupChatCreated;
        pub struct ChannelChatCreated;
        pub struct Migrate;
        pub struct Pinned;
        pub struct Invoice;
        pub struct SuccessfulPayment;
        pub struct ConnectedWebsite;
        pub struct PassportData;
        pub struct Dice;
    }

    macro_rules! impl_parser {
        ($($ty:ident,)*) => {
            $(
                impl ParseUpdate<teloxide_core::types::Message> for parser::$ty {
                    type Upd = teloxide_core::types::Message;

                    fn check(update: &teloxide_core::types::Message) -> bool {
                        matches!(update.kind, teloxide_core::types::MessageKind::$ty(_))
                    }

                    fn parse(update: teloxide_core::types::Message) -> Self::Upd {
                        match &update.kind {
                            teloxide_core::types::MessageKind::$ty(_) => update,
                            _ => unreachable!("We already checks that message is {}", stringify!($ty)),
                        }
                    }
                }
            )*
        }
    }

    impl_parser!(
        Common,
        NewChatMembers,
        LeftChatMember,
        NewChatTitle,
        NewChatPhoto,
        DeleteChatPhoto,
        GroupChatCreated,
        SupergroupChatCreated,
        ChannelChatCreated,
        Migrate,
        Pinned,
        Invoice,
        SuccessfulPayment,
        ConnectedWebsite,
        PassportData,
        Dice,
    );

    pub struct MessageParser<ParentParser, Parser, Init>
    where
        ParentParser: ParseUpdate<Update, Upd = Message>,
    {
        parent: Service<Update>,
        service: Service<Message>,
        phantom: PhantomData<(ParentParser, Parser, Init)>,
    }

    impl<ParentParser, Parser> MessageParser<ParentParser, Parser, NotInitialized>
    where
        ParentParser: ParseUpdate<Update, Upd = Message>,
        Parser: ParseUpdate<ParentParser::Upd> + 'static,
        ParentParser::Upd: 'static,
    {
        pub fn new(parent: Service<Update>) -> Self {
            let service = Service::new(Guards::new().add(Parser::check), || async move {});
            MessageParser {
                parent,
                service,
                phantom: PhantomData,
            }
        }

        pub fn to<F, H, Fut>(self, f: F) -> MessageParser<ParentParser, Parser, Initialized>
        where
            H: Handler<Message, Fut> + 'static,
            F: HandlerInto<H>,
            Fut: Future<Output = ()> + Send + 'static,
        {
            let MessageParser {
                parent,
                service: Service { guards, .. },
                ..
            } = self;
            let handler = f.into_handler();
            let service = Service::new(guards, move |update: ParentParser::Upd| {
                handler.handle(update)
            });
            MessageParser {
                parent,
                service,
                phantom: PhantomData,
            }
        }
    }

    impl<ParentParser, Parser> Into<Service<Update>>
        for MessageParser<ParentParser, Parser, Initialized>
    where
        ParentParser: ParseUpdate<Update, Upd = Message>,
    {
        fn into(self) -> Service<Update> {
            let MessageParser {
                parent,
                service: message_service,
                ..
            } = self;
            let Service { guards, .. } = parent;
            let Service {
                guards: message_guards,
                handler,
            } = message_service;
            Service::new(guards, move |upd: Update| {
                let mes = ParentParser::parse(upd);
                if message_guards.check(&mes) {
                    futures::future::Either::Left(handler.handle(mes))
                } else {
                    futures::future::Either::Right(async move {})
                }
            })
        }
    }

    impl<Parser> UpdateParser<Update, Parser, NotInitialized>
    where
        Parser: ParseUpdate<Update, Upd = Message>,
    {
        pub fn common(self) -> MessageParser<Parser, parser::Common, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn new_chat_members(
            self,
        ) -> MessageParser<Parser, parser::NewChatMembers, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn left_chat_member(
            self,
        ) -> MessageParser<Parser, parser::LeftChatMember, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn new_chat_title(self) -> MessageParser<Parser, parser::NewChatTitle, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn new_chat_photo(self) -> MessageParser<Parser, parser::NewChatPhoto, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn delete_chat_photo(
            self,
        ) -> MessageParser<Parser, parser::DeleteChatPhoto, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn group_chat_created(
            self,
        ) -> MessageParser<Parser, parser::GroupChatCreated, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn supergroup_chat_created(
            self,
        ) -> MessageParser<Parser, parser::SupergroupChatCreated, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn channel_chat_created(
            self,
        ) -> MessageParser<Parser, parser::ChannelChatCreated, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn migrate(self) -> MessageParser<Parser, parser::Migrate, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn pinned(self) -> MessageParser<Parser, parser::Pinned, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn invoice(self) -> MessageParser<Parser, parser::Invoice, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn successful_payment(
            self,
        ) -> MessageParser<Parser, parser::SuccessfulPayment, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn connected_website(
            self,
        ) -> MessageParser<Parser, parser::ConnectedWebsite, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn passport_data(self) -> MessageParser<Parser, parser::PassportData, NotInitialized> {
            MessageParser::new(self.into_inner())
        }

        pub fn dice(self) -> MessageParser<Parser, parser::Dice, NotInitialized> {
            MessageParser::new(self.into_inner())
        }
    }
}
