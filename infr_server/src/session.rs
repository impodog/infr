use crate::prelude::*;

/// Stores a game session, with object layouts and other added information.
pub struct Session {
    pub layout: Layout,
    pub steps: usize,
}

/// This is used for countering z3 errors(z3 doesn't contain Rc). Layouts may only be used under Mutex.
unsafe impl Send for Session {}
