//! Mock context module for testing

use alkanes_support::context::Context;
use std::cell::RefCell;
use crate::reset_mock_environment;

// Import the thread_local mock context
thread_local! {
    static MOCK_CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

// Helper function to get the mock context
pub fn get_mock_context() -> Option<Context> {
    // Check if we're in a test environment
    if reset_mock_environment::is_test_environment() {
        MOCK_CONTEXT.with(|c| c.borrow().clone())
    } else {
        None
    }
}

// Helper function to set the mock context
pub fn set_mock_context(context: Context) {
    MOCK_CONTEXT.with(|c| {
        // First clear any existing context to ensure proper cleanup
        if c.borrow().is_some() {
            *c.borrow_mut() = None;
        }
        
        // Then set the new context
        *c.borrow_mut() = Some(context);
    });
}

// Alias for set_mock_context for compatibility with existing code
pub fn set_context(context: Context) {
    set_mock_context(context);
}

// Helper function to clear the mock context
pub fn clear_mock_context() {
    MOCK_CONTEXT.with(|c| {
        // Explicitly drop the current context if it exists
        if c.borrow().is_some() {
            let mut context_ref = c.borrow_mut();
            // Take ownership of the context and drop it
            let _ = context_ref.take();
        }
    });
}
