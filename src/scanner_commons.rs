/// Outcome of feeding input to a scanner.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ScanOutcome<T> {
    /// Input not relevant for the scanner.
    Unhandled,
    /// Input temporarily consumed by the scanner and final result depends on the next input.
    Pending,
    /// Input consumed by the scanner and result is clear now.
    Complete(T),
}
