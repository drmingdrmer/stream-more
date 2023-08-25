//! `stream-more` is a library for working with streams.

mod stream_more;

pub use compare::Compare;

pub use crate::stream_more::coalesce::Coalesce;
pub use crate::stream_more::comparators;
pub use crate::stream_more::kmerge::KMerge;
pub use crate::stream_more::StreamMore;
