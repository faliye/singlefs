//! 一段的缓冲与两份回读在同一个显存不足作用域里建：显存不足交回建不成、不 panic，等作用域时读调用方在停的标记。

use super::*;

/// 在内存里建的一份缓冲：建的时候要了多少字节，建没建成（显存不足就没建成）。
#[derive(Debug, PartialEq, Eq)]
struct BufferInMemoryForTests {
    bytes_requested: u64,
    is_created: bool,
}

/// 建一段缓冲的设备替身：卡上此刻空闲 `free_bytes`，每建一份照它扣，不够就建不成、记一条显存不足。记法照 wgpu 的错误范围：
/// 有范围开着时那条错归最里层那一个范围（每个范围只留它接住的第一条），一个范围都没开着时记成没接住的错（真卡上那条错走设备的没接住错误回调、
/// 这一批报 `UncapturedErrorOrDeviceLost`）。另记卡上被本包占着的最多那一刻的字节数（`peak_bytes_held`）。
struct DeviceOfAChunkInMemoryForTests {
    free_bytes: Cell<u64>,
    bytes_held: Cell<u64>,
    peak_bytes_held: Cell<u64>,
    out_of_memory_captured_by_each_open_scope: std::cell::RefCell<Vec<Option<String>>>,
    uncaptured_out_of_memory: std::cell::RefCell<Vec<String>>,
    closed_scope_answers: ClosedScopeAnswersOfTheDeviceForTests,
}

/// 替身关上一个范围之后，等它接住的显存不足的那个 future 什么时候交出来。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClosedScopeAnswersOfTheDeviceForTests {
    /// 一轮询就交出（本机 wgpu 后端的样子）。
    AtOnce,
    /// 一直不交出：驱动挂在交回错误范围那一等上的卡。
    Never,
}

/// 替身开的一个范围：开时范围栈里的位置。
struct OutOfMemoryScopeInMemoryForTests {
    position_in_the_open_scopes: usize,
}

impl DeviceOfAChunkInMemoryForTests {
    fn with_free_bytes(free_bytes: u64) -> Self {
        Self::with_free_bytes_and_the_closed_scope_answering(
            free_bytes,
            ClosedScopeAnswersOfTheDeviceForTests::AtOnce,
        )
    }

    fn with_free_bytes_and_the_closed_scope_answering(
        free_bytes: u64,
        closed_scope_answers: ClosedScopeAnswersOfTheDeviceForTests,
    ) -> Self {
        Self {
            free_bytes: Cell::new(free_bytes),
            bytes_held: Cell::new(0),
            peak_bytes_held: Cell::new(0),
            out_of_memory_captured_by_each_open_scope: std::cell::RefCell::new(Vec::new()),
            uncaptured_out_of_memory: std::cell::RefCell::new(Vec::new()),
            closed_scope_answers,
        }
    }

    fn buffer_of(&self, bytes: u64) -> BufferInMemoryForTests {
        let free_bytes = self.free_bytes.get();
        if bytes <= free_bytes {
            self.free_bytes.set(free_bytes - bytes);
            self.bytes_held.set(self.bytes_held.get() + bytes);
            self.peak_bytes_held
                .set(self.peak_bytes_held.get().max(self.bytes_held.get()));
            return BufferInMemoryForTests {
                bytes_requested: bytes,
                is_created: true,
            };
        }
        let reason = format!("建 {bytes} 字节的缓冲：卡上只剩 {free_bytes} 字节");
        let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
        match open_scopes.last_mut() {
            Some(innermost_scope) => {
                if innermost_scope.is_none() {
                    *innermost_scope = Some(reason);
                }
            }
            None => self.uncaptured_out_of_memory.borrow_mut().push(reason),
        }
        BufferInMemoryForTests {
            bytes_requested: bytes,
            is_created: false,
        }
    }
}

impl DeviceCreatingTheBuffersOfAChunk for DeviceOfAChunkInMemoryForTests {
    type Buffer = BufferInMemoryForTests;
    type OutOfMemoryScope = OutOfMemoryScopeInMemoryForTests;

    fn open_the_out_of_memory_scope(&self) -> OutOfMemoryScopeInMemoryForTests {
        let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
        open_scopes.push(None);
        OutOfMemoryScopeInMemoryForTests {
            position_in_the_open_scopes: open_scopes.len() - 1,
        }
    }

    fn readback_of_one_kernel(&self, outcome_bytes: GpuMemoryInBytes) -> BufferInMemoryForTests {
        self.buffer_of(outcome_bytes.0)
    }

    fn bytes_of_the_buffer(&self, buffer: &BufferInMemoryForTests) -> GpuMemoryInBytes {
        GpuMemoryInBytes(buffer.bytes_requested)
    }

    fn out_of_memory_captured_by(
        &self,
        scope: OutOfMemoryScopeInMemoryForTests,
    ) -> impl Future<Output = Option<String>> {
        let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
        assert_eq!(
            scope.position_in_the_open_scopes + 1,
            open_scopes.len(),
            "关的是最里层那个范围"
        );
        let mut captured_until_handed_out =
            Some(open_scopes.pop().expect("上面判过：关的那个范围还开着"));
        let closed_scope_answers = self.closed_scope_answers;
        std::future::poll_fn(move |_context| match closed_scope_answers {
            ClosedScopeAnswersOfTheDeviceForTests::AtOnce => Poll::Ready(
                captured_until_handed_out
                    .take()
                    .expect("交出之后等的那一方不再轮询"),
            ),
            ClosedScopeAnswersOfTheDeviceForTests::Never => Poll::Pending,
        })
    }
}

/// 这一段自己那几样缓冲合起来的字节数与一份核的结果字节数（两份回读暂存各这么大）：只用来造「那几样建得成、回读暂存建不成」。
const BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK: u64 = 1000;
/// 这一段记在账上的字节：自己那几样映射着建、连同暂存，加两份回读暂存。
const BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT: GpuMemoryInBytes =
    COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
        .bytes_of_the_copies(GpuMemoryInBytes(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK))
        .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK));

/// 卡上空闲的装得下这一段自己那几样缓冲，
/// 装不下两份回读暂存——回读暂存建在同一个只接显存不足的范围里，那一条显存不足被范围接住：交回 `OutOfMemoryCreatingTheBuffers`
/// （外壳照它放掉这一段的账、走「这一批转给别的卡」），一条没接住的错都没有。回读暂存建在范围之外时，那一条显存不足没被接住、这一段照「建成了」交回。
#[test]
fn readbacks_short_of_free_memory_are_caught_by_the_out_of_memory_scope_of_the_chunk() {
    let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(
        BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + OUTCOME_BYTES_OF_THE_CHUNK - 1,
    );
    let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
        &device,
        &budget_of_a_card_waiting_at_most(Duration::from_secs(1)),
        BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT,
        GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
        || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
    );
    assert!(
            matches!(
                created,
                Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk,
                    ..
                }) if bytes_held_by_the_chunk == BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT
            ),
            "回读暂存建不成的那一条显存不足被这一段的范围接住、报成建缓冲显存不足（带这一段的账）：{:?}",
            created.as_ref().map(|_buffers| "建成了")
        );
    assert_eq!(
        *device.uncaptured_out_of_memory.borrow(),
        Vec::<String>::new(),
        "没有一条显存不足落到范围之外"
    );
}

/// 同一条的阳性对照：空闲的装得下这一段自己那几样与两份回读暂存——交回建成的那几样与两份各 64 字节的回读暂存，没有错、范围都关上了。
#[test]
fn the_chunk_and_both_readbacks_within_the_free_memory_are_created_in_one_out_of_memory_scope() {
    let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(
        BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
    );
    let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
        &device,
        &budget_of_a_card_waiting_at_most(Duration::from_secs(1)),
        BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT,
        GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
        || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
    )
    .expect("空闲的正好装得下：建得成");
    assert_eq!(
        (created.buffers, created.readbacks),
        (
            [BufferInMemoryForTests {
                bytes_requested: BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK,
                is_created: true,
            }],
            [
                BufferInMemoryForTests {
                    bytes_requested: OUTCOME_BYTES_OF_THE_CHUNK,
                    is_created: true,
                },
                BufferInMemoryForTests {
                    bytes_requested: OUTCOME_BYTES_OF_THE_CHUNK,
                    is_created: true,
                },
            ]
        )
    );
    assert_eq!(device.free_bytes.get(), 0, "三样都照字节数扣了");
    assert!(
        device.uncaptured_out_of_memory.borrow().is_empty()
            && device
                .out_of_memory_captured_by_each_open_scope
                .borrow()
                .is_empty(),
        "没有错、范围关上了"
    );
}

/// 关上范围之后一直不交出的那张卡上，建缓冲等错误范围那一等的上限：比放弃那一刻（一个短等间隔的量级）长得多。
const WAIT_LIMIT_OF_THE_SCOPE_NEVER_ANSWERING: Duration = Duration::from_secs(5);

/// 驱动挂在交回错误范围那一等上（替身关上范围之后一直不交出），这张卡等一批的结果读调用方在停的标记、标记已置
/// （D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交」）：建缓冲等错误范围那一等放弃、交回 `GaveUpWaitingForTheCallerIsStopping`，
/// 在等的上限之内交回（一个短等间隔的量级），不等满上限、不报等超过上限。那一等不读标记时要等满 5 秒、交回等超过上限，这一条红。
#[test]
fn the_wait_for_the_out_of_memory_scope_gives_up_within_the_wait_limit_once_the_caller_is_stopping()
{
    let device = DeviceOfAChunkInMemoryForTests::with_free_bytes_and_the_closed_scope_answering(
        BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
        ClosedScopeAnswersOfTheDeviceForTests::Never,
    );
    let (caller_is_stopping, handle_setting_the_flag) =
        caller_stopping_flag_and_the_handle_setting_it();
    let mut budget = budget_of_a_card_waiting_at_most(WAIT_LIMIT_OF_THE_SCOPE_NEVER_ANSWERING);
    budget.read_the_caller_stopping_in_the_waits_of_the_batches(
        &CallerStoppingSeenByTheWait::ReadBetweenShortWaits(caller_is_stopping),
    );
    handle_setting_the_flag.tell_the_implementation_the_caller_is_stopping();
    let started_at = Instant::now();
    let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
        &device,
        &budget,
        BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT,
        GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
        || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
    );
    let waited_for = started_at.elapsed();
    assert!(
        matches!(
            created,
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping)
        ),
        "调用方在停：建缓冲等错误范围那一等放弃：{:?}",
        created.as_ref().map(|_buffers| "建成了")
    );
    assert!(
        waited_for < WAIT_LIMIT_OF_THE_SCOPE_NEVER_ANSWERING,
        "在等的上限之内交回、不等满（等了 {waited_for:?}）"
    );
}

/// 这张卡的上限与卡上空闲的都是 2500 字节，这一段记在账上
/// 2128 字节（这一段自己那几样 1000 连同暂存 2000、两份回读暂存各 64），建的时候多建了一块 300 字节的缓冲——连同暂存卡上真占的到了 2728 字节，比账多：
/// 账核真建的缓冲字节，当场断言，不带着比账多的显存往下交。
#[test]
#[should_panic(expected = "这一段真建的缓冲合起来的字节数与记在这张卡账上的不等")]
fn the_chunk_creating_one_buffer_more_than_it_holds_on_the_account_panics() {
    const BYTES_OF_THE_EXTRA_BUFFER: u64 = 300;
    let memory_limit_in_bytes = 2500;
    let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(memory_limit_in_bytes);
    let bytes_held_by_the_chunk = BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT;
    let mut account =
        GpuMemoryAccount::<u32>::with_nothing_held(Some(GpuMemoryInBytes(memory_limit_in_bytes)));
    account.hold(bytes_held_by_the_chunk);
    let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
        &device,
        &budget_of_a_card_waiting_at_most(Duration::from_secs(1)),
        bytes_held_by_the_chunk,
        GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
        || {
            [
                device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK),
                device.buffer_of(BYTES_OF_THE_EXTRA_BUFFER),
            ]
        },
    );
    // 走到这里就是账没核真建的缓冲：卡上真占的峰值比账上的多，用例照「峰值不超账」红在这里。
    assert!(
        device.peak_bytes_held.get() <= account.bytes_held().0,
        "卡上真占的峰值 {} 字节超过了账上的 {} 字节（{:?}）",
        device.peak_bytes_held.get(),
        account.bytes_held().0,
        created.as_ref().map(|_buffers| "建成了")
    );
}

/// 同一条的阳性对照：noop 后端上照产品的建法建一份装着内容的缓冲，建得成、内容写进去了，没有错。
#[test]
fn the_buffer_with_contents_on_a_noop_device_is_created_and_written_within_the_scope() {
    let (device, _queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor {
        label: Some("singlefs-background-gpu-noop-test"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    });
    let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
        &device,
        &budget_of_a_card_waiting_at_most(Duration::from_secs(10)),
        COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(GpuMemoryInBytes(8))
            .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK)),
        GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
        || {
            [buffer_with_contents_created_within_the_scope_of_the_caller(
                &device,
                &[1, 2, 3, 4, 5],
                wgpu::BufferUsages::STORAGE,
            )]
        },
    )
    .expect("noop 后端上 8 字节建得成");
    assert_eq!(
        created.buffers[0].size(),
        8,
        "5 个字节的内容按整字补成 8 字节，与切批记账同一种算法"
    );
}
