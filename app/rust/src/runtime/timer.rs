use futures::stream::Stream;
use futures::sync::oneshot;
use futures::{future, Future};
use std::convert::Into;
use std::time::{Duration, Instant};
use tokio_timer::{Delay, Interval};

pub fn panic_on_error<I, E, F>(f: F) -> impl Future<Item = I, Error = ()>
where
    F: Future<Item = I, Error = E>,
    E: std::fmt::Debug,
{
    f.map_err(|err| adb_debug!(format!("Future got unexpected error: {:?}", err)))
}

pub fn set_timeout<F>(
    cb: F,
    delay: u32,
) -> (
    impl Future<Item = (), Error = ()>,
    futures::sync::oneshot::Sender<()>,
)
where
    F: FnOnce() -> (),
{
    let (tx, rx) = oneshot::channel();
    let duration = Duration::from_millis(delay.into());

    let rx = panic_on_error(rx);
    let delay_task = panic_on_error(Delay::new(Instant::now() + duration))
        .and_then(|_| {
            cb();
            Ok(())
        });

    (delay_task, tx)
}

pub fn set_interval<F>(
    cb: F,
    delay: u32,
) -> (
    impl Future<Item = (), Error = ()>,
    futures::sync::oneshot::Sender<()>,
)
where
    F: Fn() -> (),
{
    let (tx, rx) = oneshot::channel();
    let delay = Duration::from_millis(delay.into());

    let rx = panic_on_error(rx);
    let interval_task = Interval::new(Instant::now() + delay, delay)
        .for_each(move |_| {
            cb();
            future::ok(())
        })
        .map_err(|e| adb_debug!(e));

    (interval_task, tx)
}
