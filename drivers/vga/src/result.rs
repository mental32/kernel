/// An enum of failure reasons.
pub enum FailureReason {
    /// An access was attempted out of the bounds of the writers buffer.
    OutOfBounds((usize, usize)),
}

/// A helper type alias that is a partial core::result::Result
/// over crate::FailureReason.
pub type Result<T> = core::result::Result<T, FailureReason>;
