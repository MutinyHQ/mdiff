use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::Write;
use tokio::sync::mpsc;

/// Events emitted by the PTY runner.
#[derive(Debug)]
pub enum PtyEvent {
    /// Raw output bytes from the PTY.
    Output(usize, Vec<u8>),
    /// Process exited with the given code.
    Done(usize, i32),
}

/// Manages a PTY-based agent subprocess.
pub struct PtyRunner {
    event_rx: mpsc::UnboundedReceiver<PtyEvent>,
    master_write: Box<dyn Write + Send>,
    master_pty: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
}

impl PtyRunner {
    /// Spawn an agent subprocess in a PTY. Returns a PtyRunner that can be
    /// polled for output and written to for interactive input.
    pub fn spawn(run_id: usize, command: &str, rows: u16, cols: u16) -> Self {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("failed to open PTY");

        let mut cmd = CommandBuilder::new("sh");
        cmd.arg("-c");
        cmd.arg(command);

        let child = pair.slave.spawn_command(cmd).expect("failed to spawn");
        // Drop the slave side - the child owns it now.
        drop(pair.slave);

        let master_write = pair.master.take_writer().expect("failed to get PTY writer");
        let reader = pair
            .master
            .try_clone_reader()
            .expect("failed to clone PTY reader");

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Read loop in a blocking thread (portable-pty reads are synchronous).
        tokio::task::spawn_blocking(move || {
            use std::io::Read;
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if event_tx
                            .send(PtyEvent::Output(run_id, buf[..n].to_vec()))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            event_rx,
            master_write,
            master_pty: pair.master,
            child,
        }
    }

    /// Non-blocking poll for events.
    pub fn try_recv(&mut self) -> Option<PtyEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Write raw input bytes to the PTY.
    pub fn write_input(&mut self, bytes: &[u8]) {
        let _ = self.master_write.write_all(bytes);
        let _ = self.master_write.flush();
    }

    /// Resize the PTY.
    pub fn resize(&self, rows: u16, cols: u16) {
        let _ = self.master_pty.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }

    /// Kill the child process.
    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }

    /// Check if the child has exited. Returns Some(exit_code) if done.
    pub fn try_wait(&mut self) -> Option<i32> {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                // portable-pty ExitStatus doesn't directly expose code on all
                // platforms, but success() is reliable.
                Some(if status.success() { 0 } else { 1 })
            }
            _ => None,
        }
    }
}

/// Convert a crossterm KeyEvent to the bytes that a terminal would send.
pub fn key_event_to_bytes(key: &KeyEvent) -> Vec<u8> {
    // Handle Ctrl+letter first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            let byte = (c.to_ascii_lowercase() as u8)
                .wrapping_sub(b'a')
                .wrapping_add(1);
            return vec![byte];
        }
    }

    match key.code {
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            s.as_bytes().to_vec()
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::F(n) => match n {
            1 => b"\x1bOP".to_vec(),
            2 => b"\x1bOQ".to_vec(),
            3 => b"\x1bOR".to_vec(),
            4 => b"\x1bOS".to_vec(),
            5 => b"\x1b[15~".to_vec(),
            6 => b"\x1b[17~".to_vec(),
            7 => b"\x1b[18~".to_vec(),
            8 => b"\x1b[19~".to_vec(),
            9 => b"\x1b[20~".to_vec(),
            10 => b"\x1b[21~".to_vec(),
            11 => b"\x1b[23~".to_vec(),
            12 => b"\x1b[24~".to_vec(),
            _ => vec![],
        },
        _ => vec![],
    }
}
