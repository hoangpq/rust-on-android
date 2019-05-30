use futures::{Async, Future, IntoFuture, Poll, Stream};

#[derive(Clone, Debug)]
pub struct TakeUntil<S, F> {
    stream: S,
    until: F,
    free: bool,
}

pub trait StreamExt: Stream {
    fn take_until<U>(self, until: U) -> TakeUntil<Self, U::Future>
    where
        U: IntoFuture<Item = (), Error = ()>,
        Self: Sized,
    {
        TakeUntil {
            stream: self,
            until: until.into_future(),
            free: false,
        }
    }
}

impl<S> StreamExt for S where S: Stream {}

impl<S, F> Stream for TakeUntil<S, F>
where
    S: Stream,
    F: Future<Item = (), Error = ()>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if !self.free {
            match self.until.poll() {
                Ok(Async::Ready(_)) => {
                    return Ok(Async::Ready(None));
                }
                Err(_) => {
                    self.free = true;
                }
                Ok(Async::NotReady) => {}
                Ok(_) => {}
            }
        }
        self.stream.poll()
    }
}
