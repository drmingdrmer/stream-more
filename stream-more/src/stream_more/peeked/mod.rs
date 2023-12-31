#[cfg(test)] mod peeked_test;

/// Contains a stream and the peeked next item of the stream.
pub enum Peeked<D = ()> {
    /// Not yet peeked.
    No,

    /// Peeked some value.
    Yes(D),
}

impl<D> Peeked<D> {
    /// Return `true` if it has been filled with a peeked value.
    pub fn has_peeked(&self) -> bool {
        match &self {
            Peeked::Yes(_) => true,
            Peeked::No => false,
        }
    }

    /// Take the peeked value, and reset it to [`Peeked::No`] state.
    pub fn take(&mut self) -> Option<D> {
        match std::mem::replace(self, Peeked::No) {
            Peeked::Yes(v) => Some(v),
            Peeked::No => None,
        }
    }
}
