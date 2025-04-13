//! Mock context for testing the bonding contract

use alkanes_support::context::Context;
use std::cell::RefCell;

// Thread-local storage for mock context
thread_local! {
    pub static MOCK_CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

/// Set the mock context
pub fn set_mock_context(context: Context) {
    MOCK_CONTEXT.with(|c| {
        *c.borrow_mut() = Some(context);
    });
}

/// Get the mock context
pub fn get_mock_context() -> Option<Context> {
    MOCK_CONTEXT.with(|c| {
        c.borrow().clone()
    })
}

/// Clear the mock context
pub fn clear_mock_context() {
    MOCK_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });
}
