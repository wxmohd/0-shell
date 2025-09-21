// SIGCHLD-based async reaping would live here.
// For now we rely on exec::maybe_reap() on Unix and do nothing on non-Unix.
