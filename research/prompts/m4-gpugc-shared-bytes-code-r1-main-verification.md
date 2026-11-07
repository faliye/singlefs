# m4-gpugc-shared-bytes-code-r1：判决（2026-11-07）

正文 `research/prompts/_m4-gpugc-shared-bytes-code-r1-body.md`，改动全文 `research/prompts/_m4-gpugc-shared-bytes-code-r1-diff.md`，开工快照 `research/prompts/m4-gpugc-shared-bytes-code-r1-snapshot/`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-shared-bytes-code-r1-sonnet-output.md` | T1–T4 都判照旧；五处 `make_mut` 没有变异行直接钉，要跑才知道；`mount.rs:1102`–`:1103` 字段注释与代码不一致 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-shared-bytes-code-r1-opus-output.md`，模型 `research/prompts/m4-gpugc-shared-bytes-code-r1-opus-model/` | T2「那一刻只有一个引用」的声明打中、语义没打中；T1、T3、T4 没打中；`publish_lookup_by_identity.rs:1021` 测试注释说的危险在安全 Rust 里造不出来 |

主 agent 的核查（逐条现查或复跑，量的标「量的」）：

1. 开工快照：`sha256sum -c research/prompts/m4-gpugc-shared-bytes-code-r1-snapshot/crates-sha256.txt` 在主工作区 13 份全 OK（量的），腿跑的期间被判文件没被改过，不用倒推；`crates/mutations.tsv` 在派腿之前已从快照里删掉（它不是被判的产品代码，留在快照里挡别的会话追加变异行）。
2. 引文：`python3 scripts/agents/cite-check.py <两份腿报告> --root /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo --background research/prompts/_m4-gpugc-shared-bytes-code-r1-background.md`，末行「✓ 核了 51 处引文、169 处裸行号，对上 49 处，没判 171 处」（量的）；没对上的两处一处是路径写短了、一处是只给行号没给原句，都不改结论。
3. 攻方腿的引用数探针，任务 9602（`research/prompts/m4-gpugc-shared-bytes-code-r1-opus-model/tools/rerun.sh`，工作目录 `~/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-main-check/opus-rerun-2/`）退 0（量的）：`with_writes` 一共打 3326 次，`strong_count=1` 3325 次、`strong_count=2` 1 次，那一次落在 `publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish`，用例绿；`TransactionOutput`、`PublishedUnit`、`PoolVersion` 都是 `Send + Sync` 的编译期断言编得过（`OPUS_PROBE site=send_and_sync_compiled`）。其余几处的读数：`floor_raise_after_the_row_publish strong_count=2` 30 次、`mount_continuation_floor_raise strong_count=2` 38 次、`session_floor_raise_for_space` 1 次 28 次 / 2 次 3 次、`unmount` 1 次 3 次 / 2 次 5 次。
4. 正推腿的变异：五份副本各把一处 `Arc::make_mut` 换成「改一份临时拷贝」，跑同一组四个测试目标，任务 9550–9554 都退 101、各有用例 FAILED（量的，例 9551「a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write ... FAILED」），五处都被现有测试钉住。
5. 第 88 批：正推腿的单元图 dump 9555、9556 退 0；攻方腿的闭包 9523 退 0，`region_record_kernels.rs::packed_region_record_chunk` 的闭包里有 `fn singlefs_background_gpu::live_slot_kernels::low_and_high_words_of_the_slot crates/singlefs-background-gpu/src/live_slot_kernels.rs:73-78`（量的），去重之后本该如此；两份原文逐字相同两腿一致。
6. 注释：冻结副本 `crates/singlefs-core/src/mount.rs:1102`–`:1103` 原文「（两处指着同一份，不拷一整版；写行那一版与暖机各版同样与 `Mounted::current` 共享）」，`:8628` 起 `let mut current = row_publish.clone();`、每次暖机 `warm_up_publishes.push(next.clone()); current = next;`，只有最后一版与现行那一版同一份（现读）；`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_lookup_by_identity.rs:1021` 原文「就地清零会连 version 一起改坏」，单元字节是 `Arc<[u8]>`，安全 Rust 拿不到可写引用（现读）。

「没打中」两腿各只抽了一份样本（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）；本判决的「照旧」不靠腿的否定结论，靠第 3、4、5 条主 agent 量到的读数。

## 二、判决

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/transaction.rs` | 照旧。`with_writes` 的 `make_mut` 引用数 2 只在冻结重发一处出现（核查第 3 条），拷一份交回的内容与不拷相同，不落成 `.claude/kb/decisions/16-发布语义.md` 已定项 17 禁的「照旧复用同一份内存态重试」，也不碰 C381（根已落盘之后发布失败，分配器仍退回）；第 86 批实现员报告那句「那一刻只有一个引用，就地改」不成立，那是报告的话，不改代码 |
| `crates/singlefs-core/src/mount.rs` | 照旧；`:1102`–`:1103` 字段注释改成只有最后一版共享（只改注释） |
| `crates/singlefs-core/src/mounted_session.rs` | 照旧（核查第 3、4 条） |
| `crates/singlefs-core/src/recovery.rs` | 照旧（只是类型跟着改，腿与主 agent 都没找到读身份或改内容的地方） |
| `crates/singlefs-core/src/instance_table_sweep.rs` | 照旧（同上） |
| `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs` | 照旧（只改两条测试名） |
| `crates/singlefs-background-gpu/src/adapter_inventory.rs` | 照旧（`GIBIBYTE` 去重成一份，值同） |
| `crates/singlefs-background-gpu/src/background_gpu_compute.rs` | 照旧（测试条目改名） |
| `crates/singlefs-background-gpu/src/checksum_kernels.rs` | 照旧（测试条目改名） |
| `crates/singlefs-background-gpu/src/gpus_of_this_host.rs` | 照旧（测试条目改名） |
| `crates/singlefs-background-gpu/src/live_slot_kernels.rs` | 照旧（`low_and_high_words_of_the_slot` 改 `pub(crate)`，模块外只有 `region_record_kernels.rs` 引它） |
| `crates/singlefs-background-gpu/src/region_record_kernels.rs` | 照旧（改为 use 那一份，WGSL 源串与入口名不变，核查第 5 条） |
| `crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs` | 照旧（测试条目改名） |

测试代码：`publish_lookup_by_identity.rs:1021` 的注释改成说真的理由（单元字节在 `Arc<[u8]>` 里改不了，换掉 `malformed` 自己那一份）。

这一轮没有打中语义的格，不开第二轮。

### 交用户的

- 攻方腿提的三样收严被攻过零轮，这一轮不做，登记去向：`TransactionOutput`、`PublishedUnit`、`PoolVersion` 三样 `Send + Sync` 的编译期断言（腿的探针在副本里编过，量的）；一道检查钉住「产品代码不读 `Arc` 的指针身份」（`ptr_eq`、`strong_count`、`as_ptr`）；`resend_the_frozen_publish` 重发又失败时把同一份 `FrozenPublish` 放回去，与 D16（发布语义） 已定项 17 射程里「重做又失败」那一格字面上有张力（按值时代就有，与这一批无关，推的）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 17 | 不影响 | 2026-11-07 不受影响：冻结重发那一刻引用数 2 时拷一份、交回内容与按值时代相同，条款不动 |
| D26（后台整理与放置回收） 已定项 13 | 不影响 | 2026-11-07 不受影响：第 88 批只改名与去重，行为不变 |
