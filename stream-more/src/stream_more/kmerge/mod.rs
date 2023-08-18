pub(crate) mod heap_entry;
#[allow(clippy::module_inception)] mod kmerge;
pub use kmerge::KMerge;

#[cfg(test)] mod kmerge_tests;
