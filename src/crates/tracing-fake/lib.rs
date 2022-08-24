use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone, Debug)]
pub struct Span;

pub struct Entered;

pub struct Event;

impl Span {
    pub fn enter(&self) -> Entered {
        Entered
    }

    pub fn in_scope<F: FnOnce() -> T, T>(&self, f: F) -> T {
        f()
    }
}

pub trait Instrument: Sized {
    fn instrument(self, _: Span) -> Instrumented<Self> {
        Instrumented(self)
    }

    fn in_current_span(self) -> Instrumented<Self> {
        Instrumented(self)
    }
}

impl<T: Sized> Instrument for T {}

pub struct Instrumented<T>(T);

impl<T: Future> Future for Instrumented<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0).poll(cx) }
    }
}

pub mod instrument {
    pub use super::Instrument;
    pub use super::Instrumented;
}

#[rustfmt::skip]
mod macros {
    #[macro_export] macro_rules! info { ($($arg:tt)*) => {{}}; }
    #[macro_export] macro_rules! warn { ($($arg:tt)*) => {{}}; }
    #[macro_export] macro_rules! trace { ($($arg:tt)*) => {{}}; }
    #[macro_export] macro_rules! debug { ($($arg:tt)*) => {{}}; }
    #[macro_export] macro_rules! error { ($($arg:tt)*) => {{}}; }
    #[macro_export] macro_rules! trace_span { ($($arg:tt)*) => {{ ::tracing::Span }}; }
    #[macro_export] macro_rules! debug_span { ($($arg:tt)*) => {{ ::tracing::Span }}; }
}
