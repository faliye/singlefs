//! 在卡上做一段活、在它交回的那一刻记下设备丢没丢，与同一个设备上一起开的几段活各自报到、等齐了再认失败。

use super::*;

/// 在卡上做一段活 `work`、接住它的 panic，并在它交回的那一刻（接住之后、做别的任何事之前）记下设备丢没丢
/// （[`DeviceConditionOfACard::device_when_the_work_returns`]）：崩溃放量接 Out of Memory 的那一层
/// （`singlefs_verification::crash_amplification::on_the_card_catching_out_of_memory_ending_together`）照它认这段活的失败
/// （这一步抽到这里，交回那一刻的读法与位置不要卡就测得到）。
pub fn work_caught_noting_the_device_when_it_returned<Outcome>(
    condition: &DeviceConditionOfACard,
    work: impl FnOnce() -> Outcome,
) -> (std::thread::Result<Outcome>, DeviceWhenTheWorkReturned) {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    let device_when_the_work_returned = condition.device_when_the_work_returns();
    (outcome, device_when_the_work_returned)
}

/// 同一个设备上一起开的几段活（崩溃放量单元校验和对拍时一块切成的几段，`singlefs_verification::crash_amplification`）：每段收完报到一次，
/// 认它是不是显存不够之前等其余几段都报到（[`DeviceConditionOfACard::end_the_work_and_explain_its_failure`]）。先把每一段的报到都领出来
/// （[`Self::join`]），再起那几段：领出来的没报到就丢掉（那一段 panic 了）也算报到，别的段不会等它一辈子。
pub struct WorksEndingTogetherOnADevice {
    pub(super) works_not_yet_ended: Mutex<usize>,
    every_work_ended: std::sync::Condvar,
}

/// 一段都没领（还没开活）就是起点，没有别的不变量。
impl Default for WorksEndingTogetherOnADevice {
    fn default() -> Self {
        Self::new()
    }
}

impl WorksEndingTogetherOnADevice {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            works_not_yet_ended: Mutex::new(0),
            every_work_ended: std::sync::Condvar::new(),
        }
    }

    /// 领一段活的报到（这一段起之前领）。
    #[must_use]
    pub fn join(&self) -> WorkEndingTogether<'_> {
        // 持锁只为改一个计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        *self
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner) += 1;
        WorkEndingTogether {
            together: self,
            has_ended: false,
        }
    }

    /// 领了报到、还没报到的段数。
    #[must_use]
    pub fn works_not_yet_ended(&self) -> usize {
        *self
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn end_one_work(&self) {
        let mut works_not_yet_ended = self
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *works_not_yet_ended = works_not_yet_ended
            .checked_sub(1)
            .expect("每一份报到只报一次：领一次加一，报一次减一");
        if *works_not_yet_ended == 0 {
            self.every_work_ended.notify_all();
        }
    }
}

/// 一段活的报到（[`WorksEndingTogetherOnADevice::join`]）：[`Self::end_and_wait_for_the_others`] 报到；没报到就丢掉，丢的时候补报（不等）。
pub struct WorkEndingTogether<'together> {
    together: &'together WorksEndingTogetherOnADevice,
    has_ended: bool,
}

impl WorkEndingTogether<'_> {
    /// 这一段收完了：报到，等一起开的其余几段都报到（或丢掉了报到）再交回。
    pub fn end_and_wait_for_the_others(mut self) {
        self.has_ended = true;
        self.together.end_one_work();
        let mut works_not_yet_ended = self
            .together
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while *works_not_yet_ended > 0 {
            works_not_yet_ended = self
                .together
                .every_work_ended
                .wait(works_not_yet_ended)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}

impl Drop for WorkEndingTogether<'_> {
    fn drop(&mut self) {
        if !self.has_ended {
            self.together.end_one_work();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{work_caught_noting_the_device_when_it_returned, WorksEndingTogetherOnADevice};
    use crate::gpu_memory_ledger::{
        CauseOfTheDeviceLoss, DeviceConditionOfACard, DeviceWhenTheWorkReturned,
        WorkOnTheCardOfThisThread,
    };

    /// （攻方 `record-the-caught-after-waiting`）：一段范围里接到的 Out of Memory 先记进设备再报到：
    /// 它报到的那一刻（还在等同组别的段）设备上已经记着，先收完的别的段这时来认就认得出。改成等完之后再记，报到的那一刻设备上一条都没有。
    #[test]
    fn piece_records_what_its_scope_caught_before_it_reports_its_ending() {
        let condition: &'static DeviceConditionOfACard =
            Box::leak(Box::new(DeviceConditionOfACard::default()));
        let together: &'static WorksEndingTogetherOnADevice =
            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
        let held_ending = together.join();
        let caught_piece_ending = together.join();
        let caught_piece = std::thread::spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            condition.end_the_work_and_explain_its_failure(
                &work,
                reports_before,
                condition.device_when_the_work_returns(),
                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                caught_piece_ending,
            )
        });
        let waiting_since = std::time::Instant::now();
        while together.works_not_yet_ended() > 1 {
            assert!(
                waiting_since.elapsed() < std::time::Duration::from_secs(60),
                "接到 Out of Memory 的那一段一分钟都没报到"
            );
            std::thread::yield_now();
        }
        let reports_when_it_reported = condition.out_of_memory_reports();
        drop(held_ending);
        let explained = caught_piece.join().expect("那一段不 panic");
        assert_eq!(
            reports_when_it_reported, 1,
            "报到的那一刻范围里接到的已经记进设备"
        );
        assert_eq!(
            explained,
            Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string())
        );
    }

    /// （攻方 `endings-notify-one`）：一组三段，两段先报到、在等；最后一段报到时叫醒等着的每一段，
    /// 两段都在一分钟之内等完（一块的段数是核对线程数，本机 32）。只叫醒一段时另一段一直等（这条用例一分钟之后判红）。
    #[test]
    fn the_last_piece_to_report_wakes_every_piece_waiting_for_it() {
        let together: &'static WorksEndingTogetherOnADevice =
            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
        let waiting_endings = [together.join(), together.join()];
        let last_ending = together.join();
        let (ended_sender, ended_receiver) = std::sync::mpsc::channel();
        for ending in waiting_endings {
            let ended_sender = ended_sender.clone();
            std::thread::spawn(move || {
                ending.end_and_wait_for_the_others();
                let _ = ended_sender.send(());
            });
        }
        drop(ended_sender);
        let waiting_since = std::time::Instant::now();
        while together.works_not_yet_ended() > 1 {
            assert!(
                waiting_since.elapsed() < std::time::Duration::from_secs(60),
                "先报到的两段一分钟都没报到"
            );
            std::thread::yield_now();
        }
        // 报到之后到进等待之间还有几条指令：给那两段时间都等上
        std::thread::sleep(std::time::Duration::from_millis(200));
        last_ending.end_and_wait_for_the_others();
        for waiting_piece in 0..2 {
            assert_eq!(
                ended_receiver.recv_timeout(std::time::Duration::from_secs(60)),
                Ok(()),
                "等着的第 {waiting_piece} 段一分钟之内被叫醒、等完"
            );
        }
    }

    /// 一段领了报到、没报到就丢掉（那一段 panic 了），算它报到了，别的段不等它。
    #[test]
    fn piece_dropping_its_ending_without_ending_does_not_keep_the_others_waiting() {
        let together: &'static WorksEndingTogetherOnADevice =
            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
        let dropped_ending = together.join();
        let waiting_ending = together.join();
        let (ended_sender, ended_receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            waiting_ending.end_and_wait_for_the_others();
            let _ = ended_sender.send(());
        });
        drop(dropped_ending);
        assert_eq!(
            ended_receiver.recv_timeout(std::time::Duration::from_secs(60)),
            Ok(()),
            "丢掉的那一份算报到：另一段一分钟之内等完"
        );
        assert_eq!(together.works_not_yet_ended(), 0);
    }

    /// 一段活交回的那一刻记设备丢没丢：活里设备丢了，交回那一刻记成丢了；
    /// 活 panic 时设备没丢、之后才丢，交回那一刻记成没丢（之后丢的不把这段活真正的 panic 说成显存不够）。
    #[test]
    fn work_notes_the_device_at_the_moment_it_returns_not_before_and_not_after() {
        let lost_during_the_work = DeviceConditionOfACard::default();
        let (outcome, device_when_the_work_returned) =
            work_caught_noting_the_device_when_it_returned(&lost_during_the_work, || {
                lost_during_the_work.record_the_device_lost(
                    "Unknown：Out of memory".to_string(),
                    CauseOfTheDeviceLoss::GpuMemoryShort,
                );
                7
            });
        assert!(matches!(outcome, Ok(7)));
        assert_eq!(
            device_when_the_work_returned,
            DeviceWhenTheWorkReturned::Lost,
            "活里设备丢了：交回那一刻记成丢了"
        );
        let lost_after_the_work = DeviceConditionOfACard::default();
        let (panicked_outcome, device_when_the_panicked_work_returned) =
            work_caught_noting_the_device_when_it_returned(&lost_after_the_work, || -> u32 {
                panic!("这段活自己真 panic 了")
            });
        lost_after_the_work.record_the_device_lost(
            "Unknown：Out of memory".to_string(),
            CauseOfTheDeviceLoss::GpuMemoryShort,
        );
        assert!(panicked_outcome.is_err(), "panic 接住了");
        assert_eq!(
            device_when_the_panicked_work_returned,
            DeviceWhenTheWorkReturned::NotLost,
            "活 panic 时设备没丢、之后才丢：交回那一刻记成没丢"
        );
    }
}
