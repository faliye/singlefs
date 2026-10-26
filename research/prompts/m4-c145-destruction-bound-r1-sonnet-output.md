# m4-c145-destruction-bound-r1 云端正推腿报告（2026-10-26）

立场：从 D5 已定项 12 原文、d5-item12 三轮判决与 `crates/` 今天的实现推 B1 的结论与依据，给 B2 / B3 的推荐形态。不读攻方腿本轮的产出（禁读清单照守）。

## 0　复跑、哈希与读过什么

- 模型目录 `research/prompts/m4-c145-destruction-bound-r1-sonnet-model/`（`tools/rerun.sh` 的复跑记录在 0.1 末一行）。
- 复跑命令（工作目录随意，整条经内存包装，线程上限 31）：
  `bash scripts/hosts/run-with-memory-cap.sh 5G bash research/prompts/m4-c145-destruction-bound-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>`
- 冻结副本只读；我把它的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo` 拷进自己草稿目录的 `repo/` 里编（补丁、探针只在那里）。冻结副本里的 walk.rs 与工作区此刻的 walk.rs、`crash_judge_checker_scan.wgsl` 不同（行号差：walk.rs 这一带 +106，内核这一带 +19；`transaction.rs`、`records.rs`、`mount.rs` 与工作区逐字节相同）。凡引 walk.rs / 内核的行，下面写成**覆盖两边的区间**（冻结副本的行 – 工作区此刻的行），区间里逐字含那句原文。

### 0.1　文件与 sha256
- `destruction_bound_model.py`　2a22cbbc2b801a92d07ffb904104159a4c3c2afb345702ae6c42e0c6033c3341
- `out/control-core-mutation.txt`　c36a9fd1eabc18fa97f16f5a316f8aebdccf8ab7f07dc3d0091028d28d332f4d
- `out/control-unit-test.txt`　343e0d863a3d1487e1b120baa604c209f199806a513ea737b4ab2423f6efca4f
- `out/model-baseline.txt`　f0051396411712574a63207fa4bbef6b3d296de3f4b11d60a0a7ed809d66170c
- `out/model-mutant-allow-create-while-destroying+allow-two-destructions.txt`　28c8ea2ca2efa1cee0e79f42ab1f6d5d1810f8c8c9d2b88ae41d139e6d4ca33d
- `out/model-mutant-allow-create-while-destroying.txt`　d2b881f270dda48cb90af9f9ffc4ffff585a05bc3a892175eceb7e8c7aa75bf9
- `out/model-mutant-allow-two-destructions.txt`　9f6b86b98f75d5b7c55529fd945f35377656643b8278e16b438dc0a86f685120
- `out/model-mutant-hi-is-latest-published.txt`　e8c9b3bca92f415516c935d736bd10578b65cd5e9a5725b6e31ca4005dcc580a
- `out/model-mutant-intent-does-not-pass-the-predecessor.txt`　183c74df21a05d43e237b7f9a34da35ca2f1cf01a93b703730e30438770cc940
- `out/model-stray-entry.txt`　5006ce6ed167a63098b1c608bf7eff5d1a836d185accdb257f35ba1692e2e974
- `out/patched-unit-test.txt`　aa2e54b39e708169503427d73e5cad489593d7a50a5f34297daf5a1ecdd3a639
- `out/probe-patched.txt`　8a759cceb89ce24b9b0a6cf5e286bd651b994eb09bcaee52f00b82f7cc8fd1e1
- `out/probe.txt`　8a759cceb89ce24b9b0a6cf5e286bd651b994eb09bcaee52f00b82f7cc8fd1e1
- `probe/destruction_bound_equivalence_probe.rs`　0be092aae333599ae77672f46d021b78b39493fadb47ddaca6d56bae8572f352
- `probe/walk-version-bound.diff`　5ea7d4ac6a8432b4191599e0361287dfdae8149f740659ec1101f9617dc47b11
- `tools/rerun.sh`　3bd558e756bc78e0f5d6297c76929b626ee6dfeb8f7b4a630b17e21d5b8c9929
- 复跑记录：`rerun.sh` 在新 shell 里经内存包装整份跑过一次，退 0，12 行 ✓、0 行 ✗（11 个产物文件与 `out/` 逐行相同，加两条控制：新单测在上界改回无穷时红、探针在 core 拿掉传前驱时红）。用时 6 分 51 秒，内存包装上限 5G。

### 0.2　实现今天的样子（我读到的，标成观测；命令在 B1 各点里）

- 销毁意图写在被销毁快照自己的码 14 条目 flags 位 0，不存 hi（`records.rs:563`；D5 已定项 12 第一版段）。
- 建快照与写销毁意图各自在发布前拒掉已有快照在销毁中的情况（`transaction.rs:9450`、`:9527`）。
- 活头 `previous_snapshot_txg` 只在建快照（换成这次的 txg）与写意图（下一侧是头时换成 P）两处变（`transaction.rs:14124-14137`）；deadlist 条目的纪元只在 `transaction.rs:6308-6310` 一处取，值是上一版活头的这个前驱。
- core 放块的 hi 在下一侧是头时取这一批自己的 txg（`transaction.rs:6182`、调用 `:6322`、`:6328`、`:9577`），checker 的待放集在下一侧是头时无上界（`walk.rs` 冻结副本 :16684-16699）。
- 回退牵涉的两版里有快照时整个拒绝（`crates/singlefs-core/src/mount.rs:9684`、`:9763`）。

## B1　等价论证成不成立

**判定：一致（成立）——限于第一版写路径能写出的历史。** 下一侧是活头时，「按 ∞ 判」「按写意图那次的 txg 判」「按放块那一批自己的 txg 判」三种读法，在盘上读得到的每一条条目上结论相同；找不到反例。反例只在拿掉两道拒绝（建快照时已有销毁中的、写意图时已有销毁中的）之后才有。

### 1.1　三种读法各出在哪（并排整行抄）

- 写意图那次的 txg（条款）：`.claude/kb/decisions/05-快照-空间记账机制.md@已定项 12：deadlist 条目的形态`：「hi 在下一侧是快照时取它的 txg，是头时取写意图那个开放 checkpoint 的号（取已发布的最大号会让区间为空、条目永远不放）」
- 写意图那次的 txg（条款，第一版段）：`.claude/kb/decisions/05-快照-空间记账机制.md@已定项 12：deadlist 条目的形态`：「hi 每批从派生链现取（下一条码 14 的诞生 txg，下一侧是头时取写意图那次发布的 txg），不存续做点（条目被放掉就是进度）；不住 logged_ops」
- 放块那一批自己的 txg（core）：`crates/singlefs-core/src/transaction.rs:6165-6166`：「（调用方给这一批自己的 txg）：删快照串行（第七节岔路 2），意图之后到最后一批之间的发布只有这次销毁的批与挂载自己的写行、暖机、抬 F，」
- 同上，取值那一行：`crates/singlefs-core/src/transaction.rs:6182`：「Some(NextSideOfASnapshot::LiveHead) | None => hi_when_the_next_side_is_the_live_head,」
- 同上，调用处：`crates/singlefs-core/src/transaction.rs:6322`：「deadlist_entries_the_destruction_of_a_snapshot_releases(previous_version, snapshot, txg)」
- ∞（checker 代码）：`crates/singlefs-checker/src/walk.rs:16695-16801`：「entry.epoch >= destroyed.snapshot_txg」（冻结副本 :16695，工作区此刻 :16801；区间同时罩住两处）
- 写意图那次的 txg（checker 注释）：`crates/singlefs-checker/src/walk.rs:16683-16789`：「所以下一侧是头时按 ∞ 判与按写意图那次的 txg 判，盘上读得到的条目上结论相同）。」（冻结副本 :16683，工作区此刻 :16789）

**两处字面冲突，没有行为后果**：(1) 条款（kb 两行）写 hi 取写意图那次发布的 txg，core 取每一批自己的 txg——三种读法在代码里各占一处（条款、core、checker）；(2) checker 注释说它「按 ∞ 判」与「按写意图那次的 txg 判」相同，代码里写的只是 ∞（`DeadlistSide`/`entry_is_pending_release` 里 `upper_bound.is_none_or`，冻结副本 `walk.rs:16690-16696`）。下面 1.2 证它们在写路径可达的状态上相同。

### 1.2　正推：为什么相同（五个承重点，每一点各指一处代码）

记被销毁的快照为 D，lo = D.txg，P = prev(D).txg，写意图那次发布的 txg 为 h，下一侧是活头。待放集 = {头 = 活头、纪元 >= lo、birth > P}（∞ 读法）；条款读法再加纪元 < h。两者的差只能是「纪元 >= h、头 = 活头、birth > P」的条目。证它们不存在：

1. **条目的纪元只有一个来路。** 全仓构造 deadlist 条目的只有一处：`crates/singlefs-core/src/transaction.rs:6308-6310` 的 `entries_killed_this_publish.push(DeadlistEntry {` 与 `epoch: previous_snapshot_txg_of_the_live_head,`（在冻结副本里 `grep -rn 'DeadlistEntry {' crates/singlefs-core/src` 原样输出四行：`crates/singlefs-core/src/records.rs:607:pub struct DeadlistEntry {`、`records.rs:613:impl DeadlistEntry {`、`records.rs:1144:        let entry = DeadlistEntry {`（单测）、`crates/singlefs-core/src/transaction.rs:6308:                entries_killed_this_publish.push(DeadlistEntry {`，只有最后一行是写路径的构造）。纪元 = 上一版活头的 `previous_snapshot_txg`（取值函数 `transaction.rs:4274`）。
2. **活头的 `previous_snapshot_txg` 只有三种写法**（`transaction.rs:14124-14137` 的 `match snapshot`）：建快照那次换成这次的 txg；写意图那次、下一侧是活头时换成 D 的前驱 P（`Some((NextSideOfASnapshot::LiveHead, predecessor)) => predecessor,`，`transaction.rs:14131`）；其余发布照上一版抄。回退那条路径（`crates/singlefs-core/src/mount.rs:9557`）也只是照抄，且 core 对「回退牵涉的两版里有快照」整个拒绝（`crates/singlefs-core/src/mount.rs:8962`、`crates/singlefs-core/src/mount.rs:9684`、`crates/singlefs-core/src/mount.rs:9763`；用例 `snapshot_release_and_destroy_through_the_deadlist.rs:738`），所以第一版的历史里没有「回退之后活头的前驱变回去」这一支。
3. **写意图之后到最后一批之间，活头的前驱恒为 P。** 因为：建快照被拒（`transaction.rs:9450` 的 `SnapshotCreationRefusal::SnapshotBeingDestroyed { being_destroyed },`；用例 `snapshot_release_and_destroy_through_the_deadlist.rs:709`），第二个写意图被拒（`transaction.rs:9527` 的 `SnapshotDestructionRefusal::AnotherSnapshotIsBeingDestroyed { being_destroyed }`），其余发布照抄。P < lo（P 是 D 的前驱，lo 是 D 自己的 txg）。所以**写意图之后新进来的条目纪元恒为 P**，落在 [lo, ∞) 之外。这个前提在盘上有现成的判别子：I-9.25 对每个走到的版本判：`crates/singlefs-checker/invariants/09-inode树结构.md:37`：「inode 树那条等于最后一条不带记号的码 14 的诞生 txg」（GPU 判器内核还没加前驱链那一半，同一行自己写了），写意图没把 P 传给下一侧的镜像在意图那一版就红（1.4 的控制实验）。
4. **写意图之前的条目纪元都 < h。** 它们的纪元是某个更早的活头前驱，即某次建快照或某次写意图的 txg，每次发布的 txg 都是上一版 +1（`transaction.rs:5624` 的 `checkpoint_txg_of_the_publish_after`，`checked_add(1)`），h 是写意图那次发布自己的 txg，大于任何更早发布的 txg。
5. 由 3、4：版本里「头 = 活头、纪元 >= lo、birth > P」的条目全是写意图之前写下的，纪元 < h。差集为空。

core 的注释 `transaction.rs:6165-6166` 与 checker 的注释（`walk.rs` 冻结副本 :16681-16683）各给了一半论证：checker 注释只有第 3 点（「写意图之后头的前驱已是 P < lo」），没有第 4 点（写意图之前的条目纪元 < h）；两点合起来才推出差集为空。

同一论证对放块那一批自己的 txg（记 t_b >= h）更宽松：条目纪元 < h <= t_b。所以 core 取批自己的 txg 与条款取 h 放的是同一组，`transaction.rs:6165-6166` 的结论成立；**但它写的理由（「发布只有……都不杀用户可见单元」）不是承重点**：就算那些发布杀了用户可见单元，纪元也是 P 而不是 >= h（第 2、3 点）。承重点是第 2、3 点那两道拒绝加「前驱只在建快照与写意图两处变」。

**下一侧是快照 T 时本来就没有这个问题**：两种读法都是 T.txg（`transaction.rs:6181` 的 `Some(NextSideOfASnapshot::Snapshot(next)) => next.birth_txg,`；checker 侧 `walk.rs:16666-16680` 区间的 `next_live_side_upper_bound_after`）。注意两处对「T 在不在链上」的口径不同：core 的链不看销毁中记号（`transaction.rs:6132`「链上不在乎「销毁中」的记号：第一版删快照串行，带记号的只有正在删的这一条」），checker 的链只数活快照（`live_snapshots`）；串行下两者相同，破串行时才分叉，见 1.3 的变异。

### 1.3　第二条路（不共用 1.2 的论证）：独立小模型穷举

`research/prompts/m4-c145-destruction-bound-r1-sonnet-model/destruction_bound_model.py`（纯 Python，不读 `crates/`，只抄条款语义）：状态 = 活头前驱、快照链、活头单元、deadlist 条目；操作 = 覆盖写（杀一个活头单元，birth <= 活头前驱则进 deadlist、纪元 = 活头前驱，否则直接释放）、建快照、写销毁意图（前驱传给下一侧）、放一批（一条）、放最后一批；建快照与第二个写意图的拒绝按 core 写成开关。广度优先到深度 8、按状态去重；每一版上比三种待放集、每一批比三种放出的集合、并跑一份照 `walk.rs` 同名判定写的 ③ ④ 逆向与 I-3.6。

```
python3 destruction_bound_model.py --depth 8
mutant=- depth=8 states=5957 flagged_head_next_states=2076 entries_in_such_states=1340 batches_checked=3444
pending_set_differs=0 release_set_differs=0 judge_reds=0
```

（原样输出，存 `out/model-baseline.txt`。）2076 个「销毁中且下一侧是头」的状态里共 1340 条 deadlist 条目，逐条比三种读法：差 0；3444 次放块比三种上界放出的集合：差 0；模型里的检查器判定在全部 5957 个状态上红 0。

**校验路径自证（拿掉开关必须红）**：

| 弄坏的开关 | 命令的 `--mutant` | 原样结果（`out/` 下同名文件） |
|---|---|---|
| 写意图不把前驱传给下一侧（对应 `transaction.rs:14131` 那一支拿掉） | `intent-does-not-pass-the-predecessor` | `pending_set_differs=0 release_set_differs=0 judge_reds=471`——三种读法仍相同，但模型里的检查器判定红：这条路径不由上界读法抓（真 checker 另有 I-9.25，模型没写它，见 1.4） |
| 建快照不拒 | `allow-create-while-destroying` | 三项都是 0 |
| 第二个写意图不拒 | `allow-two-destructions` | `pending_set_differs=0 release_set_differs=0 judge_reds=105` |
| 两道拒绝都拿掉 | `allow-create-while-destroying,allow-two-destructions` | `pending_set_differs=5 release_set_differs=5 judge_reds=179`；第一例历史 `[C, C, I(1), C, W(1), I(0), I(2), F(2)]`，条目 `[5,1,1]`（纪元 5、birth 1、单元 1）：∞ 判待放、写意图那次的 txg 判不待放、本版 txg 判待放；放块集合 `{"inf": [[5,1,1]], "intent": [], "batch": [[5,1,1]]}` |

读法：**只拿掉一道拒绝，三种读法仍相同；两道都拿掉才分叉**。所以 1.2 第 3 点里「建快照被拒」与「第二个写意图被拒」是合取的承重，缺一道不破。分叉历史是写路径写不出的（两道拒绝在 `transaction.rs:9450`、`:9527`）；谁对（∞ 还是写意图那次的 txg）不在这一轮的射程，没判。

模型没复现的一条：把 core 的 hi 换成「上一版的号」（变异开关 `hi-is-latest-published`）三项仍都是 0（深度 8，5957 个状态），所以条款「取已发布的最大号会让区间为空、条目永远不放」在第一版写路径的节奏（每次发布各占一个 txg）下我没复现出来；`research/prompts/d5-item12-r3-main-verification.md:37` 那一行的 5 步漏放出在它自己的 BFS 模型里，我没跑那份。


### 1.4　第三条路：真写路径探针（core 的真发布函数 + 真 checker，不共用模型）

`probe/destruction_bound_equivalence_probe.rs`（harness 档测试，拷进草稿目录的仓副本 `crates/singlefs-harness/tests/` 里跑，不进仓）：从新池起，用 core 的 `publish_overwrite`、`publish_create_snapshot`、`publish_snapshot_destruction_intent`、`publish_snapshot_destruction_step`（批大小压成 1）和可写挂载续做（`mount_writable`，只放在历史末尾）枚举深度 N 内的全部步序列，被拒的步序列剪掉；每个历史的最后一版上：① 取「销毁中、下一侧是活头」的快照，数盘上读得到的、∞ 读法会算进待放集的条目（头 = 活头、纪元 >= lo、birth > P），其中纪元 >= 写意图那次发布 txg 的有几条、纪元 >= 本版 txg 的有几条；② 对整张镜像跑真 checker `check_pool_image`，数 `Violated`。

命令（草稿目录的仓副本里，冻结副本的 crates 原样加这一份测试；`DEPTH=6`；7 条测试并行）：

```
TMPDIR=<草稿目录>/tmp DEPTH=6 SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 destruction_bound_equivalence_probe; 范围=1 个测试目标; 为什么=…' nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 3G cargo test --offline -p singlefs-harness --test destruction_bound_equivalence_probe -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 128.40s
```

PROBE 行（原样存 `out/probe.txt`，下表是同一份数字按列摆开）：

| 前缀（前两步） | 历史数 | 被拒的步序列 | 销毁中且下一侧是头的版本 | ∞ 读法算进待放集的条目 | 其中纪元 >= 写意图 txg | 其中纪元 >= 本版 txg | 真 checker 红 |
|---|---|---|---|---|---|---|---|
| prefix_create_snapshot_then_create_snapshot | 173 | 134 | 40 | 28 | 0 | 0 | 0 |
| prefix_create_snapshot_then_intent | 44 | 53 | 11 | 4 | 0 | 0 | 0 |
| prefix_create_snapshot_then_overwrite | 126 | 103 | 44 | 104 | 0 | 0 | 0 |
| prefix_overwrite_then_create_snapshot | 123 | 106 | 34 | 42 | 0 | 0 | 0 |
| prefix_overwrite_then_overwrite | 67 | 66 | 17 | 19 | 0 | 0 | 0 |
| single_create_snapshot | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| single_overwrite | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| 合计 | 535 | 462 | 146 | 197 | 0 | 0 | 0 |

读法：535 个能写出来的历史（被拒的 462 个步序列剪掉）；其中 146 个历史的最后一版是「销毁中且下一侧是头」，∞ 读法在它们上面把 197 条 deadlist 条目算进待放集，**纪元 >= 写意图那次 txg 的 0 条，纪元 >= 本版 txg 的 0 条**；真 checker 在这 535 个历史的最后一版上红 0 次。深度 6 之内（含挂载续做放在历史末尾）没有反例。

**探针自证（弄坏 core 它必须红）**：把 `transaction.rs:14131` 那一支改成不传前驱（`Some((NextSideOfASnapshot::LiveHead, _predecessor)) => previous_snapshot_txg_of_the_live_inode_tree_before_this_publish,`），深度 5 重跑，原样结果存 `out/control-core-mutation.txt`：`test result: FAILED. 2 passed; 5 failed`，真 checker 红 47 次，第一例 `[CreateSnapshot, CreateSnapshot, IntentOnTheLastSnapshot]` 红在 **I-9.25**（树表条目 `previous_snapshot_txg`：「inode 树那条等于最后一条不带记号的码 14 的诞生 txg」，`crates/singlefs-checker/invariants/09-inode树结构.md:37`）。所以 1.2 第 3 点那个承重前提（写意图之后活头的前驱恒为 P）**在盘上有现成的判别子**：I-9.25 对每个走到的版本判它，意图那一版就红，不用等最后一批。

没复现的：探针深度 6 到不了模型里「两道拒绝都拿掉才分叉」那条 8 步的历史，所以拒绝拿掉的变异只在模型里验过，真代码里没验。


### 1.5　反推与推翻条件（B1）

反推——假设「等价论证不成立」，本该看到：(a) 某个写路径写得出的历史里有一条纪元 >= h 的待放候选（探针 ①、模型的 `pending_set_differs` 会非零）；(b) 真 checker 在合法历史上红（探针 ②）；(c) 两道拒绝只拿掉一道就分叉（模型的单开关变异）。(a)(b)(c) 都没有出现（(c) 的两道都拿掉才分叉，见 1.3 表）。

什么现象会推翻「B1 成立」：
- 探针或模型里出现一个写路径可达的历史，使 `entries_with_epoch_at_or_after_the_intent_txg` 或 `pending_set_differs` 非零；
- core 里出现第二处构造 deadlist 条目、或第三种写活头前驱的发布（`grep -rn 'DeadlistEntry {' crates/singlefs-core/src` 多一处、`transaction.rs:14124-14137` 的 `match` 多一支）；
- 两道拒绝（`transaction.rs:9450`、`:9527`）去掉任何一道**并且**第一版又放开了「销毁中的快照之外的另一个快照被销毁」的路径；
- 回退牵涉快照时不再拒（`crates/singlefs-core/src/mount.rs:9684`、`:9763`）。

不在这一轮射程、没量的：unlink（`deleted_inodes`）与截断的发布共用 `deadlist_changes_of_this_publish`（纪元同一处取），论证上不用另证，但探针与模型都没加这两类操作；写意图之后、续做之前的 unlink 步与销毁批的相对次序见 `transaction.rs` 的 `walk_the_deletion_queue_until_it_is_empty`，我只读了次序注释（先 `deleted_inodes` 再销毁中的快照再截断），没跑。

## B2　成立时的形态

**推荐：上界不上盘，判别子取「被判这一版自己的 checkpoint_txg」，写进 I-3.6 的豁免句，与 I-3.5 逆向共用同一个待放集谓词；不另立不变量编号，C145 的第二条判定并进这一句。** 这是我从 B1 推出的收严，没被攻过（零轮）。

### 2.1　规则怎么写

I-3.6 今天的豁免句：「落在未完成销毁意图 [lo, hi) 里、birth > P 的条目（写意图之后、它被放掉之前，前驱已传给下一侧）不判；」（`crates/singlefs-checker/invariants/05-空间记账.md:12`）。I-3.5 逆向里同一个待放集：「意图 (头, lo = S.txg, hi, P = prev(S).txg) 下纪元落在 [lo, hi) 里、birth > P 的条目」（同文件 `:11`）。两处的 hi 都没有定义到盘上可读的量。改成：

> 未完成销毁意图（码 14 条目带销毁中记号的快照 D）的待放集 = 头是活头、纪元 ∈ [D.S.txg, hi)、birth > D 的前驱的条目，hi = 链上 D 之后第一个活快照的 S.txg；D 之后没有活快照（下一侧是活头）时 hi = 被判这一版根记录里的 checkpoint_txg。

与今天 checker 的差别只有一处：下一侧是头时 hi 由 ∞ 改成本版 txg。对写路径写得出的状态没有差别（B1：纪元 < h <= 本版 txg，探针与模型都量过）；对纪元不小于本版 txg 的条目有差别——盘上不可能有这样的条目（条目纪元 < 杀它那次发布的 txg <= 含它的任何版本的 txg，同一张树表单元被几版共用时它们的 deadlist 相同，哪一版的 txg 都成立），所以它是一条白捡的「纪元不在未来」检查，C145 第二条「未完成意图的区间里没有纪元 ≥ 上界的条目」在下一侧是头时的可判形态就是它。

为什么不取别的量：
- **写意图那次的 txg h**：盘上没有。意图在码 14 条目的 flags 位 0，预留不写（`crates/singlefs-core/src/records.rs:563` 的 `TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED`，条款「不存续做点」）；回退候选集里第一个带销毁中记号的版本可能已被根环盖掉，不能当 h。
- **∞**：不错，但检查不到任何东西；本版 txg 白拿一条检查，代价是 `DeadlistAndSnapshotReachOfAVersion` 加一个字段（`walk.rs` 冻结副本 :16616 起的那个结构体）、`walk.rs` 冻结副本 :17065 起的 `deadlist_and_snapshot_reach_of_a_version` 多收一个参数（调用点已持有 `VersionToScanTheSnapshotsAndTheDeadlistOf.checkpoint_txg`，冻结副本 :16351-16356）、`entry_is_pending_release` 里 `upper_bound.is_none_or(|bound| entry.epoch < bound)`（冻结副本 :16696）改成 `entry.epoch < upper_bound.unwrap_or(facts.version_txg)`。
- 核心层放块时取的就是这一批自己的 txg（B1 表第三行），checker 取本版 txg 是同一个量的镜像。

### 2.2　实现今天的样子与要改的三处

1. `crates/singlefs-checker/src/walk.rs`：上面三处，补丁见模型目录 `probe/walk-version-bound.diff`（`diff -u`，对冻结副本的 walk.rs）。
2. GPU 判器内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `s_entry_is_pending_release`（冻结副本 :5908，工作区此刻 :5927）照 walk.rs 同名函数判，且对拍逐位比不豁免下标 65（`I-3.5` 一行里写的「`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表」），所以内核必须同步改；内核侧有没有现成的「本版 txg」可用我**没核**（没读那一段的输入布局），写「待核」。
3. kb：`05-空间记账.md` 的 I-3.5、I-3.6 两行里 hi 的定义；D5 已定项 12 第 291 行「下一侧是头时取写意图那次发布的 txg」与 core 的「这一批自己的 txg」（B1 的字面冲突）要选一个写法，推荐改成「hi 每批从派生链现取（下一条码 14 的诞生 txg，下一侧是头时取这一批自己的 txg），不存续做点」；改 D5 条款不在我的射程，只列。

### 2.3　改坏它时哪个必红世界会红

| 世界 | 构造 | 期望 | 弄坏开关 |
|---|---|---|---|
| 新：纪元到本版 txg 的待放条目 | 合成世界「销毁意图与最后一批之间」（`walk.rs` 测试模块 `world_between_the_destroy_intent_of_the_second_snapshot_and_its_last_batch`）再加一条纪元 = 本版 txg、单元只被 D 引用的条目 | I-3.5 逆向红，违例说明含「没有一个活快照引用它」 | 把 `facts.version_txg` 改回 ∞（旧写法）→ 这个世界转绿，用例红 |
| 旧：合法的待放态 | 同一个世界不加条目 | I-3.5 与 I-3.6 都成立 | 把上界收得过紧（例如取 lo）→ 这个世界转红（推的，没跑） |
| 旧：违例不被豁免 | birth 晚于活头前驱的块进了 deadlist（`I-3.6` 行 :12 里说的那个） | I-3.6 红 | 豁免句拿掉 → 红（推的，没跑；这条世界是原有的） |

说明：新用例我在草稿目录的仓副本里补了并跑过（结果见 2.4），没入仓；入仓是主 agent 与实现员的事。盘上的必红世界（真镜像里手写一条纪元不小于本版 txg 的 deadlist 条目）我没做：按 `checker_snapshot_space_accounting_must_red_worlds.rs` 文件头的写法要在单元区远处手写 deadlist 叶、I-3.1 一类跟着红，只能断言点名的那一条，工作量不小，列在「没做什么」。

### 2.4　补丁在真代码上的结果（草稿目录的仓副本，不进仓）
补丁：`probe/walk-version-bound.diff`（对冻结副本的 walk.rs，23 行加、1 行改；含结构体字段、两处传参、谓词一行，以及三处合成世界补字段和一条新单测）。命令与原样结果：

```
cargo test --offline -p singlefs-checker --lib -- deadlist destroy an_entry_whose_epoch_reaches
test walk::tests::an_entry_whose_epoch_reaches_the_version_txg_is_not_in_the_pending_release_set_of_a_destroy_whose_next_side_is_the_head ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 102 filtered out; finished in 0.00s
```

（存 `out/patched-unit-test.txt`；那 10 条含相邻的 `a_legal_deadlist_under_two_live_snapshots_holds_…`、`a_deadlist_entry_born_after_the_previous_snapshot_of_its_side_violates_the_tight_bound`、`a_deadlist_entry_pinning_a_unit_no_live_snapshot_references_violates_the_reverse_direction` 等原有用例，补丁后仍过。）

**控制（新用例会不会红）**：把补丁里的上界改回无穷（`upper_bound.is_none_or(|bound| entry.epoch < bound)`）再跑同一条，原样：

```
test walk::tests::an_entry_whose_epoch_reaches_the_version_txg_is_not_in_the_pending_release_set_of_a_destroy_whose_next_side_is_the_head ... FAILED
纪元不小于本版 txg 的条目不在待放集里：要判红，实际 Holds
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 111 filtered out;
```

（`out/control-unit-test.txt`。）即今天的写法（无穷）放过这条假条目，补丁后的写法抓到它。

**补丁后的真 checker 在全部历史上不红**：补丁后重跑 1.4 的探针（深度 6），七行 PROBE 与冻结副本原样 checker 的七行逐行相同（`diff out/probe.txt out/probe-patched.txt` 无输出），真 checker 红 0。

### 2.5　推翻条件

- 找到一个写路径写得出的历史，其中某一版有「头 = 活头、纪元 >= 本版 txg、birth > P」的条目——则补丁在合法镜像上红，推荐作废；探针 `entries_with_epoch_at_or_after_the_version_txg` 非 0 就是这个现象。
- 找到一条合法条目满足 纪元 >= 写意图那次的 txg h（探针 `entries_with_epoch_at_or_after_the_intent_txg` 非 0）——则 B1 的等价不成立，B2、B3 全部重判。
- 第一版放开回退（`crates/singlefs-core/src/mount.rs:9684`、`:9763` 的拒绝去掉）或放开建快照 / 第二个写意图（`transaction.rs:9450`、`:9527`）——B1 第 2、3 点前提变了，要重跑模型与探针。

## B3　不成立时要加什么（B1 成立，所以这是条件式答案，不是推荐）

**判定：B1 成立，格式不需要动；下面写若有人推翻 B1 要付的代价，与不改格式的替代判法。**

### 3.1　盘上最少加什么

意图里存 hi：一个 txg，8 字节。落点只能是被销毁快照自己的码 14 树表条目（条款：意图写在那里，「只占 flags 位 0（销毁中），预留一个字节不写」，`.claude/kb/decisions/05-快照-空间记账机制.md@已定项 12：deadlist 条目的形态`）。今天那条 200 字节条目的尾 76 字节是预留，读者对非零一律拒收：「|| bytes[bytes.len() - 76..].iter().any(|byte| *byte != 0)」（`crates/singlefs-core/src/records.rs:738`）；flags 只认位 0，且只在码 14 上（`records.rs:563` 的常量与 `parse_with_destruction_mark`）。所以要改的是：`TreeTableEntry` 字段表在码 14 上多一个 8 字节字段（占预留 76 的前 8 字节，剩 68），`parse_with_destruction_mark` 对码 14 放行这 8 字节、别的种类仍拒；写意图那次发布把 `txg`（`transaction.rs:9515` 的 `publish_snapshot_destruction_intent` 里 `txg` 就是那次发布的号）写进去；`deadlist_entries_the_destruction_of_a_snapshot_releases` 的 hi 改读它（`transaction.rs:6182`）。条款里 hi 的定义不用改，反倒是第一版段里「下一侧是头时取写意图那次发布的 txg」这半句要改。

### 3.2　格式冻结与 layout 的影响

- 现在：「在第一个外部用户出现之前，磁盘格式是软的」（`.claude/rules/format-evolution.md:7`）。软格式期里加字段没有兼容成本，但改格式必须同步 `invariants/` 与 checker（同文件「硬约束」）；`layout/01-first-txn.md` 的树表条目行要改预留的写法（该行见 `.claude/kb/layout/01-first-txn.md:391`，「预留 76」）；D5 已定项 12 的「预留 76 字节留给别的认购者」要减账（认购表另有 C338，我没读它的正文）。
- 冻结之后：D15 判定表的第一行「新增字段在预留区 / TLV 尾部 | 忽略，继续正常工作 | compat | 否」（`.claude/kb/decisions/15-格式冻结政策.md:189`）不适用这一处——今天的读者对预留区非零是**拒收**不是忽略（上面 `records.rs:738`），按同一张表的「改变已有字段含义」一行（旧读者对非 0 位一律拒收的例子）算 incompat、要 bump。树表单元落哪一层「还没有条款」（`.claude/kb/decisions/15-格式冻结政策.md:159`，C246），所以连这个字段落进哪个组件的 `spec_hash` 都没有定；`spec_hash` 恒对 kb 字段表的完整投影求（同文件 D15 已定项 3），字段表一改 hash 就变。
- 收益几乎为零：B1 说明 hi = h 与 hi = 本版 txg 在合法镜像上没有差别，存 h 只多拦「纪元落在 [h, 本版 txg)」的条目，写路径写不出这样的条目，而 B2 的本版 txg 上界已经拦住 >= 本版 txg 的。

### 3.3　不改格式的替代判法（都是零轮）

1. **本版 txg 上界**（B2 推荐）：零字节，今天的读者就够，补丁已在真代码上跑过。
2. **已有的 I-9.25（不用新写）**：「写意图之后到最后一批之间活头的前驱恒为 P」是 B1 的承重前提，I-9.25 在每个版本上把它判死（`crates/singlefs-checker/invariants/09-inode树结构.md:37`）。真代码上的控制：拿掉 `transaction.rs:14131` 那一支，探针里真 checker 在意图那一版红 I-9.25（`out/control-core-mutation.txt`，47 次）。有它在，通过 I-9.25 的镜像里写意图之后新杀的单元纪元只能是 P（< lo），hi 在盘上与否都不改变这一点。模型里另写过一个跨版本判（相邻两版新出现的条目纪元必须 < lo，`out/` 里 `cross_version_new_entry_reaches_lo`），它是 I-9.25 的冗余，不推荐入仓。
3. **从根环里找写意图那一版取 h**：不推荐。h 只存在于第一个带销毁中记号的版本；该版若已被根环盖掉就没有，不是可靠的判别子，且读起来依赖根环深度（`crates/singlefs-checker/invariants/05-空间记账.md:11` 里「回退候选集」的口径）。

### 3.4　推翻条件

B1 的反例一出现（见 B1 与 2.5），3.1 就成为必选项，且这时「存 h」比「存 hi 的上界之一」更合适：条款本来就是「写意图那个开放 checkpoint 的号」。


## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| B1 等价论证 | 一致（成立） | 下一侧是头时，∞、写意图那次的 txg、放块那一批自己的 txg 三种读法在写路径可达的状态上逐条相同；论证靠「纪元只在一处取、活头前驱只在建快照与写意图两处变、建快照与第二个写意图被拒」；模型穷举 5957 个状态、真写路径探针 深度 6 的 535 个历史 全零差；两道拒绝都拿掉才分叉 |
| B1 条款与代码 | 冲突（字面，无行为后果） | D5 已定项 12 写 hi 取写意图那次的 txg，core 取每一批自己的 txg，checker 取无穷；三处各一种 |
| B1 注释论证 | 冲突（承重点不同） | `transaction.rs:6165-6166` 说等价靠「发布都不杀用户可见单元」，实际承重点是纪元只取活头前驱且意图把它传成 P；`walk.rs` 那句注释只证了写意图之后的半边 |
| B2 形态 | 推荐（零轮） | 并进 I-3.6 豁免句与 I-3.5 逆向共用的待放集谓词，下一侧是头时 hi 取被判这一版的 checkpoint_txg；零字节；补丁在真代码上 新单测过、改回无穷即红、补丁后探针不红 |
| B2 内核同步 | 待核 | GPU 判器内核的同名函数要同改，内核侧有没有本版 txg 可用我没核 |
| B3 要加什么 | 不需要（B1 成立）；条件式答案已给 | 若被推翻：码 14 条目预留区里存 8 字节 h，读者要改、冻结后是 incompat |
| B3 替代判法 | 有 | 本版 txg 上界（零轮，补丁已在真代码上跑过）；「写意图之后活头前驱恒为 P」这个承重前提已由 I-9.25 在每一版判死（真代码控制：拿掉传前驱那一支，意图那一版就红） |

## 没做什么

- 不判别的腿的格、不替主 agent 采纳或出判决；不碰攻方腿本轮的产出（没读）。
- 没改 `crates/`、kb、不变量清单的任何一行：补丁与探针只在草稿目录的仓副本里，模型目录里存的是补丁文件与探针源。
- 没跑重型测试、门禁、checker 档（`singlefs-checker-tier`）；没跑崩溃点重放、广谱变异、GPU。
- 没在真镜像上做 B2 的必红世界（要手写 deadlist 叶，见 2.3）；GPU 判器内核没读全，同步改法待核。
- 探针没覆盖：unlink / 截断 / 整理的发布、可写挂载之后再发布（挂载续做放在历史末尾）、第二台。
- 没跑 `d5-item12-r3` 的 BFS 模型（`hi-is-latest-published` 变异在我的模型里不红，与那份判决的 5 步漏放不是同一台模型，没对账）。
- 草稿目录里的仓副本与编译目录交回前删除（大小见交回）。
