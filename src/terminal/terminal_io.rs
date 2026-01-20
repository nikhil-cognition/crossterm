use std::io::{self, Write};

use parking_lot::Mutex;

#[cfg(unix)]
pub use crate::terminal::sys::file_descriptor::FileDesc;

#[cfg(unix)]
static TERMINAL_INPUT: Mutex<Option<FileDesc<'static>>> = Mutex::new(None);
static TERMINAL_OUTPUT: Mutex<Option<Box<dyn Write + Send>>> = Mutex::new(None);

pub struct TerminalIo {
    #[cfg(unix)]
    pub input: FileDesc<'static>,
    pub output: Box<dyn Write + Send>,
}

impl TerminalIo {
    #[cfg(unix)]
    pub fn new(input: FileDesc<'static>, output: Box<dyn Write + Send>) -> Self {
        Self { input, output }
    }
}

pub fn set_terminal_io(io: TerminalIo) {
    #[cfg(unix)]
    {
        let mut guard = TERMINAL_INPUT.lock();
        assert!(guard.is_none(), "set_terminal_io called but terminal input is already set");
        *guard = Some(io.input);
    }
    let mut guard = TERMINAL_OUTPUT.lock();
    assert!(guard.is_none(), "set_terminal_io called but terminal output is already set");
    *guard = Some(io.output);
}

pub fn clear_terminal_io() {
    #[cfg(unix)]
    {
        let mut guard = TERMINAL_INPUT.lock();
        *guard = None;
    }
    let mut guard = TERMINAL_OUTPUT.lock();
    *guard = None;
}

#[cfg(unix)]
pub(crate) fn try_get_input_fd() -> Option<parking_lot::MappedMutexGuard<'static, FileDesc<'static>>> {
    let guard = TERMINAL_INPUT.lock();
    if guard.is_some() {
        Some(parking_lot::MutexGuard::map(guard, |opt| {
            opt.as_mut().unwrap()
        }))
    } else {
        None
    }
}

/// A writer that delegates to the terminal output.
///
/// This is a zero-sized type that acquires the lock on each write/flush operation,
/// avoiding issues with lock guards being held across macro expansions.
#[derive(Clone, Copy)]
pub struct TerminalOutput;

impl Write for TerminalOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = TERMINAL_OUTPUT.lock();
        match guard.as_mut() {
            Some(output) => output.write(buf),
            None => {
                drop(guard);
                io::stdout().write(buf)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut guard = TERMINAL_OUTPUT.lock();
        match guard.as_mut() {
            Some(output) => output.flush(),
            None => {
                drop(guard);
                io::stdout().flush()
            }
        }
    }
}

/// Returns a writer that delegates to the configured terminal output.
///
/// If a custom terminal I/O has been set via [`set_terminal_io`], writes will
/// go to that output. Otherwise, writes go to stdout.
///
/// This function returns a zero-sized type that acquires the lock on each
/// write/flush operation, making it safe to use in macros that may evaluate
/// the writer expression multiple times.
pub fn terminal_output() -> TerminalOutput {
    TerminalOutput
}
