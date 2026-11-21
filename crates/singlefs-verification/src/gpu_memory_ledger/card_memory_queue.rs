//! 调用方自己起好交进来的卡上按显存排队：一趟用的卡从哪来（`GpuCardsSource`）、每张卡的容量与排进去的占位，
//! 占位还回来时叫醒等着的。

use super::*;

/// 崩溃放量产品路径（`singlefs_verification::crash_amplification`）一趟用的卡从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuCardsSource {
    /// 这一趟没有一类计算走 GPU，不要卡。
    NoCardNeeded,
    /// 这个进程里 GPU 用例共用的卡池（`singlefs_verification::gpu_unit_checks::gpu_test_start::GpuCardPool`）：照池里的卡用，
    /// 每张卡这个进程只有一个设备、只在借到的那张卡上起。崩溃放量取卡（`crash_amplification::gpu_cards_in_use`）交的就是这一种。
    TheCardPoolOfThisProcess,
    /// 卡由调用方自己起好交进来（直接调 `crash_amplification::judge_my_blocks_on_gpu_cards` 的对拍用例），不进显存账；
    /// 给定状态表与观察者交回的盘那两路在这一种卡上建判器之前经 [`CardMemoryQueue`] 按显存排队。
    DevicesStartedForThisRun,
}

/// 调用方自己起好交进来的卡（[`GpuCardsSource::DevicesStartedForThisRun`]）上同时活着的判器按显存排队：每张卡一个容量（MiB），
/// 一把锁护着每张卡上此刻排进去的占位合起来多少，占位还回来时叫醒等着的。容量由调用方给：那张卡给崩溃放量的额度扣掉驱动那一份
/// （驱动那一份这个进程在每张卡上只付一次，起设备时付过）。这种卡不进上面那份显存账（[`GpuMemoryLedger`] 只管卡池里的卡），
/// 几条线程同时往同一张卡上建判器、各自照整张卡的额度开草稿区，叠起来会撞顶（wgpu 报 Out of Memory、在建缓冲的线程上 panic）。
/// 这份只算经它排进去的，不读卡上此刻的显存：别的进程、不经它建的缓冲不在它的账上（与卡池那份账的口径不同，那份每借一次现读一次）。
pub struct CardMemoryQueue {
    capacity_of_each_card_mebibytes: Vec<u64>,
    held_on_each_card_mebibytes: Mutex<Vec<u64>>,
    some_place_returned: std::sync::Condvar,
}

/// 照卡的先后（配置的优先级）找第一张此刻放得下 `need_mebibytes` 的卡：那张卡上已排进去的加这一笔不超过它的容量。
/// 前一张放满了才用下一张（文件头：「一个显卡 任务满了才去分另外一个」）；都放不下是 `None`。
#[must_use]
pub fn first_card_with_room_in_the_queue(
    capacity_of_each_card_mebibytes: &[u64],
    held_on_each_card_mebibytes: &[u64],
    need_mebibytes: u64,
) -> Option<usize> {
    capacity_of_each_card_mebibytes
        .iter()
        .zip(held_on_each_card_mebibytes)
        .position(|(capacity_mebibytes, held_mebibytes)| {
            held_mebibytes.saturating_add(need_mebibytes) <= *capacity_mebibytes
        })
}

impl CardMemoryQueue {
    /// 每张卡一个容量（MiB），次序就是挑卡的先后；起头每张卡上一个占位都没有。
    #[must_use]
    pub fn new(capacity_of_each_card_mebibytes: Vec<u64>) -> Self {
        let held_on_each_card_mebibytes = vec![0; capacity_of_each_card_mebibytes.len()];
        Self {
            capacity_of_each_card_mebibytes,
            held_on_each_card_mebibytes: Mutex::new(held_on_each_card_mebibytes),
            some_place_returned: std::sync::Condvar::new(),
        }
    }

    fn held_locked(&self) -> MutexGuard<'_, Vec<u64>> {
        // 持锁只为加减几个计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        self.held_on_each_card_mebibytes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// 照卡的先后排进第一张此刻放得下 `need_mebibytes` 的卡；每张都放不下就等，有占位还回来就醒、照先后再找一遍。
    ///
    /// # Errors
    /// 要的比每张卡的容量都大（一张卡都没有也算）：等多久都放不下，不等，调用方判红（配了 GPU 就一定用 GPU，不悄悄换小）。
    pub fn admit_on_the_first_card_with_room(
        &self,
        need_mebibytes: u64,
    ) -> Result<PlaceInTheCardMemoryQueue<'_>, NeedLargerThanEveryCard> {
        let Some(largest_capacity_mebibytes) =
            self.capacity_of_each_card_mebibytes.iter().copied().max()
        else {
            return Err(NeedLargerThanEveryCard {
                need_mebibytes,
                largest_ceiling_mebibytes: 0,
            });
        };
        if need_mebibytes > largest_capacity_mebibytes {
            return Err(NeedLargerThanEveryCard {
                need_mebibytes,
                largest_ceiling_mebibytes: largest_capacity_mebibytes,
            });
        }
        let mut held_on_each_card_mebibytes = self.held_locked();
        // 每一圈要么排进、交回，要么等到有占位还回来：上面判过至少一张卡空着时放得下，排进去的都会还，等得到头
        loop {
            let card_with_room = first_card_with_room_in_the_queue(
                &self.capacity_of_each_card_mebibytes,
                &held_on_each_card_mebibytes,
                need_mebibytes,
            );
            if let Some(card_position) = card_with_room {
                held_on_each_card_mebibytes[card_position] += need_mebibytes;
                return Ok(PlaceInTheCardMemoryQueue {
                    queue: self,
                    card_position,
                    mebibytes: need_mebibytes,
                });
            }
            held_on_each_card_mebibytes = self
                .some_place_returned
                .wait(held_on_each_card_mebibytes)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// 排进一张卡的一个占位（[`CardMemoryQueue::admit_on_the_first_card_with_room`]）：丢掉时还回去、叫醒等着的。
/// 调用方丢它之前先丢掉在这张卡上建的缓冲、等设备把提交了的活做完（丢掉的缓冲要等设备收拾才交回驱动）。
pub struct PlaceInTheCardMemoryQueue<'queue> {
    queue: &'queue CardMemoryQueue,
    pub(super) card_position: usize,
    pub(super) mebibytes: u64,
}

impl PlaceInTheCardMemoryQueue<'_> {
    /// 排进的是第几张卡（[`CardMemoryQueue::new`] 给容量的次序）。
    #[must_use]
    pub fn card_position(&self) -> usize {
        self.card_position
    }

    /// 占的显存（MiB）。
    #[must_use]
    pub fn mebibytes(&self) -> u64 {
        self.mebibytes
    }
}

impl Drop for PlaceInTheCardMemoryQueue<'_> {
    fn drop(&mut self) {
        let mut held_on_each_card_mebibytes = self.queue.held_locked();
        let held_on_the_card_mebibytes = &mut held_on_each_card_mebibytes[self.card_position];
        *held_on_the_card_mebibytes = held_on_the_card_mebibytes
            .checked_sub(self.mebibytes)
            .expect("每个占位只还一次：排进去时那张卡上加过这么多");
        drop(held_on_each_card_mebibytes);
        self.queue.some_place_returned.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::CardMemoryQueue;
    use crate::gpu_memory_ledger::NeedLargerThanEveryCard;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    /// 等一个排队结局最多等多久：排得进的几毫秒到几百毫秒就交回，到点还没交回就是挂住了（等的那条线程留在后台，不拦这条用例判红）。
    const LONGEST_WAIT_FOR_THE_QUEUE: std::time::Duration = std::time::Duration::from_secs(5);

    /// 同卡判器按显存排队（自己起的卡那一支同时建判器撞过顶）：一张卡容量 10 MiB，三条线程同时各要 6 MiB、各占 50 毫秒，
    /// 同一刻只排得进一条：三条线程合占的峰值是 6（不是 12、18），三条都在 5 秒内排进了（前一条还回来就叫醒后面的）。
    #[test]
    fn judges_that_do_not_fit_together_on_one_card_take_turns_and_each_gets_in_once_the_previous_returns(
    ) {
        const NEED_OF_EACH_THREAD_MEBIBYTES: u64 = 6;
        const THREADS: usize = 3;
        let queue = Arc::new(CardMemoryQueue::new(vec![10]));
        let held_by_the_threads_mebibytes = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let peak_held_by_the_threads_mebibytes = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let start_together = Arc::new(std::sync::Barrier::new(THREADS));
        let (admitted_sender, admitted) = std::sync::mpsc::channel();
        for _ in 0..THREADS {
            let queue = Arc::clone(&queue);
            let held_by_the_threads_mebibytes = Arc::clone(&held_by_the_threads_mebibytes);
            let peak_held_by_the_threads_mebibytes =
                Arc::clone(&peak_held_by_the_threads_mebibytes);
            let start_together = Arc::clone(&start_together);
            let admitted_sender = admitted_sender.clone();
            std::thread::spawn(move || {
                start_together.wait();
                let place = queue
                    .admit_on_the_first_card_with_room(NEED_OF_EACH_THREAD_MEBIBYTES)
                    .expect("6 MiB 比 10 MiB 的卡小：排得进");
                let held_now = held_by_the_threads_mebibytes
                    .fetch_add(NEED_OF_EACH_THREAD_MEBIBYTES, Ordering::SeqCst)
                    + NEED_OF_EACH_THREAD_MEBIBYTES;
                peak_held_by_the_threads_mebibytes.fetch_max(held_now, Ordering::SeqCst);
                admitted_sender
                    .send(place.card_position())
                    .expect("收的那头在等");
                std::thread::sleep(std::time::Duration::from_millis(50));
                held_by_the_threads_mebibytes
                    .fetch_sub(NEED_OF_EACH_THREAD_MEBIBYTES, Ordering::SeqCst);
                drop(place);
            });
        }
        let deadline = std::time::Instant::now() + LONGEST_WAIT_FOR_THE_QUEUE;
        for admitted_so_far in 0..THREADS {
            let card_position = admitted
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .unwrap_or_else(|_| {
                    panic!(
                        "三条线程都要在 5 秒内排进（前一条还回来就叫醒后面的）：只排进了 {admitted_so_far} 条"
                    )
                });
            assert_eq!(card_position, 0, "只有一张卡");
        }
        assert_eq!(
            peak_held_by_the_threads_mebibytes.load(Ordering::SeqCst),
            NEED_OF_EACH_THREAD_MEBIBYTES,
            "两条 6 MiB 放不进 10 MiB：同一刻只排得进一条"
        );
    }

    /// 要 11 MiB 而每张卡容量 10 MiB：当场交「比每张卡都大」（等多久都放不下：不等、不挂住）。
    #[test]
    fn need_larger_than_every_card_in_the_memory_queue_is_refused_at_once_instead_of_waiting() {
        let queue = Arc::new(CardMemoryQueue::new(vec![10, 10]));
        let (outcome_sender, outcome) = std::sync::mpsc::channel();
        let queue_of_the_thread = Arc::clone(&queue);
        std::thread::spawn(move || {
            let admitted = queue_of_the_thread
                .admit_on_the_first_card_with_room(11)
                .map(|place| place.card_position());
            outcome_sender.send(admitted).expect("收的那头在等");
        });
        assert_eq!(
            outcome
                .recv_timeout(LONGEST_WAIT_FOR_THE_QUEUE)
                .expect("要的比每张卡都大：当场交回，不等"),
            Err(NeedLargerThanEveryCard {
                need_mebibytes: 11,
                largest_ceiling_mebibytes: 10
            })
        );
    }

    /// 两张卡各 10 MiB，前一张已排进 6 MiB，再要 6 MiB：排进后一张，不等前一张还。
    #[test]
    fn judge_that_does_not_fit_beside_the_one_on_the_first_card_goes_to_the_next_card_without_waiting(
    ) {
        let queue = Arc::new(CardMemoryQueue::new(vec![10, 10]));
        let on_the_first_card = queue
            .admit_on_the_first_card_with_room(6)
            .expect("空着的卡排得进 6 MiB");
        assert_eq!(on_the_first_card.card_position(), 0, "先用前一张");
        let (admitted_sender, admitted) = std::sync::mpsc::channel();
        let queue_of_the_thread = Arc::clone(&queue);
        std::thread::spawn(move || {
            let place = queue_of_the_thread
                .admit_on_the_first_card_with_room(6)
                .expect("后一张空着，排得进 6 MiB");
            admitted_sender
                .send(place.card_position())
                .expect("收的那头在等");
        });
        assert_eq!(
            admitted
                .recv_timeout(LONGEST_WAIT_FOR_THE_QUEUE)
                .expect("前一张放不下就排进后一张，不等前一张还"),
            1
        );
        drop(on_the_first_card);
    }
}
