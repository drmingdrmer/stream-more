use comparators::Ascending;
use comparators::Descending;
use comparators::FnCmp;
use compare::Compare;
use futures::Stream;

use crate::stream_more::coalesce::Coalesce;
use crate::stream_more::kmerge::KMerge;

pub mod coalesce;
pub mod comparators;
pub mod kmerge;
pub mod peeked;

/// Provide more methods for [`Stream`].
pub trait StreamMore: Stream {
    /// Create a k-way merge `Stream` that flattens `Stream`s by merging them according
    /// to the given closure `first()`.
    ///
    /// The closure `first()` is called with two elements `a`, `b` and should return `true` if `a`
    /// is ordered before `b`.
    ///
    /// If all base `Stream`s are sorted according to `first()`, the result is sorted.
    ///
    /// # Example
    ///
    /// Sort merge two streams in ascending order:
    /// ```
    /// use futures::stream::iter;
    /// use futures::executor::block_on;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::StreamMore;
    ///
    /// let m = iter([1,3]).kmerge_by(|a,b| a < b).merge(iter([2,4]));
    /// let got = block_on(m.collect::<Vec<u64>>());
    /// assert_eq!(vec![1, 2, 3, 4], got);
    /// ```
    fn kmerge_by<'a, F>(self, first: F) -> KMerge<'a, FnCmp<F>, Self::Item>
    where
        Self: Sized + Send + 'a,
        F: Fn(&Self::Item, &Self::Item) -> bool,
    {
        KMerge::by(first).merge(self)
    }

    /// Create a k-way merge stream, which chooses the item from the streams by a comparator
    /// [`Compare`].
    ///
    /// If `comparator::compare(a,b)` returns [`Ordering::Greater`], `a` will be chosen first over
    /// `b`, where `a` and `b` are next item from different streams.
    ///
    /// # Example
    ///
    /// Sort merge two streams in ascending order:
    /// ```
    /// use futures::stream::iter;
    /// use futures::executor::block_on;
    /// use stream_more::comparators::Ascending;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::StreamMore;
    ///
    /// let m = iter([1,3]).kmerge_by_cmp(Ascending).merge(iter([2,4]));
    /// let got = block_on(m.collect::<Vec<u64>>());
    /// assert_eq!(vec![1, 2, 3, 4], got);
    /// ```
    ///
    /// [`Ordering::Greater`]: `std::cmp::Ordering`
    fn kmerge_by_cmp<'a, C>(self, cmp: C) -> KMerge<'a, C, Self::Item>
    where
        Self: Sized + Send + 'a,
        C: Compare<Self::Item>,
    {
        KMerge::by_cmp(cmp).merge(self)
    }

    /// Convert this stream to a [`KMerge`] streams which merge streams by choosing the maximum
    /// item from the streams, behaving like a max-heap.
    ///
    /// # Example
    ///
    /// ```
    /// use futures::stream::iter;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::StreamMore;
    /// # futures::executor::block_on(async {
    /// let m = iter([3,1]).kmerge_max().merge(iter([4,2])).merge(iter([5]));
    /// let got = m.collect::<Vec<u64>>().await;
    /// assert_eq!(vec![5, 4, 3, 2, 1], got);
    /// # });
    /// ```
    fn kmerge_max<'a>(self) -> KMerge<'a, Descending, Self::Item>
    where
        Self: Sized + Send + 'a,
        Self::Item: Ord,
    {
        KMerge::max().merge(self)
    }

    /// Convert this stream to a [`KMerge`] streams which merge streams by choosing the minimum
    /// item from the streams, behaving like a min-heap.
    ///
    /// # Example
    ///
    /// ```
    /// use futures::stream::iter;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::StreamMore;
    /// # futures::executor::block_on(async {
    /// let m = iter([3,1]).kmerge_min().merge(iter([4,2]));
    /// let got = m.collect::<Vec<u64>>().await;
    /// assert_eq!(vec![3,1,4,2], got);
    /// # });
    /// ```
    fn kmerge_min<'a>(self) -> KMerge<'a, Ascending, Self::Item>
    where
        Self: Sized + Send + 'a,
        Self::Item: Ord,
    {
        KMerge::min().merge(self)
    }

    /// Return a stream adaptor that uses the passed-in closure to optionally merge together
    /// consecutive items.
    ///
    /// The closure `f` is passed two items `previous` and `current` and may return either:
    /// - (1) `Ok(combined)` to merge the two values or
    /// - (2) `Err((previous, current))` to indicate they can’t be merged.
    ///
    /// In (2), the value `previous` is emitted.
    /// Either (1) `combined` or (2) `current` becomes the previous value when coalesce continues
    /// with the next pair of items to merge. The value that remains at the end is also
    /// emitted.
    ///
    /// The stream item type is `Self::Item`.
    ///
    /// ```
    /// use futures::stream::iter;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::StreamMore;
    /// # futures::executor::block_on(async {
    ///
    /// // sum same-sign runs together
    /// let got = iter(vec![-1, -2, -3, 3, 1, 0, -1])
    ///             .coalesce(|x, y|
    ///                 if x * y >= 0 {
    ///                     Ok(x + y)
    ///                 } else {
    ///                     Err((x, y))
    ///                 })
    ///             .collect::<Vec<_>>().await;
    /// assert_eq!(vec![-6, 4, -1], got);
    /// # });
    /// ```
    fn coalesce<'a, F>(self, f: F) -> Coalesce<'a, Self::Item, F>
    where
        Self: Sized + Send + 'a,
        F: FnMut(Self::Item, Self::Item) -> Result<Self::Item, (Self::Item, Self::Item)>,
    {
        Coalesce::new(Box::pin(self), f)
    }
}

impl<T: ?Sized> StreamMore for T where T: Stream {}
