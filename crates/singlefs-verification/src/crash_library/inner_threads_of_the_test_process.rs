//! 单测进程里的线程额度：每台默认重活线程数分成并跑的用例与它们里层各一份，临时服务的名额，取了不还就等。

#[cfg(test)]
use super::*;

/// 单测进程里用例自己开的线程共用的那一份额度分成的两份（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。
/// libtest 已经照每台默认重活线程数一起跑那么多条用例，用例里再开的线程不再各照这个数开：一次性库的临时服务同时至多
/// `temporary_servers` 份（每份照 [`THREADS_OF_A_TEMPORARY_SERVER`] 条记，至多占一半），余下的 `inner_threads` 条给里层的核对线程
/// （[`InnerThreadsTaken`]），至少 [`FEWEST_INNER_THREADS_OF_THE_SHARE`] 条。两份合起来不超过每台默认重活线程数（它不小于
/// [`THREADS_OF_A_TEMPORARY_SERVER`] 加 [`FEWEST_INNER_THREADS_OF_THE_SHARE`] 时；更小时两份的下限合起来就超过它）。
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SharesOfTheTestProcess {
    pub(crate) temporary_servers: usize,
    pub(crate) inner_threads: usize,
}

/// 一份临时服务常驻的线程（接 socket、接 TCP、提交各一条，加工作线程）与一条连接的两条（读帧、写回），
/// 照 `prune_server::server` 在这个进程里起服务的那几处数。
#[cfg(test)]
pub(crate) const THREADS_OF_A_TEMPORARY_SERVER: usize =
    3 + prune_server::server::WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS + 2;

/// 里层那一份至少几条：一处里层并行（一块切成几段、一段一条线程）至少切得成两段。每台默认重活线程数照分到的份额给、
/// 小到装不下一份临时服务再加这么多条时（合入后验证几个测试目标并在一片里跑，份额 6 这类），里层照样给这么多条，
/// 不按扣掉临时服务之后的余数给 1 条。
#[cfg(test)]
pub(crate) const FEWEST_INNER_THREADS_OF_THE_SHARE: usize = 2;

/// 每台默认重活线程数 `heavy_threads` 分成的两份（[`SharesOfTheTestProcess`]）：临时服务至少一份（不然一次性库永远起不来），
/// 里层至少 [`FEWEST_INNER_THREADS_OF_THE_SHARE`] 条。
#[cfg(test)]
pub(crate) fn shares_of_the_test_process(heavy_threads: usize) -> SharesOfTheTestProcess {
    let temporary_servers = (heavy_threads / 2 / THREADS_OF_A_TEMPORARY_SERVER).max(1);
    SharesOfTheTestProcess {
        temporary_servers,
        inner_threads: heavy_threads
            .saturating_sub(temporary_servers * THREADS_OF_A_TEMPORARY_SERVER)
            .max(FEWEST_INNER_THREADS_OF_THE_SHARE),
    }
}

/// 一处里层并行要 `wanted` 条线程时拿到几条：不过里层那一份 `inner_threads_of_the_share`，至少一条。要么整份拿到、要么等，不拆开拿：
/// 同一处每次拿到的条数一样，切段的界不随别的用例此刻占了多少变。
#[cfg(test)]
pub(crate) fn inner_threads_granted(wanted: usize, inner_threads_of_the_share: usize) -> usize {
    wanted.min(inner_threads_of_the_share).max(1)
}

/// 此刻里层已占 `in_use` 条时，再拿 `granted` 条放不放得下里层那一份 `inner_threads_of_the_share`。
#[cfg(test)]
pub(crate) fn inner_threads_fit(
    in_use: usize,
    granted: usize,
    inner_threads_of_the_share: usize,
) -> bool {
    in_use + granted <= inner_threads_of_the_share
}

/// 单测进程此刻占着的：里层几条线程、几份临时服务，与哪条线程起着几份临时服务（同一条线程再起一份不再等：
/// 它手里那一份要等它自己往下走才还，等就是自己等自己）。
#[cfg(test)]
struct ThreadsInUseInTheTestProcess {
    inner_threads: usize,
    temporary_servers: usize,
    temporary_servers_started_by_each_thread: Vec<(std::thread::ThreadId, usize)>,
}

#[cfg(test)]
static THREADS_IN_USE_IN_THE_TEST_PROCESS: Mutex<ThreadsInUseInTheTestProcess> =
    Mutex::new(ThreadsInUseInTheTestProcess {
        inner_threads: 0,
        temporary_servers: 0,
        temporary_servers_started_by_each_thread: Vec::new(),
    });

/// 还了线程或名额就叫醒等着的。
#[cfg(test)]
static THREADS_RETURNED_IN_THE_TEST_PROCESS: std::sync::Condvar = std::sync::Condvar::new();

#[cfg(test)]
fn shares_of_this_test_process() -> SharesOfTheTestProcess {
    shares_of_the_test_process(crate::heavy_threads::default_heavy_threads().get())
}

/// 里层一处并行拿到的核对线程：产品构建里就是要的那么多条；单测构建里从单测进程的那一份额度拿（`SharesOfTheTestProcess`），
/// 拿不到就等，丢掉时还回去。
pub(crate) struct InnerThreadsTaken {
    threads: usize,
}

impl InnerThreadsTaken {
    /// 要 `wanted` 条（至少一条）：产品构建里照给。
    #[cfg(not(test))]
    pub(crate) fn take(wanted: usize) -> Self {
        Self { threads: wanted }
    }

    /// 要 `wanted` 条：单测构建里照 [`inner_threads_granted`] 定拿几条，里层那一份放不下就等别的用例还。
    #[cfg(test)]
    pub(crate) fn take(wanted: usize) -> Self {
        let inner_threads_of_the_share = shares_of_this_test_process().inner_threads;
        let threads = inner_threads_granted(wanted, inner_threads_of_the_share);
        let mut in_use = THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // 迭代上界是别的用例还线程的次数：每醒一次重判一次
        while !inner_threads_fit(in_use.inner_threads, threads, inner_threads_of_the_share) {
            in_use = THREADS_RETURNED_IN_THE_TEST_PROCESS
                .wait(in_use)
                .unwrap_or_else(PoisonError::into_inner);
        }
        in_use.inner_threads += threads;
        Self { threads }
    }

    pub(crate) fn threads(&self) -> usize {
        self.threads
    }
}

#[cfg(test)]
impl Drop for InnerThreadsTaken {
    fn drop(&mut self) {
        THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .inner_threads -= self.threads;
        THREADS_RETURNED_IN_THE_TEST_PROCESS.notify_all();
    }
}

/// 单测构建里一份一次性库的临时服务占的名额（[`SharesOfTheTestProcess`] 的 `temporary_servers`）：起之前拿，满了就等；
/// 同一条线程已经起着一份的不再等、不另占。丢掉时那条线程起的都丢了才还。
/// 次序：先取名额（开一次性库）、后拿改环境的锁（`crate::crash_amplification::EnvironmentLockOfThisTestBinary`）。
#[cfg(test)]
pub(super) struct TemporaryServerPlaceInTheTestProcess {
    started_by: std::thread::ThreadId,
}

/// 拿着改环境的锁去新占一次性库名额时 panic 的那句话。
#[cfg(test)]
const PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK: &str =
    "这条线程拿着改环境的锁去新占一次性库名额：次序要先开一次性库（取名额）、后拿改环境的锁（CrashAmplificationEnvironment、\
     one_test_changes_the_environment_at_a_time）；反过来拿，名额满着时拿着名额等锁的用例与拿着锁等名额的用例互相等、整个测试进程挂住。\
     把开库挪到拿锁之前";

#[cfg(test)]
impl TemporaryServerPlaceInTheTestProcess {
    /// # Panics
    /// 要新占名额（这条线程还没起着一份）而这条线程拿着改环境的锁（[`PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK`]）：
    /// 在等名额之前当场 panic，不挂住。这条线程已经起着一份的不等名额、不查（拿着锁再开一份不会互等）。
    pub(super) fn take() -> Self {
        let started_by = std::thread::current().id();
        let temporary_servers_of_the_share = shares_of_this_test_process().temporary_servers;
        let mut in_use = THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some((_thread, started)) = in_use
            .temporary_servers_started_by_each_thread
            .iter_mut()
            .find(|(thread, _started)| *thread == started_by)
        {
            *started += 1;
            return Self { started_by };
        }
        if crate::crash_amplification::this_thread_holds_the_environment_lock() {
            // 先放占用计数的锁再 panic：不让它中毒，别的用例照常取还
            drop(in_use);
            panic!("{PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK}");
        }
        // 迭代上界是别的用例还名额的次数：每醒一次重判一次
        while in_use.temporary_servers >= temporary_servers_of_the_share {
            in_use = THREADS_RETURNED_IN_THE_TEST_PROCESS
                .wait(in_use)
                .unwrap_or_else(PoisonError::into_inner);
        }
        in_use.temporary_servers += 1;
        in_use
            .temporary_servers_started_by_each_thread
            .push((started_by, 1));
        Self { started_by }
    }
}

#[cfg(test)]
impl Drop for TemporaryServerPlaceInTheTestProcess {
    fn drop(&mut self) {
        let mut in_use = THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let position = in_use
            .temporary_servers_started_by_each_thread
            .iter()
            .position(|(thread, _started)| *thread == self.started_by)
            .expect("名额拿到时记下了起它的那条线程，那条线程起的都丢了才删这一项");
        in_use.temporary_servers_started_by_each_thread[position].1 -= 1;
        if in_use.temporary_servers_started_by_each_thread[position].1 == 0 {
            in_use
                .temporary_servers_started_by_each_thread
                .swap_remove(position);
            in_use.temporary_servers -= 1;
        }
        drop(in_use);
        THREADS_RETURNED_IN_THE_TEST_PROCESS.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 场景：一条新线程先拿改环境的锁（`CrashAmplificationEnvironment::cleared_then_set`）、再开一次性库（反着的次序）。
    /// 预期：开库取名额那一步在等名额之前当场 panic（名额满着也不挂住），消息写明先开一次性库、后拿改环境的锁；
    /// 那条线程 panic 之后锁已放，这条线程接着拿得到。
    /// 判别力（`crates/mutations.tsv`）：取名额不问这条线程拿没拿着改环境的锁（那条线程取到名额、起了临时服务，不 panic），这条红。
    #[test]
    fn taking_a_disposable_library_place_while_holding_the_environment_lock_panics_at_once_naming_the_order(
    ) {
        let reversed_order = std::thread::spawn(|| {
            let _environment =
                crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
            // 一次性库留着：自证反着的次序当场 panic（次序对了这一步取名额、起临时服务）
            DisposableLibrary::new("place-taken-while-holding-the-environment-lock")
                .map(|_library| ())
        })
        .join();
        let panic_payload =
            reversed_order.expect_err("拿着改环境的锁去开一次性库，取名额那一步当场 panic");
        let panic_message = panic_payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                panic_payload
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_string())
            })
            .expect("assert! 带格式串的 panic 交回 String");
        assert_eq!(
            panic_message, PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK,
            "panic 的就是取名额那一步，消息写明先开库、后拿锁"
        );
        assert!(
            !crate::crash_amplification::this_thread_holds_the_environment_lock(),
            "前提：这条线程没拿改环境的锁"
        );
        let _environment_after_the_panic =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
    }

    /// 场景：这条线程先开一份一次性库（新占一个名额），再拿改环境的锁，拿着锁再开第二份。
    /// 预期：第二份不等名额、不查次序（同一条线程已经起着一份，拿着锁再开不会互等），开得成、不 panic；两份是两段名字空间。
    /// 判别力（`crates/mutations.tsv`）：取名额那一步把「拿着改环境的锁」的检查挪回开头（这条线程已经起着一份的也查），这条红。
    #[test]
    fn a_thread_already_holding_a_disposable_library_place_opens_another_while_holding_the_environment_lock(
    ) {
        // 一次性库留着：自证这条线程起着一份之后拿着改环境的锁再开一份不查次序
        let library_opened_before_the_lock =
            DisposableLibrary::new("place-held-before-the-environment-lock")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let environment =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
        assert!(
            crate::crash_amplification::this_thread_holds_the_environment_lock(),
            "前提：开第二份时这条线程拿着改环境的锁"
        );
        // 一次性库留着：拿着改环境的锁开的第二份，自证不等名额、不 panic
        let library_opened_under_the_lock =
            DisposableLibrary::new("second-place-under-the-environment-lock")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        assert_ne!(
            library_opened_before_the_lock.key_space(),
            library_opened_under_the_lock.key_space(),
            "两份一次性库两段名字空间"
        );
        drop(environment);
        drop(library_opened_under_the_lock);
        drop(library_opened_before_the_lock);
    }

    /// 场景：每台默认重活线程数从一份临时服务加里层下限（[`FEWEST_INNER_THREADS_OF_THE_SHARE`]）起到 124，外层 1 条到那么多条用例同时跑，
    /// 每条里层要的条数取 1、2、每台默认重活线程数与它的两倍（核对线程变量设得比它大）几样，照拿线程的算术（[`shares_of_the_test_process`]、
    /// [`inner_threads_granted`]、[`inner_threads_fit`]）一条接一条拿、放得下才拿。比这个起点小的，两份的下限合起来就超过每台默认重活线程数，
    /// 不在这一条的射程（里层下限那一侧由 `the_inner_share_grants_at_least_two_threads_at_every_heavy_thread_default` 钉）。
    /// 预期：每条拿到的不过里层那一份、至少一条；一起拿到的合起来不过里层那一份；里层那一份加临时服务那一份不过每台默认重活线程数。
    /// 判别力（`crates/mutations.tsv`）：里层拿的不照那一份封顶，这条红；放不放得下不看已占的，这条红。
    #[test]
    fn inner_threads_of_the_cases_running_together_stay_within_the_heavy_thread_default() {
        for heavy_threads in THREADS_OF_A_TEMPORARY_SERVER + FEWEST_INNER_THREADS_OF_THE_SHARE..=124
        {
            let shares = shares_of_the_test_process(heavy_threads);
            assert!(
                shares.temporary_servers >= 1
                    && shares.temporary_servers * THREADS_OF_A_TEMPORARY_SERVER
                        + shares.inner_threads
                        <= heavy_threads,
                "每台默认 {heavy_threads} 条：临时服务至少一份，两份合起来不过它：{shares:?}"
            );
            for wanted in [1, 2, heavy_threads, 2 * heavy_threads] {
                let granted = inner_threads_granted(wanted, shares.inner_threads);
                assert!(
                    (1..=shares.inner_threads).contains(&granted),
                    "每台默认 {heavy_threads} 条、要 {wanted} 条：拿到的 {granted} 条在 1 到里层那一份 {} 条之间",
                    shares.inner_threads
                );
                for cases_running_together in 1..=heavy_threads {
                    let mut in_use = 0;
                    for _case in 0..cases_running_together {
                        if inner_threads_fit(in_use, granted, shares.inner_threads) {
                            in_use += granted;
                        }
                    }
                    assert!(
                        in_use <= shares.inner_threads && in_use <= heavy_threads,
                        "每台默认 {heavy_threads} 条、外层 {cases_running_together} 条各要 {wanted} 条：一起拿到的 {in_use} 条不过里层那一份 {} 条",
                        shares.inner_threads
                    );
                }
            }
        }
    }

    /// 场景：外层 8 条线程同时各要每台默认重活线程数那么多条里层线程（[`InnerThreadsTaken::take`]，这个单测进程用的那一份额度），拿到后各持 20 毫秒。
    /// 预期：每条拿到的都是里层那一份封顶后的条数；这 8 条同一时刻持着的合起来不过里层那一份；8 条都拿到过（还了的叫得醒等着的）。
    /// 别的用例同时也在拿：它们占的只让这 8 条多等，这 8 条合起来的条数照样不过那一份。
    /// 判别力（`crates/mutations.tsv`）：放不放得下不看已占的，这条红。
    #[test]
    fn threads_taken_by_cases_running_together_never_add_up_past_the_inner_share() {
        const CASES_RUNNING_TOGETHER: usize = 8;
        let heavy_threads = crate::heavy_threads::default_heavy_threads().get();
        let inner_threads_of_the_share = shares_of_the_test_process(heavy_threads).inner_threads;
        let held_now = std::sync::atomic::AtomicUsize::new(0);
        let most_held_at_once = std::sync::atomic::AtomicUsize::new(0);
        let granted_of_each_case: Vec<usize> = std::thread::scope(|scope| {
            let cases: Vec<_> = (0..CASES_RUNNING_TOGETHER)
                .map(|_case| {
                    scope.spawn(|| {
                        let taken = InnerThreadsTaken::take(heavy_threads);
                        let held =
                            held_now.fetch_add(taken.threads(), Ordering::SeqCst) + taken.threads();
                        most_held_at_once.fetch_max(held, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_millis(20));
                        held_now.fetch_sub(taken.threads(), Ordering::SeqCst);
                        taken.threads()
                    })
                })
                .collect();
            cases
                .into_iter()
                .map(|case| case.join().expect("拿线程的那条不 panic"))
                .collect()
        });
        assert_eq!(
            granted_of_each_case,
            vec![
                inner_threads_granted(heavy_threads, inner_threads_of_the_share);
                CASES_RUNNING_TOGETHER
            ],
            "每条拿到的都是里层那一份封顶后的条数"
        );
        assert!(
            most_held_at_once.load(Ordering::SeqCst) <= inner_threads_of_the_share,
            "同一时刻持着的 {} 条不过里层那一份 {inner_threads_of_the_share} 条",
            most_held_at_once.load(Ordering::SeqCst)
        );
    }

    /// 场景：每台默认重活线程数从 1 到 64 逐个取（合入后验证把几个测试目标并在一片里跑时，这个数照分到的份额给，会小到 6 这类），
    /// 里层一处并行要 2 条、6 条、31 条，照 [`shares_of_the_test_process`] 与 [`inner_threads_granted`] 算拿到几条。
    /// 预期：每一样都至少拿到 2 条，一块才切得成两段、各一条线程（`crash_amplification` 里 CPU 判段与观察者录入段那两条
    /// 「一块的各核对线程共用一份枚举表」的单测靠这个）；里层那一份的两个绝对值另外算出来钉住：每台默认 6 条时是 2 条
    /// （6 条扣掉一份临时服务的 7 条不剩，取下限），31 条时是 17 条（31 − 2 份临时服务 × 7 条）。
    /// 判别力（`crates/mutations.tsv`）：「单测进程线程额度：里层那一份下限改回 1 条」这一行让这一条红。
    #[test]
    fn the_inner_share_grants_at_least_two_threads_at_every_heavy_thread_default() {
        const PIECES_A_BLOCK_IS_SPLIT_INTO_AT_LEAST: usize = 2;
        for heavy_threads in 1..=64 {
            let shares = shares_of_the_test_process(heavy_threads);
            for wanted in [2, 6, 31] {
                let granted = inner_threads_granted(wanted, shares.inner_threads);
                assert!(
                    granted >= PIECES_A_BLOCK_IS_SPLIT_INTO_AT_LEAST,
                    "每台默认 {heavy_threads} 条、里层要 {wanted} 条：拿到 {granted} 条，一块切不成两段（{shares:?}）"
                );
            }
        }
        assert_eq!(
            shares_of_the_test_process(6),
            SharesOfTheTestProcess {
                temporary_servers: 1,
                inner_threads: 2,
            },
            "每台默认 6 条：临时服务一份，里层取下限 2 条"
        );
        assert_eq!(
            shares_of_the_test_process(31),
            SharesOfTheTestProcess {
                temporary_servers: 2,
                inner_threads: 17,
            },
            "每台默认 31 条：临时服务两份（31 / 2 / 7 向下取整），里层 31 − 2 × 7 = 17 条"
        );
    }
}
