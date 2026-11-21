# GPU 服务 OOM 治理文件轮第一轮：正推腿（Sonnet）报告

日期：2026-11-19。立场：正推（第 1 到 4 问）。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 读了什么、在哪棵树上

- 读的是此刻主工作区的文件。开工时对着开工快照核过哈希：`sha256sum -c research/prompts/gpu-oom-card-governance-r1-snapshot/judged-sha256.txt` 共 17 行，16 行 `OK`，`task-server/task_server_selftest.py` 一行 `FAILED`（开工之后被别的会话改过）。本报告只在第 4 问末引了它的一处（`task-server/task_server_selftest.py:2451`，按此刻的树读，开工快照里的行号可能不同、没倒推）；别处引到的文件都是 `OK` 的那 16 个里的。
- 这一轮服务补丁的 10 个文件，当前哈希与 `merge-and-verify` 结局文件里「文件」行的计划改后哈希逐个相同（我现算 `sha256sum` 对 `apply-outcome.tsv` 的第 5 列，10 对 10 相同）；`tests/card_memory_matches_the_verification_package.rs` 不在这一份补丁里（`crates.patch` 的 `diff --git` 行共 10 条，没有它）。
- 只有我这一条腿读到的范围：`wgpu-core-30.0.1`、`wgpu-30.0.1` 的源码（`~/.cargo/registry/src/*/` 下，用来核「新表两份」的前提）。其余文件攻方腿也在读。
- 没读：别的腿这一轮的提示与产出、`research/prompts/gpu-oom-card-governance-r1-main-verification.md`。

## 要主 agent 知道的三件事（先说结论）

1. 第 1 问不成立两处、推不出一处：`card_worker.rs` 文件头最后一句把「核的时候 panic」说成报卡坏，与同一文件头开头一段和代码相反；`service_loop.rs` 文件头与 README 说核对组放回之后「先腾空那张卡」，代码挑的是任一张换出全部驻留判器后借得到的卡；文件头说「显存还给驱动」，而一波里先装完全部判器、之后才有第一次提交，装判器那一段撞上 OOM 时前面已装上的判器的表与暂存还被 wgpu 的待写列表拿着，`clear()` 放不掉（读 wgpu-core 源码推的，要真卡量；推不出，算打中）。
2. 第 2 问读代码三种情形各有确定的判，与文件头一致；但真卡那一份 `after_a_panic` 没有任何测试调过（替身那一份只有一个标记，没有计数、没有丢设备优先），所以真卡上的行为是推的；探针两条已写好（在仓副本里，待主 agent 跑）。
3. 第 3 问「新表两份」三处调用、登记、驻留账、对拍测试逐处都对得上；顺带查出一处文件头说过头的话：`service_loop.rs` 文件头说「池里不留永远派不出去的态」，登记之后大卡坏了、而小卡装不下这条流时，再入池的块没人拒、一直挂在池里（读代码推出，探针已写好待跑）。第 4 问整条链字面逐环对得上，另有一处注释点名了不存在的常量（`control` 第 311 行），README 一处与代码不符（第 94 行，同第 1 问）。

## 探针与交接（我写了什么、没跑什么）

我没起任何项目代码，没跑 cargo。下面两份探针写在仓副本里，只过了 `rustfmt --check`（语法），没编译，没跑：

- 仓副本：`/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-sonnet/repo-copy`（`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs <副本> --with-test-inputs` 建的，58M）。副本里相对仓根只改了三处：
  - `gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs` 末尾追加一条探针用例（替身判器、不上卡）；
  - 新文件 `gpu-verification-server/tests/probe_after_a_panic_on_a_real_card.rs`（两条用例，要卡，经卡池借）；
  - `gpu-verification-server/Cargo.toml` 末尾给上面那个新文件加一个 `[[test]]`（`required-features = ["gpu"]`）。
- 哈希（`sha256sum`）：
  - `df0ea4a0c5b55d1634f895374b33ea905127279a45fc377a14132dd67a4494cb`  `repo-copy/gpu-verification-server/tests/probe_after_a_panic_on_a_real_card.rs`
  - `cbf5191385ffbd20223163c21fe8492051ff72597c87620ee408df4d158b1dc0`  `repo-copy/gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs`
  - `07645a9184e9ebbd1421f9bad1afed421c1d10937e9487d5728735165d1c9e5a`  `repo-copy/gpu-verification-server/Cargo.toml`
- 没写模型（读代码就能推到头的几处，起一个独立模型只会把同一段逻辑抄一遍），所以没有 `tools/rerun.sh`。

## 第 1 问：「接住 Out of Memory」文件头逐句对代码

整问判定：**不成立**（有两句对不上，见句 6 与句 17），另有一句**推不出**（句 2 括号里那半句，算打中）。其余各句一致。下面每一句一个判定加代码依据；「一致」都指我读过代码、顺着那几行走通，不是跑过。

`gpu-verification-server/src/card_worker.rs` 文件头与 `card_worker_loop`：

- 句 1：「判的时候（或核的时候）panic，先问判器缓存这张卡的设备怎样」（`gpu-verification-server/src/card_worker.rs:6`）。**一致**。判一波与核一组都包在 `catch_unwind` 里（`gpu-verification-server/src/card_worker.rs:175`、`gpu-verification-server/src/card_worker.rs:179`），出错走同一个分支「if let Err(panic) = carried_out {」（`gpu-verification-server/src/card_worker.rs:184`），再「match judges.after_a_panic() {」（`gpu-verification-server/src/card_worker.rs:193`）；panic 的字句只放进 `reason` 带走（`gpu-verification-server/src/card_worker.rs:185`），不参与分支。附注：同一个 `catch_unwind` 还罩着换出与装上（`gpu-verification-server/src/card_worker.rs:230`、`gpu-verification-server/src/card_worker.rs:233`），装判器时撞上 OOM 同样被接住；文件头没写「装」，不冲突。
- 句 2：「设备没丢、这一回记过 Out of Memory：判器缓存已丢掉卡上全部判器（显存还给驱动），报一声接住了」（`gpu-verification-server/src/card_worker.rs:7`）。前半**一致**：真卡那一份在答 `RanOutOfMemory` 之前先「self.judges.clear();」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:313`），工作线程再报 `OutOfMemoryCaught { card, judges_dropped, reason }`（`gpu-verification-server/src/card_worker.rs:197`）。括号里「显存还给驱动」**推不出**（打中）：
  - `clear()` 只是丢掉 Rust 一侧的 wgpu 句柄；wgpu-core 的缓冲在最后一份引用没了时才销毁（`~/.cargo/registry/src/*/wgpu-core-30.0.1/src/resource.rs:524` 起的 `impl Drop for Buffer`），而设备上做过提交的资源还被生命期跟踪表拿着引用，要等下一次 `maintain`（提交里的 cleanup 那一段跑它，注释写「schedule destruction of all resources that are no longer needed by the user but used in the command stream」，`wgpu-core-30.0.1/src/device/queue.rs:1538-1539`；`maintain` 的参数是 `poll_type`，`device.poll` 走的也是它：`wgpu-core-30.0.1/src/device/resource.rs:826`）才放；
  - 更要紧的是判表缓冲的暂存：判表缓冲用 `create_buffer_init` 建，`unmap` 时 wgpu-core 把暂存缓冲与目标缓冲一起放进队列的待写列表（「pending_writes.consume(staging_buffer);」，`wgpu-core-30.0.1/src/resource.rs:1032`；「pending_writes.insert_buffer(self);」，`wgpu-core-30.0.1/src/resource.rs:1033`），要等下一次提交把待写列表刷出去（`flush_pending_writes`，`wgpu-core-30.0.1/src/device/queue.rs:1306`）才放这两份引用（`PendingWrites::insert_buffer` 把 `buffer.clone()` 放进 `dst_buffers`：`wgpu-core-30.0.1/src/device/queue.rs:419`；`consume` 把暂存缓冲放进 `temp_resources`：`wgpu-core-30.0.1/src/device/queue.rs:445-446`）。`GpuCrashJudge` 的构造里没有提交：`grep -n 'queue().submit' crates/singlefs-verification/src/crash_judge_dispatch.rs` 命中 1837、2025、2386 三处，1837 与 2025 在派活的方法里（`judge_states` 在 1535 行起），2386 在 `persisted_masks_on_the_card`（2294 行起），都不在构造（1357 行起）里。工作线程又是一波里先装完全部判器、再判各批（`gpu-verification-server/src/card_worker.rs:233`、`gpu-verification-server/src/card_worker.rs:241`）。所以在装判器那一段撞上 OOM（判表一次分配最大，最可能出在这里：推的）时，这一波前面已装上的判器，表与暂存还被待写列表拿着引用，`clear()` 丢句柄放不掉它们，要等下一次提交；紧接着单独判的那一波却先 `load_the_judge`、再有第一次提交。这一条能解释「单独在腾空的卡上又 OOM」会比账给人的印象多（推的，没量）。
  - `after_a_panic` 到下一波 `load_the_judge` 之间，仓里没有一处 `poll`（我 grep 了 `gpu-verification-server/src/gpu_judges_on_a_card.rs` 全文：没有 `poll(`；`GpuCrashJudge` 也没有 `Drop` 实现：`grep -n 'impl.*Drop for' crates/singlefs-verification/src/crash_judge_dispatch.rs` 零命中）；
  - 验证包自己写过同一件事：「丢掉的缓冲等设备下一次收拾才还给驱动」（`gpu-verification-server/src/card_memory_ledger.rs:14`，说的是核对器的趟）。
  - 所以「腾空的卡」在工作线程那一侧是句柄腾空，驱动那一侧腾空没了没有保证；单独判紧接着就装判器、建缓冲（`gpu-verification-server/src/card_worker.rs:233`）。能不能量出来要真卡：量法见本问末与「要主 agent 跑的命令」第 4 条。
- 句 3：「接着报空出来了、照常收下一波」（`gpu-verification-server/src/card_worker.rs:8`）。**一致**：`RanOutOfMemory` 的臂不 `return`，落到「report(CardWorkerReport::RoundFinished { card });」（`gpu-verification-server/src/card_worker.rs:216`），`while let Ok(command) = commands.recv()` 接着收。
- 句 4：「设备丢了、或设备没丢也没记过 Out of Memory（判法自己的 panic）：报这张卡坏了」（`gpu-verification-server/src/card_worker.rs:9`）。**一致**：`DeviceLost` 报 `CardFailed { device_lost: true }` 后 `return`（`gpu-verification-server/src/card_worker.rs:203`），`PanickedOtherwise` 走「CardAfterAPanic::RanOutOfMemory { .. } | CardAfterAPanic::PanickedOtherwise => {」（`gpu-verification-server/src/card_worker.rs:210`）报 `device_lost: false` 后 `return`。线程退出之后命令那一头收不到，服务另有「工作线程已经退出」那一支兜（`gpu-verification-server/src/service_loop.rs:875`）。
- 句 5：「弄坏开关 out-of-memory-taken-for-a-card-failure 下设备没丢的 Out of Memory 也报卡坏、线程退出。」（`gpu-verification-server/src/card_worker.rs:11`）。**一致**：第一臂带守卫（`gpu-verification-server/src/card_worker.rs:195`），守卫为假时 `RanOutOfMemory` 落进句 4 末尾那一臂。
- 句 6：「核的时候 panic 同判的时候 panic：报这张卡坏了。」（`gpu-verification-server/src/card_worker.rs:19`）。**冲突**。同一份文件头两句并排：
  - `gpu-verification-server/src/card_worker.rs:6`：「判的时候（或核的时候）panic，先问判器缓存这张卡的设备怎样」
  - `gpu-verification-server/src/card_worker.rs:19`：「核的时候 panic 同判的时候 panic：报这张卡坏了。」
  - 代码：核对组与判一波走同一个分支（`gpu-verification-server/src/card_worker.rs:179`、`gpu-verification-server/src/card_worker.rs:184`），核的时候 OOM 报 `OutOfMemoryCaught`、不报卡坏；`service_loop.rs` 文件头第 16 行也写核对组 OOM 放回排头或报拒（`gpu-verification-server/src/service_loop.rs:16`），测试 `a_facts_verifier_job_that_ran_out_of_memory_is_verified_again_on_an_emptied_card`（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:878`）钉的也是「卡不坏」。
  - 第 19 行是这一轮之前的话没跟着改（推的：我没翻 git 历史；它与第 6 行直接矛盾，哪一行先写的不影响判）。


`gpu-verification-server/src/service_loop.rs` 文件头与 `take_an_out_of_memory`、`place_the_lone_blocks`：

- 句 7：「工作线程报这张卡显存不够而设备没丢」（`gpu-verification-server/src/service_loop.rs:10`）。**一致**：`OutOfMemoryCaught` 事件进「) => self.take_an_out_of_memory(card, judges_dropped, reason),」（`gpu-verification-server/src/service_loop.rs:947`）。
- 句 8：「这张卡驻留的判器从账上清掉，卡照常空出来、照常排；别的卡不动」（`gpu-verification-server/src/service_loop.rs:11`）。**一致**：清账是「self.residents.remove(&card);」（`gpu-verification-server/src/service_loop.rs:1225`）与 `loading_by_card.remove`（`gpu-verification-server/src/service_loop.rs:1227`）；卡不进 `failed`，随后的「self.idle.insert(card);」（`gpu-verification-server/src/service_loop.rs:968`）照常；这一个函数只按 `card` 这一把键动五张表（驻留、装判器账、单独判的块、在判的块、核对组），别的卡的键没碰。附注：一块若有几截派在两张卡上，另一张卡上那一截交回时会被丢掉重判（「if self.withdrawn.contains(&piece.block) {」，`gpu-verification-server/src/service_loop.rs:1069`）；丢的是那一截的结论、不是那张卡的账，不算「动」。
- 句 9：「它这一波在判的块（判完交回了的除外）整块撤回」（`gpu-verification-server/src/service_loop.rs:12`）。**一致**：在判的块取自「self.blocks_in_flight_by_card.insert(」（`gpu-verification-server/src/service_loop.rs:855`）记下的那一波；判完交回的块在「self.admitted.remove(&judged.block);」（`gpu-verification-server/src/service_loop.rs:1076`）已从 `admitted` 删掉，这里 `admitted.get` 找不到就跳过（`gpu-verification-server/src/service_loop.rs:1236`）。
- 句 10：「照原来的流、序号与入池时刻重新入池，判了一半的结论不要，回话改挂到新的号上」（`gpu-verification-server/src/service_loop.rs:12` 的下一半，见下面三处）。**一致**：
  - 判了一半的不要：「self.blocks.remove(&block).map(|held| held.stream)」（`gpu-verification-server/src/pending_pool.rs:288`），块连同已记的结论整块删；
  - 入池时刻不变：`add_block` 收 `admitted.entered_at`（`gpu-verification-server/src/service_loop.rs:1201`）；
  - 回话：「self.replies.insert(again, reply);」（`gpu-verification-server/src/service_loop.rs:1205`）。
  - 次序：新号来自单调递增的「self.next_identifier += 1;」（`gpu-verification-server/src/pending_pool.rs:157`）；循环取的 `blocks_in_flight_by_card` 是 `BTreeSet`（`gpu-verification-server/src/service_loop.rs:399`），按旧号升序撤，所以新号与旧号同序；`waiting_for_an_emptied_card` 是按新号排的 `BTreeMap`，单独判的先后就是撤回的先后。新号比池里别的块都大，等于入池先后变晚，但这些块不交调度（下一句），不影响别的块的次序。
- 句 11：「撤回的块不交调度，等一张腾空的卡单独判」（`gpu-verification-server/src/service_loop.rs:13`）。**一致**：调度的输入先滤掉它们（「.filter(|block| !self.waiting_for_an_emptied_card.contains_key(&block.block))」，`gpu-verification-server/src/service_loop.rs:525`）。同一行括号里「撤回它的那张卡没坏就等那一张，坏了取头一张空着的没坏的卡」也**一致**：「.find(|card| card.card == origin || self.failed.contains(&origin))」（`gpu-verification-server/src/service_loop.rs:601`），在 `cards` 的顺序里找头一张。每一圈先 `place_the_lone_blocks`、后问调度，所以撤回它的那张卡一空出来，先给单独判的块、再给调度。
- 句 12：「那张卡腾空了也装不下这条流的整块报拒」（`gpu-verification-server/src/service_loop.rs:14`）。**一致**：`plan_a_lone_block_on_an_emptied_card` 只在「if !building.fit_together(&[stream], card.mebibytes_for_the_judges()) {」时交 `None`（`gpu-verification-server/src/batch_scheduling.rs:777`），主循环对 `None` 整块报拒（`gpu-verification-server/src/service_loop.rs:620`）。
- 句 13：`plan_a_lone_block_on_an_emptied_card` 的文件头「这张卡上驻留的判器全换出，只装这一块那条流的判器（草稿区照整张卡分），把这一块没派的态连着切成几批，一波判完。」（`gpu-verification-server/src/batch_scheduling.rs:18`）。**一致**：换出是该卡全部驻留（`gpu-verification-server/src/batch_scheduling.rs:785` 起），装的只有这一条流，草稿区取整张卡减新表两份（「building.tables_while_loading(&[stream])」，`gpu-verification-server/src/batch_scheduling.rs:780`），批数按「let batches = (block.undispatched.end - block.undispatched.start).div_ceil(states_per_dispatch);」（`gpu-verification-server/src/batch_scheduling.rs:795`）。`SchedulingInputs` 里 `residents` 给空（`gpu-verification-server/src/batch_scheduling.rs:770`），计划经账收时「留着的」为空，所以这一波之后账上驻留的只有这一条流的判器，与工作线程那一侧换出全部、装一个对得上。
- 句 14：「单独在腾空的卡上判的那一块又显存不够：整块报拒」（`gpu-verification-server/src/service_loop.rs:15`）。**一致**：单独判的块记在「self.lone_block_on_card.insert(card.card, block);」（`gpu-verification-server/src/service_loop.rs:617`），OOM 时取出「let lone = self.lone_block_on_card.remove(&card);」（`gpu-verification-server/src/service_loop.rs:1228`），命中「if lone == Some(block)」（`gpu-verification-server/src/service_loop.rs:1239`）就报 `OutOfMemoryOnAnEmptiedCard`；这条路不把卡插进 `failed`，卡随 `RoundFinished` 照常空出来。
- 句 15：核对组：「放回排头、下一回先把那张卡驻留的判器全换出再核」（`gpu-verification-server/src/service_loop.rs:16`）与「已经腾空了还显存不够的整组报拒」（`gpu-verification-server/src/service_loop.rs:16`）。后半**一致**：派的那一刻算「on_an_emptied_card = residents.is_empty();」（`gpu-verification-server/src/service_loop.rs:661`），OOM 时「on_card.on_an_emptied_card」为真就报拒（`gpu-verification-server/src/service_loop.rs:1272`）。「放回排头」**一致**：「self.facts_jobs_waiting.push_front(WaitingFactsJob {」（`gpu-verification-server/src/service_loop.rs:1285`），带「needs_an_emptied_card: true,」（`gpu-verification-server/src/service_loop.rs:1288`）。
- 句 16：同一行的「那张卡」**冲突**（措辞）。并排：
  - 文件头：「下一回先把那张卡驻留的判器全换出再核」（`gpu-verification-server/src/service_loop.rs:16`），README 同写「下一回先腾空那张卡再核」（`gpu-verification-server/README.md:94`）；
  - 代码：放回来的那一组只带「|| (must_empty_the_card && evicted_count != order.len())」（`gpu-verification-server/src/service_loop.rs:734`），即换出个数必须等于那张卡驻留个数，卡则是所有空着的卡里换出最少的那一张（同一文件头另一处：「空着的卡里挑换出最少的那一张」，`gpu-verification-server/src/service_loop.rs:28`；函数文档「只试换出全部驻留的判器」，`gpu-verification-server/src/service_loop.rs:687`）。OOM 的那张卡此刻驻留是 0（`gpu-verification-server/src/service_loop.rs:1225` 清了），别的空着的卡若也是 0 驻留且卡序在前，核对组落到那一张，不是撤回它的那张。
  - 判断：行为合理（空卡更稳），字面不成立；这份测试文件里没有带两张卡的核对组 OOM 用例（`StandInFaultTarget::Verifying` 只在一处设定，`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:887`），钉不住是不是同一张卡（那一例只有一张卡：`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:878` 起的用例里 `vec![card_leaving(0, capacity)]`）。
- 句 17：「撤回与报拒的块、放回与报拒的核对组、原因」（`gpu-verification-server/src/service_loop.rs:17`）。**一致**：「if !breaks.is_on(BreakSwitch::OutOfMemoryCaughtNotTold) {」（`gpu-verification-server/src/service_loop.rs:1292`）里交的 `OutOfMemoryCaught` 字段正是这几样（`gpu-verification-server/src/service_loop.rs:1293` 起）。
- 句 18：「一张卡判的时候坏了（设备丢了，或判法自己 panic）：这张卡不再排」（`gpu-verification-server/src/service_loop.rs:7`），即「设备丢了照旧进 failed」。**一致**：`CardFailed` 事件（`gpu-verification-server/src/service_loop.rs:970`）进「self.take_a_failed_card(card, reason, device_lost, pci_bus_identifier);」（`gpu-verification-server/src/service_loop.rs:976`），先「self.failed.insert(card);」（`gpu-verification-server/src/service_loop.rs:1096`），在判的块整块拒（`gpu-verification-server/src/service_loop.rs:1141` 起那一段）、在核的核对组拒（「let _ = reply.send(Err(ServiceRefusal::CardFailed {」，`gpu-verification-server/src/service_loop.rs:1116`）；测试钉着：`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:814`。撤回的块等的是已坏的那张卡时改派给头一张空着的卡（句 11 末），卡都坏了的整池拒（「for block in self.pool.undispatched_blocks() {」，`gpu-verification-server/src/service_loop.rs:1159`，先「self.waiting_for_an_emptied_card.remove(&block.block);」，`gpu-verification-server/src/service_loop.rs:1160`）。

第 1 问「什么现象会推翻它」：
- 推翻句 6 的冲突：把 `card_worker.rs` 第 19 行的话当成「核的时候 panic 最终也报卡坏」读成立，要求核对组 OOM 的测试（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:878`）里那张卡进 `summary.cards_failed`；该测试断言的是 `summary.cards_failed.is_empty()`，现象相反。
- 推翻句 16 的冲突：造两张空卡，OOM 发生在卡号大的那一张，核对组放回后若落在卡号大的那一张，说明「那张卡」成立。
- 推翻句 2 的「推不出」：真卡上 `after_a_panic` 返回之后立刻读 `nvidia-smi` 的空闲显存，看是否已回到装判器之前的水平；回到了就说明「显存还给驱动」在实际路径上成立（只是代码不保证）。量法见「要主 agent 跑的命令」第 4 条的说明。

## 第 2 问：`after_a_panic` 的「OOM 次数比上一回多」

整问判定：读代码，三种情形各有确定的判，**都与文件头一致**：**成立**。真卡上这一份函数没有任何测试调过，所以「真卡上也这么答」**推不出**（算打中），探针已写好待跑。

文件头：「丢了交丢失原因；没丢而记的 Out of Memory 次数比上一回多」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:7`）。函数体顺序（`gpu-verification-server/src/gpu_judges_on_a_card.rs:301` 起）：

1. 读计数「let reports = condition.out_of_memory_reports();」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:303`）；
2. 与上次比「let ran_out_of_memory = reports > self.out_of_memory_reports_seen;」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:304`），**不管结局如何**先把上次数更新「self.out_of_memory_reports_seen = reports;」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:305`）；
3. 丢设备在先：「if let Some(reason) = condition.lost_reason() {」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:306`）答 `DeviceLost`；
4. 没比上次多：「return CardAfterAPanic::PanickedOtherwise;」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:310`）；
5. 否则清判器答 `RanOutOfMemory`（`gpu-verification-server/src/gpu_judges_on_a_card.rs:313`、`gpu-verification-server/src/gpu_judges_on_a_card.rs:314`）。

「上一回」的初值是造这一份缓冲时设备上已有的次数（「out_of_memory_reports_seen: card.device_condition().out_of_memory_reports(),」，`gpu-verification-server/src/gpu_judges_on_a_card.rs:211`），所以造这一份之前这个进程在这台设备上已经记过的（例如开卡那一步）不算。

三种情形：

- **同一张卡连着 OOM 两回**：**成立（与文件头一致）**。每一回答之前第 2 步已把 `seen` 抬到当时的计数，第二回的计数又比它大，两回都答 `RanOutOfMemory`；第二回的 `judges_dropped` 是两回之间重新装上的个数。代码依据：`gpu-verification-server/src/gpu_judges_on_a_card.rs:304`、`gpu-verification-server/src/gpu_judges_on_a_card.rs:305`、`gpu-verification-server/src/gpu_judges_on_a_card.rs:313`。
  - 一次 panic 之前设备上记了不止一次（计数 +2）也只答一回，答完 `seen` 追平；探针的第一条用例里有这一格。
- **OOM 之后判法自己又 panic**（两回之间设备上没有新记 OOM）：**成立（与文件头一致）**。第 2 步比不出多，落到 `PanickedOtherwise`（`gpu-verification-server/src/gpu_judges_on_a_card.rs:310`），工作线程报 `CardFailed { device_lost: false }` 后退出（`gpu-verification-server/src/card_worker.rs:210`），与文件头「设备没丢也没记过 Out of Memory（判法自己的 panic）：报这张卡坏了」（`gpu-verification-server/src/card_worker.rs:9`）同。前提是两回之间没有迟到的 OOM 记录：计数是整个设备上的原始次数（「.push(OutOfMemoryReport {」，`crates/singlefs-verification/src/gpu_memory_ledger.rs:2189`），不分是哪一段活的，别的线程（编内核另起的线程）晚到的一条会让下一次无关的 panic 被当成显存不够。这一点**推不出**（要真卡、要看 wgpu 回调到的时序）。
- **OOM 与设备丢失同时到**：**成立（与文件头一致）**。丢失在先判（`gpu-verification-server/src/gpu_judges_on_a_card.rs:306`、`gpu-verification-server/src/gpu_judges_on_a_card.rs:307`），答 `DeviceLost { reason }`，不清判器（那条线程要退）；之后任何一次 panic 都是 `DeviceLost`，因为丢失「记了就不抹」（`crates/singlefs-verification/src/gpu_memory_ledger.rs:2104`）。两点附注：
  - 因显存不够丢的设备（`CauseOfTheDeviceLoss::GpuMemoryShort`，`crates/singlefs-verification/src/gpu_unit_checks.rs:362` 起认）在服务里**报卡坏**而不是接住：`lost_reason()` 只交 `why`（`crates/singlefs-verification/src/gpu_memory_ledger.rs:2289`），缘由没人读。这合理，因为服务持着的卡是固定的一组 `GpuCard`（「cards: Vec<GpuCard>,」，`gpu-verification-server/src/gpu_judges_on_a_card.rs:108`），没有重起设备的路；但文件头没把这一类单列，README 也没说，读的人会以为「显存不够」一律接住；
  - 丢失回调晚于 panic 到达时（读代码定不了时序）：第 3 步读不到丢失，若 OOM 已记就答 `RanOutOfMemory`，这张卡被当好卡接住，下一波再 panic 才答 `DeviceLost`；代价是一块被撤回、单独判一次、第二次 panic 时再按卡坏报拒。**推不出**（打中，要真卡）。

没有测试盖到这一份：

- 我 grep 了 `gpu-verification-server/tests` 与 `gpu-verification-server/src`（排除定义它的两个文件）里的 `after_a_panic`，只有两份替身实现：`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:201` 与 `gpu-verification-server/tests/library_pending_pool_on_a_temporary_library.rs:806`；真卡对拍文件 `gpu-verification-server/tests/gpu_service_path_agrees_with_the_direct_judge.rs` 没有。
- 替身的答法是「match self.fault_of_the_panic.take() {」（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:202`）：一次一个标记，没有计数、没有「丢失优先」，所以上面三种情形的差别没有任何测试能钉住。弄坏开关里与这一份有关的 `out-of-memory-taken-for-a-card-failure` 动的是工作线程那一侧（`gpu-verification-server/src/card_worker.rs:195`），不碰计数。
- 探针（真卡，待主 agent 跑）：`probe_after_a_panic_counts_out_of_memory_reports_since_the_last_answer`（无记录 → 记一回 → 再记一回 → 不记 → 同一回记两次）与 `probe_after_a_panic_answers_a_lost_device_before_out_of_memory`（OOM 与丢失同记）。它们用 `DeviceConditionOfACard` 公开的 `record_an_out_of_memory` 与 `record_the_device_lost` 造设备情况，不真撑爆显存，所以只能证「读到这些记录时函数怎么答」，证不了「wgpu 真的按什么时序记」。

第 2 问「什么现象会推翻它」：探针第一条用例里任一个 `assert_eq!` 不成立，就推翻「连着两回 / 先 OOM 后判法 panic」的判；第二条里第一个断言不成立（答成了 `RanOutOfMemory`）就推翻「丢失优先」。

## 第 3 问：「新表两份」

整问判定：三处调用、登记流时的判、文档串、驻留账、对拍测试逐处**成立**；另有一处文件头说过头的话**不成立**（登记之后卡坏，见 3.6），和一处测试盖不到的缺口（3.5，推的）。

### 3.1 文档串本身（`mebibytes_while_loading_a_judge`）

- 代码：「mebibytes_of_a_judge(facts, requested_states_per_dispatch) + facts.table_mebibytes」（`gpu-verification-server/src/card_memory_ledger.rs:86`），即判器占的再加一份表。
- 文档串说「新装一个判器的那一刻卡上要的（MiB）：判器占的」（`gpu-verification-server/src/card_memory_ledger.rs:78`）再加一份表，与代码相同。
- 文档串的前提「wgpu 给不带 `MAP_WRITE` 的缓冲另开一份同样大的暂存缓冲，下一次提交做完才放；一波里的判器全装完才判头一批」（`gpu-verification-server/src/card_memory_ledger.rs:79`）三半句各核一次：
  - `create_buffer_init` 建缓冲用 `mapped_at_creation: true`（`wgpu-30.0.1/src/util/device.rs:65`），判表缓冲的用途只有 `STORAGE`、没有 `MAP_WRITE`（`crates/singlefs-verification/src/crash_judge_dispatch.rs:1406` 起那一段）；wgpu-core 对这一类另开暂存：`MAP_WRITE` 那一支在 `wgpu-core-30.0.1/src/device/resource.rs:1144`，否则走 `StagingBuffer::new(...)`（`wgpu-core-30.0.1/src/device/resource.rs:1164`）。前半句**成立**（读源码）。
  - 「下一次提交做完才放」：`unmap` 时暂存缓冲进 `temp_resources`、目标缓冲进 `dst_buffers`（`wgpu-core-30.0.1/src/resource.rs:1032`、`wgpu-core-30.0.1/src/resource.rs:1033`；`wgpu-core-30.0.1/src/device/queue.rs:419`、`wgpu-core-30.0.1/src/device/queue.rs:445-446`），刷出要等下一次提交（`wgpu-core-30.0.1/src/device/queue.rs:1306`）：「下一次提交」那半句**成立**（读源码）；引用在提交「做完」才放那半句我没读到，**推不出**（不影响账：账按「同时都在」算，偏保守）。
  - 「一波里的判器全装完才判头一批」：工作线程先全部换出、再全部装上、最后才判各批（「for stream in &wave.plan.evictions {」，`gpu-verification-server/src/card_worker.rs:230`；「for (load, materials) in wave.plan.loads.iter().zip(&wave.materials_of_the_loads) {」，`gpu-verification-server/src/card_worker.rs:233`；「for batch in &wave.plan.batches {」，`gpu-verification-server/src/card_worker.rs:241`）。**成立**。

### 3.2 `tables_while_loading` 的调用处

函数本身：「fn tables_while_loading(&self, streams: &[StreamKey]) -> u64 {」（`gpu-verification-server/src/batch_scheduling.rs:361`），每张表乘 2（弄坏开关打开时乘 1）。文件头：「新装那一刻：新装的判器的表在装的那一刻占两份」（`gpu-verification-server/src/batch_scheduling.rs:14`），「都照每张新表两份算，留着的判器照驻留的账算；这一波之后驻留的照旧只记一份表」（`gpu-verification-server/src/batch_scheduling.rs:15`）。

| 调用 | 位置 | 判定 | 依据 |
|---|---|---|---|
| 挑新流（装不装得下） | `fit_together` 里 | 一致 | 「let tables = self.tables_while_loading(streams);」（`gpu-verification-server/src/batch_scheduling.rs:378`），挑新流两处用它：「if self.fit_together(&widened, free_evicting_every_idle_judge) {」（`gpu-verification-server/src/batch_scheduling.rs:667`）、「while !self.fit_together(&newcomers, free) {」（`gpu-verification-server/src/batch_scheduling.rs:692`）；留着的判器按驻留的账（一份）从 `free` 里扣，见 `kept_mebibytes` 与 `idle_mebibytes` |
| 新装分草稿区 | `load_new_streams_onto_the_card` | 一致 | 「let tables = self.tables_while_loading(&newcomers);」（`gpu-verification-server/src/batch_scheduling.rs:705`）、「let sizes = self.split_the_scratch(free - tables, &newcomers);」（`gpu-verification-server/src/batch_scheduling.rs:706`）；`free - tables` 不会下溢：上面的循环退出条件就是 `fit_together` 为真，即 `free >= tables`（`gpu-verification-server/src/batch_scheduling.rs:379`） |
| 不饿死那一遍 | `rescue_the_starving_streams` | 一致 | 「let tables = self.tables_while_loading(&reloaded);」（`gpu-verification-server/src/batch_scheduling.rs:541`）；整卡重装时原来的判器全换出，重装的全是新表，每张都该两份；挑「装不装得下」另用 `fit_together`（`gpu-verification-server/src/batch_scheduling.rs:523`） |

另有两处用到它、不在「三处」的清单里，也一致：单独判那一块的 `fit_together`（`gpu-verification-server/src/batch_scheduling.rs:777`）与分草稿区（`gpu-verification-server/src/batch_scheduling.rs:780`，整张卡减新表两份）。文件头（第 14、15 行）只点了挑新流与分草稿区，没写不饿死与单独判，漏写不矛盾。

### 3.3 登记流时的判

- 代码：「mebibytes_while_loading_a_judge(&facts, 1)」（`gpu-verification-server/src/service_loop.rs:1005`）与「if judge_mebibytes_at_one_state > largest_card_mebibytes_for_the_judges {」（`gpu-verification-server/src/service_loop.rs:1014`）。文件头：「登记流时判它新装那一刻装不装得进任何一张卡」（`gpu-verification-server/src/service_loop.rs:22`）。**一致**。
- 调度侧不调这个函数，另用「表两份 + 一个态的草稿区」的一套写法；两套写法等价（推的，代数）：一个态的草稿区取整是 `mebibytes_of_one_state_rounded_up()`，登记那边 `mebibytes_of_a_judge(facts, 1) = 表 + 一个态取整`，再加一份表就是 `2×表 + 一个态取整`；调度侧 `fit_together` 的条件「scratch / count >= mebibytes_of_one_state_rounded_up()」（`gpu-verification-server/src/batch_scheduling.rs:383`）对单条流、整张卡的额度就是 `额度 − 2×表 ≥ 一个态取整`，与登记的 `2×表 + 一个态取整 ≤ 额度` 同一个不等式。所以登记过了的流，在空卡上调度侧（单独判那条路，`gpu-verification-server/src/batch_scheduling.rs:777`）也装得下。

### 3.4 驻留的账仍只记一份

- 收判器进账：「let judge_mebibytes = mebibytes_of_a_judge(facts, states_per_dispatch);」（`gpu-verification-server/src/card_memory_ledger.rs:229`），一份表。
- 主循环那一侧的账同样：驻留的合计用 `mebibytes_of_a_judge`（`gpu-verification-server/src/service_loop.rs:833`），装上一个判器时加的也是「let judge_mebibytes = mebibytes_of_a_judge(&facts, states_per_dispatch);」（`gpu-verification-server/src/service_loop.rs:1316`），「装上」那一行的 `free_mebibytes` 照它减（`gpu-verification-server/src/service_loop.rs:1327`），README 日志表那一行也写「账不记它」（`gpu-verification-server/README.md:87`）。
- 在 `crates`、`gpu-verification-server/src`、`gpu-verification-server/tests` 三处目录里，`while_loading` 的命中文件只有四个（README 与别处没搜）：`gpu-verification-server/src/service_loop.rs`、`gpu-verification-server/src/batch_scheduling.rs`、`gpu-verification-server/src/card_memory_ledger.rs`、`gpu-verification-server/tests/batch_scheduling_requirements.rs`（我跑了 `grep -rln 'while_loading' crates gpu-verification-server/src gpu-verification-server/tests`，四行输出）；账本里没有第二处记两份的地方。**成立**。
- 测试钉着：每个场景断言「这一波之后驻留的照旧只记一份表」（`gpu-verification-server/tests/batch_scheduling_requirements.rs:883`），式子是「on_the_card.resident_after_the_wave().occupied_mebibytes() + tables_loaded,」（`gpu-verification-server/tests/batch_scheduling_requirements.rs:881`）。

### 3.5 `card_memory_matches_the_verification_package` 比的是不是同一个量

**是同一个量，所以它没盖到新增的量。**

- 测试比的是「mebibytes_of_a_judge(&facts, requested),」（`gpu-verification-server/tests/card_memory_matches_the_verification_package.rs:30`）对「mebibytes_a_judge_needs(&tables, usize::try_from(requested).expect("装得进 usize")),」（`gpu-verification-server/tests/card_memory_matches_the_verification_package.rs:31`），即驻留的那个量（一份表）。这份文件不在这一轮的补丁里（`crates.patch` 十条 `diff --git` 没有它），导入的也只有 `mebibytes_of_a_judge`（`gpu-verification-server/tests/card_memory_matches_the_verification_package.rs:7`）。
- 新增的 `mebibytes_while_loading_a_judge` 在验证包那一侧没有对应物：`grep -rn 'while_loading' crates/singlefs-verification/src` 零命中。它唯一的独立来源是文档串里那个 wgpu 前提（3.1）与 `gpu-verification-server/tests/batch_scheduling_requirements.rs:809` 那个测试辅助函数（辅助函数用它当尺子去量调度侧的 `tables_while_loading`，两边是同一个公式的两种写法，不是独立量出来的数）。
- 缺口（推的）：调度测试只有三个场景（独占空卡、两条流一起新装、留一个再新装，`gpu-verification-server/tests/batch_scheduling_requirements.rs:832`、`gpu-verification-server/tests/batch_scheduling_requirements.rs:837`、`gpu-verification-server/tests/batch_scheduling_requirements.rs:845`），没有不饿死那一遍（`gpu-verification-server/src/batch_scheduling.rs:541`）与单独判那一条（`gpu-verification-server/src/batch_scheduling.rs:780`）；登记那一处的边界也没有：登记测试用的表是「table_mebibytes: 100_000,」（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:421`），远大于任何卡，不卡在「一份装得下、两份装不下」那条边上，而弄坏开关 `table-staging-not-reserved` 点名的只有那个调度测试（`gpu-verification-server/src/break_switches.rs:652`）；把登记那一处改回一份表，没有测试会红（推的，没试）。

### 3.6 文件头说过头的一句（不成立）

- 文件头：「装不进的当场拒：池里不留永远派不出去的态。」（`gpu-verification-server/src/service_loop.rs:23`）。
- 代码：判「装不装得进」只在登记那一刻、只对「没坏的卡」取最大额度（「let largest_card_mebibytes_for_the_judges = self」，`gpu-verification-server/src/service_loop.rs:1007`）；之后入池不再判（`gpu-verification-server/src/service_loop.rs:1026` 起的 `admit_the_block` 只看在不在停、卡是不是全坏）。卡坏时拒的是在判的块、核对组与「卡都坏了」的整池（`gpu-verification-server/src/service_loop.rs:1141` 起、`gpu-verification-server/src/service_loop.rs:1159`），没有「剩下的没坏的卡装不下这条流」那一支（核对组有：「(!fits).then_some(ServiceRefusal::FactsVerifierFitsNoCard {」，`gpu-verification-server/src/service_loop.rs:784`，在卡坏时重判）。
- 推出的现象：两张卡一大一小，某条流只有大卡装得下，登记过了；大卡坏了之后再入池的块，调度判 `HoldNothingPlaceableOnTheIdleCards`，一直挂在池里；收到停止信号之后主循环要等「池里回话都交完」才退（`gpu-verification-server/src/service_loop.rs:467` 起的退出条件含 `self.replies.is_empty()`），所以也停不下来，只能等单元的停止限时到了被 SIGKILL。这一条不是这一轮引入的行为，但文件头这句话是这一轮写进去的（它带着 `mebibytes_while_loading_a_judge` 的名字）。
- 探针（不上卡，待主 agent 跑）：`probe_a_block_only_the_failed_big_card_could_hold_is_refused_not_left_hanging`，在仓副本里的 `gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs` 末尾。它的做法：先放一条小流、让小卡也坏，借「卡都坏了」把挂着的块放出来，再看那块 3 秒里有没有回话；有则 `hung` 为假。推翻这一条的现象：探针通过。

第 3 问「什么现象会推翻它」：
- 推翻「三处一致」：任何一个调用处用的不是两份（例：把 `tables_while_loading` 的系数改成 1 之后，`a_newly_loaded_judge_leaves_room_for_a_second_copy_of_its_table_while_it_loads` 不红，就说明那个场景没走到那一处）。
- 推翻「驻留账一份」：`grep -rn 'while_loading' gpu-verification-server/src/card_memory_ledger.rs` 出现在 `admit` 里。
- 推翻「3.3 等价」：找出 `2×表 + 一个态取整 ≤ 额度` 为真而 `fit_together(&[流], 整张卡额度)` 为假（或相反）的一组 (表, 额度)。

## 第 4 问：卡坏上报一条链

整问判定：四环字面逐环**成立**（我对着源码逐字对了前缀、字段名与次序、分隔符、台标签、子命令名，全部相同；读代码，没跑）。另查出：一处注释点名了不存在的常量（环 2，**不成立**，不影响行为）、一处 README 与代码不符（README 第 94 行，同第 1 问句 16）、一处 README 没写条件（环 4）。

### 环 1：服务打的三种行 → control 认的

- 前缀：服务「pub const CARD_FAILED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARD_FAILED";」（`gpu-verification-server/src/serving_from_the_library.rs:97`）、「pub const OUT_OF_MEMORY_CAUGHT_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT";」（`gpu-verification-server/src/serving_from_the_library.rs:99`）、「pub const JUDGE_LOADED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_JUDGE_LOADED";」（`gpu-verification-server/src/serving_from_the_library.rs:101`）。control 的：「CARD_FAILED_LINE_PREFIX = "GPU_VERIFICATION_SERVER_CARD_FAILED"」（`gpu-verification-server/control:307`）、「OUT_OF_MEMORY_CAUGHT_LINE_PREFIX = "GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT"」（`gpu-verification-server/control:308`）。逐字相同。control 不读 `JUDGE_LOADED`。**一致**。
- 字段：服务打的卡坏行是「"{CARD_FAILED_LINE_PREFIX} card={} nvidia_smi_index={} name=\"{}\" pci={} device_lost={} reason=\"{}\"",」（`gpu-verification-server/src/serving_from_the_library.rs:262`）；control 文件头写的模板是「GPU_VERIFICATION_SERVER_CARD_FAILED card=<服务里的号> nvidia_smi_index=<序号> name="<卡名>" pci=<PCI 总线号> device_lost=<yes|no> reason="<原因>"」（`gpu-verification-server/control:94`）。字段名、次序、引号位置相同。接住那一行的字段（`blocks_put_back`、`facts_jobs_put_back`、`put_back=`、`refused=`、`reason`）与 README 日志表第 88 行次序相同（`gpu-verification-server/src/serving_from_the_library.rs:246`）。
- 从打出到 journald：每一行经「self.pool.print_a_service_line(line);」（`gpu-verification-server/src/serving_from_the_library.rs:1123`）进 `eprintln!("{line}")`（`gpu-verification-server/src/library_pending_pool.rs:880`），服务日志过滤只丢三种逐次派活的行、别的原样转（`gpu-verification-server/README.md:62`），所以 journald 的 `MESSAGE` 就是这一行本身，开头无别的前缀；control 用 `startswith` 认「前缀加一个空格」（「prefixes = [f"{CARD_FAILED_LINE_PREFIX} "]」，`gpu-verification-server/control:1048`），服务打的是前缀、空格、`card=`，对得上；卡坏前缀不是接住前缀的前缀（`..._CARD_FAILED ` 与 `..._OUT_OF_MEMORY_CAUGHT ` 互不以对方起头），OOM 那一行进不了卡坏的判。**成立**。
- 自证：control 的自证格从源码里正则读前缀（「failed = re.search(r'pub const CARD_FAILED_LINE_PREFIX: &str = "([^"]+)";', source)」，`gpu-verification-server/control:2235`）再比。接住前缀那一行现在 98 个字符（`awk 'NR==99{print length($0)}' gpu-verification-server/src/serving_from_the_library.rs` 打 98），仓根与服务目录都没有 rustfmt 配置（`ls rustfmt.toml .rustfmt.toml gpu-verification-server/rustfmt.toml` 三个都不存在），默认行宽 100，改长三个字符它就会折行、正则读不到：读不到时自证返回「找不到……的定义」那条问题，是红、不是静默过（`gpu-verification-server/control:2238` 起那一段，推的：没改源码试）。

### 环 2：control 的 `card_failed_lines` 与 `cards-failed`

- 读哪个进程：单元此刻的主进程号（`unit_main_process`），命令是「command = [paths.journalctl, "--user", "-b", f"_PID={main}", "-o", "json", "--output-fields=MESSAGE,_PID", "--no-pager"]」（`gpu-verification-server/control:1041`），逐行只留进程号等于主进程号的（「if process != main and not control_broken("card-failure-of-earlier-processes-counted"):」，`gpu-verification-server/control:1060`）；交出「lines.append((process, message))」（`gpu-verification-server/control:1066`），即 (进程号, 行原样)。README 起停第 41 条写的 `journalctl --user -b _PID=<主进程号>` 与它同（`gpu-verification-server/README.md:41`）。**一致**。
- `check` 用它：「refused = card_failure_refusal(paths) or refused」（`gpu-verification-server/control:1143`），读不了退 1（`gpu-verification-server/control:1079`），读到退 `CARD_FAILED_EXIT = 1`（`gpu-verification-server/control:310`）。**一致**。
- `cards-failed` 的输出：每台一行「say(f"{CARDS_FAILED_READ_PREFIX} host={label} state={state} lines={len(lines)}")」（`gpu-verification-server/control:1450`），每一行卡坏一行「say(f"{FAILED_CARD_OF_A_HOST_PREFIX} host={label} process={process} {line}")」（`gpu-verification-server/control:1452`）。文件头与 README 的写法相同：「GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host=<台> state=<read|not-running|unreadable|unreachable> lines=<行数>，每一行卡坏行一行」（`gpu-verification-server/control:46`）。状态取值来自「CARDS_FAILED_HOST_STATES = ("read", "not-running", "unreadable")」（`gpu-verification-server/control:314`）加 `unreachable`（`gpu-verification-server/control:315`）。**一致**。
- 第二台：经 `forward` 起同一条、收它的标准输出（`gpu-verification-server/control:1433`），按空格切 4 段读（「words = line.split(" ", 3)」，`gpu-verification-server/control:1436`）、只认那一台自称的 `host=this-host`（`gpu-verification-server/control:339`），退出码不是 0、状态行不是恰好一条、状态不在三个里就记 `unreachable`（「if code != 0 or len(states) != 1 or states[0] not in CARDS_FAILED_HOST_STATES:」，`gpu-verification-server/control:1441`）。子命令在主函数里分到「if arguments[:1] == ["cards-failed"]:」（`gpu-verification-server/control:2651`）。**一致**。
- 不成立的一处：control 的注释「task-server/task_server_lib.py 的 GPU_CARDS_FAILED_READ_PREFIX、GPU_FAILED_CARD_OF_A_HOST_PREFIX 逐字相同」（`gpu-verification-server/control:311`）点名了两个常量，`task_server_lib.py` 里只有后一个（「GPU_FAILED_CARD_OF_A_HOST_PREFIX = "GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST"」，`task-server/task_server_lib.py:2200`）。我跑了 `grep -rn 'GPU_CARDS_FAILED_READ_PREFIX' --include=*.py --include=*.sh --include=*.md --include=control .`（排除 `research/`），只命中 `gpu-verification-server/control` 第 311 行这一句注释自己。任务服务那一侧不读状态行（见环 3），所以没有对应常量是对的，是注释写多了。行为无影响。

### 环 3：`failed_cards_of_the_report`

- 读法：「words = line.split(" ", 3)」（`task-server/task_server_lib.py:2209`），不是四段、开头不是「GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST」就跳过（「if len(words) != 4 or words[0] != GPU_FAILED_CARD_OF_A_HOST_PREFIX:」，`task-server/task_server_lib.py:2210`）；前缀常量与 control 的「FAILED_CARD_OF_A_HOST_PREFIX = "GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST"」（`gpu-verification-server/control:313`）逐字相同；台标签「GPU_THIS_HOST_LABEL = "this-host"」（`task-server/task_server_lib.py:2201`）与 control 的 `THIS_HOST_LABEL` 相同。
- 台、进程、卡号：台与进程取「host=」「process=」之后（「host, process = words[1].partition("=")[2], words[2].partition("=")[2]」，`task-server/task_server_lib.py:2212`），卡号取第四段里头一个「card=」开头的词（「card = next((word.partition("=")[2] for word in words[3].split() if word.startswith("card=")), "?")」，`task-server/task_server_lib.py:2213`）；第四段就是服务那一行原样，头一个词是行首前缀、第二个词就是 `card=<号>`，对得上。键是「found.setdefault(f"{host}:{process}:{card}"」（`task-server/task_server_lib.py:2215`）。**一致**。
- 状态行（`..._CARDS_FAILED_READ`）这一环**不读**：「return sorted(found.items())」（`task-server/task_server_lib.py:2216`）前只有卡坏那一种行。后果（推的）：某一台读不了或没在跑（`unreadable`、`not-running`、`unreachable`）时命令仍退 0、没有卡坏行，`read_gpu_cards_failed` 就拿空表换掉上一回读到的（「self.gpu_cards_failed = found」，`task-server/task_server.py:5575`），已经报过的坏卡在下一轮被当「好了」忘掉，journald 一恢复可读又会对名下会话重发一条（告警记录那一条不重发，见环 4）。这是读代码推的，没造假输入跑过；要验证得改自证里 `health-gpu-card-failed-notified` 那一格的假 control（`task-server/task_server_selftest.py` 不在我的写范围），所以没写探针。

### 环 4：`gpu_card_failures` 发给谁、写什么

- 起的命令：「command = [fake] if fake else [sys.executable, "-B", lib.GPU_SERVICE_CONTROL, "cards-failed"]」（`task-server/task_server.py:5563`），`GPU_SERVICE_CONTROL` 指仓里 `gpu-verification-server/control`（「GPU_SERVICE_CONTROL = os.path.join(REPOSITORY_ROOT, "gpu-verification-server", "control")」，`task-server/task_server_lib.py:1752`），子命令名与 control 主函数认的相同。
- 频率与限时：另起一条线程读、至多每 `GPU_CARD_FAILURE_CHECK_SECONDS = 60`（`task-server/task_server.py:1323`，起线程在 `task-server/task_server.py:5554`），一回至多等「GPU_CARDS_FAILED_READ_SECONDS = 150」（`task-server/task_server.py:1324`）。退出码不是 0 或没跑成的留着上一回、记一行日志（`task-server/task_server.py:5568` 起那一段）。
- 发给谁：「recipients = self.health_recipients()」（`task-server/task_server.py:5585`）里的会话，也就是名下有在排、在跑任务的会话；消息是「task-server：服务健康 GPU 服务卡坏」「问题：」加那一句、「出路：」加 `GPU_CARD_FAILED_NEXT_STEP`（「f"出路：{next_step}"」，`task-server/task_server.py:5607`），那一句是「<这一台|第二台 X>的 gpu-verification-server（进程 N）报卡坏：<卡坏行原样>」，里面有 PCI 总线号。
- 写什么告警：健康问题的「种类」带键（`gpu-card-failed:<台>:<进程号>:<卡号>`，「return [(f"{lib.GPU_CARD_FAILED_ALERT_KIND}:{key}", "GPU 服务卡坏", line) for key, line in found]」，`task-server/task_server.py:5555`），用来做「同一样对同一个会话只发一次」的键；**告警记录里写的种类不带键**：「self.alert(lib.GPU_CARD_FAILED_ALERT_KIND, 0, line, next_step)」（`task-server/task_server.py:5600`），种类就是「GPU_CARD_FAILED_ALERT_KIND = "gpu-card-failed"」（`task-server/task_server_lib.py:2198`），任务号写 0，同一个健康种类只写一回（「if gpu_card_failed and kind not in self.gpu_cards_failed_alerted:」，`task-server/task_server.py:5598`），记在内存里（服务重启后同一张还坏着的再写一条，`task-server/task_server.py:5599`）。背景材料第 12 问把「gpu-card-failed:<台>:<进程>:<卡号>」当告警种类，那只是健康种类；告警记录里区分两台、两张卡只能靠 `line` 的正文。README 写的「往它的告警记录写一条 `gpu-card-failed`」（`gpu-verification-server/README.md:42`）与代码一致。
- 条件（README 没写）：没有名下有在排、在跑任务的会话时，`watch_health` 在「if not recipients:」（`task-server/task_server.py:5586`）就返回、先「self.health_notified.clear()」（`task-server/task_server.py:5587`），不读卡坏也不写告警记录。README 第 42 行写「任务服务每分钟至多读一回，读到就往名下有在排、在跑任务的会话发……往它的告警记录写一条」，没说没有会话时不读不写（`task-server/task_server.py` 文件头 ⑥ 写了「没有要发给的会话时不查」）。后果是坏卡发生在没有任务的时段，要等下一个会话有任务之后的一轮才读到（推的）；这不是永远收不到。
- 观察（不判缺陷）：`read_gpu_cards_failed` 复位「self.gpu_cards_failed_reading = False」（`task-server/task_server.py:5577`；3001 行是初值）写在 `try` 之后、没有 `finally`；`try` 只接 `OSError` 与 `TimeoutExpired`（`task-server/task_server.py:5567` 起），别的异常会让这条线程退出而 `reading` 一直为真、之后永远不再读。现在那一段里能抛别的异常的只有解析函数与 `say`，都没有已知的抛法（推的）。

### README「日志」「起停」两节与代码

| 句 | 判定 | 依据 |
|---|---|---|
| 日志表第 87 行 `JUDGE_LOADED` 的字段与「账不记它」 | 一致 | 字段与次序 `gpu-verification-server/src/serving_from_the_library.rs:220`、`gpu-verification-server/src/serving_from_the_library.rs:221`；账的算法见第 3 问 3.4 |
| 第 88 行 `OUT_OF_MEMORY_CAUGHT` 的触发与字段 | 一致 | `gpu-verification-server/src/serving_from_the_library.rs:246` |
| 第 89 行 `CARD_FAILED`「设备丢了，或判法自己 panic」 | 一致 | 「一张卡坏了、不再排（设备丢了，或判法自己 panic）」（`gpu-verification-server/README.md:89`）对 `gpu-verification-server/src/card_worker.rs:210` |
| 第 90 行 `STOPPED` 带 `cards_failed`、接住几回、不再取的块数 | 一致 | 打的是 `{summary:?}`（`gpu-verification-server/src/main.rs:376`），字段在 `gpu-verification-server/src/serving_from_the_library.rs:303`、`gpu-verification-server/src/serving_from_the_library.rs:305`、`gpu-verification-server/src/serving_from_the_library.rs:307` |
| 第 94 行「下一回先腾空那张卡再核」 | 冲突 | 见第 1 问句 16：代码换出的个数等于驻留个数，卡是换出最少的那一张，不一定是撤回它的那张 |
| 第 95 行「新装判器的那一刻表占两份……登记流时判装不装得下也照两份」 | 一致 | 第 3 问 3.2、3.3 |
| 起停第 41 条 `check` 读 `journalctl --user -b _PID=<主进程号>`、`CARD_FAILED` 退 1、只有接住行不算、出路 | 一致 | `gpu-verification-server/control:1041`、`gpu-verification-server/control:1048`、`gpu-verification-server/control:1143`、`gpu-verification-server/control:318` 起的 `RESET_THE_CARD_HOWTO` |
| 起停第 42 条 `cards-failed` 的输出格式、退 0、没开 GPU 只打 `why=gpu-off` | 一致 | `gpu-verification-server/control:1450`、`gpu-verification-server/control:1452`、`gpu-verification-server/control:1468` |
| 起停第 42 条「任务服务每分钟至多读一回，读到就往……发……写一条 `gpu-card-failed`」 | 一致，缺一个条件 | 缺「没有名下有任务的会话时不读不写」，见环 4 |
| 文件表第 23 行「设备丢了才报卡坏」（不在这两节里，顺带查到） | 冲突 | 「显存不够而设备没丢的丢掉全部判器、报接住、照常收活，设备丢了才报卡坏」（`gpu-verification-server/README.md:23`）对 `gpu-verification-server/src/card_worker.rs:210`：设备没丢也没记 OOM 的判法自己 panic 也报卡坏（`device_lost=no`），README 第 89 行与代码一致、第 23 行漏了这一类 |

第 4 问「什么现象会推翻它」：
- 推翻「四环字面相同」：把 `control` 的 `FAILED_CARD_OF_A_HOST_PREFIX` 或 `task_server_lib.py` 的同名常量改一个字，两边各自的自证都仍然绿（环 3 的自证格拿 `lib.GPU_FAILED_CARD_OF_A_HOST_PREFIX` 造假输出：`task-server/task_server_selftest.py:2451`，它与 control 各用各的常量，没有一格拿 control 真打的输出喂 `failed_cards_of_the_report`），说明这一环只靠注释与人对，不靠测试；我没改文件试，是读出来的；三个前缀与台标签的字面相同用只读 grep 核过，原样输出在「只读核对」一节。
- 推翻环 3 的后果：喂一段「state=unreadable lines=0」的输出给 `failed_cards_of_the_report`，它交空表（读代码即得）；要看服务那一侧是否真把上一回的坏卡忘掉，要跑任务服务自证或造假 control（`TASK_SERVER_SELFTEST_GPU_CARDS_FAILED`，`task-server/task_server_lib.py:398`），我没跑。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 1 问整问 | 不成立（两句）、推不出（一句） | `card_worker.rs` 文件头第 19 行与第 6 行、代码相反；核对组「那张卡」措辞过头；「显存还给驱动」代码里没有保证 |
| 1：`card_worker.rs:6` 判与核 panic 先问设备 | 一致 | 判与核共用同一个 `catch_unwind` 后的分支 |
| 1：`card_worker.rs:7` 丢判器、报接住 | 一致 | `clear()` 之后报 `OutOfMemoryCaught` |
| 1：`card_worker.rs:7`「显存还给驱动」 | 推不出（打中） | `clear()` 只丢句柄；本波已装判器的表与暂存还在 wgpu 待写列表里，要下一次提交才放；装判器与单独判之间没有提交；要真卡量 |
| 1：`card_worker.rs:8` 接着报空出来 | 一致 | 臂不 `return`，落到 `RoundFinished` |
| 1：`card_worker.rs:9` 设备丢了或判法自己 panic 报卡坏 | 一致 | `DeviceLost` 与 `PanickedOtherwise` 都 `return` |
| 1：`card_worker.rs:11` 弄坏开关 | 一致 | 守卫在第一臂 |
| 1：`card_worker.rs:19` 核的时候 panic 报卡坏 | 冲突 | 与第 6 行、`service_loop.rs:16`、测试 `…ran_out_of_memory_is_verified_again…` 相反 |
| 1：`service_loop.rs:10-15` 清账、撤回、次序、回话、单独判、再 OOM 报拒 | 一致 | 逐行对过，见句 7 到句 14 |
| 1：`service_loop.rs:16` 核对组放回排头、腾空再 OOM 报拒 | 一致 | `push_front` 与 `on_an_emptied_card` |
| 1：`service_loop.rs:16`、README 第 94 行「那张卡」 | 冲突（措辞） | 核对组落到换出最少的那张空卡，不一定是撤回它的那张 |
| 1：`service_loop.rs:7` 设备丢了照旧进 failed | 一致 | `take_a_failed_card` 照旧 |
| 第 2 问 `after_a_panic` 三种情形 | 成立（读代码）、真卡行为推不出 | 连着两回都 `RanOutOfMemory`；OOM 后判法 panic 是 `PanickedOtherwise`；OOM 与丢失同时是 `DeviceLost`（丢失优先、只记不抹）；真卡那一份无测试调过 |
| 2：丢失回调晚于 panic、迟到的 OOM 记录 | 推不出（打中） | 计数是设备上的原始次数，读设备情况时不等回调；探针只能证函数怎么答，证不了时序 |
| 第 3 问整问 | 成立；另一处不成立 | 三处调用、登记、驻留账、对拍测试逐处对得上；文件头「池里不留永远派不出去的态」在登记之后卡坏时不成立 |
| 3：文档串对代码 | 一致 | 再加一份表；wgpu 暂存前提读源码成立，「做完才放」那半句推的 |
| 3：`tables_while_loading` 挑新流、分草稿区、不饿死 | 一致 | `batch_scheduling.rs:378`、`:705`、`:541`；另有单独判两处也一致 |
| 3：登记流时的判 | 一致 | 与调度侧不等式同一个（代数，推的） |
| 3：驻留账仍一份 | 一致 | `admit` 与主循环的账都用 `mebibytes_of_a_judge` |
| 3：`card_memory_matches_the_verification_package` | 同一个量 | 比的是一份表那个量，不在补丁里；新增的两份表没有验证包侧的对照物，登记那一处边界无测试（推的） |
| 3：`service_loop.rs:23` 池里不留派不出去的态 | 不成立 | 登记之后大卡坏、小卡装不下，再入池的块没人拒；探针待跑 |
| 第 4 问整问 | 成立（四环字面相同）；注释一处不成立、README 两处不符 | 前缀、字段、分隔、台标签、子命令名逐字对过 |
| 4 环 1：服务三种行与 control 的前缀、字段 | 一致 | 前缀逐字相同，卡坏行字段次序与模板相同 |
| 4 环 2：`card_failed_lines`、`cards-failed` 输出 | 一致 | 只读这个进程的行，输出格式与文件头、README 相同 |
| 4 环 2：`control:311` 注释点名的 `GPU_CARDS_FAILED_READ_PREFIX` | 不成立 | 全仓（排除 `research/`）只有这一句注释自己，任务服务那边没有这个常量 |
| 4 环 3：`failed_cards_of_the_report` | 一致 | 切四段、取 `card=`、键 `<台>:<进程>:<卡>`；状态行不读，读不了的台等于交空表（推的） |
| 4 环 4：`gpu_card_failures` 发给谁、写什么 | 一致 | 发名下有任务的会话；告警记录的种类不带键；没有会话时不读不写（README 没写这个条件） |
| 4：README 第 94 行「先腾空那张卡再核」 | 冲突 | 同第 1 问 |
| 4：README 第 23 行「设备丢了才报卡坏」 | 冲突（不在两节里） | 判法自己 panic 也报卡坏 |
| 4：README「日志」「起停」其余各句 | 一致 | 字段、退出码、读哪个进程、出路逐句对过 |

## 要主 agent 跑的命令

我一条也没交。三条探针与一个「复核不了」。命令一行一条，先建好的仓副本在草稿目录（`.deliverables` 里登记成「交接」，用完请主 agent 删：`du -sh` 量过 58M，路径 `/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-sonnet/repo-copy`）。副本里新加的测试不进库，所以没写命名检查那两条。三个文件都只过 `rustfmt --check`，没编过：编译失败就是我的探针写错，不是结论，退回给我改。

1. 不上卡，替身判器：第 3 问 3.6 的登记之后大卡坏。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=gpu-verification-server 的测试目标 service_loop_with_judges_off_the_card 里的 1 条测试; 范围=1 个测试; 为什么=核登记之后大卡坏了、再入池的块有没有被拒，文件头写了池里不留永远派不出去的态' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-sonnet/repo-copy -- cargo test -p gpu-verification-server --test service_loop_with_judges_off_the_card -- probe_a_block_only_the_failed_big_card_could_hold_is_refused_not_left_hanging --exact`
   - 在哪个目录起：上面 `--cwd` 那个副本根。
   - 看哪几行：任务末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`，与 `task-server/task log <号>` 里的 `test result:` 行。打中（我的读法对）：`FAILED`，且 panic 信息含「3 秒里没有回话」；没打中：`test result: ok. 1 passed`；`type=timed-out` 或别的：我推的路径漏了别的卡住点，结论作废、退回给我。

2. 上卡（卡池借，带 `--features gpu`）：第 2 问连着两回 OOM、OOM 后判法 panic。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=gpu-verification-server 的测试目标 probe_after_a_panic_on_a_real_card 里的 1 条测试; 范围=1 个测试; 为什么=核真卡那一份 after_a_panic 连着两回记 Out of Memory 与记后判法自己 panic 时各答什么，没有测试调过它' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-sonnet/repo-copy -- cargo test -p gpu-verification-server --features gpu --test probe_after_a_panic_on_a_real_card -- probe_after_a_panic_counts_out_of_memory_reports_since_the_last_answer --exact`
   - 在哪个目录起：同上。
   - 看哪几行：`test result: ok. 1 passed` 且日志里没有「本次未跑」那一行（没卡时它打那一行、仍退 0，那不算过）；`FAILED` 时看哪个 `assert_eq!` 的说明，对应第 2 问哪一格被推翻。

3. 上卡：第 2 问 OOM 与丢失同时到。这条会让借来的那张卡的设备记成丢了（卡池下一次借照它重起，并打一行 `GPU_DEVICE_LOST`），所以单列、不和上一条合。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=gpu-verification-server 的测试目标 probe_after_a_panic_on_a_real_card 里的 1 条测试; 范围=1 个测试; 为什么=核真卡那一份 after_a_panic 在显存不够与设备丢失同时记下时先答哪一个，没有测试调过它' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-sonnet/repo-copy -- cargo test -p gpu-verification-server --features gpu --test probe_after_a_panic_on_a_real_card -- probe_after_a_panic_answers_a_lost_device_before_out_of_memory --exact`
   - 在哪个目录起：同上。
   - 看哪几行：同上一条。

4. 复核不了：第 1 问句 2「显存还给驱动」。原因：要在真卡上量「`after_a_panic` 返回之后、下一波装判器之前，驱动那一侧的空闲显存有没有回到装判器之前的水平」，得新写一个把一个接近额度的判器装满、记一次 OOM、调 `after_a_panic`、立刻读 `nvidia-smi` 的空闲显存的真卡实验；仓里没有现成的入口，我也不能起真卡。建议交实验设计员写，两条臂：`after_a_panic` 之后立刻读，与之后再做一次 `device.poll`、再读。

## 只读核对（我自己跑的，原样输出）

对着开工快照核哈希：`sha256sum -c research/prompts/gpu-oom-card-governance-r1-snapshot/judged-sha256.txt`，其中 16 行 `OK`，一行：

```
task-server/task_server_selftest.py: FAILED
```

前缀与台标签字面相同：对 `gpu-verification-server/control`、`gpu-verification-server/src/serving_from_the_library.rs`、`task-server/task_server_lib.py` 各 `grep -nF`，输出：

```
== GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST"
gpu-verification-server/control:313:FAILED_CARD_OF_A_HOST_PREFIX = "GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST"
task-server/task_server_lib.py:2200:GPU_FAILED_CARD_OF_A_HOST_PREFIX = "GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST"
== GPU_VERIFICATION_SERVER_CARD_FAILED"
gpu-verification-server/src/serving_from_the_library.rs:97:pub const CARD_FAILED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARD_FAILED";
gpu-verification-server/control:307:CARD_FAILED_LINE_PREFIX = "GPU_VERIFICATION_SERVER_CARD_FAILED"
== GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT"
gpu-verification-server/src/serving_from_the_library.rs:99:pub const OUT_OF_MEMORY_CAUGHT_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT";
gpu-verification-server/control:308:OUT_OF_MEMORY_CAUGHT_LINE_PREFIX = "GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT"
== = "this-host"
task-server/task_server_lib.py:2201:GPU_THIS_HOST_LABEL = "this-host"   # cards-failed 里这一台（任务服务所在的这一台）写的台名
gpu-verification-server/control:339:THIS_HOST_LABEL = "this-host"
```

`GPU_CARDS_FAILED_READ_PREFIX` 的全仓（排除 `research/`）命中：只有 `gpu-verification-server/control:311` 一行。`grep -rln 'while_loading' crates gpu-verification-server/src gpu-verification-server/tests` 输出四个文件；`grep -rn 'while_loading' crates/singlefs-verification/src | wc -l` 输出 `0`。

## 没做什么

- 不判攻方腿的格（第 5 到 12 问）；上面凡碰到它们的地方（迟到的 OOM 记录、告警刷屏、没有会话收不到、登记之后卡坏）只当第 1 到 4 问的附带观察写出，不替主 agent 采纳。
- 没起任何项目代码、没跑 cargo、没交任务；三条探针与副本里的三处改动没编过，只过了 `rustfmt --check`。判决要引的数没有，探针结果由主 agent 复跑。
- 没写模型，所以没有 `tools/rerun.sh` 与复跑命令；没量 wgpu 的真实时序（丢失回调何时到、缓冲何时还给驱动）。
- `wgpu-core` 里引用在提交「做完」才放那一半没读（只读到待写列表持有引用、刷出要等下一次提交）；`task_server_selftest.py` 只引了一处，且是此刻的树。
- 没翻 `git log` 查 `card_worker.rs` 第 19 行是哪一轮写的（报告里标了「推的」）。
- 没改任何主工作区文件；写过的只有本报告、草稿目录与草稿目录里的仓副本。
