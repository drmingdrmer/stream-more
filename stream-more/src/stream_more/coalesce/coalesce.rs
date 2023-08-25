use std::pin::Pin;
use std::task::ready;
use std::task::Context;
use std::task::Poll;

use futures::stream::BoxStream;
use futures::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// A [`Stream`] that coalesces adjacent items by user provided function.
    pub struct Coalesce<'a, T, F> {
        #[pin]
        prev: Option<T>,
        finished: bool,
        inner: BoxStream<'a, T>,
        f: F,
    }
}

impl<'a, T, F> Coalesce<'a, T, F>
where F: FnMut(T, T) -> Result<T, (T, T)>
{
    pub fn new(stream: BoxStream<'a, T>, f: F) -> Self {
        Coalesce {
            prev: None,
            finished: false,
            inner: stream,
            f,
        }
    }
}

impl<'a, T, F> Stream for Coalesce<'a, T, F>
where
    T: Unpin,
    F: FnMut(T, T) -> Result<T, (T, T)>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        let mut this = self.project();
        loop {
            let current = ready!(this.inner.as_mut().poll_next(cx));
            let prev = this.prev.take();

            match (prev, current) {
                (None, None) => {
                    return Poll::Ready(None);
                }
                (Some(p), None) => {
                    *this.finished = true;
                    return Poll::Ready(Some(p));
                }
                (None, Some(c)) => {
                    *this.prev = Some(c); // continue;
                }
                (Some(p), Some(c)) => {
                    let res = (this.f)(p, c);
                    match res {
                        Ok(x) => {
                            *this.prev = Some(x);
                        }
                        Err((prev, current)) => {
                            *this.prev = Some(current);
                            return Poll::Ready(Some(prev));
                        }
                    }
                }
            }
        }
    }
}
