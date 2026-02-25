//! Task Control Block (TCB) definitions.

/// Minimal Task Control Block placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskControlBlock {
    /// Unique task identifier.
    pub tid: u64,
}
