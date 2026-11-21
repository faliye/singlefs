//! 借据与按需起的设备：列卡那一刻没等成的借据、按状态数借到的草稿区、账上的一笔借据与它丢掉之前等设备的结局，
//! 借据丢掉时那一步，与每张卡要时才起的设备 `DeviceStartedOnDemand`。

use super::*;

/// 列卡那一刻一张卡上等设备没等成而照旧记着的那几笔（[`GpuMemoryLedger::leases_kept_after_a_failed_wait_on_each_card`]）：
/// 卡池列卡之后对这张卡留着的设备 poll，交回队列空了只销这几笔（[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]）。
/// 列卡之后才记成没等成的那一笔不在里面：它的活可以是 poll 之后才提交的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeasesKeptAfterAFailedWaitWhenListed {
    pub(super) card_position: usize,
    /// 借据号 → 它等的那一台。
    pub(super) leases_kept_on_each_device: BTreeMap<u64, DeviceNumberOnTheCard>,
}

impl LeasesKeptAfterAFailedWaitWhenListed {
    /// 在池里第几张卡上。
    #[must_use]
    pub fn card_position(&self) -> usize {
        self.card_position
    }

    /// 列出来的借据号（从小到大）。
    #[cfg(test)]
    #[must_use]
    pub fn lease_identifiers(&self) -> Vec<u64> {
        self.leases_kept_on_each_device.keys().copied().collect()
    }
}

/// 按状态数在一张卡上借到的草稿区：借了几个状态（在锁里照此刻能借的压过）与那一笔借据（丢掉时还）。
pub struct StatesBookedOnACard<'ledger> {
    pub states: u64,
    pub booking: GpuMemoryBooking<'ledger>,
}

/// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡能借给这个进程的；丢掉时销账（等设备没等成的照旧记着，
/// [`Self::take_in_the_wait_before_returning`]）。
pub struct GpuMemoryBooking<'ledger> {
    pub(super) ledger: &'ledger GpuMemoryLedger,
    pub(super) card_position: usize,
    pub(super) mebibytes: u64,
    pub(super) lease_identifier: u64,
    pub(super) card_room_mebibytes_when_booked: u64,
    pub(super) when_dropped: LeaseWhenTheBookingIsDropped,
}

/// 借据丢掉之前等设备的结局（卡池的借据丢掉时等这张卡的设备把提交了的活都做完，`singlefs_verification::gpu_unit_checks::gpu_test_start::GpuCardLease`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceWaitBeforeReturningTheLease {
    /// 提交了的活都做完了：丢掉的缓冲已经交回驱动，可以销账。
    TheSubmittedWorkFinished,
    /// 没等成（超时、wgpu 报错或 panic）：缓冲可能还占着卡，等的是在用的那一台时这一笔先不销账（文件头「口径」）。
    TheWaitFailed,
}

/// 一笔借据丢掉时账上怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LeaseWhenTheBookingIsDropped {
    /// 销账（等完设备了，这一笔没在卡上建过缓冲，或等的是丢了还没放掉的那一台）。
    ReturnedToTheDriverAndOffTheBooks,
    /// 等在用的那一台没等成：照旧记着，那一台上提交的活后来做完了、记成丢了、或放掉了时销（`BooksOfACard::leases_kept_after_a_failed_wait`）。
    KeptOnTheBooksAfterAFailedWait,
}

impl GpuMemoryBooking<'_> {
    /// 记在池里第几张卡上。
    #[must_use]
    pub fn card_position(&self) -> usize {
        self.card_position
    }

    #[must_use]
    pub fn mebibytes(&self) -> u64 {
        self.mebibytes
    }

    /// 这一笔的借据号（[`GpuMemoryLedger::is_the_lease_on_the_books`] 照它查）。
    #[must_use]
    pub fn lease_identifier(&self) -> u64 {
        self.lease_identifier
    }

    /// 记的那一刻这张卡能借给这个进程的（MiB，这一笔之前）。
    #[must_use]
    pub fn card_room_mebibytes_when_booked(&self) -> u64 {
        self.card_room_mebibytes_when_booked
    }

    /// 这一笔此刻还记在账上没有（[`GpuMemoryLedger::is_the_lease_on_the_books`]；借据活着时恒为真，核账用）。
    #[must_use]
    pub fn is_on_the_books(&self) -> bool {
        self.ledger
            .is_the_lease_on_the_books(self.card_position, self.lease_identifier)
    }

    /// 丢掉这一笔之前等过设备（卡池的借据丢掉时调「改用乙」；`device` 是这笔借据那张卡上的设备）：
    /// 等完了，丢掉时照常销账，同一台上之前没等成而照旧记着的那几笔也一起销（这一次等的是到此刻为止的全部提交，它们的活也做完了）；
    /// 没等成而等的是在用的那一台，这一笔丢掉时不销账、照旧算借着，到那一台上提交的活后来做完了、或那一台记成丢了、放掉了才销
    /// （文件头「口径」：账不多给，代价是这一段少借这一笔）；没等成而等的是丢了还没放掉的那一台，照常销账（这个进程占着的那时照分不出算，
    /// 它的缓冲在那一台放掉之前交回驱动）。
    ///
    /// # Panics
    /// `device` 不是这本账、这张卡上的（别处已经在 panic 时不断言：丢借据是收尾的一部分，两次 panic 会让整个进程中止）。
    pub fn take_in_the_wait_before_returning(
        &mut self,
        device: &DeviceStartedOnTheBooks<'_>,
        wait: DeviceWaitBeforeReturningTheLease,
    ) {
        assert!(
            (std::ptr::eq(device.ledger, self.ledger) && device.card_position == self.card_position)
                || std::thread::panicking(),
            "第 {} 张卡上的借据号 {} 等的设备记在第 {} 张卡上（或别的账上）：借据只等它那张卡上的设备",
            self.card_position,
            self.lease_identifier,
            device.card_position
        );
        let mut books = self.ledger.books_locked();
        let books_of_the_card = &mut books[self.card_position];
        match wait {
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {
                books_of_the_card.return_the_leases_kept_after_a_failed_wait_on(device.device);
            }
            DeviceWaitBeforeReturningTheLease::TheWaitFailed => {
                if !books_of_the_card
                    .lost_devices_not_released
                    .contains(&device.device)
                {
                    books_of_the_card
                        .leases_kept_after_a_failed_wait
                        .insert(self.lease_identifier, device.device);
                    self.when_dropped =
                        LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;
                }
            }
        }
    }
}

impl Drop for GpuMemoryBooking<'_> {
    fn drop(&mut self) {
        match self.when_dropped {
            LeaseWhenTheBookingIsDropped::ReturnedToTheDriverAndOffTheBooks => {
                let removed = self.ledger.books_locked()[self.card_position]
                    .return_a_lease(self.lease_identifier);
                // 别处已经在 panic（丢借据是收尾的一部分）时不再断言：两次 panic 会让整个进程中止
                assert!(
                    removed || std::thread::panicking(),
                    "第 {} 张卡上借据号 {} 不在账上：一笔借据只在它丢掉时销一次账",
                    self.card_position,
                    self.lease_identifier
                );
            }
            LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait => {}
        }
    }
}

/// 卡池的一笔借据丢掉时那一步（`singlefs_verification::gpu_unit_checks::gpu_test_start::GpuCardLease` 的 `drop` 只调它）：
/// 照 `wait_for_the_device` 等这笔借据那张卡上的设备 `device`（wgpu 在等的时候 panic 了接住），`wait_of` 照等的结局判等没等成，
/// 没等成时先交 `report_the_failed_wait` 打一行，再把结局记进这一笔（[`GpuMemoryBooking::take_in_the_wait_before_returning`]）。
/// 等的时候 panic 了：不在展开中，记完照样往外抛；已经在展开中（这笔借据是别处 panic 收尾时丢掉的），只记没等成、不再抛——展开中再抛会让
/// 整个进程中止，同一进程里别的用例都没了结局（攻方 H3，改法 F-H3）。
pub fn wait_for_the_device_and_take_in_the_outcome<Polled>(
    booking: &mut GpuMemoryBooking<'_>,
    device: &DeviceStartedOnTheBooks<'_>,
    wait_for_the_device: impl FnOnce() -> Polled,
    wait_of: impl FnOnce(&std::thread::Result<Polled>) -> DeviceWaitBeforeReturningTheLease,
    report_the_failed_wait: impl FnOnce(&std::thread::Result<Polled>),
) {
    let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait_for_the_device));
    let wait = wait_of(&polled);
    match wait {
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {}
        DeviceWaitBeforeReturningTheLease::TheWaitFailed => report_the_failed_wait(&polled),
    }
    booking.take_in_the_wait_before_returning(device, wait);
    if let Err(panic) = polled {
        if !std::thread::panicking() {
            std::panic::resume_unwind(panic);
        }
    }
}

/// 一张卡上这个进程的设备（卡池每张卡一格）：要时有留着的、没丢的就拿它；没有（第一次要、上一次起不来）或丢了就起一个，
/// 起好了留着，起不来交为什么、不留着（起设备失败不缓存成这个进程永久的错；设备丢了之后借这张卡的
/// 不拿丢了的那个）。不碰 wgpu：设备、怎么起、怎么算丢了由卡池给，单测换成假的。
/// 两把锁：起设备时只拿 `starting`（同一张卡上同时要起的等这一次起完），留着的那一份只在取、换的那一下拿 `started`，
/// 取留着的设备（[`Self::started_now`]，卡池借之前 poll 用）不等正在起的设备（
/// 正推量到一次 300 毫秒的重起里借之前那一下等了 250 毫秒，哪怕借的是别的卡）。
pub struct DeviceStartedOnDemand<Device> {
    /// 留着的设备：持锁的那几段只取克隆、换一份，不起设备。
    started: Mutex<Option<Device>>,
    /// 起设备的那一段拿着。
    starting: Mutex<()>,
}

/// 空着（还没起设备）就是起点，没有别的不变量。
impl<Device: Clone> Default for DeviceStartedOnDemand<Device> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Device: Clone> DeviceStartedOnDemand<Device> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            started: Mutex::new(None),
            starting: Mutex::new(()),
        }
    }

    /// 留着的那一份的克隆（`started` 只在这一下拿）。持锁的这一段只克隆，不 panic；别的线程 panic 留下的毒照用。
    fn clone_of_the_device_kept(&self) -> Option<Device> {
        self.started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// 留着的设备 `is_lost` 不为真就交它的克隆；否则调 `start` 起一个，起好了留着、交克隆，起不来交 `start` 的错（不留着）。
    /// 起的时候拿着这张卡起设备的那把锁：同一张卡上同时要起的等这一次起完、再看一次留着的（别的卡不等）；
    /// 取留着的设备（[`Self::started_now`]、[`Self::is_started`]）不等。
    ///
    /// # Errors
    /// `start` 的错。
    pub fn started_or_start<Failure>(
        &self,
        is_lost: impl Fn(&Device) -> bool,
        start: impl FnOnce() -> Result<Device, Failure>,
    ) -> Result<Device, Failure> {
        if let Some(device) = self.clone_of_the_device_kept() {
            if !is_lost(&device) {
                return Ok(device);
            }
        }
        // 持这把锁时 panic 只会在 `start` 里（起设备的断言）：那时留着的还是旧的（没有或丢了的），别的线程照用、照起
        let _starting = self.starting.lock().unwrap_or_else(PoisonError::into_inner);
        // 等这把锁的那一段里别的线程可能已经起好了：再看一次，起好了就用它，不再起
        if let Some(device) = self.clone_of_the_device_kept() {
            if !is_lost(&device) {
                return Ok(device);
            }
        }
        let device = start()?;
        // 换下来的旧设备在放开 `started` 之后才丢：它可能是这个进程里那台设备的最后一个克隆，丢的时候 wgpu 要等设备空了再收拾
        let replaced = self
            .started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .replace(device.clone());
        drop(replaced);
        Ok(device)
    }

    /// 起过、留着一个设备没有（丢了的也算留着，下一次要时重起）；不等正在起的设备。
    #[must_use]
    pub fn is_started(&self) -> bool {
        self.started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// 留着的设备的克隆（丢了的也交；没起过、上一次没起来是 `None`）；不起设备，也不等正在起的设备：起着的时候交的是起之前留着的那一份
    /// （没有或丢了的；卡池借之前 poll 留着的设备用，丢了的不 poll）。
    #[must_use]
    pub fn started_now(&self) -> Option<Device> {
        self.clone_of_the_device_kept()
    }
}

#[cfg(test)]
mod tests {
    use super::{wait_for_the_device_and_take_in_the_outcome, DeviceStartedOnDemand};
    use crate::gpu_memory_ledger::tests::{
        ledger_reading, reading_of_an_rtx_5090, replaceable_readings, wait_of_a_fake_device,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    /// 卡池每张卡的设备第一次起不来不留着，下一次要时再起；起好了留着给后面要的；留着的那个丢了就重起一个。
    #[test]
    fn device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced() {
        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
        let starts = AtomicUsize::new(0);
        let start_as = |device: u32| {
            starts.fetch_add(1, Ordering::SeqCst);
            Ok::<u32, String>(device)
        };
        let never_lost = |_device: &u32| false;
        assert_eq!(
            slot.started_or_start(never_lost, || Err::<u32, String>("显存不够".to_string())),
            Err("显存不够".to_string()),
            "第一次起不来：交起不来的那一句"
        );
        assert!(!slot.is_started(), "起不来不留着");
        assert_eq!(
            slot.started_or_start(never_lost, || start_as(1)),
            Ok(1),
            "下一次要时再起，起得来"
        );
        assert_eq!(
            slot.started_or_start(never_lost, || start_as(2)),
            Ok(1),
            "起好了留着：后面要的拿同一个，不再起"
        );
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        assert_eq!(
            slot.started_or_start(|device: &u32| *device == 1, || start_as(3)),
            Ok(3),
            "留着的那个丢了：重起一个"
        );
        assert_eq!(starts.load(Ordering::SeqCst), 2);
        assert!(slot.is_started());
    }

    /// （正推 `probe_started_now_blocks.rs`：取留着的设备与起设备共用一把锁，
    /// 300 毫秒的重起里借之前那一下等了 250 毫秒）：留着的设备 1 丢了，一条线程在重起、起到一半停住；这时取留着的设备不等，
    /// 交起之前留着的那一台（丢了的 1，卡池借之前不 poll 丢了的）；重起完了交新起的 2。
    #[test]
    fn taking_the_device_kept_does_not_wait_for_a_restart_in_progress() {
        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
        assert_eq!(
            slot.started_or_start(|_device: &u32| false, || Ok::<u32, String>(1)),
            Ok(1)
        );
        let slot_shared = &slot;
        let (restart_began_sender, restart_began) = std::sync::mpsc::channel::<()>();
        let (finish_the_restart, restart_may_finish) = std::sync::mpsc::channel::<()>();
        let (taken_sender, taken) = std::sync::mpsc::channel::<Option<u32>>();
        std::thread::scope(|scope| {
            let restarting = scope.spawn(move || {
                slot_shared.started_or_start(
                    |device: &u32| *device == 1,
                    move || {
                        restart_began_sender.send(()).expect("用例在等重起起头");
                        // 用例断言失败、放开之前就丢了发的那一头时也收尾
                        let _finished_or_the_test_gave_up = restart_may_finish.recv();
                        Ok::<u32, String>(2)
                    },
                )
            });
            restart_began.recv().expect("重起起头了");
            scope.spawn(move || {
                taken_sender
                    .send(slot_shared.started_now())
                    .expect("用例在等取到的那一台");
            });
            let taken_during_the_restart = taken.recv_timeout(std::time::Duration::from_secs(10));
            finish_the_restart.send(()).expect("重起在等放开");
            assert_eq!(
                taken_during_the_restart,
                Ok(Some(1)),
                "重起着的时候取留着的设备不等：交起之前留着的那一台（丢了的 1）"
            );
            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
        });
        assert_eq!(
            slot.started_now(),
            Some(2),
            "重起完了：留着的是新起的那一台"
        );
    }

    /// （K4-2 拆了锁之后）：同一张卡上一条线程在重起，另一条也要这张卡的设备：等这一次起完、拿它起好的那一台，不另起一台。
    #[test]
    fn second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device() {
        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
        assert_eq!(
            slot.started_or_start(|_device: &u32| false, || Ok::<u32, String>(1)),
            Ok(1)
        );
        let slot_shared = &slot;
        let starts_by_the_second = AtomicUsize::new(0);
        let starts_by_the_second_shared = &starts_by_the_second;
        let (restart_began_sender, restart_began) = std::sync::mpsc::channel::<()>();
        let (finish_the_restart, restart_may_finish) = std::sync::mpsc::channel::<()>();
        let (second_sender, second) = std::sync::mpsc::channel::<Result<u32, String>>();
        std::thread::scope(|scope| {
            let restarting = scope.spawn(move || {
                slot_shared.started_or_start(
                    |device: &u32| *device == 1,
                    move || {
                        restart_began_sender.send(()).expect("用例在等重起起头");
                        let _finished_or_the_test_gave_up = restart_may_finish.recv();
                        Ok::<u32, String>(2)
                    },
                )
            });
            restart_began.recv().expect("重起起头了");
            scope.spawn(move || {
                second_sender
                    .send(slot_shared.started_or_start(
                        |device: &u32| *device == 1,
                        || {
                            starts_by_the_second_shared.fetch_add(1, Ordering::SeqCst);
                            Ok::<u32, String>(3)
                        },
                    ))
                    .expect("用例在等第二条拿到的那一台");
            });
            let second_before_the_restart_finished =
                second.recv_timeout(std::time::Duration::from_millis(500));
            finish_the_restart.send(()).expect("重起在等放开");
            assert_eq!(
                second_before_the_restart_finished,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout),
                "第一条还在起：第二条等它起完，不先交回"
            );
            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
            assert_eq!(
                second.recv_timeout(std::time::Duration::from_secs(10)),
                Ok(Ok(2)),
                "第二条拿的是第一条起好的那一台"
            );
        });
        assert_eq!(
            starts_by_the_second.load(Ordering::SeqCst),
            0,
            "第二条没另起一台"
        );
    }

    /// 一台假设备：最后一个克隆丢掉时（wgpu 收设备那一下要等设备空了）先报一声、等用例放开才收完；`number` 认是哪一台。
    struct DeviceWhoseLastDropWaits {
        number: u32,
        dropping_began: Mutex<Option<std::sync::mpsc::Sender<()>>>,
        drop_may_finish: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
    }

    impl Drop for DeviceWhoseLastDropWaits {
        fn drop(&mut self) {
            if let Some(dropping_began) =
                self.dropping_began.lock().expect("只在丢的这一下拿").take()
            {
                // 用例已经不等了（断言失败、先丢了收的那一头）时照样往下收
                let _sent_or_the_test_gave_up = dropping_began.send(());
            }
            if let Some(drop_may_finish) = self
                .drop_may_finish
                .lock()
                .expect("只在丢的这一下拿")
                .take()
            {
                let _finished_or_the_test_gave_up = drop_may_finish.recv();
            }
        }
    }

    /// （换下来的旧设备改回在 `started` 的锁里丢照绿）：留着的设备 1 丢了，
    /// 一条线程重起、换上 2，丢 1 时卡住（等设备空了）；这时取留着的设备不等丢旧设备的那一下，交新起的 2。
    #[test]
    fn taking_the_device_kept_does_not_wait_for_the_replaced_device_to_be_dropped() {
        let slot: DeviceStartedOnDemand<Arc<DeviceWhoseLastDropWaits>> =
            DeviceStartedOnDemand::new();
        let (dropping_began_sender, dropping_began) = std::sync::mpsc::channel::<()>();
        let (finish_the_drop, drop_may_finish) = std::sync::mpsc::channel::<()>();
        let first_started = slot.started_or_start(
            |_device: &Arc<DeviceWhoseLastDropWaits>| false,
            move || {
                Ok::<Arc<DeviceWhoseLastDropWaits>, String>(Arc::new(DeviceWhoseLastDropWaits {
                    number: 1,
                    dropping_began: Mutex::new(Some(dropping_began_sender)),
                    drop_may_finish: Mutex::new(Some(drop_may_finish)),
                }))
            },
        );
        assert_eq!(first_started.map(|device| device.number), Ok(1));
        let slot_shared = &slot;
        let (taken_sender, taken) = std::sync::mpsc::channel::<Option<u32>>();
        std::thread::scope(|scope| {
            let restarting = scope.spawn(move || {
                slot_shared
                    .started_or_start(
                        |device: &Arc<DeviceWhoseLastDropWaits>| device.number == 1,
                        || {
                            Ok::<Arc<DeviceWhoseLastDropWaits>, String>(Arc::new(
                                DeviceWhoseLastDropWaits {
                                    number: 2,
                                    dropping_began: Mutex::new(None),
                                    drop_may_finish: Mutex::new(None),
                                },
                            ))
                        },
                    )
                    .map(|device| device.number)
            });
            let dropping_the_replaced_device =
                dropping_began.recv_timeout(std::time::Duration::from_secs(10));
            scope.spawn(move || {
                // 用例已经不等了时照样收尾
                let _sent_or_the_test_gave_up =
                    taken_sender.send(slot_shared.started_now().map(|device| device.number));
            });
            let taken_while_the_replaced_device_drops =
                taken.recv_timeout(std::time::Duration::from_secs(10));
            finish_the_drop.send(()).expect("丢旧设备那一下在等放开");
            assert_eq!(
                dropping_the_replaced_device,
                Ok(()),
                "重起换上 2 之后丢旧设备 1"
            );
            assert_eq!(
                taken_while_the_replaced_device_drops,
                Ok(Some(2)),
                "丢旧设备的那一下不拿着留着的那一份：取留着的设备不等，交新起的 2"
            );
            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
        });
    }

    /// （正推第 4 点：`GpuCardLease::drop` 记没记结局没有不要卡的测试）：
    /// 借据丢掉时那一步没等成，打一行、这一笔照旧记着；等完了不打、照常销账，之前没等成的那一笔也一起销。
    #[test]
    fn the_wait_before_returning_a_lease_reports_a_failed_wait_and_takes_in_either_outcome() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        let mut timed_out = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        let timed_out_identifier = timed_out.lease_identifier();
        let device = ledger.record_the_device_started_on_the_card(0);
        let mut finished = ledger.book_on_the_card(0, 1_000).expect("借得到");
        let finished_identifier = finished.lease_identifier();
        let reports = std::cell::Cell::new(0);
        wait_for_the_device_and_take_in_the_outcome(
            &mut timed_out,
            &device,
            || false,
            wait_of_a_fake_device,
            |_polled| reports.set(reports.get() + 1),
        );
        drop(timed_out);
        assert_eq!(reports.get(), 1, "没等成：打一行");
        assert!(
            ledger.is_the_lease_on_the_books(0, timed_out_identifier),
            "没等成：结局记进了这一笔，丢掉之后照旧记着"
        );
        wait_for_the_device_and_take_in_the_outcome(
            &mut finished,
            &device,
            || true,
            wait_of_a_fake_device,
            |_polled| reports.set(reports.get() + 1),
        );
        drop(finished);
        assert_eq!(reports.get(), 1, "等完了：不打");
        assert!(!ledger.is_the_lease_on_the_books(0, finished_identifier));
        assert!(
            !ledger.is_the_lease_on_the_books(0, timed_out_identifier),
            "同一台上等完了：之前没等成的那一笔一起销"
        );
    }

    /// （`catch_unwind` 拿掉照绿）：不在展开中，等的时候 wgpu panic 了，
    /// 这一笔记成没等成（丢掉之后照旧记着），那个 panic 照样往外抛。
    #[test]
    fn panic_while_waiting_before_returning_a_lease_is_taken_as_a_failed_wait_and_goes_on() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        let mut lease = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        let lease_identifier = lease.lease_identifier();
        let device = ledger.record_the_device_started_on_the_card(0);
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            wait_for_the_device_and_take_in_the_outcome(
                &mut lease,
                &device,
                || -> bool { panic!("等的时候 wgpu panic 了") },
                wait_of_a_fake_device,
                |_polled| {},
            );
        }));
        let panic = unwound.expect_err("不在展开中：等的时候的 panic 照样往外抛");
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"等的时候 wgpu panic 了"),
            "抛出去的是等的时候那一个"
        );
        drop(lease);
        assert!(
            ledger.is_the_lease_on_the_books(0, lease_identifier),
            "等的时候 panic 了：记成没等成，丢掉之后照旧记着"
        );
    }
}
