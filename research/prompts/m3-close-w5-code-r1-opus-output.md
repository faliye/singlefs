# m3-close-w5-code-r1 攻方腿（Opus）报告（2026-10-19）

<!-- doc-lint:not-numbers W1 W2 W3 W4 W5 M1 M2 M3 M4 M5 M6 M7 -->

立场：攻「哪一格会错」。代码行号都按树副本 `~/.local/share/singlefs/work/m3-close-w5-code-r1-tree` 算；被判的 9 份文件与主工作区逐份同 sha256（现查：`sha256sum` 两边比，9 份都「同」），所以下面写成 `crates/...` 的仓内相对路径。量都在树副本的拷贝上做（副本上的数，不是入库装置上的数）。

## 复跑

模型目录 `research/prompts/m3-close-w5-code-r1-opus-model/`：攻方用例一份（harness 档测试目标 `attack_histories_of_multi_file_writes_and_unlinks`）、`tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`）、`out/` 原样输出。复跑命令（仓根起）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-w5-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-w5-code-r1-tree --work <新的草稿目录>
```

它把冻结副本拷进草稿目录、补仓根的 `Cargo.toml` / `Cargo.lock` / `.cargo` / 线程登记、放进用例、`cargo test --no-run` 编、直接执行测试二进制（定向用例全跑，交错历史只跑种子 200 与 1051）。rerun.sh 没有端到端跑过（新编译目录要从头编 harness 的依赖，前台 240 秒放不下，见「这条腿自己的限度」）；它的每一步与我跑过的命令同形。

各文件 sha256（`find . -type f | sort | xargs sha256sum`，模型目录里跑）：

| 文件 | sha256 |
|---|---|
| attack_histories_of_multi_file_writes_and_unlinks.rs | ee651f61659baee68c801fc2e6d7a1d53cf8b7c2af23a33fb709b36011421875 |
| tools/rerun.sh | a1da0f5d00d9c7e07dede330e3ad5045a0b9f579692d76a6f28d4f0475fab2d0 |
| out/attack-all-but-fuzz.txt | 9a5fe80255148b095416333cbd679192e7ca586afa65499c4a9a1cc5237c6a57 |
| out/run-beta.log | b75130da9123760253aa125bddc8f98b3f4533d688fa8a00a4cb8ecf156127cc |
| out/run-seed200-trace.log | 5d3393c4b296db383872ed796af011c48ec111d9e9b8591797f762cdf2b11913 |
| out/b3-5.log（与 out/fuzz-batch3-part5.log 同一份） | e6f5d39857eece4a2a42abdfb8bb1391d1e32465af2b4e8aee349430ac56129e |
| out/mut-min-summary.txt | d9ddc0a1ba79f031b59e785282d9f69e83ffef93ad966bf2a9845fc6394ac5db |
| out/mut-guard-summary.txt | 6b813a15e37366ccad241f617ae8e7de370f6840fa74f0dd75d6e70f92fdf5fc |
| out/mut-guard-attack.log | 80d6bbb1a2a61fc2304124625726611afc4a3196fc91d4381a0cf0e3e2d15d1b |
| out/base-summary.txt | 259e7bbb6d2d856f6873bf05e9b1b9d4024d39f79afa47be79805d860ada29a0 |
| out/run-crash2.log | 6908010c89c0e2e216a3b81d1fdcfd8785a33af7a6a35dd396c2ed64fd273c4a |
| out/run-level2.log | b69e9794084ecf78ee41e66348f08cf5c2b0efb8ef881210c57757b208e313f2 |
| out/run-rest.log | a05541671def03c9ea9caf8e03ea715df8c8f5a6259b4a04b82ad663f6eed7e8 |
| out/fuzz-batch-first.log | 4f97e918cb0a55401d11e7ba19e2c050da976b59ac1ba4d7c4ca8b5a736d5bfe |
| out/fuzz-batch2-part1.log … part4.log | 725faab4…、1c3814da…、e737f970…、c64b6b60…（全值见 `sha256sum` 现算） |
| out/fuzz-batch3-part1.log … part6.log | 1bf06d2c…、979fff43…、f60199d1…、1e2a7130…、e6f5d398…、b0604b00… |

`out/` 里 `run-beta.log`、`run-rest.log`、`run-level2.log`、`run-crash2.log` 与 fuzz-batch 两批是用例文件往后追加用例之前那几版跑的；被跑的那几条用例的函数体之后没改（只在文件尾部追加、加了 `ATTACK_TRACE` 打印）。`attack-all-but-fuzz.txt` 是定稿那一份用例在未改动的树副本拷贝上的整跑。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| W1 写入口与条款 | 没打中 | 交错写、空内容写、快照下写，checker 每版全绿，读回逐字节对；准入没攻 | W1 |
| W2 上段规范形 | 没打中（一处结构偏离，推的） | 长到层级 2 再删到一个、再删空，降两层照抄成根，I-9.40 全绿；截断那条路不经统一计划（只因截断不改 S 才等价，推的） | W2 |
| W3 删除各步 | 没打中 | 别的号批大小 1 的每一批、最后一批与段边界状态都续做得完；三种删序终态上段同为 (0, 0)；删除中的别的号只读挂载答没有文件 | W3 |
| W4 β「取死亡 txg 最大」 | **打中**（量过） | 删 1 → 建 3 → 回退到删 1 之前 → 再删 1 → 回退到建 3 那一版 → 写 3 做成；这条时间线上 3 生于 1 删完之后，D8 已定项 16 射程「删完之后不再建带数据的文件」被越过；池级 checker 0 条红 | W4-一 |
| W4 墓碑「每个对象一条」 | 判决里那句外推过头（量过，不是代码错） | 回退到 2 活着那一版再删一次，同一对象 (2, 出生代) 两条墓碑；checker 全绿；判决 U4 行「墓碑每对象一条」在回退历史上不成立 | W4-二 |
| W4 恢复、续做 | 没打中 | 段边界状态 33 + 33 个都可写挂载续做到底；全量崩溃状态数 1.5 × 10⁸ / 2.2 × 10⁸，超过 10⁶，没跑 | W4-三 |
| W5 测试与变异 | **打中**（量过） | β 两处实现员自己定的句子没有一条会红的测试：`.max()` 改 `.min()`、删掉「inode 1 记录在就不判」那一句，各自让现有三个 β 相关目标全绿；随机历史的模型同取最大、同样放过 | W5 |

## W4-一　β「取死亡 txg 最大」放过删完之后建的 inode（打中，量过）

**历史**（定向用例 `beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through`，模型目录那份用例第 267 行起）：新池新建文件（inode 1）→ 建 inode 2、按号写它两个单元（让删完 1 之后这一版仍是带文件的一版）→ 记下这一版 v0 → 删 1（意图 txg 6 = d1，产品批大小）→ 建 inode 3（出生 txg 8）→ 这时写 3 被 β 拒（对）→ 管理员回退到 v0（1 又活着）→ 再删 1（意图 txg 10 = d2）→ 管理员回退到建 3 那一版（1 不在、3 在）→ 按号写 3。

**每一步许可它的那一句**：
- 回退的候选集只看根环、F 与带文件：`crates/singlefs-core/src/mount.rs:8863`「- 候选集：根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件（[`rollback_candidate`]）。」，建 3 那一版在第二次回退时仍是候选（两次回退都做成，原样输出里没有拒）。
- 墓碑随现行那一版、回退不取目标的：`crates/singlefs-core/src/mount.rs:8813`「tombstone_containers: current.tombstone_containers.clone(),」。第二次回退之后墓碑树里有 d1、d2 两条 inode 1 的墓碑。
- β 只取最大那条：`crates/singlefs-core/src/transaction.rs:3518`「.filter(|tombstone| tombstone.object.0 == FIRST_INODE_NUMBER)」之后 `crates/singlefs-core/src/transaction.rs:3520`「.max()」；文档注释给的理由是 `crates/singlefs-core/src/transaction.rs:3507`「回退之后又删一次时有两条，」与 `crates/singlefs-core/src/transaction.rs:3508`「管着这一版的是最后那一次删）」。
- 写入口比的是它：`crates/singlefs-core/src/transaction.rs:8535`「if record.object_birth > death_txg_of_the_first_file {」，8 > 10 不成立，放过。

**期望与实际**：这一版的父链是 …→ 6（删 1 的意图）→ … → 8（建 3）→ 第二次回退；d2 = 10 落在第二次回退抛下的那一段里，管着这一版的是 d1 = 6。3 生于 8 > 6，是「删完之后才建的 inode」，期望拒（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md:53`「只拒对象出生代晚于 inode 1 墓碑死亡 txg 的 inode 写数据」，以及 `.claude/kb/decisions/08-核心索引结构.md@已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空` 射程「删完之后不再建带数据的文件（K-U5，记欠账）」）。实际写成，读回 32629 字节，池级 checker 0 条红。

**原样输出**（`out/run-beta.log`，去掉排队那一行）：

```
test beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through ... d1 = CheckpointTxg(6)，b3 = CheckpointTxg(8)
回退之前写 3：Err(DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { inode: 3, object_birth: CheckpointTxg(8), death_txg_of_the_first_file: CheckpointTxg(6) })
d2 = CheckpointTxg(10)
回退之后：3 的出生 CheckpointTxg(8)，β 读到的死亡 txg Some(CheckpointTxg(10))
回退到建 3 那一版之后写 3：Ok(())
写成之后读回 3：Ok(32629) 字节；checker 红 []
```

**复现命令**（草稿目录 `~/.local/share/singlefs/work/m3-close-w5-code-r1-opus`，树副本拷贝在其下 `repo/`，用例已放进 `repo/crates/singlefs-harness/tests/`）：

```
cd ~/.local/share/singlefs/work/m3-close-w5-code-r1-opus/repo && CARGO_TARGET_DIR=../target bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 31 cargo test --offline -p singlefs-harness --test attack_histories_of_multi_file_writes_and_unlinks --no-run
cd .. && bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 850M target/debug/deps/attack_histories_of_multi_file_writes_and_unlinks-98db3e89b1a27288 --nocapture --test-threads 1 beta_after_two_rollbacks
```

**放开用户动作扫过**：两次回退的目标与中间的写删由我自己的交错历史放开抽（W5 节末「交错历史」：建 inode、按号写、删、写第一个文件、回退到之前任一版、可写挂载拷贝，种子 48 段、共 1857 步），种子 200 与 1051 两段自己走到同一形，没有人写死回退目标。种子 200 的轨迹（`out/run-seed200-trace.log`，`ATTACK_TRACE=1` 跑的）：

```
种子 530887153800 第 2 步：删 1，意图 txg 6
种子 530887153800 第 11 步：建 159 个 inode，txg 14
种子 530887153800 第 20 步：回退到 txg 5：做成
种子 530887153800 第 21 步：删 1，意图 txg 20
种子 530887153800 第 33 步：回退到 txg 14：做成
种子 530887153800 第 38 步：写 269（生于 14，这条时间线上 1 死于 Some(6)）做成了
```

种子 1051（`out/b3-5.log`）：「种子 2789811993219 第 34 步：写 13（生于 13，这条时间线上 1 死于 Some(10)）做成了」，同段另有写 61、写 51 两次。我那个小模型的 β 判法是「这条时间线上 1 的意图 txg」（回退时连同模型状态一起回到目标那一版），不是实现员的取最大。

**四句**：
1. 分不分辨臂：分辨「取最大」与「按这一版的父链取」两种读法；这段历史上「取最小」也对（d1），但「取最小」在「回退到删 1 之前、建 X、再删 1」那一形上会把 X 错拒（推的，没跑），所以它不说明取最小对。
2. 被判的系统看不看得到判别它的东西：看得到。谱系记录随现行那一版、每次回退追加一条 (T_rb, T_old)（`.claude/kb/decisions/18-块里携带什么信息.md@已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录` 第 5 条「T_rb 是回退那次发布的 txg，T_old 是 R_old 的 txg」），`TransactionOutput` 带着它（`crates/singlefs-core/src/transaction.rs:3148`「pub lineage_containers: Vec<LineageContainerVersion>,」）。d2 = 10 落在第二次回退的 (T_old = 8, T_rb) 之间，按谱系判得出它不在这一版的父链上。
3. 满足的是判据字面的哪一个分句：β 那句「晚于 inode 1 墓碑的死亡 txg」没说几条墓碑时取哪条，这是读法空白；越过的是 D8 已定项 16 射程「删完之后不再建带数据的文件」（3 是这条时间线上删完之后建的，写成了带数据的文件）。
4. 跑前条款给的改法在这几格上还中不中：判决与岔路单只写了「依赖墓碑第一版不回收，墓碑回收落地时要换读法」，没有针对回退的改法；没有可核的跑前改法。

**改法（只在我的模型上量过、被攻过零轮）**：
- 甲：死亡 txg 只取落在这一版父链上的墓碑——按谱系记录把每段 (T_old, T_rb) 开区间里的死亡 txg 排除，再取最大。推的：没实现、没跑；我的小模型用的是等价的「随目标回退的那一个值」，在 48 段上与实现只在 β 这一处不同。
- 乙：把「inode 1 删于哪个 txg」放进随用户可见树回退的结构（例如回退取目标那一版的一个字段），不读随现行那一版的墓碑树。推的：要动格式或固定点，代价没量。

## W4-二　同一对象两条墓碑（判决那句外推过头，量过，不是代码错）

用例 `rolling_back_into_the_middle_of_the_deletion_of_another_file_is_continued_by_the_mount`：写 2 三个单元 → 批大小 1 删 2 → 写第一个文件 → 回退到删 2 第一批之后（可写挂载续做把 2 删完，checker 全绿）→ 回退到删 2 的意图那一版 → 回退到 2 活着那一版（读回逐字节对）→ 再删 2。原样输出「2 的墓碑条数：2」（`out/attack-all-but-fuzz.txt` 同一行在 `out/run-rest.log`）。判决 U4 行写「墓碑每对象一条」，`.claude/kb/decisions/18-块里携带什么信息.md@已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录` 本身没说每个对象一条；代码那句注释（`crates/singlefs-core/src/transaction.rs:3507`）也认「回退之后又删一次时有两条」。所以这是判决那一句的射程问题，代码与条款原文不冲突；它正是 W4-一 的前提（同一对象几条墓碑、取哪条没有条款）。checker 在这段历史上全绿。

## W4-三　恢复与续做（没打中）

- 段边界状态：S = {1, 150}，批大小 1 删 150（意图、留 2、留 1、最后一批降层）与删第一个文件（150 留着）两段历史，按 `singlefs_harness::memory_pool::writes_and_segments` 切段，每段末尾那个状态加「至多 2 次写的短段」全展开，各判 33 个状态：每个可写挂载（续做批大小 1）做成、续做之后队列空、第一个文件或 150 逐字节读回、删的那个要么整份在要么一个单元不剩、池级 checker 全绿。原样输出（`out/run-crash2.log`）：「删 150（降层）：写 116 次，段长 [26, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2]，闭式状态数 152043545」「删 150（降层）：判了 33 个状态（段边界加短段全展开）」；「删 1（150 留着）：写 122 次，段长 [26, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2, 26, 2, 1, 2]，闭式状态数 218103833」「删 1（150 留着）：判了 33 个状态（段边界加短段全展开）」。
- 全量崩溃状态没跑：一段历史 1.5 × 10⁸ 与 2.2 × 10⁸ 个状态，超过每次约 10⁶ 的上限（定义第 3b 步「一段历史自己就超过约 10⁶ 的，那段不跑」）；单次发布的单元写段就有 20–26 次写（2²⁶ ≈ 6.7 × 10⁷），缩历史也压不到 10⁶ 以下。33 个状态是这两段历史的一个子集，不当崩溃点重放的结论用。原地覆写的撕裂第三态也不在里面。
- 回退到别的号删除中途、可写挂载续做把它删完；回退到意图那一版、再回到它活着那一版：都做成、checker 全绿（上一节那条用例）。
- 交错历史里每次可写挂载一份拷贝都做成（48 段里抽到的每一次，没有一次报错）。

## W5　β 两处自己定的句子没有会红的测试（打中，量过）

实现员定的「β 只在 inode 1 的记录不在时判；有几条墓碑时取死亡 txg 最大的那条」落在 `crates/singlefs-core/src/transaction.rs:3512`「if self.inode_record.is_some() {」与 `crates/singlefs-core/src/transaction.rs:3520`「.max()」。`crates/mutations.tsv` 里改 core 的 β 行只有一行（另一行 4862 改的是理想模型）：`crates/mutations.tsv:4852`「K-U5 β：对象出生代与 inode 1 墓碑死亡 txg 比反」（改比较方向），这两句各没有一行。我在树副本拷贝上各改一次、只编 harness、跑与 β 有关的三个现有目标：

| 改法 | `unlink_of_files_other_than_the_first` | `writes_by_inode_number_of_several_files` | `writes_by_inode_number_in_random_histories_judged_by_the_model` | 判 |
|---|---|---|---|---|
| 不改（基线） | 10 passed | 6 passed | 5 passed, 1 ignored | 量过 |
| `.max()` → `.min()` | 10 passed | 6 passed | 5 passed, 1 ignored | 量过：活下来 |
| `if self.inode_record.is_some() {` → `if false && self.inode_record.is_some() {` | 10 passed | 6 passed | 5 passed, 1 ignored | 量过：活下来 |

原样输出（`out/base-summary.txt`、`out/mut-min-summary.txt`、`out/mut-guard-summary.txt` 各行，摘三行）：

```
unlink_of_files_other_than_the_first-18eea8402292e592 rc=0 test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.64s
writes_by_inode_number_in_random_histories_judged_by_the_model-5507cf63c2321401 rc=0 test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 18.63s
unlink_of_files_other_than_the_first-18eea8402292e592 rc=0 test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.29s
```

（第一行是取最小那一趟、后两行是删 guard 那一趟；三份 summary 的全部行在模型目录 `out/`。）

- 删 guard 那一句会改行为：我加的 `a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta`（删 1 → 回退到删 1 之前 → 建 3 → 写 3，1 活着）在不改的树上过、在删 guard 的树上红，原样「1 活着时写 3 照写: DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { inode: 3, object_birth: CheckpointTxg(8), death_txg_of_the_first_file: CheckpointTxg(5) }」（`out/mut-guard-attack.log`）。
- 取最小那一改在 W4-一 那段历史上反而判对（我的 W4-一 用例在取最小的树上过：`out/mut-min-summary.txt` 末行「attack_histories_of_multi_file_writes_and_unlinks-98db3e89b1a27288 rc=0 test result: ok. 8 passed」）；取最大与取最小各在一种回退历史上错，现有测试一种都没有。
- 随机历史的理想模型照抄同一读法：`crates/singlefs-harness/src/model.rs:1524`「意图那次写成这次的 txg，别的发布照抄；墓碑树随」现行那一版取最大，所以模型对拍罩不到 W4-一（我的交错历史用另一个读法才抓到）。
- 随机历史不在第一个文件删完之后删别的号：多文件删除模型实现员报告（草稿目录 `m3-close-w5-multi-file-unlink-model-writer/report.md` 第 35 行，仓外，未核进仓）「只在第一个文件那条记录还在时按号删」，同一份报告说快档 56 次抽到的 unlink 里 49 次没调。「删完第一个文件之后删到只剩一个、再删空」只有定向用例罩着，随机历史一次都走不到。

## W1　写入口与条款（没打中）

- 交错历史（攻方自己的小模型，见下面「交错历史」）每次按号写之后池级 checker 全绿、活着的每个 inode 只读挂载读回与模型逐字节相同；号落在上段第 0、1、多片叶上（一次建至多 159 个，号过 143 就换叶、过 21021 就长到层级 2 的那一形在定向用例里）。
- 空内容按号写（`an_empty_write_to_another_inode_mounts_and_is_deleted`）：原样「写空内容之后：2 的单元数 Some(1)，size Some(0)，有条目的号 {1, 2}」，与第一个文件「长度 0 也写一个单元」同一条规则；之后可写挂载、只读读回 0 字节、批大小 1 删掉，都做成。
- 活快照之下按号改写 2、删 3、删 1、删 2（`deleting_another_file_in_batches_under_a_live_snapshot_keeps_every_version_green`），每一版 checker 全绿，删空之后可写挂载做成。
- 准入扣的量（`admission.rs`、`write_accounting.rs`）没攻：没造近满的盘，见限度。

## W2　上段规范形（没打中；一处结构偏离，推的）

- 长到层级 2 再删到一个、再删空（`the_upper_segment_grown_to_level_two_is_deleted_down_to_one_file_and_then_to_none`，建 21100 个 inode）。原样（`out/run-level2.log`，节点位置照抄、只缩写成 (层级, 序号)）：写完 (0,0)(0,1)(0,147)(1,0)(1,1)(2,0) 根层级 2；删 1 之后 (0,1)(0,147)(1,0)(1,1)(2,0)；删 21100 之后 (0,1)(1,0)——降两层、(1,0) 照抄成根；删 150 之后 (0,0)。每一步 checker（含 I-9.40）全绿、可写挂载一份拷贝做成、留下的文件逐字节读回。
- 三种删序（{1, 2, 150}，149 个 size 0 的 inode 留着）终态上段都是 (0, 0)：「各删序终态：[([1, 2, 150], [ExtentUpperNodePosition { level: 0, index: 0 }], false), ([150, 2, 1], [ExtentUpperNodePosition { level: 0, index: 0 }], false), ([2, 1, 150], [ExtentUpperNodePosition { level: 0, index: 0 }], false)]」。
- 结构偏离（推的，没量）：截断那条路的 extent 计划不经 `plan_the_extent_upper_segment`（`crates/singlefs-core/src/extent_tree.rs:212`「pub fn plan_the_extent_upper_segment(」），而是自己按上一版根层级重写 inode 1 那一串：`crates/singlefs-core/src/truncate.rs:276`「let path_of_the_file = upper_path_of_inode(inode, upper_root_level);」，由 `crates/singlefs-core/src/transaction.rs:7156`「plan_the_extent_tree_after_a_size_change(」调。它与统一计划等价只因为截断留至少一个单元（`crates/singlefs-core/src/truncate.rs:242`「"截断留至少一个单元、变长不加单元"」），S 不变。背景材料第三节 W2 那一问「写与删两边是否都只经 `plan_the_extent_upper_segment` 定形」按字面不成立（截断是第三条路）。将来截断成 0 不留单元，或第一个文件之外的号能截断，这里就会写出与规范形不同的形状。
- 实现员定的第 3 条（模型里上段根之下的节点占槽取「这一版上段的全部节点」当上界，草稿目录 `m3-close-w5-multi-inode-model-writer/report.md` 第 42 行）：只放宽模型的单元区墙，不让准入低估（准入按实际重写的角色数，模型那一格是判别力变弱）。推的。
- 第 4 条（GPU `MAX_UPPER_ENTRIES` 64，`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:259`「const MAX_UPPER_ENTRIES: u32 = 64u;」）、第 5 条（必红世界「少一个祖先」落成「少一片叶」）：只读了内核的分批循环（正好 64 条时 `batch_first >= upper_entry_total` 退出，下一批 0 条而还有条目时记判不了），没找到错；checker 档没跑，判不了。

## W3　删除各步（没打中）

- 别的号批大小 1：每一批之后、最后一批之后 checker 全绿（`MemPool::unlink` 每一步都判），删 150、删 2、删 21100、删 3 都走到最后一批；最后一批删记录与 `deleted_inodes` key 同一次发布这一句由 W4-三 的段边界状态间接罩着（没有一个状态出现「记录没了、key 还在」或反过来而挂载做成的）。
- 删除中的别的号只读挂载答没有文件（`a_read_only_mount_answers_no_file_for_another_inode_from_its_intent_on`）：原样「意图之后只读挂载打开 2：Err("打开 2：NoSuchInode { inode: InodeNumber(2) }")」「第一批之后只读挂载打开 2：Err("打开 2：NoSuchInode { inode: InodeNumber(2) }")」。
- 删第一个文件之后别的文件还活着：交错历史与定向用例里这一版都归带文件的一版、可写挂载做成；只剩 size 0 的 inode 时同样归带文件（层级 2 用例末行「成员 带文件」）。
- 两种删序的终态逐字节相同：判决 U3 删序那一行已定不要求；我只比了上段位置与成员，相同。

## 交错历史（攻方自己的小模型）

用例 `interleaved_histories_against_the_attack_model`：一步抽一样——建 1–159 个 inode、按号写（1–4 个单元，号在活着的号里抽，含 1）、删（批大小 1，号在活着的号里抽，含 1）、回退到之前任一版（拒了就跳过）、可写挂载一份拷贝。每次发布之后 checker 全绿；每一步之后活着的每个 inode 读回与模型相同；按号写的结局与「这条时间线上 1 的意图 txg」那一读法的 β 比。量：48 段、1857 步（`grep` 各批日志里「走了 N 步」那一行加总），每批的段数按先跑一小段估的：2 段 × 30 步 29 秒，于是 4 件并行、每件 8–10 段 40 步；第二批 4 件被 200 秒前台时限切掉，只数打出了「走了」的段。β 之外没有一处对不上。

## 没打中的形状

| 形状 | 取样 |
|---|---|
| 上段长到层级 1、2 之后删到一个、删空 | 层级 2：一段（21100 个 inode）；层级 1：交错历史与 W4-三 两段 |
| 别的号删一半崩溃、续做 | 两段历史的段边界加短段 66 个状态；全量没跑 |
| 回退到别的号删除中途、续做 | 一段 |
| 快照之下删别的号、删第一个文件、删空 | 一段 |
| 空内容按号写、再删 | 一段 |
| 三种删序的终态上段 | 三段 |
| 交错写删回退挂载 | 48 段、1857 步 |
| 删除中的别的号只读挂载 | 意图之后、第一批之后两版 |

## 这条腿自己的限度

- 只跑了 harness 档与我自己写的用例；checker 档一条没跑，GPU 判器那两处（`MAX_UPPER_ENTRIES` 64、必红世界换成「少一片叶」）只读代码、判不了。
- 崩溃状态没按定义第 3b 步调 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 的全量枚举：harness 测试拿不到 checker 档的库，且这两段历史的全量各 1.5 × 10⁸、2.2 × 10⁸ 个状态，超过约 10⁶，按定义不跑。跑了的 66 个段边界状态是子集（只取每次写两态，原地覆写的撕裂第三态不在里面），不当崩溃点重放的结论。这一条腿全部原型合计判过的崩溃状态 66 个。
- 盘用的是内存稀疏盘（`SparseBlockDevice`）；每段历史从 mkfs 起各建一次池，没有从一份起点池拷（历史短、建池几十毫秒，没省）。
- 内存包装：起初 slice 只剩约 0.9 GiB（别的会话的门禁与测试占着），用例取 850M 上限排队跑；第一版崩溃枚举把状态表先攒成一张大表，撞了 850M（`run-with-memory-cap` 报「撞了内存上限 850M」、退 250），改成逐个状态判之后不再撞。那一次的输出不算结果。
- 交错历史里我的 β 读法（这条时间线上 1 的意图 txg）本身是一个模型：它与实现在 β 之外一处都没对不上，β 那几处我用定向用例与轨迹逐步核过（d1、d2、出生代、回退目标都在原样输出里）。
- 变异只做了 β 那两句、各一条、只跑三个现有目标加我的目标；没跑 `prove-red.sh`，没跑 crates 变异整表。
- `tools/rerun.sh` 没端到端跑：新草稿目录要从头编 harness 依赖，加上用例运行约 5–6 分钟，前台 240 秒放不下；子 agent 不起后台。它的各步与我实际跑过的命令同形。
- 准入（W1 第二样）没攻；「回退之后水位取现行那一版」（实现员第 2 条）只读了 `mount.rs` 的回退注释与模型那句，交错历史里回退之后建 inode 的号与读回都对得上，没另造历史。

## 要主 agent 跑的命令

- 复跑模型目录（判决要引复跑数时）：照「复跑」一节那条命令，`--work` 给一个新的草稿目录，8G 上限；看 `name=rerun_directed` 那一行（期望 `9 passed; 1 failed`，红的是 `beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through`）与两行 `name=rerun_fuzz`（种子 200、1051 各 `β 对不上` ≥ 1 次）。线程照登记值。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没读禁读清单里的正推腿报告与它的草稿目录。
- 没改仓里任何文件，只新建了这份报告与模型目录。

## 删了什么

交回前删了自己建的：编译目录 `~/.local/share/singlefs/work/m3-close-w5-code-r1-opus/target`（3.1G）、树副本拷贝 `~/.local/share/singlefs/work/m3-close-w5-code-r1-opus/repo`（34M）、改变异时留的原文件备份 `transaction.rs.pristine`（780K）。草稿目录里只剩日志与辅助脚本（380K）。
