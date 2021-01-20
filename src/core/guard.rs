pub trait Guard<Upd: ?Sized> {
    fn check(&self, update: &Upd) -> bool;
}

impl<F, Upd> Guard<Upd> for F
where
    F: Fn(&Upd) -> bool,
{
    fn check(&self, update: &Upd) -> bool {
        self(update)
    }
}

pub struct Guards<Upd> {
    guards: Vec<Box<dyn Guard<Upd>>>,
}

impl<Upd> Guards<Upd> {
    pub fn new() -> Self {
        Guards { guards: Vec::new() }
    }

    pub fn add<T>(mut self, data: T) -> Self
    where
        T: Guard<Upd> + 'static,
    {
        self.guards.push(Box::new(data));
        self
    }

    pub fn add_guard<T>(&mut self, data: T)
    where
        T: Guard<Upd> + 'static,
    {
        self.guards.push(Box::new(data));
    }

    pub fn check(&self, update: &Upd) -> bool {
        Guard::check(self, update)
    }

    pub fn with(mut self, other: Self) -> Self {
        self.guards.extend(other.guards.into_iter());
        self
    }
}

impl<Upd> Guard<Upd> for Guards<Upd> {
    fn check(&self, update: &Upd) -> bool {
        self.guards.iter().all(|guard| guard.check(update))
    }
}
