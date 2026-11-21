//! 按墙钟打进度行：收消息时每过一整段间隔就该打一行。

use super::*;

/// 长跑的枚举与核对按挂钟至少隔这么久打一行进度（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」的进度间隔 5 分钟）：
/// 一片、一块跑得比它久时，等着收的那条线程照样隔这么久打一行，不至于几个小时零输出。
pub const PROGRESS_LINE_INTERVAL_IN_SECONDS: u64 = 300;

/// 上一次收到消息、或上一次到期打按挂钟那一行是在什么时候（从这一趟开跑算起），离下一行按挂钟的到期还要等多久。
/// 时刻由调用方的时钟给，用例换成假的时钟。收到消息记的是收到的时刻，不是调用方真打出它那一行的时刻：调用方收到一条就打一行时两者只差
/// 打那一行的工夫；收到不打的（崩溃放量 CPU 块那一支，行在核对线程发消息之前打）差一个排队的时间，下一行按挂钟的最多晚这么一截。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WallClockProgressLines {
    interval: std::time::Duration,
    last_message_or_wall_clock_line_at: std::time::Duration,
}

impl WallClockProgressLines {
    #[must_use]
    pub fn new(interval: std::time::Duration, started_at: std::time::Duration) -> Self {
        Self {
            interval,
            last_message_or_wall_clock_line_at: started_at,
        }
    }

    /// 间隔取 [`PROGRESS_LINE_INTERVAL_IN_SECONDS`]。
    #[must_use]
    pub fn every_progress_line_interval(started_at: std::time::Duration) -> Self {
        Self::new(
            std::time::Duration::from_secs(PROGRESS_LINE_INTERVAL_IN_SECONDS),
            started_at,
        )
    }

    /// 离下一行到期还剩多久；已经到期是 0。
    #[must_use]
    pub fn wait_before_the_next_line_is_due(
        &self,
        now: std::time::Duration,
    ) -> std::time::Duration {
        self.interval
            .saturating_sub(now.saturating_sub(self.last_message_or_wall_clock_line_at))
    }

    /// 上一次收到消息或打按挂钟那一行之后，时钟走过了整个间隔。
    #[must_use]
    pub fn wall_clock_line_is_due(&self, now: std::time::Duration) -> bool {
        now.saturating_sub(self.last_message_or_wall_clock_line_at) >= self.interval
    }

    /// 记下 `now`：刚收到一条消息，或刚到期打了一行按挂钟的；下一行按挂钟的从这一刻起再等一个间隔。
    pub fn restart_the_interval_at_a_received_message_or_a_wall_clock_line(
        &mut self,
        now: std::time::Duration,
    ) {
        self.last_message_or_wall_clock_line_at = now;
    }
}

/// [`receive_until_the_senders_are_gone_with_wall_clock_lines`] 交给调用方的一件事：收到一条消息（调用方照旧按片、按块打它那一行），
/// 或者上一行之后过了整个间隔还没有新消息（调用方打一行按挂钟的进度）。
pub enum ReceivedOrWallClockLineDue<Message> {
    Received(Message),
    WallClockLineDue { elapsed: std::time::Duration },
}

/// 一直收到发送端全部丢掉为止：每收到一条交给 `handle`（调用方照它的约定按片、按块打一行或不打），记下收到的时刻；等的时候最多等到下一行到期，
/// 到期还没收到就交一件 [`ReceivedOrWallClockLineDue::WallClockLineDue`]，让调用方打一行按挂钟的进度。`clock` 交回从这一趟开跑算起的时刻。
/// `handle` panic 时原样往外传（接收端随调用方的栈一起丢掉，发送的线程下一次发不出去就退）。
pub fn receive_until_the_senders_are_gone_with_wall_clock_lines<Message>(
    receiver: &mpsc::Receiver<Message>,
    progress: &mut WallClockProgressLines,
    clock: &dyn Fn() -> std::time::Duration,
    handle: &mut dyn FnMut(ReceivedOrWallClockLineDue<Message>),
) {
    loop {
        let wait = progress.wait_before_the_next_line_is_due(clock());
        match receiver.recv_timeout(wait) {
            Ok(message) => {
                handle(ReceivedOrWallClockLineDue::Received(message));
                progress.restart_the_interval_at_a_received_message_or_a_wall_clock_line(clock());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let now = clock();
                if progress.wall_clock_line_is_due(now) {
                    handle(ReceivedOrWallClockLineDue::WallClockLineDue { elapsed: now });
                    progress.restart_the_interval_at_a_received_message_or_a_wall_clock_line(now);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

#[cfg(test)]
mod wall_clock_progress_line_tests {
    use super::{
        receive_until_the_senders_are_gone_with_wall_clock_lines, ReceivedOrWallClockLineDue,
        WallClockProgressLines,
    };
    use std::cell::{Cell, RefCell};
    use std::sync::mpsc;
    use std::time::Duration;

    const INTERVAL: Duration = Duration::from_secs(300);
    /// 假时钟每读一次走这么久：比间隔长，每一次等都已经到期，不真等。
    const CLOCK_STEP_LONGER_THAN_THE_INTERVAL: Duration = Duration::from_secs(301);
    /// 假时钟读到这么多次就丢掉发送端，接收循环随之收尾（间隔判断坏了也收得了尾，用例红而不是卡住）。
    const CLOCK_READS_BEFORE_THE_SENDER_IS_DROPPED: u32 = 12;

    /// 一条消息都没来、假时钟每读一次走过一个间隔以上：每一次等完都要交一行按挂钟的进度，相邻两行之间假时钟走过的不超过两次读数。
    #[test]
    fn wall_clock_line_is_due_whenever_a_whole_interval_passes_without_a_message() {
        let (sender, receiver) = mpsc::channel::<u64>();
        let sender_still_open = RefCell::new(Some(sender));
        let clock_reads = Cell::new(0u32);
        let clock = || {
            let reads = clock_reads.get() + 1;
            clock_reads.set(reads);
            if reads >= CLOCK_READS_BEFORE_THE_SENDER_IS_DROPPED {
                sender_still_open.borrow_mut().take();
            }
            CLOCK_STEP_LONGER_THAN_THE_INTERVAL * reads
        };
        let mut progress = WallClockProgressLines::new(INTERVAL, Duration::ZERO);
        let mut lines_at: Vec<Duration> = Vec::new();
        receive_until_the_senders_are_gone_with_wall_clock_lines(
            &receiver,
            &mut progress,
            &clock,
            &mut |event| match event {
                ReceivedOrWallClockLineDue::Received(message) => {
                    panic!("没有发过消息，却收到了 {message}")
                }
                ReceivedOrWallClockLineDue::WallClockLineDue { elapsed } => lines_at.push(elapsed),
            },
        );
        assert!(
            lines_at.len() >= 5,
            "假时钟走过 {} 次间隔以上，按挂钟的进度只交了 {} 行：{lines_at:?}",
            CLOCK_READS_BEFORE_THE_SENDER_IS_DROPPED,
            lines_at.len()
        );
        let mut previous = Duration::ZERO;
        for line_at in &lines_at {
            assert!(
                line_at.saturating_sub(previous) <= CLOCK_STEP_LONGER_THAN_THE_INTERVAL * 2,
                "两行之间假时钟走了 {:?}，过了间隔却没有一行：{lines_at:?}",
                line_at.saturating_sub(previous)
            );
            previous = *line_at;
        }
    }

    /// 消息接连到、时钟不走：每条消息交给调用方（它按片、按块打行），不多交一行按挂钟的进度。
    #[test]
    fn no_wall_clock_line_is_due_while_messages_arrive_within_the_interval() {
        let (sender, receiver) = mpsc::channel::<u64>();
        for message in 0..3 {
            sender.send(message).expect("接收端还在");
        }
        drop(sender);
        let clock = || Duration::from_secs(5);
        let mut progress = WallClockProgressLines::new(INTERVAL, Duration::ZERO);
        let mut received = Vec::new();
        let mut wall_clock_lines = 0usize;
        receive_until_the_senders_are_gone_with_wall_clock_lines(
            &receiver,
            &mut progress,
            &clock,
            &mut |event| match event {
                ReceivedOrWallClockLineDue::Received(message) => received.push(message),
                ReceivedOrWallClockLineDue::WallClockLineDue { .. } => wall_clock_lines += 1,
            },
        );
        assert_eq!(received, vec![0, 1, 2], "每条消息按次序交给调用方");
        assert_eq!(wall_clock_lines, 0, "间隔没过，不该有按挂钟的行");
    }
}
