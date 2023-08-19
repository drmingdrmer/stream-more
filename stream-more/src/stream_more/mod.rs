use comparators::Ascending;
use comparators::Descending;
use comparators::FnCmp;
use compare::Compare;
use futures::Stream;

use crate::stream_more::kmerge::KMerge;

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

    /// Convert this stream to a [`KMerge`] streams which merge streams by choosing the smallest
    /// item from the streams.
    ///
    /// # Example
    ///
    /// ```
    /// use futures::stream::iter;
    /// use futures::executor::block_on;
    /// use stream_more::comparators::Descending;
    /// # use futures::StreamExt;
    /// # use crate::stream_more::StreamMore;
    ///
    /// let m = iter([3,1]).kmerge_max().merge(iter([4,2])).merge(iter([5]));
    /// let got = block_on(m.collect::<Vec<u64>>());
    /// assert_eq!(vec![5, 4, 3, 2, 1], got);
    /// ```
    fn kmerge_max<'a>(self) -> KMerge<'a, Descending, Self::Item>
    where
        Self: Sized + Send + 'a,
        Self::Item: Ord,
    {
        KMerge::by_cmp(Descending).merge(self)
    }

    /// Convert this stream to a [`KMerge`] streams which merge streams by choosing the smallest
    /// item from the streams.
    fn kmerge_min<'a>(self) -> KMerge<'a, Ascending, Self::Item>
    where
        Self: Sized + Send + 'a,
        Self::Item: Ord,
    {
        KMerge::by_cmp(Ascending).merge(self)
    }
}

impl<T: ?Sized> StreamMore for T where T: Stream {}
