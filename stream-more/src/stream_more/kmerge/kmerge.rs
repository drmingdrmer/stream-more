use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use binary_heap_plus::BinaryHeap;
use binary_heap_plus::PeekMut;
use compare::Compare;
use futures::ready;
use futures::Stream;

use crate::stream_more::comparators::FnCmp;
use crate::stream_more::kmerge::heap_entry::HeapEntry;
use crate::stream_more::kmerge::heap_entry::HeapEntryCmp;
use crate::stream_more::peeked::Peeked;

/// A [`Stream`] that merges multiple streams in user specified order.
///
/// `HeapEntry` is stored in a max-bin-heap and the order is decided by `Peeked` and
/// `C`. `Peeked::No` is uninitialized state, and will be placed at top of the max-bin-heap, so that
/// all streams will be peeked before returning any item.
///
/// When all streams are initialized, i.e., `peeked.is_peeked() == true`, the order is decided
/// by `C(&D, &D)`.
///
/// To do a merge sort, the stream **should** output items in the same order as `C` decided,
/// otherwise the result is undefined.
///
/// Example:
/// ```
/// use futures::stream::iter;
/// use futures::executor::block_on;
/// use stream_more::comparators::Ascending;
/// # use futures::StreamExt;
/// # use stream_more::KMerge;
///
/// let m = KMerge::by_cmp(Ascending)
///                .merge(iter([1,3]))
///                .merge(iter([2,4]));
///
/// let got = block_on(m.collect::<Vec<u64>>());
/// assert_eq!(vec![1, 2, 3, 4], got);
/// ```
pub struct KMerge<'a, C, D>
where C: Compare<D>
{
    curr_id: u64,
    heap: BinaryHeap<HeapEntry<'a, D>, HeapEntryCmp<D, C>>,
}

impl<'a, F, D> KMerge<'a, FnCmp<F>, D>
where
    F: Fn(&D, &D) -> bool,
    FnCmp<F>: Compare<D>,
{
    /// Return an empty `Stream` adaptor `KMerge` that flattens `Stream`s by merging them according
    /// to the given closure `first()`.
    ///
    /// The closure `first()` is called with two elements `a`, `b` and should return `true` if `a`
    /// is ordered before `b`.
    ///
    /// If all base `Stream`s are sorted according to `first()`, the result is sorted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use futures::stream::iter;
    /// use futures::executor::block_on;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::KMerge;
    ///
    /// let m = KMerge::by(|a,b| a < b)
    ///             .merge(iter([1,3]))
    ///             .merge(iter([2,4]));
    /// let got = block_on(m.collect::<Vec<u64>>());
    /// assert_eq!(vec![1, 2, 3, 4], got);
    /// ```
    pub fn by(first: F) -> Self {
        Self::by_cmp(FnCmp(first))
    }
}

impl<'a, D, C> KMerge<'a, C, D>
where C: Compare<D>
{
    /// Merge streams by a comparator [`Compare`], if `Compare::compare(a,b)` returns
    /// `Ordering::Greater`, `a` will be chosen prior to `b`.
    ///
    /// # Example
    ///
    /// Sort merge two streams in ascending order:
    /// ```
    /// use futures::stream::iter;
    /// use futures::executor::block_on;
    /// use stream_more::comparators::Ascending;
    /// # use crate::stream_more::KMerge;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::StreamMore;
    ///
    /// let m = KMerge::by_cmp(Ascending).merge(iter([1,3])).merge(iter([2,4]));
    /// let got = block_on(m.collect::<Vec<u64>>());
    /// assert_eq!(vec![1, 2, 3, 4], got);
    /// ```
    pub fn by_cmp(cmp: C) -> Self {
        KMerge {
            curr_id: 0,
            heap: BinaryHeap::<HeapEntry<D>, _>::from_vec_cmp(vec![], HeapEntryCmp::new(cmp)),
        }
    }

    /// Append another stream to the merging stream.
    ///
    /// This method can be called any time after the stream is created.
    pub fn merge(mut self, stream: impl Stream<Item = D> + Send + 'a) -> Self {
        self.curr_id += 1;
        self.heap.push(HeapEntry::new(Box::pin(stream)).with_id(self.curr_id));
        self
    }
}

impl<'a, D, C> Stream for KMerge<'a, C, D>
where
    D: Unpin,
    C: Compare<D> + Unpin,
{
    type Item = D;

    /// ## Algorithm to merge streams by item order:
    ///
    /// 1. The first round is to peek the first item of each stream.
    ///
    ///    At first the `peeked` is `No` and will be placed at the top of the heap.
    ///    Once a stream is peeked, it will be moved to the tail of heap because
    ///    `Peeked::No > Peeked::Yes(T)`.
    ///    And continue the loop, examine the new heap head, until all streams are peeked.
    ///
    ///    So all streams will be peeked before returning any item.
    ///
    /// 2. Return the peeked item, poll the heap-head stream for the next item, then replace the
    ///    currently `peeked` of the head stream with the `next`, let BinaryHeap adjust the
    ///    order(if `next` is `Some`) or just remove the stream(if `next` is `None`).
    ///
    ///    Do this until all of them are exhausted.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let Some(mut peek_mut) = self.heap.peek_mut() else {
                return Poll::Ready(None);
            };

            if peek_mut.peeked.has_peeked() {
                return Poll::Ready(peek_mut.peeked.take());
            }

            let next = ready!(peek_mut.stream.as_mut().poll_next(cx));
            if let Some(t) = next {
                // Put it back to adjust the order in the heap.
                peek_mut.peeked = Peeked::Yes(t);
            } else {
                PeekMut::pop(peek_mut);
            }
        }
    }
}
