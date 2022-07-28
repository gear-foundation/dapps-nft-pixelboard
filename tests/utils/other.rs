use super::prelude::*;
use core::{fmt::Debug, marker::PhantomData};
use gstd::ActorId;
use gtest::{Log, Program as InnerProgram, RunResult, System};

pub fn initialize_system() -> System {
    let system = System::new();
    system.init_logger();
    system
}

pub trait Program {
    fn inner_program(&self) -> &InnerProgram;

    fn actor_id(&self) -> ActorId {
        self.inner_program().id().as_ref().try_into().unwrap()
    }
}

pub struct MetaStateReply<T>(pub T);

impl<T: Debug + PartialEq> MetaStateReply<T> {
    #[track_caller]
    pub fn check(self, value: T) {
        assert_eq!(self.0, value);
    }
}

pub struct Action<F, T, R>(pub RunResult, pub F, pub PhantomData<(T, R)>)
where
    F: FnOnce(T) -> R,
    R: Encode;

impl<F, T, R> Action<F, T, R>
where
    F: FnOnce(T) -> R,
    R: Encode,
{
    #[track_caller]
    pub fn check(self, value: T) {
        assert!(self.0.contains(&Log::builder().payload(self.1(value))));
    }

    #[track_caller]
    pub fn failed(self) {
        assert!(self.0.main_failed())
    }
}
