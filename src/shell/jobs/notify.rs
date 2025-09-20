/// In a full implementation, this module would:
/// - install a SIGCHLD handler
/// - reap children (waitpid(WNOHANG))
/// - update Job state and print notifications like:
///   [1]+  Stopped                 python
///
/// Stubbed for now.
