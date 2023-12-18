/// Short-circuits the surrounding function returning Result when there is a
/// problem creating a listener for the signal passed in.
/// Otherwise, unwraps and returns the underlying created signal.
#[macro_export]
macro_rules! short_curcuit_unix_signal_unwrap {
    ( $tokio_signal_kind:expr ) => {{
        match tokio::signal::unix::signal($tokio_signal_kind) {
            Err(e) => return Err(e),
            Ok(s) => s,
        }
    }};
}

/// registers a list of signals to listen to and runs the given expression
/// The run_expr must return a Result of some sort
#[macro_export]
macro_rules! receive_signal_and_run_expr {
    ( $run_expr:expr, $($signal_literals:literal => $signals_to_receive:expr) *) => {
        tokio::select! {
            $(
                _ = $signals_to_receive.recv() => {
                    tracing::info!("Received signal: {}", $signal_literals);
                    match $run_expr {
                        Err(e) => {
                            return Err(::std::io::Error::new(::std::io::ErrorKind::Other, e));
                        }
                        Ok(r) => {
                            tracing::info!("Cleanup task result: {r:?}")
                        }
                    }
                }
            )*
        }
    };
}
