# 第 10z 批第六块代码轮第一轮：攻方（opus）报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G2 F1 F2 F4 F5 M05 M10 M13 M14 M17 M22 M23 X1 X7 X17 H1 H2 H3 H4 H5 H6 -->

轮名 m3-batch10z-f-code-r1。被判对象是冻结副本 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree/`（开工时对 `research/prompts/m3-batch10z-f-code-r1-snapshot/crates-sha256.txt` 跑 `sha256sum -c`，不过的 0 条）。下面引代码的路径都相对这个冻结副本。腿自己量的数都是副本或腿的模型上的数，不是入库装置上的数。

## 复跑

仓根下跑（经内存包装 3G、线程上限 8，只编、跑 harness 档与一个不碰卡的小程序）：

```
bash research/prompts/m3-batch10z-f-code-r1-opus-model/rerun.sh <草稿目录>
```

它在草稿目录建两份冻结副本拷贝（`copy-a` 原样，`copy-b` 打攻方改法 `fix-fk1-fk3.diff`），各用自己的编译目录，产出 `out-a.txt`、`out-b.txt`、`mutants.txt`、`unwind.txt`；与模型目录里的 `output-a.txt`、`output-b.txt`、`output-mutants.txt`、`output-unwind.txt` 比。本腿在 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1-opus/rerun2/` 跑过一遍，`ATTACK` 行与开发时的三份日志逐行相同（`diff` 无输出），整条 67 秒左右。

模型目录 `research/prompts/m3-batch10z-f-code-r1-opus-model/` 每个文件的 sha256（`sha256sum *` 原样）：

```
89f9302f8bb5eb97e5d64d7bccc0b34495f52b283e3bd1bef725e12158d3a4e4  all-mutants.tsv
d6bd07c2dada8ec7a2afdb7594a13818813dd8e147dceb915a548951d3412530  attack_10zf_ledger.rs
ddb12067dd2f9d2e95daef00f0b259e71e1484e724438613cba09e96a014ddc0  attack_10zf_rounds.rs
e8b6078ab4b66dbcee6a18996ef07dadf5a4bba6a6f4f628633d6b07cdf8c326  extra-mutants.tsv
3abc444cab97ee6e4d202a69241dd5a9ebad225a32beb9b2cce334e6e100a0ad  fix-fk1-fk3.diff
614b33deef0ad8d9e862c6940450bf2e68acd762da3079454694e885ec5bf2b5  mutate-harness.sh
9181cb955ea4623bda86d00aa14129726b6b9ae06e1ec40b8e323f383a560dff  output-a.txt
8f3611e77cfc20de610fe6c714bb5205645ca843b5b75a463b53694086f13e5d  output-b.txt
d33dd533e137f957077880d7df3bf1b4873e7dbe8660244971a5ffa90f735812  output-mutants.txt
4d3850d6281bb45d765682a3205af3d5187a3100a72bdfcf678d472d0a07621f  output-unwind.txt
7bdf88296650f93aecbfad721a4435713efff9eb3b32a86c75190eb4d587add8  rerun.sh
c90ed34f1f1a97e2c0890baeccca77ed95071271b9f1675829cc67b01a64ab20  unwind_in_drop.rs
```

模型怎么搭：`attack_10zf_ledger.rs` 原样调入库的 `GpuMemoryLedger`（harness 档），读数由一张「真卡」给：`occupied` 是这个进程此刻真占的，`growth` 是之后还要长上去的（活着的借据没建的、驱动那一份长到 D 的）；真能再借的 = 空卡 32 110 − occupied − growth，再与额度 32 000 − D − 活着的借据取小。D = 2 560、驱动实占 600、额度 32 000，与入库用例 `a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more` 同一组数。旧设备连它上面的缓冲什么时候还给驱动，是我定的：旧设备的最后一个句柄丢掉的那一下（池里那一格换成新设备，或拿着旧设备的借据的卡丢掉，后发生的那一下；依据是 wgpu-core 30.0.1 `Queue::drop` 先 `wait_for_idle` 再收拾，见 K1-3 节）。`attack_10zf_rounds.rs` 原样调 `RoundsOfTheCards` 一族，checker 档外层循环照次序抄成单线程。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 每条交回缓冲的路先等了没有 | **没打中**（不等就销账的路一条都没造出）；**另报一形 H3**：用例已经 panic、借据在展开中丢掉、等的那一下 wgpu 又 panic，`resume_unwind` 让整个测试进程中止 | Rust 语义在不碰卡的小程序上量过（rc=134）；wgpu 在丢了的设备上等时 panic 是读源码推的 |
| K1-2 等失败那一支 | **打中 H1**：没等成的那一笔一直记着，卡上早还了；两次超时之后这张卡账上能借的是 0、真能借 29 440，之后每一笔都报本次未跑，直到这张卡的设备丢了 | 模型量过；超时在本机可不可达是推的（要卡） |
| K1-3 丢了的设备两种状态 | **打中 H2**：重起之后才丢掉的旧设备借据要是没等成（等的是丢了的设备），「那一刻活着的」永远空不了，这张卡一直照分不出算，直到下一次丢设备；6 种次序里 2 种，账 0、真 14 440 | 模型量过；旧设备上等会不会失败是推的（要卡）。多给：重起之前那一下多给 6 960，重起时照分不出再判一次，记下不算打中 |
| K1-4 「卡上可能还没还」拿干净没有 | **没打中** | crates 里 `returned_possibly_still` 0 处 |
| K2-1 叫停记住 | **没打中** | 卡线程那一个包着整趟那一个 |
| K2-2 头一句 OOM 那一行 | **分歧，不判打中**：照「这一趟」打而照「这一轮」不打的 21 / 91 段，判哪种读法对交主 agent | 模型量过 |
| K2-3 七条用例与抽到 harness 档的两处 | **没打中**：harness 档 22 条变异行在副本上逐条改坏都红 | 量过 |
| K3-1 账多给 | **没打中**（稳态 0；重起之前那一下见 K1-3） | 模型量过 |
| K3-2 放得下却本次未跑、判红、弃卡、压并发 | **打中**，就是 H1（本次未跑）、H2（本次未跑）、H3（判红）三形 | 同上 |
| K4 测试判别力 | **打中 H4、H5、H6**：碰卡用例在提交和丢之间夹了一次 `nvidia-smi`，改回 Poll 照推是照绿（要卡核）；harness 档两处本批新加的行删掉照绿（X1、X7，量过）；`catch_unwind`、`resume_unwind`、超时 30 秒三处没有会红的用例、没有变异行 | X1、X7 量过；H4 推的 |

打中各形的四句与改法表在「打中之后的四句」「改法表」两节。正文第二节「已知、在修」的两处（`next_card_to_start` 多问一次叫停、库单测打两行 `NOT_RUN_NO_GPU_ADAPTER`）没有报。

## K1　G2 乙做到没有

### K1-1 每一条交回缓冲、然后销账的路

逐条走了五条路：

| 路 | 先等了没有 | 依据 |
|---|---|---|
| 卡池借据 `GpuCardLease` 丢掉 | 等：`Wait { submission_index: None, timeout: 30 秒 }`，等成才销账 | 「timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1266`）；「self.booking.take_in_the_wait_before_returning(wait);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1280`） |
| 借到了、起设备失败（头一回起或重起） | 不等，借据直接丢掉销账；这一笔上没建过缓冲，不用等 | 起设备在交成 `GpuCardLease` 之前，「.prepare_the_driver_share_before_starting_the_device(card_position)」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1327`） |
| 重起时照重起记 | 没等成而记着的那几笔这时销账，不再等；它们在旧设备上，此刻照分不出算（占着的当 0），只会少借 | 「for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:847`）、「DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => return 0,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:206`） |
| 不走卡池的设备 | 不进账，没有销账这一步 | 文件头「没起过卡池的进程照旧不进账」那一句 |
| panic 展开 | 等的那一下 wgpu panic：接住、记成没等成、照样往外抛，账不销 | 「let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1263`）、「std::panic::resume_unwind(panic);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1282`） |

调用方那一侧：缓冲都在借据丢掉之前丢（`crash_judge_dispatch.rs` 第 948–949 行先丢判器再丢借据；第 ① 段 `ScratchOfTheCards` 的借据在最后一个字段，「leases: Vec<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>,」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2767`），判器在 `thread::scope` 里先收掉）。不等就销账的路没造出来。

**H3（另报一形，展开中再抛）**：用例本身已经在 panic（设备丢了，提交或没接住的错误 panic），借据在展开中丢掉；等的是丢了的设备，wgpu-core 把等时设备上的错交给 `handle_hal_error`，wgpu 前端对不是超时的错走 `handle_error_fatal` panic（`wgpu-30.0.1/src/backend/wgpu_core.rs:1924`：「self.context.handle_error_fatal(err, "Device::poll")」；`wgpu-core-30.0.1/src/device/resource.rs:885`：「let hal_error: WaitIdleError = self.handle_hal_error(e).into();」；只有超时与提交号不对转成 `PollError`，`wgpu-core-30.0.1/src/device/life.rs:150`：「WaitIdleError::Timeout => Some(wgt::PollError::Timeout),」）。`catch_unwind` 接得住这一次 panic，但随后的 `resume_unwind` 是从展开中的析构里往外抛，Rust 中止整个进程。不碰卡的小程序 `unwind_in_drop.rs` 照原样写了这一形（`poll` 换成必 panic 的闭包），量到：

「UNWIND arm=as-shipped rc=134 stdout_lines=0 abort_line=1」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-unwind.txt:1`）
「UNWIND arm=not-while-panicking rc=0 stdout_lines=2 abort_line=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-unwind.txt:2`）

stderr 里是 `panic in a destructor during cleanup` / `thread caused non-unwinding panic. aborting.`。中止的是整个测试二进制：同一进程里别的 GPU 用例都没了结局，也打不出「本次未跑」那一行，门禁记成判红。改前用 `Poll` 时，`get_fence_value` 在丢了的设备上报错也同样会在析构里 panic（推的，没上卡），所以这不一定是这一批带进来的；这一批的写法让「接住再抛」成了确定的结局。改法 F-H3：`std::thread::panicking()` 时只记没等成、不再抛（小程序上量过，见上面第二行）。

### K1-2 等失败那一支

**H1 打中**。模型 `k1_2_…`：借 K 20 000、起设备、建满；K 丢掉时设备上还排着比 30 秒长的活，`Wait` 超时；之后活做完、缓冲交回驱动。

「ATTACK arm=timeout-kept step=k_wait_timed_out_buffers_still_in_flight ledger_room=9440 truth_room=9550 under=110 over=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:2`）
「ATTACK arm=timeout-kept step=k_work_done_memory_back ledger_room=9440 truth_room=29440 under=20000 over=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:3`）
「ATTACK arm=timeout-kept step=book_15000 result=refused:NoCardHasRoomNow{need_mebibytes:15000,card_rooms_mebibytes:[9440]}」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:4`）
「ATTACK arm=timeout-kept step=after_50_leases_waited_and_returned_normally ledger_room=9440 truth_room=29440 under=20000 over=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:5`）
「ATTACK arm=timeout-kept step=after_2_timeouts ledger_room=0 truth_room=29440 under=29440 over=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:6`）
「ATTACK arm=timeout-kept step=after_a_device_loss_and_restart_judged ledger_room=29440 truth_room=29440 under=0 over=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:8`）

- 多给：没有。超时那一刻缓冲还在卡上，账少给 110（驱动还要长的那一份照 D 算），之后只会更少。
- 「一直少借」最坏少到多少：没有上界，等于所有没等成的那几笔之和，封顶是整张卡。每次借满此刻能借的、都超时，两次之后账上能借的是 0（上面第 6 行），之后连 1 MiB 也借不到（第 7 行 `NoCardHasRoomNow{need_mebibytes:1,…[0]}`）。空卡上放得下，所以是借不到、不是放不下：卡池用例报本次未跑，崩溃放量第 ① 段 `book_states_on_the_cards_with_room` 交 `NoCardHasRoomNow`，这一趟本次未跑。单卡机器上就是这个进程之后的 GPU 活全部本次未跑；多卡机器上这张卡等于弃了。
- 出不来：之后 50 笔照常等成、还掉，一个 MiB 都没回来（第 5 行）。唯一的出口是这张卡的设备丢了、有人来重起（第 8 行）；设备不丢就到进程结束。
- 超时可不可达（推的，要卡）：`Wait { submission_index: None }` 等的是这个设备上此刻最后一次提交（`wgpu-core-30.0.1/src/device/resource.rs:861` 起那一支取 `last_successful_submission_index`），同一进程里别的用例、别的崩溃放量趟在同一张卡的同一个设备上排的活都要等完。代码自己说一块可能比进度间隔长：「// 卡上判一块、编内核都可能比进度间隔久：等着收的这条线程隔一个间隔打一行按挂钟的」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3066`），进度间隔「pub const PROGRESS_LINE_INTERVAL_IN_SECONDS: u64 = 300;」（`crates/singlefs-checker-tier/src/crash.rs:2989`）。一块是一次提交还是几次，我没查到，所以「排着的活超过 30 秒」是推的。E174 两段一次都没超时（`wait_always_returns=true`），但它最长只量到另一条线程提交 400 毫秒的活、等 0.3 到 0.69 秒那一形。

### K1-3 丢了的设备两种状态

**H2 打中（少给出不来）**。历史：旧设备上借着 A（10 000，建满）；设备丢了；另一条用例借 L（5 000）去起卡，见设备丢了、照重起记（这时「那一刻活着的」是 {A, L}）、起好新设备，状态进 `DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`。之后三步（重起、A 丢掉、L 用完还）的先后由别的用例定，A 的等成不成由设备定，这几样全放开扫了一遍（3 种次序 × 2 种等的结局；L 要先起卡才建得了，所以 L 还在重起之后）。最后一条长借据 Q 15 000 建满，再来一条要 10 000 的，再照常借还 100 笔 1 000：

| 次序 | A 的等 | 最后少给 | 要 10 000 的那笔 | 100 笔 1 000 借不到的 | 出处行 |
|---|---|---|---|---|---|
| 重起 → A 丢 → L 还 | 等成 | 0 | 借到 | 0 | `output-a.txt` 第 16 行 |
| 重起 → A 丢 → L 还 | **没等成** | **14 440** | **借不到**（真放得下） | **100** | 第 20、21、24 行 |
| 重起 → L 还 → A 丢 | 等成 | 0 | 借到 | 0 | 第 32 行 |
| 重起 → L 还 → A 丢 | **没等成** | **14 440** | **借不到** | **100** | 第 36、37、40 行 |
| A 丢 → 重起 → L 还 | 等成 | 0 | 借到 | 0 | 第 48 行 |
| A 丢 → 重起 → L 还 | 没等成 | 0 | 借到 | 0 | 第 56 行 |

「ATTACK arm=lost-device:Restart>DropA>ReturnL-a_wait_failed step=q_15000_built ledger_room=0 truth_room=14440 under=14440 over=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:20`）
「ATTACK arm=lost-device:Restart>DropA>ReturnL-a_wait_failed step=100_leases_of_1000 refused=100」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:21`）
「ATTACK k1_3 histories_refusing_a_lease_that_fits=2 of 6」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:57`）

机理：A 没等成，丢掉时只记进「没等成的」、不销账（「self.when_dropped = LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;」，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1414`；「LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait => {}」，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1434`），于是它也一直留在「那一刻活着的」里；状态回到 `DeviceStarted` 只在销账时那一处（「if self.leases_alive_when_the_device_was_lost.is_empty() {」，`crates/singlefs-harness/src/gpu_memory_ledger.rs:871`）。没等成的那几笔只在照重起记时销，可 A 丢掉时重起已经做过了，要等这张卡的新设备也丢了、下一次照重起记才出来。这期间 A 的 10 000 一直在账上，新设备与新借据建上的也全照「占着的当 0」算，账同时少给这两份。真卡上 A 的卡是旧设备最后一个句柄，A 丢掉时旧设备连 A 的缓冲就还了（`wgpu-core-30.0.1/src/device/queue.rs:281`：「match unsafe { self.raw.wait_for_idle() } {」，`Queue::drop` 等空了再收拾）。

可达性（推的，要卡）：没等成有两种来路。一是旧设备在驱动那一层真丢了（驱动重置一类）：在它上面 `Wait`，Vulkan 交 `DEVICE_LOST`，wgpu 走 `handle_error_fatal` panic，接住记成没等成（K1-1 那一段的源码）。二是旧设备只是 wgpu 记成丢了（显存不够一类），而它上面还排着超过 30 秒的活。两种都要「另一条用例先借、先重起，A 后丢」这个先后，而几条用例共用一张卡时这正是常态：头一个撞上丢失的用例让设备记成丢了，下一个借的用例重起，别的用例手里的借据这时才陆续丢。

只看 `A 丢 → 重起` 那一行：A 先丢、照重起记时作为「没等成的」销掉，出得来。这个先后不是这一批能定的。

多给：两种状态里稳态没造出多给。造出一处窗口：`A 丢 → 重起 → L 还`、A 等成，A 丢掉那一刻账上状态还是 `DeviceStarted`（还没有人来照重起记），旧设备连 A 的缓冲还在池里那一格上占着卡，读数里这个进程占着的把它算成 L 已经建上的：

「ATTACK arm=lost-device:DropA>Restart>ReturnL-a_wait_finished step=DropA ledger_room=21510 truth_room=14550 under=0 over=6960」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:41`）

多给 6 960，正好是 L 还没建的 5 000 加新设备驱动那一份要长的 1 960。这一下之后谁来借都要先起卡，起卡时见设备丢了、照重起记、再照「占着的当 0」判一次放不放得下（「|started: &GpuCard| started.device_condition().is_lost(),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1320`），第 1327 行起 `prepare_the_driver_share_before_starting_the_device`），借到的这一笔要是放不下就当场拒、报本次未跑，建不上缓冲。这与第 10z 批第三块判决 K1-1 记下的「设备在卡上丢了、账还没被问到的那一下读数多给」同一个窗口；改前那一下最多一份驱动，因为销了账的 A 当时还记在「销了账、卡上可能还没还」里扣着，乙拿掉那一份之后这一下变成 L 没建的加一份驱动。窗口有重起那一道再判兜着，我没造出照它建上缓冲的历史，记下、不算打中。

少给的那一段（另一问）：除了 H2，两种状态下的少给都在「那一刻活着的」都还了之后回来（表里等成的三行最后少给都是 0）。

### K1-4 「卡上可能还没还」那一份拿干净没有

`grep -rn 'returned_possibly_still' crates --include=*.rs | wc -l` 输出 `0`，`crates/mutations.tsv` 里同样 0 处。按旧口径往下压的那一步（`returned_possibly_still_on_the_card_after_a_reading`）整个删了，往上加的没找到；文件头「口径」一段改成乙的说法。另有一处同名的「上一趟丢掉的可能还占着」是 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD`（一笔借据里几趟派活的缓冲怎么算），与销账无关，照旧。没打中。

## K2　第五块判决第三节三条做到没有

### K2-1 叫停记住

整趟只有一处读叫停文件：「let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5594`），交下去的是记住的那一个：「let stop_is_requested = || stop_request.is_requested();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5595`）。第 ① 段里卡线程与一轮收尾问的是另建的一个：「let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2947`），它包着整趟那一个。里面那个答叫停，一定是它现读外面那个时外面答了叫停，外面那个从此恒答叫停；外面那个先答了叫停，里面那个下一次问到时现读、也答叫停。所以卡线程、一轮收尾、整趟收尾（「let stop_was_requested = gpu_memory_was_short || stop_is_requested();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5677`））不会一处见过叫停而另一处没见过。叫停文件在头一次被读到之前就删了，是四处都没见过，一致。没打中。

### K2-2 头一句 OOM 那一行

打的那一句（「if let Some(not_run_line) = rounds.not_run_line_of_a_stop_after_an_out_of_memory(」，`crates/singlefs-checker-tier/src/crash_amplification.rs:3137`）取的是这一趟头一张报 Out of Memory 的卡（「.map(|reason| gpu_not_run_line(run_name, reason))」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2397`），前一行取 `self.first_out_of_memory`）。实现员自定第 5 处：比「这一轮」宽。

模型 `attack_10zf_rounds.rs`：两张卡，头一轮状态全装在卡 0、卡 1 不起；卡 0 判第 `oom_at` 块时报 Out of Memory（0–5 与不报）；叫停文件在第 `stop_at` 次问叫停时出现（0–11 与不叫停），共 91 段：

「ATTACK k2_2 summary histories=91 printed_this_trip=33 printed_this_round=12 differ=21 differ_that_complete_without_the_oom=15」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-a.txt:79`）

21 段两种读法分得开，全是同一形：头一轮卡 0 报了 Out of Memory、收尾再起一轮用卡 1，叫停落在第二轮，第二轮没有卡报，照叫停收尾。照「这一趟」打一行 `NOT_RUN_NO_GPU_ADAPTER … reason=显存不够：…`，门禁 feature-gated-tests 格多记一条本次未跑；照「这一轮」不打，外面只看得到一趟提早收尾。这 21 段里 15 段在同一时刻叫停、卡 0 不报的对照段里能判完（Out of Memory 拖慢了、叫停才截住），6 段对照段同样被叫停截住（`oom_at` 0–2、叫停来得早的那几段）。哪种读法对，看这一行要说的是「这一趟的显存出过事」还是「没判完是因为显存」：前一种照现在打，后一种那 6 段是多记的本次未跑。条款没写到这一层，我不判打中，分歧交主 agent。

### K2-3 七条会红用例与抽到 harness 档的两处

在副本 `copy-a` 上照 `crates/mutations.tsv` 第 3348–3369 行（这一批 harness 档的 22 行）逐条改坏、跑点名的测试、还原，22 条都红（`output-mutants.txt` 第 4–25 行全是 `result=caught`）。其中 M05、M10、M13、M14、M17、M22、M23 七条（第 19–25 行）、交回那一刻记设备（F4，第 17 行）、起下一张卡看叫停（第 18 行）、叫停记住两条（第 14、15 行）、叫停收尾打那一行（第 16 行）。checker 档的两行（3373、3374 源码扫描，3370–3372 碰卡或 checker 档 lib）是重型，没跑。没打中。

## K3　只防 OOM、不压并发两头

### K3-1 账多给

模型里稳态没有多给：K1-2 那一臂每一步 `over=0`（`output-a.txt` 第 1–8 行），K1-3 六种次序里只有 K1-3 节那一处重起之前的窗口（6 960，有重起再判兜着）。新代码里还有一处我推了、没量：读数里这个进程占着的里，有一块是分配器按块拿、块里另一笔借据还活着而销了账的那一截（E174 第一段「并存的长借据」那一组见过一整块 256 MiB 在读数里），乙把它算成许出去而已经建上的，最多多算一块；改前「销了账、卡上可能还没还」那一份会把销了账的那一笔扣回来。要卡才分得出块里是哪一笔，没量，不算打中。

### K3-2 放得下却本次未跑、判红、弃卡、压并发

- 本次未跑：H1（等超时一次就少一笔，两次之后整张卡借不出；单卡就是整个进程之后全部本次未跑）、H2（旧设备借据重起之后没等成，到下一次丢设备之前这张卡一直少给，模型里 100 笔 1 000 全借不到而真放得下）。
- 判红：H3（展开中再抛，整个测试二进制中止）。
- 压并发：`Wait` 只拿着 wgpu-core 设备的那把读锁等（`poll_and_return_closures` 里 `snatchable_lock.read()`），别的线程提交拿的也是读锁；拿写锁的只有显式 `destroy` 与延后的销毁，crates 里没有 `.destroy()` 调用（`grep -rn '\.destroy()' crates --include=*.rs` 零命中）。所以等的时候别的线程照常提交，压不到并发；代价是丢借据的那条线程自己要等这个设备上排着的活，E174 第二段量过 0.3 到 0.69 秒，这是用户定乙时认的那一份，不报。

## K4　测试判别力

**H4（推的，要卡核）**：变异行 3370「借据丢掉时改回只收一次（Poll，不等设备做完）」点名的碰卡用例，提交之后、丢缓冲之前先读了一次显存：「lease.card().queue().submit([encoder.finish()]);」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:83`），「let occupied_while_the_buffers_live = this_process_occupancy_on_the_card_mebibytes(&bus);」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:84`），然后「drop(buffers);」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:89`）、「drop(lease);」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:90`）。读一次 `nvidia-smi -q` 要几十毫秒，清两张 1 GiB 早就做完了，借据丢掉时活已经不在飞，这正是 E174 的「已做完」档。E174 第一段里不等待臂（就是 Poll）在「已做完」档、2 GiB、两张卡上第一份读数都不含：`research/results/e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out` 第 55、56、115、116 行四栏 `q2_next_reading=absent`、`q2_remainders_mebibytes=0;0;0;0;0`（那四行里 `q1_submit_to_poll_microseconds` 约 12 万微秒、`q1_work_microseconds` 约 5 万微秒，活做完了才 poll），kb 页 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:130`：「活已经做完的档（已做完、逐趟）下一次读数不含」。所以改回 Poll，这条用例照推是绿的，3370 那一行抓不住。要它分得出，提交的得是在飞的活（E174「在飞·中」那样空转 250 毫秒、S ≥ 512 MiB），显存读数挪到提交之前，提交之后紧接着丢缓冲、丢借据。

**H5（量过，副本上）**：两处这一批新加的行删掉，harness 档 `gpu_memory_ledger` 94 条测试照绿（`output-mutants.txt` 第 1、2 行 `result=survived`）：
- X1：照重起记时没等成的那几笔只从借据里删、不从「那一刻活着的」里删（删「self.leases_alive_when_the_device_was_lost」那两行，第 849–850 行）。H2 那一形里 A 从此永远在「那一刻活着的」里、借据里却没了，`return_a_lease` 再也删不到它，照分不出算就不止「到下一次丢设备」，是到进程结束（推的，按代码）。
- X7：重起好了、旧设备的借据还在时这张卡又丢一次，不再照重起记（`DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain` 那一支只改成没起好）。第二台设备上的借据不记成「那一刻活着的」、没等成的不销，第一台设备的借据一还完就回到照读数分，第二台设备留在卡上的被当成第三台的（推的，按代码；方向是多给）。
- 对照：X17（销账时不从「那一刻活着的」里删这一笔）红了（第 3 行 `result=caught`），所以模型的改坏、还原那一套是灵的。

**H6（读码）**：`GpuCardLease::drop` 的三样没有会红的用例，也没有变异行：`catch_unwind` 拿掉（等的那一下 panic 直接往外抛，借据走正常销账，`std::thread::panicking()` 时也不断言）；`resume_unwind` 拿掉或改成展开中不抛；超时 30 秒改成 0 或 `None`（「std::time::Duration::from_secs(30);」，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:27`）。3371 那一行改的是结局怎么分（`device_wait_before_returning_a_lease`），碰不到这三样。超时改成 0：碰卡用例里活已做完，`Wait` 立刻交 `QueueEmpty`，照推照绿；改成 `None`：等不到头的那一形没有用例。

其余：K2-3 那 22 行都红（见上）。

## 打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足判据字面的哪一分句 | 跑前条款给的改法在打中的那几格上还中不中 |
|---|---|---|---|---|
| H1 等超时永久少借 | 分辨：同一段历史，攻方改法 F-K1（之后有一笔在 `DeviceStarted` 下等成就把没等成的那几笔一起销）在第 5 行就回到真值（`output-b.txt` 第 5 行 `ledger_room=29440 truth_room=29440`）；实现员另一支「记成设备丢了」会让下一次借去重起、照重起记时销掉（推的） | 看得到：之后同一设备上任何一次 `Wait { None }` 等成，就说明更早提交的活都做完了 | K1-2「「一直少借」最坏少到多少，会不会整趟弃卡或报本次未跑？」后一半：会；K3-2「卡放得下却报本次未跑」 | 正文没给跑前改法；F-K1 修第 5 行那一形，两次超时之间没有别的等成时（第 6 行）照旧借不出，要等下一次等成（`output-b.txt` 第 6 行照样 `ledger_room=0`） |
| H2 旧设备借据重起后没等成 | 分辨：同一次序、A 等成的那一行最后少给 0，没等成的 14 440；F-K3（没等成的那一笔若在「那一刻活着的」里，借据丢掉时照销）6 种都出得来（`output-b.txt` 第 57 行 `histories_refusing_a_lease_that_fits=0 of 6`） | 看得到：A 丢掉时账知道它在「那一刻活着的」里 | K1-3「少给的那一段会不会永远出不来（活着的借据一直不还）？」：会，到下一次丢设备；K3-2「卡放得下却报本次未跑」 | 正文没给；F-K1 碰不到（状态不是 `DeviceStarted`）；F-K3 修 |
| H3 展开中再抛 | 分辨：同一程序两条臂 rc=134 与 rc=0（`output-unwind.txt`） | 看得到：`std::thread::panicking()` | K1-1「panic 展开」那一路：账没被销（记成没等成），但进程中止；K3-2「判红」 | 正文没给；F-H3 修 |
| H4 碰卡用例分不出 Poll | 分辨的是测试，不是臂：改法是改用例 | 用例当时能控制活在不在飞 | K4「有没有一形「改回去」照绿」 | — |
| H5 X1、X7 照绿 | 同上 | — | K4 同上 | 补用例：H2 那段历史（X1）、两次丢设备之间旧借据还在（X7） |

## 改法表

攻方改法都只在我的模型上量过、被攻过零轮。「量过」贴的是副本 `copy-b`（打了 `fix-fk1-fk3.diff`）或小程序上的原样输出；「推的」是按代码推、没实现或没跑。

| 改法 | H1 等超时永久少借 | H2 旧设备借据重起后没等成 | H3 展开中再抛 | H4 碰卡用例 | 别处 |
|---|---|---|---|---|---|
| F-K1：`take_in_the_wait_before_returning` 等成、而这张卡是 `DeviceStarted` 时，把「没等成的」一起销账 | 量过：之后一笔等成就回到真值，「ATTACK arm=timeout-kept step=after_50_leases_waited_and_returned_normally ledger_room=29440 truth_room=29440 under=0 over=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-b.txt:5`）；量过：两次超时之间没有等成的那一形照旧 0（同文件第 6 行） | 推的：碰不到（那时状态不是 `DeviceStarted`） | 碰不到 | 碰不到 | 量过：副本上 harness 档 `gpu_memory_ledger` 94 条照绿（`test result: ok. 94 passed`）；推的：安全性靠「`Wait { None }` 等成说明更早提交的都做完、这一次 maintain 已收掉它们」，与乙本身同一个前提 |
| F-K3：没等成的那一笔若在「那一刻活着的」里（拿着旧设备），借据丢掉时照常销账 | 碰不到 | 量过：「ATTACK k1_3 histories_refusing_a_lease_that_fits=0 of 6」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-b.txt:57`） | 碰不到 | 碰不到 | 量过：同上 94 条照绿；推的：靠「这笔借据的卡是旧设备最后一个句柄、丢掉时 `Queue::drop` 已等空收拾」；旧设备另有不在账上的句柄时，最后一笔旧借据销账之后回到照读数分，旧设备留在卡上的会被当成新的，多给最多那一份 |
| F-H3：`std::thread::panicking()` 时只记没等成、不 `resume_unwind` | 碰不到 | 碰不到 | 量过（小程序）：「UNWIND arm=not-while-panicking rc=0 stdout_lines=2 abort_line=0」（`research/prompts/m3-batch10z-f-code-r1-opus-model/output-unwind.txt:2`） | 碰不到 | 推的：展开中那一笔照没等成记着，进程照常往下 |
| F-H4：碰卡用例提交在飞的活（空转约 250 毫秒、S ≥ 512 MiB），显存读数挪到提交之前，提交后紧接着丢 | — | — | — | 推的：照 E174「在飞·中」不等待臂第一份读数含整笔，Poll 会红；要卡 | — |
| 实现员另一支「记成设备丢了」 | 推的：下一次借去重起、照重起记时销掉，出得来；实现员判它借与重起之间会多给，我没量 | 推的：同 H1 | 碰不到 | 碰不到 | — |

## 没打中的形状

| 试过的形状 | 取样 | 结果 |
|---|---|---|
| K1-1 不等就销账的路：起设备失败、重起、不走卡池、借据在缓冲之前丢（调用方 `crash_judge_dispatch.rs` 第 948–949 行、第 ① 段 `ScratchOfTheCards` 字段次序、第 ② 段与单元校验和的借据） | 读码，五条路 | 没有 |
| K1-3 两种状态的稳态多给 | 3 种次序 × A 等成或不成，之后 Q 15 000、100 笔 1 000 | 0（`over_max` 只在重起之前那一下窗口出现） |
| K1-4 旧口径残留 | crates 全量 grep | 0 处 |
| K2-1 叫停中途删、晚到 | 读码（里面那个包着外面那个） | 一致 |
| K2-2 一趟打两行 | 91 段 | 没有；只有「这一趟 / 这一轮」两种读法之分 |
| K2-3 22 行变异 | 副本上逐条 | 全红 |
| K3-2 等的时候锁住别的线程提交 | 读 wgpu-core 30.0.1 `poll_and_return_closures` 与锁 | 只拿读锁，压不到 |
| 提交之前 `write_buffer` 了、没提交就丢缓冲（挂在 pending writes 上，`Wait { None }` 罩不到） | 读码，没造 | 只在出错那一路；改前 `Poll` 一样；没量，不报 |
| 分配器按块拿、块里销了账的那一截（K3-1） | 读码 | 最多一块；要卡，没量 |

## 这条腿自己的限度

- 「真卡」是我定的：旧设备连它上面的缓冲在最后一个句柄丢掉那一下还给驱动；驱动那一份实占 600、要长到 D。换一种（比如丢了的设备在驱动那边还要过一阵才还），H2 的少给数会变，方向不变（账照旧照分不出算）。
- 要卡的都没做：H1 的 30 秒超时可不可达、H2 的旧设备上等会不会失败、H3 里 wgpu 在丢了的设备上等时是否 panic、H4 改回 Poll 照不照绿，都是读源码与 E174 推的。
- 两个模型都是单线程按次序走，不是真线程；K2-2 用「问了几次叫停」当时刻。
- 「没打中」各格是一次抽样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不能单独撑结论。
- checker 档测试（含 3370–3374 五行变异）没跑：重型。
- 没跑任何崩溃状态枚举，原型全量状态数 0。

## 要主 agent 跑的命令

核 H4（重型、要卡，只在提交时或用户要求时）：在冻结副本的一份拷贝里照 `crates/mutations.tsv` 第 3370 行把 `Wait { … }` 那三行换成 `device.poll(wgpu::PollType::Poll)`，再跑

```
SINGLEFS_HEAVY_TESTS=user-request bash /home/fy5090/code/singlefs/scripts/crash/gpu-quiesce-run.sh -- bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 8 cargo test -p singlefs-checker-tier --features gpu --test gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card -- --nocapture
```

工作目录是那份拷贝；看 `test result:` 那一行。照绿就是 H4 坐实（那一行变异抓不住），红就是我推错了。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；副本与模型上的数不算入库装置上的数。
- 没上卡、没跑 checker 档任何测试与 GPU 测试；H1、H2、H3 的可达性与 H4 都要卡（见上一节）。
- 没给 K2-2 下打中的结论（两种读法交主 agent）。
- 没量分配器按块拿那一形（K3-1）、`write_buffer` 挂在 pending writes 上那一形。
- 过程里收到主 agent 的消息：没有。
- 跑的时候内存包装的 slice 被别的会话一条 38G 的 checker 档用例占着：我第一次用 8G 起的模型编译排了 13 分钟没起，我停了自己那一条（pid 1741413，`proc.py stop`，包装退 251，那一次一行没跑），改用 3G 重排，又排了约 15 分钟才起；之后每条都在一分钟内跑完。

## 草稿目录

`~/.local/share/singlefs/work/m3-batch10z-f-code-r1-opus/`：建过的两份冻结副本拷贝（`copy-a`、`copy-b`）、两个编译目录（`target-a`、`target-b`）与两次复跑（`rerun1`、`rerun2`，各含两份拷贝与编译目录）交回前都删了。留下的：`dev/`（模型的开发版与小程序 `unwind_in_drop` 的二进制，4.5 MB，模型目录里是同一份源码）、`rerun2-outputs/`（第二次复跑的输出，与模型目录里四份 `output-*.txt` 同源）、`mut-logs/`（每条改坏一份 cargo 日志）、`run*.log`、`progress.md`。都不入库，模型目录里的输出与 `rerun.sh` 能重新造出来。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-batch10z-f-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree` 末行：

```
  ✓ 核了 41 处引文，对上 20 处，没判 21 处（逐处列在上面）
```

没判的 21 处是引模型目录、kb 页与 wgpu 源码的（不在冻结副本下）。加上仓根与 cargo 源码目录两个根（`--root <冻结副本> --root . --root ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`）再跑一次，末行：

```
  ✓ 核了 41 处引文，对上 41 处，没判 0 处（逐处列在上面）
```
