//! A local test listener on a port that no earlier listener of this test
//! process used.
//!
//! HitIt keeps process-wide request stores (`request_reuse::shared()`). A
//! store key holds the endpoint of the request, for example
//! `http://127.0.0.1:41234`. A stored response stays for some minutes. When
//! a test stops its server, the system can give the same port to a
//! subsequent test. That test then gets the stored response of the earlier
//! test, and it counts fewer requests. Each test listener comes from this
//! function, so no two test servers of one process have the same endpoint.

use std::collections::HashSet;
use std::io;
use std::net::TcpListener;
use std::sync::{Mutex, PoisonError};

static GIVEN_PORTS: Mutex<Option<HashSet<u16>>> = Mutex::new(None);

/// Binds `127.0.0.1:0` until the system gives a port that this function did
/// not give before in this process.
///
/// A listener with a port that was given before is closed, and the function
/// binds again. The function keeps all such listeners open until it finds a
/// new port, so the system cannot give the same old port again in the loop.
///
/// # Errors
///
/// Returns the error of [`TcpListener::bind`].
pub(crate) fn fresh_local_listener() -> io::Result<TcpListener> {
    let mut given = GIVEN_PORTS.lock().unwrap_or_else(PoisonError::into_inner);
    let given = given.get_or_insert_with(HashSet::new);
    let mut held = Vec::new();
    loop {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        if given.insert(port) {
            return Ok(listener);
        }
        held.push(listener);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The helper gives a different port each time, also after the earlier
    /// listener is closed.
    #[test]
    fn fresh_local_listener_never_gives_a_port_again() {
        let mut ports = HashSet::new();
        for _ in 0..200 {
            let listener = fresh_local_listener().expect("bind a local port");
            let port = listener.local_addr().expect("local address").port();
            assert!(ports.insert(port), "port {port} was given two times");
            drop(listener);
        }
    }
}
