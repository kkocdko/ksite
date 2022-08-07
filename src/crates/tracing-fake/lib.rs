use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct Span;

pub struct Entered;

pub struct Event;

pub trait Instrument: Sized {
    fn instrument(self, _: Span) -> Instrumented<Self> {
        Instrumented { inner: self }
    }

    fn in_current_span(self) -> Instrumented<Self> {
        self.instrument(Span)
    }
}

impl<T: Sized> Instrument for T {}

pin_project! {
    pub struct Instrumented<T> {
        #[pin]
        inner: T,
    }
}

impl<T: Future> Future for Instrumented<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}

impl Span {
    pub fn enter(&self) -> Entered {
        Entered
    }

    pub fn in_scope<F: FnOnce() -> T, T>(&self, f: F) -> T {
        f()
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
