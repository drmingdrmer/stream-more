//! [`Compare`] implementations

use std::cmp::Ordering;

use compare::Compare;

/// Sort merge in descending order
pub struct Descending;

impl<T> Compare<T> for Descending
where T: Ord
{
    fn compare(&self, l: &T, r: &T) -> Ordering {
        l.cmp(r)
    }
}

/// Sort merge in ascending order
pub struct Ascending;

impl<T> Compare<T> for Ascending
where T: Ord
{
    fn compare(&self, l: &T, r: &T) -> Ordering {
        r.cmp(l)
    }
}

/// A wrapper of choosing function `Fn(&D, &D) -> bool` to implement `Compare<D>`.
pub struct FnCmp<F>(pub F);

impl<D, F> Compare<D> for FnCmp<F>
where F: Fn(&D, &D) -> bool
{
    fn compare(&self, l: &D, r: &D) -> Ordering {
        if self.0(l, r) {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}
