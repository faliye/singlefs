//! 这个进程里开着的库：按键空间各一份连接、库锁与累计持锁时长、同一摘要的值只读回一次、
//! 判段与录入段每次读写现取现放，以及按测试类别开的那一份库。

use super::*;

// ============================== 这个进程里开着的库 ==============================

/// 这个进程里开着的库在登记表里的键：连哪一处（`RouteAddress` 的写法）与键空间的前缀字节（全仓那一份是空的）。
type KeyOfALibraryOpenInThisProcess = (String, Vec<u8>);

/// 开着的一份：库，一把锁一次一个读写（容错客户端追加日志的一次性目录在它连着的那一端里，随最后一个放手的删掉）；这把锁的累计时长；
/// 这条连接读回过的流与它们的计划；持锁时长行里报的连接名（[`connection_name_of_an_opening`]）。
struct OpenLibrary {
    connection_name: String,
    store: Mutex<VerdictStore>,
    lock_times: LibraryLockTimes,
    flows_read_back:
        ValuesReadBackOnceInThisProcess<crate::crash_amplification::RecordedFlowAndItsPlan>,
}

/// 这个进程里开过几次连接：连接名带上开的是第几次，同一个进程里关了再开同一份库，两次的行不同名（各自的 `seconds_open` 各从零数）。
static OPENINGS_OF_A_LIBRARY_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 持锁时长行里的连接名：连哪一处、键空间前缀（十六进制，全仓那一份是空的）、这个进程里开的第几次，三段用 `#` 接起来，空白换成 `_`
/// （行按空格切字段）。
fn connection_name_of_an_opening(key: &KeyOfALibraryOpenInThisProcess) -> String {
    let opening = OPENINGS_OF_A_LIBRARY_IN_THIS_PROCESS.fetch_add(1, Ordering::Relaxed);
    format!("{}#{}#{opening}", key.0, hexadecimal_text(&key.1))
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("_")
}

fn seconds_of_nanoseconds(nanoseconds: u64) -> f64 {
    std::time::Duration::from_nanos(nanoseconds).as_secs_f64()
}

impl OpenLibrary {
    /// `CRASH_LIBRARY_LOCK_TIME connection= pid= seconds_open= seconds_held= seconds_held_in_round_trips= seconds_held_outside_round_trips=
    /// seconds_waited= most_askers_at_once= acquisitions= held_share_of_the_wall_clock= flows_read_over_the_connection=`，
    /// 字段都是从连接开着起累计的：一趟的读数按 `connection=` 与 `pid=` 各取这一趟日志里那条连接的最后一行。
    /// `seconds_held_in_round_trips` 是锁里花在与 prune-server 往返上的秒数（[`VerdictStore::nanoseconds_in_round_trips_so_far`]），
    /// `most_askers_at_once` 是同时在等这把锁的最多线程数。
    fn lock_time_line(&self, time_open: std::time::Duration) -> String {
        let times = &self.lock_times;
        let seconds_open = time_open.as_secs_f64();
        let held_nanoseconds = times.nanoseconds_held.load(Ordering::Relaxed);
        let held_in_round_trips_nanoseconds = times
            .nanoseconds_held_in_round_trips
            .load(Ordering::Relaxed);
        let held = seconds_of_nanoseconds(held_nanoseconds);
        let waited = seconds_of_nanoseconds(times.nanoseconds_waited.load(Ordering::Relaxed));
        let held_share = if seconds_open > 0.0 {
            held / seconds_open
        } else {
            0.0
        };
        format!(
            "{LIBRARY_LOCK_TIME_LINE_PREFIX} connection={} pid={} seconds_open={seconds_open:.3} seconds_held={held:.3} seconds_held_in_round_trips={:.3} seconds_held_outside_round_trips={:.3} seconds_waited={waited:.3} most_askers_at_once={} acquisitions={} held_share_of_the_wall_clock={held_share:.3} flows_read_over_the_connection={}",
            self.connection_name,
            std::process::id(),
            seconds_of_nanoseconds(held_in_round_trips_nanoseconds),
            seconds_of_nanoseconds(held_nanoseconds.saturating_sub(held_in_round_trips_nanoseconds)),
            times.most_askers_at_once.load(Ordering::Relaxed),
            times.acquisitions.load(Ordering::Relaxed),
            self.flows_read_back
                .values_read_over_the_connection
                .load(Ordering::Relaxed)
        )
    }

    /// 按取锁的调用点各一行（[`LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX`]），持锁秒数大的在前：`connection= pid= site=<文件>:<行>
    /// seconds_held= seconds_held_in_round_trips= seconds_held_outside_round_trips= acquisitions= longest_hold_seconds=`，从连接开着起累计。
    fn lock_time_lines_of_the_sites(&self) -> Vec<String> {
        let sites = self
            .lock_times
            .of_each_site
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut ordered: Vec<(&(&'static str, u32), &LockTimesOfASite)> = sites.iter().collect();
        ordered.sort_by_key(|site| std::cmp::Reverse(site.1.nanoseconds_held));
        ordered
            .into_iter()
            .map(|((file, line), of_the_site)| {
                format!(
                    "{LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX} connection={} pid={} site={file}:{line} seconds_held={:.3} seconds_held_in_round_trips={:.3} seconds_held_outside_round_trips={:.3} acquisitions={} longest_hold_seconds={:.3}",
                    self.connection_name,
                    std::process::id(),
                    seconds_of_nanoseconds(of_the_site.nanoseconds_held),
                    seconds_of_nanoseconds(of_the_site.nanoseconds_held_in_round_trips),
                    seconds_of_nanoseconds(
                        of_the_site
                            .nanoseconds_held
                            .saturating_sub(of_the_site.nanoseconds_held_in_round_trips)
                    ),
                    of_the_site.acquisitions,
                    seconds_of_nanoseconds(of_the_site.longest_hold_nanoseconds),
                )
            })
            .collect()
    }

    /// 连接那一行连同各调用点那几行打进标准错误。
    fn print_the_lock_time_lines(&self, time_open: std::time::Duration) {
        eprintln!("{}", self.lock_time_line(time_open));
        for line in self.lock_time_lines_of_the_sites() {
            eprintln!("{line}");
        }
    }
}

impl Drop for OpenLibrary {
    fn drop(&mut self) {
        self.print_the_lock_time_lines(self.lock_times.opened_at.elapsed());
    }
}

/// 持锁时长行的头（[`OpenLibrary::lock_time_line`]）：只打进日志，不进任何键。
pub const LIBRARY_LOCK_TIME_LINE_PREFIX: &str = "CRASH_LIBRARY_LOCK_TIME";

/// 按调用点的持锁时长行的头（[`OpenLibrary::lock_time_lines_of_the_sites`]）：只打进日志，不进任何键。
pub const LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX: &str = "CRASH_LIBRARY_LOCK_TIME_OF_A_SITE";

/// 连接开着时隔多久打一行持锁时长：实验装置把引擎放在 `static` 里，连接开到进程退出、放手那一行打不出来，读数靠这一行。
const SECONDS_BETWEEN_LIBRARY_LOCK_TIME_LINES: u64 = 60;

/// 一条连接上那把锁的累计时长。线程数越多等锁的秒数加起来越大，可以超过连接开着的秒数；判收窄成不成看持锁占比。
struct LibraryLockTimes {
    opened_at: std::time::Instant,
    nanoseconds_held: AtomicU64,
    /// 持锁的纳秒里花在与 prune-server 往返上的那一部分。
    nanoseconds_held_in_round_trips: AtomicU64,
    nanoseconds_waited: AtomicU64,
    /// 此刻在等这把锁的线程数，与它到过的最大值。
    askers_now: AtomicU64,
    most_askers_at_once: AtomicU64,
    acquisitions: AtomicU64,
    next_line_at_second: AtomicU64,
    /// 按取锁的调用点（文件，行）各记一份。
    of_each_site: Mutex<BTreeMap<(&'static str, u32), LockTimesOfASite>>,
}

/// 一个取锁调用点的累计：持锁纳秒、其中往返的纳秒、取锁次数、最长一次持锁的纳秒。
#[derive(Default)]
struct LockTimesOfASite {
    nanoseconds_held: u64,
    nanoseconds_held_in_round_trips: u64,
    acquisitions: u64,
    longest_hold_nanoseconds: u64,
}

impl LibraryLockTimes {
    fn starting_at_this_opening() -> Self {
        Self {
            opened_at: std::time::Instant::now(),
            nanoseconds_held: AtomicU64::new(0),
            nanoseconds_held_in_round_trips: AtomicU64::new(0),
            nanoseconds_waited: AtomicU64::new(0),
            askers_now: AtomicU64::new(0),
            most_askers_at_once: AtomicU64::new(0),
            acquisitions: AtomicU64::new(0),
            next_line_at_second: AtomicU64::new(SECONDS_BETWEEN_LIBRARY_LOCK_TIME_LINES),
            of_each_site: Mutex::new(BTreeMap::new()),
        }
    }

    /// 一次持锁放手时记进这一次的调用点：持了多久、其中往返多久。
    fn note_a_hold_at_the_site(
        &self,
        site: &'static std::panic::Location<'static>,
        nanoseconds_held: u64,
        nanoseconds_held_in_round_trips: u64,
    ) {
        self.nanoseconds_held_in_round_trips
            .fetch_add(nanoseconds_held_in_round_trips, Ordering::Relaxed);
        let mut sites = self
            .of_each_site
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let of_the_site = sites.entry((site.file(), site.line())).or_default();
        of_the_site.nanoseconds_held += nanoseconds_held;
        of_the_site.nanoseconds_held_in_round_trips += nanoseconds_held_in_round_trips;
        of_the_site.acquisitions += 1;
        of_the_site.longest_hold_nanoseconds =
            of_the_site.longest_hold_nanoseconds.max(nanoseconds_held);
    }

    /// 到了下一行该打的时候交回连接开了多久；几条线程同时到点只有抢到的那一条拿到。在取锁之前调，打这一行不算进持锁时长。
    fn time_open_when_a_lock_time_line_is_due(
        &self,
        now: std::time::Instant,
    ) -> Option<std::time::Duration> {
        let time_open = now.duration_since(self.opened_at);
        let next_line_at_second = self.next_line_at_second.load(Ordering::Relaxed);
        if time_open.as_secs() < next_line_at_second {
            return None;
        }
        self.next_line_at_second
            .compare_exchange(
                next_line_at_second,
                time_open.as_secs() + SECONDS_BETWEEN_LIBRARY_LOCK_TIME_LINES,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_line_due_before| time_open)
    }
}

fn nanoseconds_of_a_library_lock_duration(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).expect("一次取锁或持锁的纳秒数装得进 u64：五百多年")
}

thread_local! {
    /// 这条线程此刻握着几把库锁（[`LibraryLockHeld`] 造出来加一、丢掉减一；它不能跨线程）。
    static LIBRARY_LOCKS_HELD_BY_THIS_THREAD: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// 纯计算（排计划、序列化、算摘要、打包、解码）的路上判：这条线程没握着库锁。别的线程握着不算，它们本来就该能同时算。
/// 这几步挪回锁里时当场 panic，不等到量出并行上不去。
///
/// # Panics
/// 这条线程握着库锁。
pub(crate) fn assert_this_thread_holds_no_library_lock(pure_work: &str) {
    let locks_held = LIBRARY_LOCKS_HELD_BY_THIS_THREAD.with(std::cell::Cell::get);
    assert_eq!(
        locks_held, 0,
        "{pure_work}是纯计算，要挪到判定库的锁外：握着锁算，这个进程里别的线程读写判定库都得等它"
    );
}

/// 手里握着的库锁：照 [`VerdictStore`] 用，丢掉时放锁、把握了多久（其中往返多久）记进 [`LibraryLockTimes`]，按取锁的调用点各记一份。
pub struct LibraryLockHeld<'library> {
    store: MutexGuard<'library, VerdictStore>,
    times: &'library LibraryLockTimes,
    acquired_at: std::time::Instant,
    site: &'static std::panic::Location<'static>,
    nanoseconds_in_round_trips_at_acquisition: u64,
}

impl std::ops::Deref for LibraryLockHeld<'_> {
    type Target = VerdictStore;

    fn deref(&self) -> &VerdictStore {
        &self.store
    }
}

impl std::ops::DerefMut for LibraryLockHeld<'_> {
    fn deref_mut(&mut self) -> &mut VerdictStore {
        &mut self.store
    }
}

impl Drop for LibraryLockHeld<'_> {
    fn drop(&mut self) {
        self.times.nanoseconds_held.fetch_add(
            nanoseconds_of_a_library_lock_duration(self.acquired_at.elapsed()),
            Ordering::Relaxed,
        );
        self.times.note_a_hold_at_the_site(
            self.site,
            nanoseconds_of_a_library_lock_duration(self.acquired_at.elapsed()),
            self.store
                .nanoseconds_in_round_trips_so_far()
                .saturating_sub(self.nanoseconds_in_round_trips_at_acquisition),
        );
        LIBRARY_LOCKS_HELD_BY_THIS_THREAD.with(|locks_held| locks_held.set(locks_held.get() - 1));
    }
}

/// 一条连接最多留多少字节读回的值（按读回前的载荷算；解码之后的 Rust 结构大一点，粗估在载荷的两倍以内，推的）：超了从最早读回的放起。
/// E171（三oracle合谋测试）两条流，载荷大的那条约 7.2 MiB，用不到这个数；比这个数还大的流不留，下一次照读。
const MOST_BYTES_OF_FLOWS_HELD_OF_A_LIBRARY: u64 = 512 * 1024 * 1024;

/// 一个摘要一个格子：几条线程同时要同一个摘要时，头一条握着格子去读，其余等它、拿它读回的那一份，不各读一遍。
type SlotOfAValueReadBack<Value> = Arc<Mutex<Option<Arc<Value>>>>;

struct SlotsOfTheValuesReadBack<Value> {
    slot_of_digest: BTreeMap<[u8; DIGEST_BYTES], SlotOfAValueReadBack<Value>>,
    /// 读回过的（摘要，载荷字节数），最早读回的在前。
    held_in_the_order_read: VecDeque<([u8; DIGEST_BYTES], u64)>,
    bytes_held: u64,
}

/// 同一摘要的值只从库里读回一次、留在内存里给这条连接上后面要它的人：内容由摘要定，读回时核过摘要，再读一遍读到的是同样的字节。
/// 只留读回的，不留录入段内存里的那一份（录入与判两段中间只经库），不写结论，不是第二份判定库。
struct ValuesReadBackOnceInThisProcess<Value> {
    most_bytes_held: u64,
    slots: Mutex<SlotsOfTheValuesReadBack<Value>>,
    values_read_over_the_connection: AtomicU64,
}

impl<Value> ValuesReadBackOnceInThisProcess<Value> {
    fn holding_at_most(most_bytes_held: u64) -> Self {
        Self {
            most_bytes_held,
            slots: Mutex::new(SlotsOfTheValuesReadBack {
                slot_of_digest: BTreeMap::new(),
                held_in_the_order_read: VecDeque::new(),
                bytes_held: 0,
            }),
            values_read_over_the_connection: AtomicU64::new(0),
        }
    }

    /// 这个摘要留着就交回留着的；没留着调 `read` 读（交回值与它的载荷字节数）、留下。`read` 没读回值（只录、只判了这台机器的一份、出错）什么都不留，
    /// 下一次照调。同一摘要同时只有一条线程在 `read` 里，它不握着 `slots`，别的摘要不等它。
    ///
    /// # Errors
    /// `read` 的错。
    fn held_or_read_once(
        &self,
        digest: &[u8; DIGEST_BYTES],
        read: impl FnOnce() -> Result<StageOutcome<(Value, u64)>, String>,
    ) -> Result<StageOutcome<Arc<Value>>, String> {
        let slot: SlotOfAValueReadBack<Value> = Arc::clone(
            self.slots
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .slot_of_digest
                .entry(*digest)
                .or_default(),
        );
        let mut value_in_the_slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(value) = &*value_in_the_slot {
            return Ok(StageOutcome::Judged(Arc::clone(value)));
        }
        let (value, payload_bytes) = match read()? {
            StageOutcome::RecordedOnly => return Ok(StageOutcome::RecordedOnly),
            StageOutcome::JudgedMyShareOnly {
                blocks_judged_here,
                blocks_left_to_other_hosts,
            } => {
                return Ok(StageOutcome::JudgedMyShareOnly {
                    blocks_judged_here,
                    blocks_left_to_other_hosts,
                })
            }
            StageOutcome::Judged((value, payload_bytes)) => (Arc::new(value), payload_bytes),
        };
        self.values_read_over_the_connection
            .fetch_add(1, Ordering::Relaxed);
        *value_in_the_slot = Some(Arc::clone(&value));
        drop(value_in_the_slot);
        self.hold_within_the_limit(digest, payload_bytes);
        Ok(StageOutcome::Judged(value))
    }

    /// 记下刚读回的这一份，超了额度从最早读回的放起（它自己比额度还大就连它一起放）。已经拿到手的 [`Arc`] 不受影响，放的只是这里留的那一份。
    fn hold_within_the_limit(&self, digest: &[u8; DIGEST_BYTES], payload_bytes: u64) {
        let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        slots
            .held_in_the_order_read
            .push_back((*digest, payload_bytes));
        slots.bytes_held += payload_bytes;
        // 迭代上界：留着的份数，每一轮放一份
        while slots.bytes_held > self.most_bytes_held {
            let Some((digest_let_go, bytes_let_go)) = slots.held_in_the_order_read.pop_front()
            else {
                break;
            };
            slots.bytes_held -= bytes_let_go;
            slots.slot_of_digest.remove(&digest_let_go);
        }
    }
}

/// 这个进程里开着的判定库：按（连哪一处，键空间）各记一份。同一个进程里几处开同一份（并行跑的几条用例、几类测试）共用这一份连接，
/// 判定、复用、写入都经它，库里的一把锁让它们一次一个读写。
static LIBRARIES_OPEN_IN_THIS_PROCESS: Mutex<
    BTreeMap<KeyOfALibraryOpenInThisProcess, Weak<OpenLibrary>>,
> = Mutex::new(BTreeMap::new());

/// 一处手里的那份库：这个进程里同一份已经开着就共用那一份，没开着才连服务、开库；最后一个放手的关掉连接。
pub struct LibraryOpenInThisProcess {
    key: KeyOfALibraryOpenInThisProcess,
    open: Option<Arc<OpenLibrary>>,
}

impl LibraryOpenInThisProcess {
    /// 开这个进程的那一份（全仓那一份；这个进程设了一次性库就是那一份，[`DisposableLibrary::make_it_the_library_of_this_process`]），
    /// 这个进程里已经开着就共用。全仓那一份连不上、读写不成：打一行 [`PRUNE_SERVER_FAILURE_LINE_PREFIX`]、照 prune-server 的退出码
    /// 退出进程（75 服务不可用），不当成「库里没有」去现判；改连接地址的变量被拒、哪一处都没写服务的地址同样打那一行、退 3
    /// （[`PruneServerRouteRefusal::exit_code`]）；
    /// 一次性库那一份交回错误。
    ///
    /// # Errors
    /// 一次性库那一份连不上、开不了；建不了追加日志的一次性目录；全仓那一份布局不是这一版、判定向量表不合布局。
    pub fn open_or_share() -> Result<Self, String> {
        let chosen = DISPOSABLE_LIBRARY_OF_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let (route, key_space, failure_handling) = match chosen {
            Some(DisposableLibraryChosen { route, key_space }) => {
                (route, key_space, PruneServerFailureHandling::ReturnTheError)
            }
            None => match prune_server_route_of_the_configuration() {
                Ok(route) => (
                    route,
                    LibraryKeySpace::TheOneLibrary,
                    PruneServerFailureHandling::ExitTheProcess,
                ),
                Err(refusal) => {
                    eprintln!("{}", refusal.failure_line());
                    std::process::exit(refusal.exit_code())
                }
            },
        };
        let key = (format!("{route:?}"), key_space.key_prefix().to_vec());
        let mut open_libraries = LIBRARIES_OPEN_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(open) = open_libraries.get(&key).and_then(Weak::upgrade) {
            return Ok(Self {
                key,
                open: Some(open),
            });
        }
        let storage = PruneServerStorage::connect_with_a_journal_of_its_own(
            route,
            failure_handling,
            key_space,
        )?;
        let store = VerdictStore::open_the_prune_server(storage)
            .map_err(|error| format!("开不了 prune-server 里那一份崩溃判定库：{error:?}"))?;
        let open = Arc::new(OpenLibrary {
            connection_name: connection_name_of_an_opening(&key),
            store: Mutex::new(store),
            lock_times: LibraryLockTimes::starting_at_this_opening(),
            flows_read_back: ValuesReadBackOnceInThisProcess::holding_at_most(
                MOST_BYTES_OF_FLOWS_HELD_OF_A_LIBRARY,
            ),
        });
        open_libraries.insert(key.clone(), Arc::downgrade(&open));
        Ok(Self {
            key,
            open: Some(open),
        })
    }

    fn open_library(&self) -> &OpenLibrary {
        self.open
            .as_ref()
            .expect("库在放手之前一直在：只有 Drop 取走它")
    }

    /// 库里那一把锁：一次一个读写。握着它只做与 prune-server 的往返。持锁时长按调用这一处的文件与行记（`#[track_caller]`）。
    #[track_caller]
    pub fn lock(&self) -> LibraryLockHeld<'_> {
        let site = std::panic::Location::caller();
        let open = self.open_library();
        let asked_at = std::time::Instant::now();
        if let Some(time_open) = open
            .lock_times
            .time_open_when_a_lock_time_line_is_due(asked_at)
        {
            open.print_the_lock_time_lines(time_open);
        }
        let askers_with_this_one = open.lock_times.askers_now.fetch_add(1, Ordering::Relaxed) + 1;
        open.lock_times
            .most_askers_at_once
            .fetch_max(askers_with_this_one, Ordering::Relaxed);
        let store = open.store.lock().unwrap_or_else(PoisonError::into_inner);
        open.lock_times.askers_now.fetch_sub(1, Ordering::Relaxed);
        let nanoseconds_in_round_trips_at_acquisition = store.nanoseconds_in_round_trips_so_far();
        let acquired_at = std::time::Instant::now();
        open.lock_times.nanoseconds_waited.fetch_add(
            nanoseconds_of_a_library_lock_duration(acquired_at.duration_since(asked_at)),
            Ordering::Relaxed,
        );
        open.lock_times.acquisitions.fetch_add(1, Ordering::Relaxed);
        LIBRARY_LOCKS_HELD_BY_THIS_THREAD.with(|locks_held| locks_held.set(locks_held.get() + 1));
        LibraryLockHeld {
            store,
            times: &open.lock_times,
            acquired_at,
            site,
            nanoseconds_in_round_trips_at_acquisition,
        }
    }
}

/// 判段与录入段读写判定库经它：每一次读写现取现放（[`Self::take`]），手里不留整段握着的 [`VerdictStore`]，判、看、排计划都在锁外，
/// 同一条连接上别的线程（同一个进程里并行跑的用例、给定状态表的引擎）在这期间照样读写。
pub enum LibraryTakenAtEachReadOrWrite<'library> {
    /// 这个进程里开着的那一份：每次取的是它那一把锁，持锁时长按取锁的调用点记。
    SharedInThisProcess(&'library LibraryOpenInThisProcess),
    /// 调用方独占的一份（单测与实验装置自己开的、不经这个进程登记表的那一份库）：同样一次一取，没有别的连接上的线程等它。
    HeldByThisCallerAlone(LibraryHeldByThisCallerAlone<'library>),
}

/// 调用方独占的那一份库（[`LibraryTakenAtEachReadOrWrite::HeldByThisCallerAlone`]）：一把锁护着库，另记此刻是哪条线程取着
/// （取到之后记上、丢掉时擦掉），同一条线程取着又来取时当场 panic，不等自己。
pub struct LibraryHeldByThisCallerAlone<'library> {
    store: Mutex<&'library mut VerdictStore>,
    thread_taking_it_now: Mutex<Option<std::thread::ThreadId>>,
}

impl<'library> LibraryTakenAtEachReadOrWrite<'library> {
    /// 调用方独占的那一份库照每次读写一取的办法交下去。
    #[must_use]
    pub fn held_by_this_caller_alone(store: &'library mut VerdictStore) -> Self {
        Self::HeldByThisCallerAlone(LibraryHeldByThisCallerAlone {
            store: Mutex::new(store),
            thread_taking_it_now: Mutex::new(None),
        })
    }

    /// 为这一次读写取库：取到的丢掉就放。别的线程取着时等它放。
    ///
    /// # Panics
    /// 调用方独占的那一份（[`Self::HeldByThisCallerAlone`]）由这条线程取着还没丢又来取（同一条语句里取两次，头一次取到的活到语句结束）：
    /// 等下去等的是自己，当场 panic，消息带「同一条语句取了两次库，拆成两条 let」。
    /// 这个进程里开着的那一份（[`Self::SharedInThisProcess`]）不判这一条，同一条线程取两次照旧等自己。
    #[track_caller]
    pub fn take(&self) -> StoreTakenForOneReadOrWrite<'_, 'library> {
        match self {
            Self::SharedInThisProcess(library) => {
                StoreTakenForOneReadOrWrite::UnderTheLockOfTheConnection(library.lock())
            }
            Self::HeldByThisCallerAlone(held) => {
                let this_thread = std::thread::current().id();
                let taken_by_this_thread_already = *held
                    .thread_taking_it_now
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    == Some(this_thread);
                assert!(
                    !taken_by_this_thread_already,
                    "同一条语句取了两次库，拆成两条 let：调用方独占的那一份库这条线程取着还没丢，又来取，等下去等的是自己（{}）",
                    std::panic::Location::caller()
                );
                let store = held.store.lock().unwrap_or_else(PoisonError::into_inner);
                *held
                    .thread_taking_it_now
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner) = Some(this_thread);
                StoreTakenForOneReadOrWrite::HeldByThisCallerAlone(StoreHeldByThisCallerAlone {
                    store,
                    thread_taking_it_now: &held.thread_taking_it_now,
                })
            }
        }
    }
}

/// 为一次读写取到的库（[`LibraryTakenAtEachReadOrWrite::take`]）：照 [`VerdictStore`] 用，丢掉就放。
pub enum StoreTakenForOneReadOrWrite<'taken, 'library> {
    UnderTheLockOfTheConnection(LibraryLockHeld<'taken>),
    HeldByThisCallerAlone(StoreHeldByThisCallerAlone<'taken, 'library>),
}

/// 取到的调用方独占的那一份库：丢掉时先擦掉取着它的线程、再放锁（擦掉之后、放锁之前别的线程来取照旧等锁）。
pub struct StoreHeldByThisCallerAlone<'taken, 'library> {
    store: MutexGuard<'taken, &'library mut VerdictStore>,
    thread_taking_it_now: &'taken Mutex<Option<std::thread::ThreadId>>,
}

impl Drop for StoreHeldByThisCallerAlone<'_, '_> {
    fn drop(&mut self) {
        *self
            .thread_taking_it_now
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
    }
}

impl std::ops::Deref for StoreTakenForOneReadOrWrite<'_, '_> {
    type Target = VerdictStore;

    fn deref(&self) -> &VerdictStore {
        match self {
            Self::UnderTheLockOfTheConnection(held) => held,
            Self::HeldByThisCallerAlone(held) => &held.store,
        }
    }
}

impl std::ops::DerefMut for StoreTakenForOneReadOrWrite<'_, '_> {
    fn deref_mut(&mut self) -> &mut VerdictStore {
        match self {
            Self::UnderTheLockOfTheConnection(held) => held,
            Self::HeldByThisCallerAlone(held) => &mut held.store,
        }
    }
}

impl Drop for LibraryOpenInThisProcess {
    fn drop(&mut self) {
        let mut open_libraries = LIBRARIES_OPEN_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        drop(self.open.take());
        if open_libraries
            .get(&self.key)
            .is_some_and(|open| open.strong_count() == 0)
        {
            open_libraries.remove(&self.key);
        }
    }
}

impl CrashLibrary {
    /// 开这一类的库（这个进程的那一份）。
    ///
    /// # Errors
    /// 同 [`LibraryOpenInThisProcess::open_or_share`]。
    pub fn open(class: CrashLibraryClass) -> Result<Self, String> {
        let library = LibraryOpenInThisProcess::open_or_share()?;
        let library_name = library.lock().library_name().to_string();
        Ok(Self {
            class,
            library_name,
            library,
        })
    }

    #[must_use]
    pub fn class(&self) -> CrashLibraryClass {
        self.class
    }

    /// 库的名字（连的是哪一处、哪一个键空间）：只进行里报的名字。
    #[must_use]
    pub fn library_name(&self) -> &str {
        &self.library_name
    }

    /// 库里那一把锁：一次一个读写。握着它只做与 prune-server 的往返。持锁时长按调用这一处的文件与行记。
    #[track_caller]
    pub fn lock(&self) -> LibraryLockHeld<'_> {
        self.library.lock()
    }

    /// 这一类测试的判段、录入段读写库的那一份：每一次读写现取现放（[`LibraryTakenAtEachReadOrWrite::SharedInThisProcess`]）。
    #[must_use]
    pub fn taken_at_each_read_or_write(&self) -> LibraryTakenAtEachReadOrWrite<'_> {
        LibraryTakenAtEachReadOrWrite::SharedInThisProcess(&self.library)
    }

    /// 这个摘要的流留着就交回留着的（读回过、排过计划的那一份）；没留着调 `read`（交回从库里读回的流连计划与载荷字节数）、留下。
    /// 同一条连接上同一摘要只读回一次。
    ///
    /// # Errors
    /// `read` 的错。
    pub(crate) fn recorded_flow_and_its_plan_read_back_once(
        &self,
        flow_digest: &[u8; DIGEST_BYTES],
        read: impl FnOnce() -> Result<
            StageOutcome<(crate::crash_amplification::RecordedFlowAndItsPlan, u64)>,
            String,
        >,
    ) -> Result<StageOutcome<Arc<crate::crash_amplification::RecordedFlowAndItsPlan>>, String> {
        self.library
            .open_library()
            .flows_read_back
            .held_or_read_once(flow_digest, read)
    }

    /// 这条连接从库里读回过几次流（留着的不算）：单测用。
    #[cfg(test)]
    pub(crate) fn recorded_flows_read_over_the_connection(&self) -> u64 {
        self.library
            .open_library()
            .flows_read_back
            .values_read_over_the_connection
            .load(Ordering::Relaxed)
    }

    /// 这个进程里开着的那一份（给定状态表的引擎按它共用）。
    #[must_use]
    pub fn shared_handle(&self) -> &LibraryOpenInThisProcess {
        &self.library
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash_identity::{ConclusionKind, ConclusionVersion};
    use crate::verdict_store::{
        BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EncodedVerdictVector,
        EnumerationPlanHash, JudgingDevice, NodeInputDigest, VerdictBlock, VerdictBlockKey,
    };

    /// 场景：调用方独占的那一份库，同一条线程取着还没丢又来取（同一条语句里 `(library.take(), library.take())` 那一形）。
    /// 预期：第二次取当场 panic，消息带「同一条语句取了两次库，拆成两条 let」，不等自己。
    /// 两次取放在另起的一条线程上，这条线程限时等它的结局：取不到 panic 而一直等自己时，限时一到这条线程以不带那一句的消息 panic，
    /// 这条判红，不靠测试框架一直等。不用 `std::thread::scope`：一直等自己的那条线程 scope 收尾时也要等，限时就判不出来；它留给进程退出时收。
    /// 判别力（`crates/mutations.tsv`）：「独占库 同一条线程取两次不判、照旧等自己」这一行让这条红（限时到）。
    #[test]
    #[should_panic(expected = "同一条语句取了两次库，拆成两条 let")]
    fn one_thread_taking_the_library_held_alone_twice_panics_at_once_instead_of_waiting_for_itself()
    {
        /// 等那条线程第二次取库 panic 最多等几秒：两次取只是两把进程内的锁，不碰服务。
        const SECONDS_TO_WAIT_FOR_THE_SECOND_TAKE_TO_PANIC: u64 = 30;
        let disposable = DisposableLibrary::new("library-taken-twice-by-one-thread")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let mut store = disposable
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let (outcome_sender, outcome_receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let (first_taken, second_taken) = (library.take(), library.take());
                drop((first_taken, second_taken));
            }));
            // 限时到了收的一方已经走了，发不出去不要紧
            let _sent_or_the_receiver_left = outcome_sender.send(outcome);
        });
        match outcome_receiver.recv_timeout(std::time::Duration::from_secs(
            SECONDS_TO_WAIT_FOR_THE_SECOND_TAKE_TO_PANIC,
        )) {
            Ok(Err(panic_payload)) => std::panic::resume_unwind(panic_payload),
            Ok(Ok(())) => panic!("同一条线程取着还没丢又来取，两次都取到了：独占的那一份库同时交出去两份"),
            Err(waited_out) => panic!(
                "等了 {SECONDS_TO_WAIT_FOR_THE_SECOND_TAKE_TO_PANIC} 秒，同一条线程第二次取库还没 panic，在等自己（{waited_out}）"
            ),
        }
    }

    /// 场景：调用方独占的那一份库，一条线程取着，另一条线程这时来取。
    /// 预期：后来的那条不 panic，等到先取的那条丢掉之后取到（取到时先取的那条已经放手）。
    /// 先取的那条握一秒再放，让后来的那条在这期间来取、等在锁上（推的：一秒之内来得及；来晚了照样取到、照样绿，只是没等过）。
    /// 判别力（`crates/mutations.tsv`）：「独占库 别的线程取着也当成这条线程取着」这一行让这条红（后来的那条 panic）。
    #[test]
    fn a_second_thread_taking_the_library_held_alone_waits_until_the_first_thread_lets_it_go() {
        /// 先取的那条线程握着库多久再放（毫秒）。
        const MILLISECONDS_THE_FIRST_THREAD_HOLDS_THE_LIBRARY: u64 = 1_000;
        let disposable = DisposableLibrary::new("library-taken-in-turn-by-two-threads")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let mut store = disposable
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let library_held_alone =
            LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
        let library = &library_held_alone;
        let first_thread_let_it_go = &std::sync::atomic::AtomicBool::new(false);
        let (first_taken_sender, first_taken_receiver) = mpsc::channel::<()>();
        std::thread::scope(|scope| {
            let first_thread = scope.spawn(move || {
                let first_taken = library.take();
                first_taken_sender
                    .send(())
                    .expect("后来的那条线程在等先取的那条取到");
                std::thread::sleep(std::time::Duration::from_millis(
                    MILLISECONDS_THE_FIRST_THREAD_HOLDS_THE_LIBRARY,
                ));
                first_thread_let_it_go.store(true, Ordering::SeqCst);
                drop(first_taken);
            });
            let second_thread = scope.spawn(move || {
                first_taken_receiver
                    .recv()
                    .expect("先取的那条线程取到了才发这一声");
                let second_taken = library.take();
                assert!(
                    first_thread_let_it_go.load(Ordering::SeqCst),
                    "后来的那条线程取到库时，先取的那条已经放手"
                );
                drop(second_taken);
            });
            first_thread
                .join()
                .expect("先取的那条线程取库、放库不 panic");
            second_thread
                .join()
                .expect("后来的那条线程取库不 panic：别的线程取着时照旧等它放");
        });
    }

    fn a_block_key() -> VerdictBlockKey {
        VerdictBlockKey {
            node_input_digest: NodeInputDigest([3; DIGEST_BYTES]),
            enumeration_plan_hash: EnumerationPlanHash([4; DIGEST_BYTES]),
            block_start: BlockStartStateIndex(0),
        }
    }

    fn one_clean_state(store: &mut VerdictStore) -> ConclusionBlockRecord {
        let clean = store
            .register_verdict_vector(&EncodedVerdictVector(Vec::new()))
            .expect("登记得了判定向量");
        ConclusionBlockRecord {
            block: VerdictBlock::new(vec![clean]).expect("一块一个状态"),
            violating_states: Vec::new(),
            count_vector: BlockCountVector::default(),
            judged_on: JudgingDevice::Cpu,
            recovery_words: None,
        }
    }

    /// 一份库、类当索引：几类测试开的是这个进程的同一份（同一个进程里共用一份连接），库的名字不带类名；
    /// 同一块的 GPU 与 CPU 两种结论在这一份库里是同一块下的两条记录、各自读得回。只对临时服务里的一次性库跑。
    /// 判别力：`LibraryOpenInThisProcess::open_or_share` 不看这个进程设的一次性库（照开全仓那一份），这条红在列结论那一步（读到别处的行或连不上）。
    #[test]
    fn every_class_opens_the_one_library_and_two_conclusion_kinds_of_one_block_are_two_records() {
        // 一次性库留着：自证一份库、类当索引：要一份空的临时服务
        let disposable =
            DisposableLibrary::new("one-library").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let amplification =
            CrashLibrary::open(CrashLibraryClass::CrashAmplification).expect("开得了库");
        let enumeration =
            CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration).expect("开得了库");
        let given_states = CrashLibrary::open(CrashLibraryClass::GivenStates).expect("开得了库");
        assert_eq!(amplification.library_name(), enumeration.library_name());
        assert_eq!(amplification.library_name(), given_states.library_name());
        for class in [
            CrashLibraryClass::CrashAmplification,
            CrashLibraryClass::CrashStateEnumeration,
            CrashLibraryClass::GivenStates,
            CrashLibraryClass::JudgeTablesPairing,
            CrashLibraryClass::FactsPairing,
            CrashLibraryClass::InputPairing,
        ] {
            assert!(
                !amplification.library_name().contains(class.name()),
                "库的名字不带类名 {}",
                class.name()
            );
        }
        let key = a_block_key();
        let gpu_version = ConclusionVersion([5; DIGEST_BYTES]);
        let cpu_version = ConclusionVersion([6; DIGEST_BYTES]);
        {
            let mut store = amplification.lock();
            let record = one_clean_state(&mut store);
            store
                .store_conclusion_block(
                    &key,
                    &ConclusionKind::ImageJudgement,
                    &gpu_version,
                    &record,
                )
                .expect("存 GPU 结论");
        }
        {
            let mut store = enumeration.lock();
            let record = one_clean_state(&mut store);
            store
                .store_conclusion_block(
                    &key,
                    &ConclusionKind::ImageJudgement,
                    &cpu_version,
                    &record,
                )
                .expect("存 CPU 结论");
        }
        let store = given_states.lock();
        assert_eq!(
            store.stored_conclusion_keys().expect("列结论"),
            vec![
                (key.clone(), ConclusionKind::ImageJudgement, gpu_version),
                (key.clone(), ConclusionKind::ImageJudgement, cpu_version),
            ],
            "同一批状态只存一处：一块两种结论两条记录"
        );
    }

    /// 几条线程同时要同一个摘要：只读一次，都拿到读回的那一份；之后再要也不再读。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 同一摘要每次各开一个格子」这一行让这一条红（各读各的）。
    #[test]
    fn several_threads_asking_for_one_digest_read_it_once_and_share_the_value() {
        let values = ValuesReadBackOnceInThisProcess::<u32>::holding_at_most(1024);
        let reads = AtomicU64::new(0);
        let threads_asking = 4;
        let start_together = std::sync::Barrier::new(threads_asking);
        let ask = || {
            values
                .held_or_read_once(&[1; DIGEST_BYTES], || {
                    reads.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    Ok(StageOutcome::Judged((7u32, 8u64)))
                })
                .expect("读得回")
                .judged()
                .map(|value| *value)
                .expect("读回了值")
        };
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads_asking)
                .map(|_thread| {
                    scope.spawn(|| {
                        start_together.wait();
                        ask()
                    })
                })
                .collect();
            for handle in handles {
                assert_eq!(handle.join().expect("线程不 panic"), 7);
            }
        });
        assert_eq!(ask(), 7);
        assert_eq!(reads.load(Ordering::SeqCst), 1, "同一个摘要只读一次");
    }

    /// 留的字节超了额度，从最早读回的放起；比额度还大的不留。放掉的下一次照读，留着的不再读。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 读回的流超额也不放」这一行让这一条红（放掉的那一份还留着，不再读）。
    #[test]
    fn values_read_back_beyond_the_limit_are_let_go_from_the_earliest() {
        let values = ValuesReadBackOnceInThisProcess::<u32>::holding_at_most(10);
        let reads = AtomicU64::new(0);
        let ask = |digest_byte: u8, payload_bytes: u64| {
            let value_read_or_held = values
                .held_or_read_once(&[digest_byte; DIGEST_BYTES], || {
                    reads.fetch_add(1, Ordering::SeqCst);
                    Ok(StageOutcome::Judged((
                        u32::from(digest_byte),
                        payload_bytes,
                    )))
                })
                .expect("读得回")
                .judged()
                .expect("读回了值");
            assert_eq!(*value_read_or_held, u32::from(digest_byte));
            reads.load(Ordering::SeqCst)
        };
        assert_eq!(ask(1, 6), 1, "头一份现读");
        assert_eq!(ask(1, 6), 1, "还留着");
        assert_eq!(ask(2, 6), 2, "第二份现读，超了额度放掉头一份");
        assert_eq!(ask(2, 6), 2, "第二份还留着");
        assert_eq!(ask(1, 6), 3, "头一份放掉了，照读");
        assert_eq!(ask(3, 11), 4, "比额度还大的现读");
        assert_eq!(ask(3, 11), 5, "比额度还大的不留，照读");
    }

    /// 握着库锁的线程走到纯计算当场 panic；别的线程握着不算，放了锁也不算。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 纯计算的断言不判」这一行让这一条红。
    #[test]
    fn the_pure_work_assertion_fires_only_on_the_thread_holding_the_library_lock() {
        // 一次性库留着：要一把真的库锁
        let disposable = DisposableLibrary::new("pure-work-assertion")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
        let held = library.lock();
        assert!(
            std::panic::catch_unwind(|| assert_this_thread_holds_no_library_lock("单测的纯计算"))
                .is_err(),
            "握着锁的这条线程上要 panic"
        );
        std::thread::scope(|scope| {
            scope
                .spawn(|| assert_this_thread_holds_no_library_lock("单测的纯计算"))
                .join()
                .expect("别的线程没握着锁，不 panic");
        });
        drop(held);
        assert_this_thread_holds_no_library_lock("单测的纯计算");
    }

    /// 从持锁时长行里取一个字段的数。
    fn field_of_the_lock_time_line(line: &str, field: &str) -> f64 {
        line.split(' ')
            .find_map(|word| word.strip_prefix(field)?.strip_prefix('='))
            .unwrap_or_else(|| panic!("持锁时长行里没有 {field}：{line}"))
            .parse()
            .expect("字段是数")
    }

    /// 一条线程握着锁 30 毫秒以上，另一条等它：行里持锁秒数、等锁秒数、取锁次数都记上，持锁占比是持锁秒数除以连接开着的秒数。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 放锁时不记持锁时长」这一行让这一条红。
    #[test]
    fn the_lock_time_line_adds_up_what_the_lock_was_held_and_waited() {
        // 一次性库留着：要一把真的库锁，从这条连接开着起数
        let disposable =
            DisposableLibrary::new("lock-time-line").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
        let time_held = std::time::Duration::from_millis(30);
        let the_other_thread_is_about_to_ask = std::sync::atomic::AtomicBool::new(false);
        std::thread::scope(|scope| {
            let held = library.lock();
            let other_thread = scope.spawn(|| {
                the_other_thread_is_about_to_ask.store(true, Ordering::SeqCst);
                drop(library.lock());
            });
            // 迭代上界：另一条线程起来就设它，起不来 join 那里也会报
            while !the_other_thread_is_about_to_ask.load(Ordering::SeqCst) {
                std::thread::yield_now();
            }
            std::thread::sleep(time_held);
            drop(held);
            other_thread.join().expect("另一条线程不 panic");
        });
        let line = library
            .open_library()
            .lock_time_line(std::time::Duration::from_millis(100));
        assert!(line.starts_with(LIBRARY_LOCK_TIME_LINE_PREFIX), "{line}");
        let held = field_of_the_lock_time_line(&line, "seconds_held");
        assert!(held >= time_held.as_secs_f64(), "持锁秒数没记全：{line}");
        assert!(
            field_of_the_lock_time_line(&line, "seconds_waited") >= time_held.as_secs_f64() / 2.0,
            "另一条线程等的没记上：{line}"
        );
        assert!(
            field_of_the_lock_time_line(&line, "acquisitions") >= 2.0,
            "两次取锁都数：{line}"
        );
        assert!(
            (field_of_the_lock_time_line(&line, "held_share_of_the_wall_clock") - held / 0.1).abs()
                < 0.02,
            "持锁占比是持锁秒数除以连接开着的秒数：{line}"
        );
    }

    /// 持锁时长行隔 60 秒一行：不到点不打，到点打一次，同一个点不打第二次，下一行从这一次到点起再隔 60 秒。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 持锁时长行不看到没到点」这一行让这一条红。
    #[test]
    fn the_lock_time_line_comes_once_per_interval() {
        let times = LibraryLockTimes::starting_at_this_opening();
        let at_second = |second: u64| times.opened_at + std::time::Duration::from_secs(second);
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(59)),
            None
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(60)),
            Some(std::time::Duration::from_secs(60))
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(60)),
            None
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(119)),
            None
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(120)),
            Some(std::time::Duration::from_secs(120))
        );
    }

    /// 一条线程取锁、在锁里读一次库（与 prune-server 往返一次）再放：这一次持锁记在取锁的那一处（文件与行）名下，锁里往返的纳秒数大于零、
    /// 不超过持锁的纳秒数；连接那一行与各调用点那几行都带连接名（`connection=`）与进程号（`pid=`），调用点那一行点名这一处。
    /// 同一个进程里关了再开同一份库，两次的连接名不同（各自的 `seconds_open` 各从零数，按连接名取最后一行不混）。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁时长 锁里的往返不记」这一行让这一条红。
    #[test]
    fn the_lock_time_lines_name_the_connection_and_the_site_with_the_round_trips_in_the_lock() {
        // 一次性库留着：要一把真的库锁与真的往返，从这条连接开着起数
        let disposable = DisposableLibrary::new("lock-time-lines-of-the-sites")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
        let (held, line_of_the_lock) = (library.lock(), line!());
        assert!(
            held.read_flow_facts(&[0; DIGEST_BYTES])
                .expect("读得了库")
                .is_none(),
            "空库里没有这一份事实表"
        );
        drop(held);
        let open = library.open_library();
        let held_in_round_trips = open
            .lock_times
            .nanoseconds_held_in_round_trips
            .load(Ordering::Relaxed);
        assert!(
            held_in_round_trips > 0
                && held_in_round_trips <= open.lock_times.nanoseconds_held.load(Ordering::Relaxed),
            "锁里往返的纳秒数大于零、不超过持锁的纳秒数：{held_in_round_trips}"
        );
        {
            let sites = open
                .lock_times
                .of_each_site
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let of_the_site = sites
                .get(&(file!(), line_of_the_lock))
                .expect("这一次持锁记在取锁的那一处名下");
            assert!(
                of_the_site.acquisitions == 1
                    && of_the_site.nanoseconds_held_in_round_trips > 0
                    && of_the_site.nanoseconds_held_in_round_trips <= of_the_site.nanoseconds_held,
                "那一处取了一次锁、锁里往返的纳秒数大于零而不超过持锁的纳秒数"
            );
        }
        let connection_and_process = format!(
            "connection={} pid={}",
            open.connection_name,
            std::process::id()
        );
        let line = open.lock_time_line(std::time::Duration::from_millis(100));
        assert!(
            line.starts_with(&format!(
                "{LIBRARY_LOCK_TIME_LINE_PREFIX} {connection_and_process} "
            )),
            "{line}"
        );
        let site = format!("site={}:{line_of_the_lock} ", file!());
        assert!(
            open.lock_time_lines_of_the_sites().iter().any(|site_line| {
                site_line.starts_with(&format!(
                    "{LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX} {connection_and_process} "
                )) && site_line.contains(&site)
            }),
            "调用点那几行里有点名 {site} 的一行：{:#?}",
            open.lock_time_lines_of_the_sites()
        );
        let connection_name_of_the_first_opening = open.connection_name.clone();
        drop(library);
        let reopened = LibraryOpenInThisProcess::open_or_share().expect("再开得了库");
        assert_ne!(
            reopened.open_library().connection_name,
            connection_name_of_the_first_opening,
            "关了再开，连接名换一个"
        );
    }
}
