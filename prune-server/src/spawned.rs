//! 起子进程（自证起服务与压测器、压测器起服务）：标准输出逐行收进通道，按行首等就绪行；
//! 子进程设了父线程退出时收 SIGKILL，起它的一方走了它不会留下；放手（drop）时还活着的照进程号 SIGKILL 再收尸。

use std::io::{BufRead, BufReader, Read};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 子进程的标准错误：接到起它的一方的标准错误上，或收下来给自证判。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StandardError {
    Inherit,
    Capture,
}

pub struct SpawnedProcess {
    child: Child,
    lines: Receiver<String>,
    seen_lines: Vec<String>,
    captured_standard_error: Arc<Mutex<String>>,
    standard_error_reader: Option<thread::JoinHandle<()>>,
}

impl SpawnedProcess {
    pub fn spawn(
        mut command: Command,
        standard_error: StandardError,
    ) -> Result<SpawnedProcess, String> {
        command.stdin(Stdio::null()).stdout(Stdio::piped());
        match standard_error {
            StandardError::Inherit => command.stderr(Stdio::inherit()),
            StandardError::Capture => command.stderr(Stdio::piped()),
        };
        // SAFETY: pre_exec 的闭包在 fork 之后、exec 之前的子进程里跑，只调 prctl 这一个异步信号安全的系统调用、不分配内存、不碰锁。
        unsafe {
            command.pre_exec(|| {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("起不了子进程 {command:?}：{error}"))?;
        let standard_output = child
            .stdout
            .take()
            .expect("上面设了 Stdio::piped()，标准输出在");
        let (line_sender, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(standard_output).lines() {
                let Ok(line) = line else {
                    return;
                };
                if line_sender.send(line).is_err() {
                    return;
                }
            }
        });
        let captured_standard_error = Arc::new(Mutex::new(String::new()));
        let standard_error_reader = child.stderr.take().map(|mut standard_error_stream| {
            let sink = Arc::clone(&captured_standard_error);
            thread::spawn(move || {
                // 边读边存：子进程还在跑时也能拿到已经写出的部分；读出错就停，读到的留下给判。
                let mut chunk = [0_u8; 4096];
                while let Ok(read_now) = standard_error_stream.read(&mut chunk) {
                    if read_now == 0 {
                        return;
                    }
                    sink.lock()
                        .expect("标准错误缓冲只在读线程追加与 standard_error_text 里短暂持有")
                        .push_str(&String::from_utf8_lossy(&chunk[..read_now]));
                }
            })
        });
        Ok(SpawnedProcess {
            child,
            lines,
            seen_lines: Vec::new(),
            captured_standard_error,
            standard_error_reader,
        })
    }

    pub fn process_identifier(&self) -> u32 {
        self.child.id()
    }

    /// 等一行以 `prefix` 起头的标准输出；子进程先退出或到点还没等到，回错并带上已经看到的行。
    pub fn wait_for_line_starting_with(
        &mut self,
        prefix: &str,
        timeout: Duration,
    ) -> Result<String, String> {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(remaining) {
                Ok(line) => {
                    self.seen_lines.push(line.clone());
                    if line.starts_with(prefix) {
                        return Ok(line);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    return Err(format!(
                        "{} 秒内没等到以 {prefix} 起头的一行；已看到：{:?}",
                        timeout.as_secs(),
                        self.seen_lines
                    ));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(format!(
                        "子进程没打出以 {prefix} 起头的一行就关了标准输出；已看到：{:?}",
                        self.seen_lines
                    ));
                }
            }
        }
    }

    /// 等子进程退出，到点还没退回 `None`。
    pub fn wait_for_exit(&mut self, timeout: Duration) -> Option<ExitStatus> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Some(status);
            }
            if Instant::now() >= deadline {
                return None;
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    /// 读到标准输出的流尾、收齐每一行，再等子进程退出；不设限时（自证给每格另起的进程用，限时在各格自己里头）。
    pub fn wait_for_exit_with_all_lines(&mut self) -> Result<(ExitStatus, Vec<String>), String> {
        self.seen_lines.extend(self.lines.iter());
        let status = self
            .child
            .wait()
            .map_err(|error| format!("等子进程 {} 退出：{error}", self.child.id()))?;
        Ok((status, self.seen_lines.clone()))
    }

    /// 子进程退出之后把剩下的标准输出行都收齐（读线程到流尾才关通道）。
    pub fn remaining_lines(&mut self, timeout: Duration) -> Vec<String> {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(remaining) {
                Ok(line) => self.seen_lines.push(line),
                Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                    return self.seen_lines.clone()
                }
            }
        }
    }

    /// 收下来的标准错误；子进程已经退出的，先等读线程读到流尾，拿到的是全文。
    pub fn standard_error_text(&mut self) -> String {
        if let Ok(Some(_)) = self.child.try_wait() {
            if let Some(reader) = self.standard_error_reader.take() {
                // 读线程只读与追加，不会 panic；join 只为等它读到流尾。
                let _ = reader.join();
            }
        }
        self.captured_standard_error
            .lock()
            .expect("标准错误缓冲只在读线程收尾与这里短暂持有")
            .clone()
    }

    /// 照进程号发一个信号给这个子进程（只对自己起的这一个）。
    pub fn send_signal(&self, signal: libc::c_int) -> Result<(), String> {
        let process_identifier = libc::pid_t::try_from(self.child.id())
            .map_err(|_| "子进程号放不进 pid_t".to_string())?;
        // SAFETY: kill 只收两个整数；目标是这个进程自己起、还没收尸的子进程，进程号不会被别的进程复用。
        let result = unsafe { libc::kill(process_identifier, signal) };
        if result != 0 {
            return Err(format!(
                "发信号 {signal} 给 {process_identifier} 失败：{}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
}

impl Drop for SpawnedProcess {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            // 已经退出的再 kill 回错，不要紧；随后的 wait 收尸。
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
