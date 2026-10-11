# m3-batch10z-ghi-code-r1 正推腿（Sonnet）报告（2026-10-08）

立场：正推。从 kb 已定条款与冻结副本 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree/` 今天的实现推「应该是什么」，再与被判改动逐格比。
引代码写冻结副本的路径与行号（`--root` 指冻结副本），引 kb 写 kb 文件自己的行号；每一句引文写进报告之前都在被引文件里 `grep -nF` 过一次。
自己的探针、变异脚本与原样输出放 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-model/`（`README.md` 写了复现命令）；草稿目录 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1-sonnet/`。

## 〇、先说结论

- 打中 1 条实质的：**K2-1，丙罩不住管理员回退（向前发布）当 R 时被复活的单元**。已在入库装置（`tests/common_unwitnessed_publish`、`PoolUnderTest`）上造出会红的一形，池级 checker 报 I-7.4（见第三节）。
- 打中 1 条低危的：**K4-2，卡池借之前取留着的设备（`started_now`）会等一次正在进行的设备重起**（量到 250 毫秒 / 300 毫秒），窗口只在重起头几毫秒内（第五节）。
- K5 有缺口（第六节）：丙的落点集合「改错」有三种照绿（多护已释放的；只护前一半；只护跨度 ≥ 2 的），叫停接线的扫源码用例钉不住「闭包传到下游」，借之前 poll 的扫源码用例认得出被注释掉的那一句。
- 另记一笔量出来的耗时（第三节 K2-2）：C 那一槽装着整份现行账的落点时，进程内回收那一步的扫描是 受护落点数 × 回收记录数。
- K1-1、K1-2、K1-3、K2-3、K2-4、K3-1、K3-2、K4-1：没找到会红的形；各自写了查过什么、什么现象会推翻。
- 没跑的：checker 档测试（含 e158 内联单测、gpu_unit_checks 的库单测）、任何带 gpu 特性的测试、门禁阶段。能在 harness 档与 core 复跑的变异表行全部复跑了（30 行，30 行红，`logs/rows-harness-1.log`、`logs/rows-harness-2.log`）。

## 一、基线（这一轮的观测）

- 冻结副本被判文件 sha256 与快照核过：
  `grep -E 'singlefs-core/src/(mount|allocator).rs|gpu_memory_ledger.rs|gpu_unit_checks.rs|crash_amplification.rs' research/prompts/m3-batch10z-ghi-code-r1-snapshot/crates-sha256.txt | sha256sum -c -`
  输出五行 `…: OK`（crash_amplification.rs、gpu_unit_checks.rs、allocator.rs、mount.rs、gpu_memory_ledger.rs）。
- 开工前 `ps -o pid,args -u "$(id -u)"` 里没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；没有别的 `cargo`、`gate.sh` 在跑。
- 基线：`cargo test --offline -p singlefs-harness --lib -- gpu_memory_ledger` → `test result: ok. 104 passed; 0 failed`；
  `--test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` → `ok. 11 passed`；
  `--test gpu_card_rounds_remember_a_stop_request_removed_mid_run` → `ok. 1 passed`。
  命令都经 `bash scripts/capped.sh 15 bash scripts/run-with-memory-cap.sh 8G nice -n 19 cargo test --offline …` 起。
- 变异复跑做法：背景材料改动全文里的变异表行（`logs/rows-harness-*.log` 用 `tools/mutate_rows2.py`）套到拷贝上，跑那一行点名的测试，还原。
  checker 档的扫源码用例（四条）不能在 checker 档包里跑，我把它们的逻辑原样搬进 `scan-model/`（读运行时给的源文件路径），
  基线上四条都过（`test result: ok. 4 passed; 0 failed`），再对变异后的源码复跑。这是对用例逻辑的复现，不是用例本身；标「模型」。

## 二、K1　第八块：G2 乙的补法做到没有

### K1-1　等超时那一笔活做完之后是不是总能销；借之前那一下 Poll 有没有让账多给，有没有拿锁压并发

读到的出口（今天的实现，冻结副本）。没等成的借据记在 `leases_kept_after_a_failed_wait`（借据号 → 它等的那一台），销账只有四个出口：

1. 借之前 poll：四个借的入口都先调 `take_in_the_devices_found_idle`（行 1435、1473、1521、1612，每处一句「self.take_in_the_devices_found_idle();」），
   它对账上有没等成借据的卡取留着的设备（「let Some(started) = self.cards[card_position].started.started_now() else {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1335`）），
   设备没丢就 poll 一次（「started.device().poll(wgpu::PollType::Poll)」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1342`）），只有交回 `QueueEmpty` 才销（「Ok(Ok(wgpu::PollStatus::QueueEmpty)) => true,」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:60`））。
2. 同一台上另一笔借据等成：「books_of_the_card.return_the_leases_kept_after_a_failed_wait_on(device.device);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1493`）。
3. 设备丢了重起：「for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait).into_keys() {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:866`）。
4. 那一台放掉：「self.return_the_leases_kept_after_a_failed_wait_on(device);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:901`）。

`QueueEmpty` 的含义我读了 wgpu-core 30.0.1 源码核（`~/.cargo/registry/src/*/wgpu-core-30.0.1/src/`，不在报告的引文核对根里，贴命令与输出）：

```
$ grep -nF 'self.active.is_empty()' device/life.rs                                                    → 218:        self.active.is_empty()
$ grep -nF 'let queue_empty = life_tracker.queue_empty();' device/queue.rs                            → 254:        let queue_empty = life_tracker.queue_empty();
$ grep -nF 'let queue_result = queue.maintain(current_finished_submission, &snatch_guard);' device/resource.rs → 917
$ grep -nF 'self.device.raw().destroy_buffer(raw);' resource.rs                                       → 537:                self.device.raw().destroy_buffer(raw);
```

即 `Poll` 先 `triage_submissions` 收掉做完的提交（它们的编码器带着的 `Arc<Buffer>` 随之丢掉，`Buffer::drop` 当场 `destroy_buffer`），再看 `active` 空没空；
所以「交回队列空」蕴含「这台设备上到此刻为止提交的活做完、丢掉的缓冲已交回 wgpu-hal」。交回 wgpu-hal 到交回驱动之间（小缓冲经 gpu-alloc 的块）我没查，那是 G2 乙的前提、E174 量的是 1 GiB 的缓冲，不在这一轮的范围。

**活做完之后是不是总能销**：有出口 1 在，只要之后还有一次借，队列空的那一刻就销；没有之后的借，也就没有谁需要这笔的额度。
残留（少给，不多给）：同一台设备上另有借据持续提交时 `Poll` 一直交 `Poll`、不销；要等那台上另一笔借据等成（出口 2）。这一段里这笔的 MiB 仍算借着，别的借据看到的额度少这么多。
这是 G2 乙「账不多给，代价是这一段少借」的同一条代价，今天只是比第六块短了；我没造出让它永久的形（出口 2 每笔借据丢掉时都会走）。
推翻它的现象：造出一形，活早做完、设备也没丢，而四个出口都不触发——例如同一台设备上每笔借据的等都超时、又没有新的借。

**借之前那一下 Poll 有没有让账多给**：没造出。
- 多给要求「缓冲其实还在卡上，账却销了」。出口 1 销的是 poll 之前就进了表的那几笔：它们的缓冲在借据丢掉之前已经丢了（借据先于缓冲丢的前提，文件头「管不到的」一句，行 1534 附近），提交在更早。
  这个前提今天没有会红的检查（实现员自报，我也没找到）；我读了崩溃放量里借据与判器的丢法：「借据（不进账时是空的），判器都丢掉之后才还。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2779`），没发现缓冲比借据活得久的一处。
- `take_in_that_the_submitted_work_finished` 销的是调用那一刻该台上的全部 kept，不只 poll 之前的。poll 之后才进表的一笔，要么等满 30 秒超时（`WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE`，行 27），要么等的时候立刻报错或 panic；
  后者要这台设备恰在 poll 与销账之间出错而没被前面的 `is_lost()` 滤掉（「if started.device_condition().is_lost() {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1338`））。窗口微秒级，我没造出，也不能证它不存在（标：推的，没造出）。

**拿锁压并发**：有，窄，写在第五节 K4-2（`started_now` 与 `started_or_start` 是同一把锁，实测等了 250 毫秒）。poll 本身（`PollType::Poll`）不等；账的锁只在列卡和销账时短暂拿，没有在锁里 poll。

### K1-2　丢了的设备按 GpuCard 克隆都放掉判

读到的：`GpuCard` 的最后一个字段是账上那一台（「device_on_the_books: Option<std::sync::Arc<DeviceStartedOnTheBooks<'static>>>,」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:151`）），
起设备的唯一出口（`started_card`）起好后放进去（「started.device_on_the_books = Some(std::sync::Arc::new(device_on_the_books));」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1399`）），
最后一个 Arc 丢掉时销该台（「self.ledger.books_locked()[self.card_position].release_a_device(self.device);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1552`））。

- 查「`GpuCard` 的字段从克隆里另拿出来、比卡活得久」：`grep -rn 'GpuCard {' --include=*.rs crates/` 只有结构定义和 `start_the_card` 一处构造；`grep -rn 'device()\.clone()\|queue()\.clone()\|wgpu::Device\b\|wgpu::Queue\b'` 在 `gpu_unit_checks.rs` 之外只有 `tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs` 里按引用传的参数；
  判器（`GpuCrashJudge<'card>`）带的是对 `GpuCard` 的借用，编译器管着它不比那张卡活得久。没找到旧设备另有句柄的一形。
- 字段丢的次序：多个克隆同时丢时，最后一个 Arc 的递减发生在每个克隆自己的设备、队列、管线、共用管线表之后（每个克隆的递减按它自己的声明次序，最后一次递减之前所有克隆的前几个字段都已递减过），
  所以放掉判成立时 wgpu 句柄不会还有克隆持有的。
- 「永远判不成放掉」：卡池那一格在设备重起时被换掉（`started_or_start` 里 `*started = Some(…)` 丢旧克隆），借据的卡与调用方留的 `Arc<GpuCard>` 随借据和一趟活结束而丢；没找到静态或线程局部的克隆持有者
  （`grep -n 'OnceLock\|thread_local\|static ' crates/singlefs-checker-tier/src/gpu_unit_checks.rs crash_judge_dispatch.rs crash_verify_gpu.rs` 的静态只有两处：行 224 的 `BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL`（总线号表）与行 1854 的 `GPU_CARD_POOL`（卡池本身，池里那一格放的是在用的设备）；`OnceLock` 另在行 123 的每设备管线表里；没有 `thread_local`）。
- 判错的代价方向：判成「没放掉」→ 这张卡照分不出算（取 0，少给）；判成「放掉」而其实还占着 → 多给。后一方向的前提就是上面两条，没有会红的检查。

推翻它的现象：造出一个比 `GpuCard` 克隆活得久的 wgpu 句柄（缓冲、管线、设备的 clone）；或一个永不丢的 `GpuCard` 克隆。

### K1-3　展开中不再抛

读到的：`catch_unwind` 接住等的结局，记完之后「if !std::thread::panicking() {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1576`）才「std::panic::resume_unwind(panic);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1577`）。
- 吞掉的只有「已经在展开中」时等的那一个 panic。这时线程已经在往外抛第一个 panic，用例已经红；吞掉第二个只是不让它升级成中止。要变绿，要有外层 `catch_unwind` 把第一个当成预期的 panic 接住，且这时丢的借据的等又恰好 panic。
  既提借据的接口又用 `catch_unwind` 的文件，我逐个看了（`for f in $(grep -rl catch_unwind --include=*.rs .)` 再筛含 `GpuCardLease|lease_card_or_report_not_run|lease_the_first_card_with_room_now|book_the_scratch_of_the_cards_with_room_now` 的）：`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:292` 的 `on_the_card_or_report_not_run_when_out_of_memory` 接的是 `work`，借据在调它的用例里、在 `catch_unwind` 外面（行 2288、2768、2895、3099 先借后调）；`crash_judge_dispatch.rs:1393` 是一条拒绝内核过大的库单测，不碰借据；`gpu_unit_checks.rs` 行 2059、2072 是 Out of Memory 回调的库单测。没有一处把借据放在被预期 panic 接住的展开里。
- 不是静默吞：没等成时先「DeviceWaitBeforeReturningTheLease::TheWaitFailed => report_the_failed_wait(&polled),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1572`）打 `GPU_CARD_LEASE_RETURN_WAIT_FAILED` 一行再记结局。
- 另一处吞 panic：`take_in_the_devices_found_idle` 里 poll 的 `catch_unwind`（「let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1341`））不留痕、不记；
  poll 的结局只用来决定销不销，不进任何判定，所以不会把红变绿；但 wgpu 若把别的线程那份活的异步错报在这次 poll 里，这里会吃掉它。我没造出这种形（wgpu-core 的错多半在提交那一刻由提交的线程报，丢设备记在设备状况里），记一笔。

推翻它的现象：一条把借据放在被预期 panic 的 `catch_unwind` 里的用例，等的时候真 panic 了而用例仍绿。

## 三、K2　第九块：H2 丙与第四块判决那几条做到没有

### K2-1　丙罩不罩得住被抛弃根 R 引用的全部单元（**打中**）

条款（整句抄）：D16 已定项 1「根槽这一次读坏」那一行末段写的是「那条根在可写挂载那一刻账读不出而记录齐时，C 那一槽另并上那一刻现行账里仍分配着的落点」，括注给的理由是「那条根从所选那一版继承的单元只可能在这一截里」（`.claude/kb/decisions/16-发布语义.md:37`）。
底下的总原则：「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）。

实现：丙就一句，「placements_still_allocated_in_the_account(current_records)」（`crates/singlefs-core/src/mount.rs:2008`），
它取的是挂载一开始、重建分配器之后的那份账「let current_records = allocator.records().to_vec();」（`crates/singlefs-core/src/mount.rs:2487`）里**没释放的**记录：「.filter(|record| !record.is_released)」（`crates/singlefs-core/src/mount.rs:2030`）。
被借给 C 的是「所选那一版 P 的账里仍分配着的」加「R 的记录点名的」，没有第三类。

**括注的前提不成立**：R 引用的单元，除了 R 自己写的和 P 账里仍分配着的，还有第三类——**管理员回退那次发布（向前发布）复活的单元**。
回退发布 R 取 R_old 的用户可见树，R_old 引用、P 的账记成已释放的单元被改回已分配：「/// - 复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的。」（`crates/singlefs-core/src/mount.rs:7584`），
动作是「allocator.resurrect_released_record(」（`crates/singlefs-core/src/mount.rs:7763`）。回退发布不写数据单元，记录不点名它们；它们在 P 的账里是 `is_released`，被 `!is_released` 滤掉。
所以 R 引用着、而 C 那一槽里一个也没有。回退的根「持久之后才向管理员确认」（`.claude/kb/decisions/23-journal的角色与格式.md:390`），根持久之后、系统配置轮换之前崩是条款允许的状态。

**造出来了**（入库装置上，我自己写的探针，`research/prompts/m3-batch10z-ghi-code-r1-sonnet-model/probe_rollback_revived.tail.rs`，不进库）：
单元区 384 槽那一档小盘，第一个文件之后覆盖写 4 次得 P（txg 7）；经会话调 `roll_back_by_a_forward_publish` 回退到第一个文件那一版 R_old，录这一次发布得 R（txg 8），崩在 R 的根槽落盘之后、轮换之前；
可写挂载那一刻 R 的账三段（树表、中央映射树根、分配记录树根）读回全零（与第九块丙的两条用例同一个取法），挂载后撤故障；会话覆盖写到准入推出第一串抬 F，再顺序写三次。原样输出（`logs/probe-rollback-revived.log`，sha256 在文末）：

```
PROBE rolled back: resurrected=8 released=4
PROBE mounted on (InstanceGeneration(1), CheckpointTxg(7)), previous_key (InstanceGeneration(1), CheckpointTxg(7)), r key (InstanceGeneration(1), CheckpointTxg(8))
PROBE borrowed for R = Some(44) entries; revived n = 2
PROBE revived not in C[R]: 2 of 2
PROBE after raise (after 15 overwrites; R in ring before the step: Some(true)): free revived slots = 4
PROBE after seq write 1: R in ring=true violations=["I-7.4: 被抛弃的根（实例 1、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 1412 的那一份与位置条目里的校验和对不上；…
PROBE after seq write 2: R in ring=true violations=[同上]
PROBE after seq write 3: R in ring=true violations=[同上]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 3.35s
```

读法：挂载落到 P、R 记成有效根；C 那一槽借到 44 项，复活的 2 个落点（1 个数据单元 × 两块盘）一个都不在里面；推过抬 F 的那一刻复活单元覆盖的 4 个槽（2 个落点 × 跨度 2）全空闲、可发；
顺序写之后池级 checker 在被抛弃的 R（还在根环里）上判 I-7.4 红，点名盘 0 槽 1412 的数据单元。探针最后一条断言（复活的槽一个都不空闲）红，就是命中。
**对照**：把丙那一句的过滤去掉（含已释放的记录）再跑同一个探针，`logs/probe-revived-with-bing-including-released.log`：`GREEN … test result: ok. 1 passed`——所以缺口就是「第三类单元被 `!is_released` 滤掉」，不是装置的毛病。

不在「已知没做」里：已知没做写的是「丙那一形的崩溃枚举没做」；这里是丙没写到的边角（R 是回退发布）。丙的两条新用例（R 建 inode）都不覆盖这一形。
前提三个同时成立才红：回退发布崩在轮换之前；挂载那一刻 R 的账读不出而记录齐；之后会话写到抬 F 并回收那几个单元，R 还在环里。每一个都是条款允许的状态，但合起来是窄的。
推翻它的现象：在冻结副本的拷贝上按 `README.md` 复跑探针，`revived not in C[R]` 不是全部、或末尾不红；或回退发布的记录其实点名了复活单元（我看到的是 44 项里一个都没有）。

### K2-2　丙多护的那一截什么时候放；会不会一直不放；D28 第九项计数变了之后准入与卸载有没有一形判错

- 放的时机：C 的一槽在这个进程写掉那一槽时解开：「.remove(&ring_slot)」（`crates/singlefs-core/src/allocator.rs:2184`），条款同句：「C 的全部落点只在这个进程写掉那条记录所属 txg 对应的根环槽、那次发布取完落点之后解开」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。
  丙并上的那一截放的时机与 H2 之前 C 那一槽完全一样（同一个键、同一条 `remove`），R 的根环槽比 P 的晚一个 txg 被盖，也就是比 P 本身离环晚一次发布。不会比环转一圈更久。
  环转一圈要那么多次发布；一个几乎不发布的会话里它就多留那么久，这与别的被抛弃根同一个性质，不是丙新引进的。
- 多护：R 自己那次发布释放的 P 的单元（R 不再引用）也被丙并进去（P 的账里它们仍分配）。量是一次发布的释放量，环里留着，不累积。
- D28 第九项：挂载那一刻丙并上的落点全在现行账里，豁免：「if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {」（`crates/singlefs-core/src/mount.rs:1646`），第九项不变；
  抬 F 时用的是会话此刻的现行账（`current.allocation_records`，行 4201），会话释放过的丙落点才补隔离、进第九项，与条款「只除去现行账里仍分配着的槽」（`.claude/kb/decisions/28-挂载期承诺量.md:27`）同形。11 条用例全绿，没找到判错的形。
- **耗时（量的，调试构建）**：丙把 C 那一槽从「几个记录点名的落点」变成「整份现行账的落点」。进程内回收每次发布都做 `placements_the_isolation_still_protects()`（行 2186）再对回收出来的每个槽线性扫它：
  「.any(|placement| placement.covers(record.device, SlotNumber(slot)));」（`crates/singlefs-core/src/allocator.rs:2365`），受护集合里有 C 的全部落点（`protected.extend(self.placements_named_by_unapplied_journal_records());`（`crates/singlefs-core/src/allocator.rs:1222`））。
  我在拷贝的 `allocator.rs` 测试模块里插了一条用例直接量这个函数（`probe_cost_of_the_protected_list_scan.rs`），回收的槽都不在受护集合里（最坏）：

```
PROBE-COST protected_placements=10000 reclaimed_records=1000 elapsed_ms=68
PROBE-COST protected_placements=40000 reclaimed_records=1000 elapsed_ms=277
PROBE-COST protected_placements=40000 reclaimed_records=4000 elapsed_ms=1104
```

  即 受护落点数 × 回收记录数 的乘积，4×10^4 个落点、4×10^3 条回收记录是 1.1 秒（调试构建；发布构建没量）。丙把受护落点数从几十推到整份账的单元数，所以真池上每次发布的这一步在 R 还在环里的那一圈里变慢；
  不判红、不弃卡、不涨内存，只是会话线程上的 CPU 时间。是不是要紧归主 agent 判；我只量了函数，没量真池上的单元数。
  推翻它的现象：发布构建下同样的乘积小于毫秒级，或真池上回收出来的记录数远小于 10^3。

### K2-3　K1-1 新口径：公开函数、`on_device_modes.rs`、e158 改取之后有效根槽读坏是不是照拒

- 新函数就是原来 `rollback_floor_ceiling_of_this_process` 里那一段原样抽出：已知住着根的槽除去根环表里记成被抛弃根的槽（`crates/singlefs-core/src/mount.rs:3635` 是抬 F 那一串的调用点）。有效根的槽不被除去，读坏、重读仍坏照拒。
  用例 `the_public_rollback_floor_ceiling_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_equals_the_ceiling_while_it_reads` 的阳性对照里，旧口径传整集就拒、点名 C 那一槽；
  变异（去掉除去被抛弃根的那个 `filter`）复跑：`logs/rows-harness-2.log` 里该行红（`9 passed; 2 failed`）；失败的两条是这条新用例和 `session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements`（第四块 H1 那条），用我自己的变异脚本列出的名字（`mutate_custom.py`，同一个变异）。
- 调用方都改了：`grep -rn 'rollback_floor_ceiling(' crates/` 在 `src/` 下只有 `on_device_modes.rs:332`、`bin/e158_root_choice_repair.rs:1988` 两处，另有两个 harness 用例（`admission_raises_the_floor_before_refusing.rs:2263` 传整集——背景材料「已知没做」里写了，`raise_floor_takes_…:774` 新用例）。
- 一处没改、条款与注释对不上的：公开函数仍把另外两个参数传空表（「&BTreeMap::new(),」（`crates/singlefs-core/src/mount.rs:3145`）、`:3146`），即不并「挂载时读坏的槽」，也不处置「两遍里只有一遍自证过的槽」；
  `on_device_modes.rs` 的注释说「入参与 raise_rollback_floor 自己算上限时的一样」，在这两个参数上不一样。这是改动前就有的（旧口径同样传空），没引进新的红；后果是自算上限的调用方在这两类槽存在时可能比 `raise_rollback_floor` 里的上限高，被它在写之前以 `RollbackFloorAboveCeiling` 拒掉（安全方向）。记一笔，不算打中。

### K2-4　K1-2 空条目来自坏盘记录时有没有一形让 I-7.4 红

- 这一格的代码改动只有注释和一条库单测（行为没改：`get(&ring_slot).cloned()` 本来就交 `Some(空)`）。所以问题是：`Some(空)` 的来源里有没有「照空串算」比「照读盘」少护而红的。
- 空条目的来源：(a) 凭据放行：「return Some(Vec::new());」（`crates/singlefs-core/src/mount.rs:1724`），按 C694 的 B 条款，那条根的树表槽与映射根槽已被后来的分配罩住；(b) 记录点名那一份对每条记录无条件建槽条目：「.entry(target_for_publish(record.checkpoint_txg, slots_per_region))」（`crates/singlefs-core/src/mount.rs:1824`）后接「.or_default()」，
  记录的落点全不在单元区（坏盘上的记录）时留下空条目。
- (b) 要在 `..._for_the_root_recorded_valid_on_its_ring_slot` 里被借出，那一槽的占用者必须是同 txg 的有效根，它就同时在 `roots` 里，是「较新可读根」，被 `placements_in_the_accounts_of_newer_readable_roots` 往同一条目里 `extend` 它的账（读得出）、
  映射现算（现算得出）、或丙（记录齐）。三条都是非空；能留空的只剩 (a)。记录不齐又现算不成、没有凭据时是拒（行 1988 的 `None => continue` 只在系统配置的实例代号比那条根新时）。
  所以我没造出 (b) 红的形。
- 推翻它的现象：一个有效根槽的占用者同 txg，记录全坏、账读不出、映射现算不成、系统配置的实例代号又比它新——这时 C 里是空条目，抬 F 照空串算。我没造出；也没证它造不出。

## 四、K3　第七块

### K3-1　叫停只在有下一张装了状态的卡时才问：有没有一形该看叫停时没看、叫停之后还起新卡领块

读到的：`next_card_to_start` 先找后面第一张装了状态的卡（没有就 `None`，一次叫停都不问），找到了才「if blocks.any_left_to_take(stop_of_the_cards_now()) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2294`）；
产品路径调用点把叫停交成闭包「next_card_to_start(work.states_placed, position, &work.blocks, || {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2473`）。
- 该看时没看：没有下一张卡时不读叫停，少了一次读；叫停每领一块之前仍问（「while let Some(pending_index) = blocks.take_a_block(stop_of_the_cards()) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2258`）），一轮收尾仍问记住的那个（「let stop = stop_request.stop_of_the_run();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3129`））。
  叫停是「见过一次就记住」的（`StopRequestRememberedInThisRun::is_requested`），少一处读，只是叫停文件短暂出现又被删的窗口多一点点——别的几处照读，没造出这一窗里漏掉叫停的一形。
- 叫停之后还起新卡领块：读到叫停（或判红）时 `any_left_to_take` 交 `false`，不起；读之后、新线程跑起来之前才来的叫停，新卡线程领第一块之前自己再问一次（行 2258），不领。新线程起来后先建判器（编内核）这件事与改动前相同。没造出叫停之后还领块的形。
- 判别力：变异「起下一张卡不管后面有没有装了状态的卡都先问叫停」，库单测 `the_next_card_asks_for_the_stop_once_only_when_a_later_card_holds_states` 红（`logs/rows-harness-1.log` 第一行）；
  产品路径那一行的变异（`{ let stop = work.stop_of_the_cards(); move || stop }`）靠扫源码用例 `the_stop_request_remembered_in_this_run_reaches_…` 的 B 项红（模型 `logs/scan-model-mutations.log`：`RED(caught) … 起下一张卡之前先问叫停`）；
  表里写的 GPU 用例我不能跑。
推翻它的现象：叫停文件在起最后一张卡的那一刻出现、之后又很快删掉，而别的读的点都错过了它。

### K3-2　不打印的那一半拆出去之后，产品路径上该打的本次未跑那一行是不是每一处都还打

产品路径只有三处借不到走 `print_the_not_run_line_or_refuse`：「print_the_not_run_line_or_refuse("第 ① 段判器的草稿区", &refusal)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2857`）、
「print_the_not_run_line_or_refuse("单元校验和对拍的缓冲", &refusal)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:6235`）、
「print_the_not_run_line_or_refuse("第 ② 段核对的缓冲", &refusal)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:6381`）。
打印那一半先造再打再交回：「let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2728`）、「println!("{not_run_line}");」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2729`）。
不打印的那一半在产品路径只有这一处调用（`grep -n 'not_run_line_or_refusal_of_a_lease' crash_amplification.rs`：行 2718、2722 是注释，2728 调用，2739 定义，其余是库单测）。三处调用点的前后都没有别的路径绕开它（`?` 出错是判红、不是本次未跑）。每一处都还打。
判别力（模型）：改一处调用点改调不打印的那一半，扫源码用例红（`RED(caught) … X5`）；去掉 `println!` 红（`RED(caught) … 产品路径借不到时不打本次未跑那一行`）。
这条用例的数调用点法（数 `print_the_not_run_line_or_refuse(` 恰好 4 行）有一个口子，见第六节 K5。
推翻它的现象：第四处产品路径的借不到走了不打印的那一半，或某一处调用点的 `?` 之前返回了 `Ok` 而没打。

## 五、K4　只防 OOM、不压并发两头（新代码引进来的）

### K4-1　有没有一形账给的量大于此刻真能用的

查了第八块新引进的每一处会改「给多少」的地方，没造出多给的形：
- 借之前 poll 销账：见 K1-1，销的是缓冲已经丢掉、活已做完的那几笔；多给要靠「缓冲比借据活得久」或「poll 与销账之间进来一笔快速失败的」，两样都没造出。
- 丢了的设备按放没放掉判：放掉之前这张卡照分不出算，取 0（「| DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:211`）），少给；
  放掉判成早了才会多给（K1-2：没找到旧设备另有句柄）。
- 重起时没等成的借据一起销（行 866）、记成丢了的设备上等的借据不进 kept（行 1498 的 `contains`）：两处都落在「分不出就取 0」那一段里，不会多给。
- 第九块不碰 GPU 账。
推翻它的现象：同第 K1-1、K1-2 两格；另外，任何一条「借据销了账而它的缓冲还在卡上」的运行记录（`GPU_CARD_LEASE_RETURN_WAIT_FAILED` 不出现、却读到该张卡上这个进程占着的比账上多）。

### K4-2　除用户已认的之外，有没有一形卡放得下却报本次未跑、判红、整趟弃卡，或并发被压住（**打中，低危**）

**借之前取留着的设备会等一次正在进行的设备重起**。`take_in_the_devices_found_idle` 对账上有没等成借据的每张卡先取留着的设备：「let Some(started) = self.cards[card_position].started.started_now() else {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1335`），
`started_now` 拿的是卡池那一格的锁（「pub fn started_now(&self) -> Option<Device> {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1637`）），而 `started_or_start` 在整个 `start()` 里拿着同一把锁：
「let mut started = self.started.lock().unwrap_or_else(PoisonError::into_inner);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1615`）、「let device = start()?;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1621`），文档也写着「起的时候拿着这张卡的锁：同一张卡上同时要的等这一次起完（别的卡不等）。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1605`）。
量（自己的探针，`probe_started_now_blocks.rs`，一个线程重起设备睡 300 毫秒，另一个线程在 50 毫秒后问 `started_now`）：

```
PROBE-LOCK started_now returned Some(2) after 250 ms (restart takes 300 ms)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
```

窗口有多窄：只有「列卡时这张卡上还有没等成的借据」与「它的设备正在被重起」同时成立才等。重起的头一步 `prepare_the_driver_share_before_starting_the_device`（「.prepare_the_driver_share_before_starting_the_device(card_position)」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1375`））
就把这张卡上没等成的借据销了（行 866），列卡在它之后看到的是空。所以要列卡发生在重起已拿到锁、而 prepare 还没销之间（推的，没量，量级毫秒）。代价：这次借被拖到重起做完（起设备加编一个小内核，量级秒；推的，没量真卡上的重起用时），哪怕它要借的是另一张有空的卡；
不判红、不弃卡、结果不错；起不来设备时的失败走原来的路。是新代码引进的（改动前借之前不碰这把锁）。
建议的方向（归主 agent 判）：借之前这一下取设备改成不等锁（拿不到就跳过这张卡，下一次借再来）。
推翻它的现象：重起期间列卡时这张卡上的没等成借据总是已被销掉（比如 prepare 改到拿锁之前）。

其余查过、没打中的：
- 每次借多了一次账的锁（列卡），而账的锁在别的线程借时会跨一次 `nvidia-smi` 读（「let mut books = self.books_locked();」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1111`）接着「let rooms = self.rooms_of_the_cards(&books, card_positions)?;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1112`）），
  所以每次借多排一次队；不改给多给少，只是慢一点，与已有的串行读同一种代价（背景材料第二节「用户已认」里的约 1.3 毫秒窗口是另一回事，这里不借它）。
- 少给的残留（K1-1）：同一台设备上另有借据持续提交时，没等成的那笔额度一直被占到别的借据等成。这是 G2 乙「少借」那一半，用户已认；今天比第六块短。
- 借据丢掉时那一等最多 30 秒、等的是到此刻为止这台设备上的全部提交（包括别的并发借据的活）：这是 G2 乙本身（用户已定），不报。

## 六、K5　测试判别力

### 六·一　表里的变异行复跑：能复跑的 30 行全红

背景材料改动全文里 harness 与 core 两档的变异表行（第七块 1、第八块 13、第九块 7、第四块 H1 1、第六块 5、第五块 1、第10x 批 1、第10z 批 1，合 30 行）逐行套到拷贝上跑那一行点名的测试：
`logs/rows-harness-1.log` 14 行、`logs/rows-harness-2.log` 16 行，`grep -c '^RED' ` 各为 14、16，`grep -c '^GREEN'` 都是 0。

checker 档的扫源码用例（四条）的模型：基线四条过；表里指向这四条的变异行全红（`logs/scan-model-mutations.log` 里 `diff` 行 `RED(caught)` 共 9 行：第七块 2、第八块 K4 五项 A–E 里的 5 行、第八块 GPU 取卡 2）。
同一份日志里的 7 行 `GREEN … diff` 是**我没有模型化的用例**（超时、`PollStatus` 判法、第六块 G2 乙、F5 扫 wgpu-core 的用例，它们要 wgpu 类型或 GPU），不是没拦住，也不是我验过它们拦住——这些我没跑。
读代码判：`a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty` 对 `QueueEmpty | Poll => true` 必红（它断言 `!device_found_idle_by_a_poll(&Ok(Ok(wgpu::PollStatus::Poll)))`）；`a_lease_waits_for_every_submission_…` 对超时 0 与不设超时必红（`assert_eq!(timeout, Duration::from_secs(30))` 与 `panic!` 分支，行 2291）。读的，没跑。

### 六·二　自己造的「改回去」变异，照绿的

| 变异 | 套在 | 结果 | 说明 |
|---|---|---|---|
| 丙并「含已释放的记录」（去掉 `!is_released` 过滤，`mount.rs:2030`） | 第九块新用例 11 条 + 相关 harness 用例 19 个文件（`logs/own-mutation-bing-includes-released-wide38.log`，`rc 0`） | **绿** | 多护。挂载那一刻 `placements_named_by_unapplied_journal_records` 里只有「现行账里没释放」的豁免（行 1646），已释放的（含早已回收的）会被隔离、进第九项：空间白占，方向是准入白拒。「会隔离」是读代码推的，用例结果是量的。没有任何用例钉丙的集合不多、不少 |
| 丙只并前一半记录（`.take(records.len() / 2)`） | 同上 11 条 | **绿** | 少护。新用例只核「R 与 P 共用的文件数据单元」是否落在借到的那一份里 |
| 丙只并跨度 ≥ 2 的记录 | 同上 11 条 | **绿** | 少护，漏掉跨度 1 的索引节点等。我没造出 R 与 P 共用跨度 1 单元的盘面（384 槽小盘上映射树只有一片叶，R 重写它），所以这一缺口在真树上会不会红，我没证 |
| 丙只并设备 0 的记录 | 同上 11 条 | 红（两条丙用例） | 对照：这一项被钉住 |
| 丙的跨度全当 1 | 同上 11 条 | 红（两条丙用例） | 对照：这一项被钉住 |
| 叫停闭包传给 `judge_my_blocks_on_gpu` 的那一行换成 `&|| false`（行 ~5733） | 模型 | **绿** | 扫源码用例只钉 A–E 五行（`let stop = stop_request.stop_of_the_run();` 等），不钉闭包有没有传到下游（B 项只钉起下一张卡那个函数里有一行 `work.stop_of_the_cards()`） |
| 叫停闭包传给 `check_selected_blocks` 的那一行换成 `&|| false`（行 ~5768） | 模型 | **绿** | 同上；CPU 对拍路径的叫停有没有别的用例钉，我没跑 checker 档，不知道 |
| 借之前 poll 那一句注释掉（`lease_card`） | 模型 | **绿** | `the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first` 的扫描走的是没剥注释的源码（「.find("self.take_in_the_devices_found_idle();")」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2376`）），注释里的那一句也算数；同一份文件里 `crash_amplification.rs` 那条用例是剥注释的（「.filter(|line| !line.trim_start().starts_with("//"))」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4672`）） |
| 借之前 poll 那一句包进 `if false { … }`（`card_rooms_now_and_when_empty_of`） | 模型 | **绿** | 同上，只比先后位置 |
| 产品路径一处调用点换成字符串里带 `print_the_not_run_line_or_refuse(` 的一行 | 模型 | 绿 | 数行数的口子；做作的一形，只记一笔 |
| `release_a_device` 不销该台上没等成的借据 / 不把在用的那台清成 `None`；借据只等它那张卡上的设备那条断言去掉卡位比较 | harness 库单测 104 条（`logs/own-mutations-ledger.log`） | 绿 | 前两项走不到产品路径（放掉的是在用的设备只在进程退出时），第三项是守卫断言；低价值，只记 |

### 六·三　e158 的阳性对照（读的，没跑）

新加的那一段先经同一组带故障的盘读被抛弃根的分配记录树节点，断言每一份都读不出，再抬 F。故障计划是「每次命中都失败」（`FaultOccurrence::EVERY_MATCHING_CALL`），多读几次不会挪走故障。
一个脆处：装故障时对同一块盘只取 `fault_targets` 里第一个偏移（`find`），而对照循环对 `allocation_locations` 的每一份都断言读不出；这块盘上要是有两个节点，对照会红（假红，不是假绿）。今天过（背景材料写 `95 passed`），我没跑。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 | 一致（带窄残留） | 销账四个出口；多给的形没造出；残留是别的借据持续占着同一台设备时少给；拿锁见 K4-2 |
| K1-2 | 一致 | 按克隆放掉的前提（没有句柄比卡活得久）今天成立但没有会红的检查；没找到多给或永不放的形 |
| K1-3 | 一致 | 只在已经展开中才不抛；没有借据在被预期 panic 接住的展开里；借之前 poll 的 `catch_unwind` 吞得不留痕，记一笔 |
| K2-1 | **冲突** | 丙罩不住管理员回退（向前发布）复活的单元（P 的账里已释放），探针在入库装置上红 I-7.4；对照含已释放的记录则过 |
| K2-2 | 一致（另记一笔量） | 放的时机与 H2 之前一样；第九项同形；C 那一槽装整份账后受护扫描是 落点数 × 回收记录数，4×10^4 × 4×10^3 是 1.1 秒（调试构建） |
| K2-3 | 一致 | 有效根槽读坏照拒；调用方都改了；公开函数另两个参数仍传空表，改动前就这样 |
| K2-4 | 一致（没造出红） | 空条目来源只剩凭据放行；坏盘记录那一支，按读代码，同槽的有效根同时会并进非空的账 / 映射 / 丙（读的，没造出反例） |
| K3-1 | 一致 | 没有下一张卡不问叫停；叫停每领一块之前仍问，叫停之后不起新卡领块 |
| K3-2 | 一致 | 三处产品路径调用点都经打印那一半，不打印的那一半只在它里面被调 |
| K4-1 | 一致（没造出多给） | 新引进的每一处改「给多少」的地方都落在少给或分不出取 0 的方向；前提同 K1-1、K1-2 |
| K4-2 | **冲突（低危）** | 借之前取留着的设备与设备重起共用一把锁，量到等 250 毫秒；窗口只在重起头几毫秒；不判红不弃卡 |
| K5 | **冲突（部分）** | 表里 30 行全红；丙的集合多护 / 只护一半 / 只护跨度 ≥ 2 三种照绿；叫停闭包传到下游与借之前 poll 被注释掉照绿（扫源码） |

## 八、没做什么

- 不替主 agent 采纳或出判决；上面「冲突」是我读到、量到的现象，K2-1 的前提三个同时成立才红，窄不窄归主 agent 判。
- 没跑 checker 档任何测试（含 e158 内联单测、`gpu_unit_checks`、`crash_amplification` 的库单测）、没跑任何带 gpu 特性的测试、没起 GPU 设备、没跑门禁阶段（`.claude/gate.d/` 下的）。
  checker 档四条扫源码用例我是搬了逻辑进 `scan-model/` 跑的，是复现不是用例本身。
- 没重量 G2 乙的前提（E174：等完设备之后缓冲交回驱动）。小缓冲经 wgpu-hal 的分配器是否当场还给驱动，我没查。
- 没造出丙的少护变异（只护跨度 ≥ 2）在真树上会红的盘面；没在发布构建下量 K2-2 的耗时；没量重起设备的真实用时（K4-2 的「秒级」是推的）。
- K2-1 我只找到复活这一类：`resurrect_released_record` 在产品代码里只有一处调用（`grep -rn 'resurrect_released_record' --include=*.rs crates/`：`mount.rs:7763`，allocator.rs 的另外几处是定义和库单测），即回退。回退之外的发布种类（覆盖写、顺序写、删除批、抬 F 的空发布、新建 inode）R 引用的单元是否都落在 P 没释放的账里或 R 点名的里，是读代码推的（新建 inode 一种由第九块的用例覆盖），其余种类没逐个造；也没证明没有第三种。
- 没有写任何修法进 `crates/`，没碰 kb；所有改动都在草稿目录和模型目录里。

## 九、草稿目录与清理

- 模型目录（入库候选，主 agent 定留不留）：`research/prompts/m3-batch10z-ghi-code-r1-sonnet-model/`（`README.md` 写了怎么复现；`logs/` 是上面各条的原样输出）。
  `logs/probe-rollback-revived.log` sha256 `5c0ebcfe413ed6646613b70ff8d7b1e1d58b72ae76d1e8f439f8658dd3ece761`；`probe_rollback_revived.tail.rs` sha256 `224456fe103d1a165a6738682b3b87a88642e31682f890412791a19a3d063a83`。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1-sonnet/`：我建的仓副本 `copy-a`（22 MB）、编译目录 `target-a`（6.6 GB）与 `target-scan`（13 MB）已在报告写完后删掉（逐个：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-ghi-code-r1-sonnet/copy-a`、`…/target-a`、`…/target-scan`，`ls` 核过已不在）；留下的只有三小样（`probe-rollback-revived.log`、`scan-model/`、`tools/`），模型目录里都有拷贝。主 agent 要复核，从冻结副本重新拷一份，按模型目录的 `README.md` 做。

## 十、cite-check

`python3 scripts/cite-check.py research/prompts/m3-batch10z-ghi-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree --root /home/fy5090/code/singlefs` 末行：

```
  ✓ 核了 51 处引文，对上 51 处，没判 0 处（逐处列在上面）
```
