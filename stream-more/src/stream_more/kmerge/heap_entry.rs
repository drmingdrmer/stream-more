use std::cmp::Ordering;
use std::marker::PhantomData;

use compare::Compare;
use futures::stream::BoxStream;

use crate::stream_more::peeked::Peeked;

/// An entry to store a stream and its peeked value which is stored in a **MAX** [`BinaryHeap`].
///
/// The order of [`HeapEntry`] is decided only by [`Peeked`].
///
/// [`BinaryHeap`]: `binary_heap_plus::BinaryHeap`
pub(crate) struct HeapEntry<'a, D> {
    /// Contains the peeked item of the stream.
    pub(crate) peeked: Peeked<D>,

    /// The producing stream
    pub(crate) stream: BoxStream<'a, D>,

    /// Id for debug, starts from 1 in a [`KMerge`].
    ///
    /// [`KMerge`]: crate::stream_more::kmerge::KMerge
    pub(crate) id: String,
}

impl<'a, D> HeapEntry<'a, D> {
    pub fn new(stream: BoxStream<'a, D>) -> Self {
        Self {
            peeked: Peeked::No,
            stream,
            id: "".to_string(),
        }
    }

    pub fn with_id(mut self, id: impl ToString) -> Self {
        self.id = id.to_string();
        self
    }
}

/// A [`Compare`] implementation that compares [`Peeked`] values and its containing struct
/// [`HeapEntry`].
///
/// It always put `Peeked::No` before `Peeked::Yes(_)`, so that the heap will always pop
/// uninitialized values first.
///
/// If two values are both `Peeked::Yes(_)`, then it will compare the inner values by the
/// `Compare<D>` implementation.
pub struct HeapEntryCmp<D, C: Compare<D>> {
    pub cmp: C,
    _p: PhantomData<D>,
}

impl<D, C: Compare<D>> HeapEntryCmp<D, C> {
    pub fn new(cmp: C) -> Self {
        Self { cmp, _p: PhantomData }
    }
}

impl<D, C: Compare<D>> Compare<Peeked<D>> for HeapEntryCmp<D, C> {
    fn compare(&self, l: &Peeked<D>, r: &Peeked<D>) -> Ordering {
        match (l, &r) {
            (Peeked::No, Peeked::No) => Ordering::Equal,
            (Peeked::No, Peeked::Yes(_)) => Ordering::Greater,
            (Peeked::Yes(_), Peeked::No) => Ordering::Less,
            (Peeked::Yes(a), Peeked::Yes(b)) => self.cmp.compare(a, b),
        }
    }
}

impl<'a, D, C: Compare<D>> Compare<HeapEntry<'a, D>> for HeapEntryCmp<D, C> {
    fn compare(&self, l: &HeapEntry<D>, r: &HeapEntry<D>) -> Ordering {
        self.compare(&l.peeked, &r.peeked)
    }
}
