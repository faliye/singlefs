# 根环实现第 8 批代码轮第二轮：云端正推腿报告（Sonnet，2026-10-03）

轮名 m3-root-ring-code-r2。立场：从条款原文推到冻结副本里的行，再推到钉它的测试与变异行（正文第四节分工表）。格：K1、K3、K4、K5、K6。K2 不归我。

## 零、读法与约定

- 读的是冻结副本 `/tmp/claude-1000/m3-root-ring-code-r2/tree/`。主工作区此刻与冻结副本不同的文件（`cmp` 实测）：walk.rs、image.rs、transaction.rs（第 5842 行起）、crash_judge_checker.wgsl（第 1463 行起）、mutations.tsv（第 2266 行之后追加）、writable_mount_of_a_formatted_pool.rs。这几份文件的引用写成「冻结副本 文件 第 N 行」，不用引号加位置的写法，免得 cite-check 拿主工作区去对时错位；其余 `crates/` 文件两边逐字节相同，行号两边通用。
- kb 条款一律写 kb 文件自己的行号（现查，`grep -nF` 命中）。
- 我在自己的副本 `/tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/tree/`（冻结副本的原样拷贝）上跑 harness 档：只用 `cargo test -p singlefs-harness`、`-p singlefs-core --lib`、`prove-red.sh`，全部经 `capped.sh 15` 与内存包装（上限 6G 或 8G）；checker 档（`singlefs-checker-tier`）一条没跑。改源码的探针都在副本里，每次跑完 `cmp` 与冻结副本逐字节相同再往下（探针的旧串、新串在 `research/prompts/m3-root-ring-code-r2-sonnet-model/probes/`）。
- 一个我自己踩过的坑，写在这里免得别人踩：还原源码用带原 mtime 的拷贝时，cargo 认为产物是新的，下一次「基线」其实跑的是上一条探针的产物。我发现后改成还原即 `touch`，并把受影响的几次（F 探针、K1 探针）在干净构建上重跑了一遍，结论没变；报告里引的数都来自重跑之后的日志。
- 判定三档：一致、冲突、规则没说；打中类的线索另标「线索」，由主 agent 现查坐实。
- 方案按 `crates/` 今天的实现来谈。

## 一、K1　E2、B2、RR4 落地

### 1.1 三句写回条款与代码逐字对照

| 条款句（D16 已定项 1） | 代码 | 判 |
|---|---|---|
| 「进程内记根时，根环表里这一项成了窗口里最小的那个，先重读那一槽，读得出且自证过就照读出来的根算，仍读不出才按记的最大 txg 算（用户 2026-10-03 定，零轮）」（`.claude/kb/decisions/16-发布语义.md:35`） | 「oldest < highest_txg_it_can_hold」（`crates/singlefs-core/src/allocator.rs:1151`，`if oldest_valid_root.is_some_and(...)` 那一行）：只在读不出的槽是窗口里最小的那个时才重读；重读入口 `record_root_written_by_this_process_rereading_slots_unreadable_at_mount`（allocator.rs:1860）在记根之前调；读盘一侧 mount.rs:1277 | 一致 |
| 「含抬 F 那一遍，根环表里挂载时读不出的那几项同样按上一句并进最旧有效根」（`.claude/kb/decisions/16-发布语义.md:35`） | 抬 F 那一遍取最旧有效根调 `oldest_valid_root_in_the_window_for_raising_the_floor`（冻结 mount.rs 第 2865 行，定义第 2697 行），门槛是 `reclaim_floor(new_floor, oldest_valid_root_in_the_window)`（冻结 mount.rs 第 2898 行） | 一致 |
| 「都只在一圈窗口里的根上取（用户 2026-10-03 定，零轮：窗口外的旧根，窗口已许可它的树表被复用，读不出不再拒抬 F）」（`.claude/kb/decisions/16-发布语义.md:37`） | 「is_inside_the_window(root.checkpoint_txg))」（`crates/singlefs-core/src/mount.rs:2160`）与 「let previous_candidates: &[RootRecord] = &valid;」（`crates/singlefs-core/src/mount.rs:2200`）：有效根、非空、「前一条」三处都只在 `valid`（按表有效 ∧ ≥F ∧ 在窗口里）上取 | 一致 |

### 1.2 重读那一刻四种结局各记成什么

重读函数 `reread_of_a_root_ring_slot_unreadable_at_mount`（mount.rs:1277）分四类，表的改法在 allocator.rs:1894-1910（`record_root_written_by_this_process_rereading_slots_unreadable_at_mount` 里的 `for` 循环）：

| 结局 | 代码分类 | 根环表上记成 | 条款有没有点名 |
|---|---|---|---|
| 读得出、自证过、按表有效 | 「RootRingSlotReading::SelfVerified(root) if !abandoned_by_table(&root, table) => {」（`crates/singlefs-core/src/mount.rs:1290`）→ `ValidRoot{读出的 txg}` | 换成 `ValidRootReadWhenTheSlotUnreadableAtMountWasReread{读出的 txg}`（allocator.rs:1901），不算知道住着根的槽（`crates/singlefs-core/src/allocator.rs:1027`） | 点名（「照读出来的根算」） |
| 读得出而自证不过 | 分到 `NoValidRoot`（mount.rs:1297） | 从表里拿掉（「occupancy.occupants.remove(&ring_slot);」，`crates/singlefs-core/src/allocator.rs:1907`） | **条款没点名**，实现取「当没有根」，与挂载那一刻同一处置（D16 已定项 1：「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根」，`.claude/kb/decisions/16-发布语义.md:38`）：规则没说 |
| 读得出而按表被抛弃 | 同上，`NoValidRoot` | 同上拿掉；不按被抛弃根记（它引用的落点挂载时没隔离，离开根环时也就不清） | 规则没说（窗口定义里「按实例表判仍然有效」排除它，间接推得） |
| 仍读不出 | 「RereadOfARootRingSlotUnreadableAtMount::StillUnreadable」（`crates/singlefs-core/src/mount.rs:1300`） | 不改表，门槛取该槽能住的最大 txg（allocator.rs:1164 起返回 `Some(highest_txg_it_can_hold)`） | 点名（「仍读不出才按记的最大 txg 算」） |

### 1.3 有没有一段合法历史让记下的 txg 高于真住在那一槽的根

条款原话：「挂载时读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的（零轮，只在攻方模型上量过）」（`.claude/kb/decisions/16-发布语义.md:35`）。

推导：挂载时记的最大 txg h 是该槽位在 (N − 3S, first_txg − 1] 里的最大 txg（`unreadable_ring_slots_inside_the_window`，mount.rs:1676）；进程内的窗口 txg > M − 3S，锚 M 从 first_txg − 1 起只增不减（allocator.rs:1822 `occupancy.latest_recorded_txg = checkpoint_txg;`）。同槽位的 txg 相差 3S 的整数倍，所以窗口里同槽位、txg ≤ first_txg − 1 的 txg 至多一个，且它就是 h；比 h 小的那个（h − 3S）一定已在窗口外。这个进程写到那一槽时，`occupants.insert` 把那一项换成这条根（allocator.rs:1826），重读那一遍也先把这条根自己的槽从候选里去掉（allocator.rs:1886）。结论：重读之前记的 h 要么等于真住在那里的窗口内的根，要么那里住的根在窗口外（记的是占位，只会让门槛更低、记账多，不会少，这正是第一轮 K2-e 那一形，E2 就是为它的）。

逐点枚举核这条引理（不是抽样；S = 1..8、N、first_txg、M、槽位全扫）：

```text
$ cd research/prompts/m3-root-ring-code-r2-sonnet-model && nice -n 19 python3 e2_window_lemma.py
checked=17399790 violations=0
```

- 引理的前提：该槽里真住的根 txg ≤ first_txg − 1。first_txg 取环里读得出的根与 journal 里见过的最大 txg 加一；一条根与它的记录都读不出的「整条命都读不出」那一形不在引理内（第一轮判决说另记、第二轮攻）。
- 什么会推翻：任一历史里，挂载时读不出的槽住着 txg > first_txg − 1 的有效根；或进程内窗口锚能回退。

### 1.4 E2 判「按实例表有效」用的那张表，写行之前与之后

- 写行之前（挂载时）：`allocator.note_the_instance_table_judging_the_rereads_of_root_ring_slots(newest_table.clone());`（`crates/singlefs-core/src/mount.rs:1913`），`newest_table` 取自最新那条根（mount.rs:1769）。
- 写行之后：`establish_instance` 开头（mount.rs:4243）取到要写的行之后、第一次发布之前换成写行那一张（mount.rs:4291）。这一步与第一次记根之间只有分配器拷贝上的预演（`copy.record_root_written_by_this_process(...)`，mount.rs:3350、3406，不重读），没有真记根，所以换表那一刻没有窗口最小的槽被按旧表判。
- 与池级 checker 的口径：checker 按镜像里最新那条根的实例表判，写行之后最新那条根带的就是写行那一张；两者一致（推的）。
- 钉没钉：没钉。我在副本里把重读分类的「按表有效」判断拿掉（`probes/p2`）、把「读得出且有效」整条改成当没有（`probes/p1`）、把读出的 txg 加 1000（`probes/p1b`）、把「自证不过」改成仍读不出（`probes/p3`），harness 档 17 个相关二进制与 core 的 180 条单测全绿，见 1.7。

### 1.5 B2：F 能不能越过窗口里一条读不出的有效根（8-1 报告第五节第 1 点那一形）

条款字面：算准入抬 F 的上限时「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根」（`.claude/kb/decisions/16-发布语义.md:38`），上限只取窗口里的根（`.claude/kb/decisions/16-发布语义.md:37`）。实现：算上限那一遍 `rollback_floor_ceiling`（mount.rs:2129）只在读得出的根上取，读不出的 X 当没有；「这个进程知道住着根的槽」读坏才重读一次、仍坏拒。X 属于挂载时读不出、之后没写过的那一类，不在其内（`ring_slots_known_to_hold_a_root_by` 在 mount.rs:2460 定义，抬 F 那一遍把它传给算上限那一遍）。⇒ **条款字面下这条是对的**。

我在副本里把攻方 `rfo3-17-persist` 那份盘面（洞 18、19、20，X = 21 一直读不出，推到锚 29）上的准入抬 F 改成走「抬到上限」入口，X 读不出与读得出各一次（探针 `tests/probe_k1_raise_to_the_ceiling_over_an_unreadable_x.rs`，日志 `logs/probe-k1-raise-over-x.log`），原样输出：

```text
test probe_admission_raise_to_the_ceiling_over_an_unreadable_x ... PROBE_RAISED floor_before=CheckpointTxg(0) anchor_txg=31 => Raised ceiling=CheckpointTxg(22)
PROBE_ROOT floor_after=CheckpointTxg(22)
PROBE_VIOLATIONS count=1
PROBE_VIOLATION I-7.9：实例 6 txg 30 那条根把回退下界 F 从 0 抬到 22，高于它之前的根算出的抬 F 上限 8：每块盘上最新的有效根 {0: 29, 1: 28}，非空有效根 [8, 21]，空不空判不了的有效根 []，最旧有效根 txg 8
test probe_admission_raise_to_the_ceiling_with_x_readable ... PROBEREADABLE_RAISED floor_before=CheckpointTxg(0) anchor_txg=31 => Raised ceiling=CheckpointTxg(21)
PROBEREADABLE_ROOT floor_after=CheckpointTxg(21)
```

- 两次都只红 I-7.9（池级 checker 的 I-7.9 还取全环，已知没做、进第 9 批）；I-3.1、I-3.11、I-2.1、I-7.4、I-4.8 都不红。X 读不出时 F = 22，读得出时 F = 21：差的正是 X 那一条根。
- **线索（推的，I-7.9 还没跟上窗口，量不了）**：I-7.9 的「非空有效根」已列出 [8, 21]；窗口 txg > 29 − 12 = 17 里只剩 21 一条非空根（22..29 是空发布，同一个状态），去重后不足 4 个状态，上限取窗口里最旧的有效根 21。第 9 批把 I-7.9 改成只取窗口之后，X 读不出那一格 F = 22 > 21 判红，X 读得出那一格 F = 21 成立。池级 checker 读的是镜像，挂载那一刻的读错不留在盘上（同第一轮 K2-e 的「判别子观测不到」）。条款「根槽这一次读坏」一行与 I-7.9 在这一段历史上合不起来，哪边让要交用户，我不替判。
- 什么会推翻：X 读不出的那一格上 I-7.9 改完之后仍成立（比如第 9 批把 I-7.9 的「有效根」也按「挂载读坏的槽当没有」读，镜像上做不到）；或条款「根槽这一次读坏」一行改成「不止拒知道住着根的槽」。

另一条没写在条款里的边：E2 重读读出了有效根（`ValidRootReadWhen…`）之后，这个进程已经知道那一槽住着根，但它不在知道住着根的槽里（`crates/singlefs-core/src/allocator.rs:1027`），之后那一槽再读坏，上限那一遍照旧当没有根。条款「根槽这一次读坏」一行列的是「挂载那一刻读得出」与「这个进程写过」两种，重读读出来的不在其内：规则没说。

### 1.6 RR4 用例只抬到 6，「抬到 B2 上限」那一形有没有别的钉

- 钉：`admission_raising_the_floor_counts_the_root_ring_slot_unreadable_since_the_mount_into_the_reclaim_threshold`（`crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs:203`），F 取 `FLOOR_RAISED_BELOW_X` = 6（同文件第 43 行）。变异行 2235（RR4）与 2234（B2）各自抓到，见 1.7。
- 「抬到上限」那一形：没有别的用例，也没有别的变异行。我 grep 了所有同时用 `raise_rollback_floor_to_the_admission_ceiling` / `push_one_floor_raise_within_the_admission_budget` 与读故障设备的 harness 用例（`an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`、`admission_raises_the_floor_before_refusing.rs`、`entries_after_mount_refuse_swapped_or_behind_devices.rs` 与上面那份），没有一条在「挂载时读不出的槽 + 抬到上限」上断言记账。
- 推得（代数，不是实测）：RR4 并进来的占位只会把「窗口里最旧有效根」往下拉；门槛是 `max(F, 最旧有效根)`（「effective_floor.max(oldest)」，`crates/singlefs-core/src/mount.rs:1340`）。抬到上限时 F 取 `min(每块盘最新有效根, 第 4 新非空根或最旧有效根)`（mount.rs:2266），它本身就是窗口里一条读得出的有效根的 txg，所以读得出的最旧有效根 ≤ F，门槛恒等于 F，占位多不多算都一样；卸载那一格 F = 现行那一版的 txg，同理。⇒ RR4 的补漏只对「F 低于窗口里读得出的最旧有效根」的显式抬 F（公开入口 `raise_rollback_floor`，随机历史走它）有区分力，准入推空与卸载两个入口不受 RR4 影响。8-1 报告第五节第 1 点第一趟用例没红的原因就是这个。
- 什么会推翻：找到一段准入抬到上限、门槛因占位而变的历史（我的代数把 F 当成窗口里读得出的有效根的 txg；若上限里有一支取到别的数，这条就破了）。

### 1.7 钉它的测试与变异行：现跑 prove-red 与我自己的探针

K1 相关变异行（`crates/mutations.tsv` 第 2229–2236、2238 行，冻结副本 `prove-red.sh --copy` 现跑）：

```text
$ bash research/scripts/capped.sh 15 bash research/scripts/prove-red.sh --copy <副本> --memory 8G singlefs-harness <2229 2231 2234 2235 2236 2238 行的名字>
✓ 点名 6 条：跑了 6 条，跳过 0 条，跑的都抓到了
$ ……  singlefs-core <2230 2232 2233 行的名字>
✓ 点名 3 条：跑了 3 条，跳过 0 条，跑的都抓到了
```

（日志 `logs/prove-k1-harness.out`、`logs/prove-k1-core.out`；每行抓到的用例名在日志里。）这 9 行各自红的是它名字说的行为。下面是它们**没钉到**的几处，都是我在副本里改一处、跑相关二进制，全绿：

| 探针（`probes/` 里的旧串新串） | 改了什么（mount.rs / transaction.rs 冻结行） | 跑的 | 结果 |
|---|---|---|---|
| p1 | 重读分类里「读得出且有效」永远不成立（`SelfVerified(root) if false && …`，mount.rs:1290）：有效根一律当没有 | harness 17 个二进制（`scripts/e2-targets.txt`）+ `-p singlefs-core --lib` | 17 个 `test result: ok`，core 180 过（`logs/p1-survives-harness17.out`、`logs/p1-survives-core-lib.out`） |
| p1b | 重读读出的 txg 加 1000 | E2 用例 + 抬 F 用例 | 全绿（`logs/p1b-survives.out`） |
| p2 | 「按表有效」判断拿掉（`… \|\| true`，mount.rs:1290） | 4 个相关二进制 | 全绿（`logs/p2-survives.out`） |
| p3 | 「自证不过」分到「仍读不出」 | 同上 | 全绿（`logs/p3-survives.out`） |
| q1 / q2 / q3 | 记根调用点三处（mount.rs:3257、4491，transaction.rs:1757）从「先重读再记」换成不重读的 `record_root_written_by_this_process` | 7 个相关二进制 | 全绿（`logs/q3-q1-survive.out`、`logs/q2-survives.out`） |
| p4（对照，确认探针灵敏） | 重读「仍读不出」分到「当没有根」 | 抬 F 用例 | 红：`admission_raising_the_floor_counts_the_root_ring_slot_unreadable_since_the_mount_into_the_reclaim_threshold ... FAILED`（`logs/p4-red.out`） |

七个二进制每次都是 8 过、21 过 1 忽略、13 过、1 过、2 过、1 过、15 过，0 红。

意思：E2 的四种结局里，只有「仍读不出」（RR4 用例，`Every` 读错）与「读得出但在窗口外」（E2 用例读出 txg 0 的根，与「当没有根」在这条用例里不可分）被钉到；「读得出且有效且在窗口里」「读得出而自证不过」「读得出而被抛弃」三种结局、「按哪张表判有效」、以及四处记根调用点里除 `publish_admitted`（变异行 477 管它）以外的三处，**没有测试也没有变异行**。这不是实现错，是钉不住；**线索**，改法是补用例（要的盘面：挂载时读不出的槽正好是窗口里最旧的那一槽，即进程要写的下一个 txg 的槽，在写行那次发布就会被重读，见 1.8 的推导）。

### 1.8 E2 在哪一次记根上第一次触发，与 K1 判定

进程窗口在记 txg M 这条根时是 txg > M − 3S。挂载后第一次记根是写行那次发布（M = first_txg），窗口里最旧的 txg 是 first_txg − 3S + 1，它的槽位 ≡ first_txg + 1（模 3S），就是下一个 txg 要写的那一槽。环里最旧的那一槽挂载时读不出，重读就发生在写行那次发布（或随后的暖机），不是 E2 用例里的第 47 次发布。写行那次发布按上一版是什么走三条路：带文件的经 `publish_version` 走 `publish_admitted`（变异行 477 管它）；无文件而要写行的走 `prepare_the_row_publish_on_a_version_without_file`（transaction.rs:1757）；无文件而不写行（第一次可写挂载）的零单元发布走 mount.rs:4491；暖机与会话里的零单元空发布走 mount.rs:3257。后三处没有任何钉（上表 q1–q3）。**线索，不是错**：改法是补一条「挂载时读不出的槽正好是窗口最旧的槽、里面住的是上一圈的根或自证不过的内容」的用例，并给三处各补一行变异。

| K1 子问 | 判 | 什么会推翻 |
|---|---|---|
| 三句条款与代码 | 一致（「读得出而自证不过」「被抛弃」两种结局条款没点名：规则没说，实现取当没有根） | 条款改成点名别的处置 |
| 记下的 txg 高于真住的根（不安全方向） | 没找到：引理枚举 17399790 点 0 违例，前提写明 | 挂载时读不出的槽住着 txg > first_txg − 1 的有效根 |
| 换表那一刻窗口最小的槽判得对吗 | 一致（推的）；没有钉 | 找到换表前后判有效结果不同、且发生在记根之间的历史 |
| F 越过窗口里读不出的有效根 X | 条款字面一致；与 I-7.9 按镜像判的口径合不起来（推的，线索） | 见 1.5 |
| RR4 与 B2 的钉 | B2、RR4（F = 6）各有用例加变异行，现跑都红；「抬到上限」形无用例也无变异行，且推得 RR4 对该形无区分力 | 见 1.6 |

## 二、K3　I-3.5 出现规则

条款是 `.claude/kb/invariants.md` I-3.5 那一行（第 146 行）；定下的 TF、β、gap、gap-row、N3 在 `research/prompts/m3-birth-floor-r2-main-verification.md` 第三节与 `research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 6 行。代码是冻结副本 walk.rs（行号都是冻结副本的）。

### 2.1 逐要素对照

| 要素 | 条款原文（invariants.md:146） | 代码（冻结 walk.rs） | 用例（`checker_judges_reference_intervals_and_every_new_root_segment_field.rs`）与变异行（mutations.tsv） | 判 |
|---|---|---|---|---|
| 候选集、升序、同 txg 并组、逐对 | 「取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判」（`.claude/kb/invariants.md:146`） | 候选筛：第 7539 行 `let walked =`（最新根 ∨ 非抛弃 ∧ ≥F ∧ 在窗口里）；记录施加版：第 6508 行 `versions_applied_only_by_records`；分组：第 518–546 行（`judge_the_appearance_rule_of_adjacent_candidate_versions` 从第 505 行起） | 候选组成无专门用例；同 txg 并组：**无用例、无变异行**（同 txg 两条合法候选根造不出） | 一致；并组无钉 |
| (a) 诞生代号 > R⁻.txg | 「满足 (a) birth(b) > R⁻.txg」（`.claude/kb/invariants.md:146`） | 第 581 行 `let born_after_the_previous_version = birth_txg > previous_txg;` | 第 440 行用例 | 2250 现跑红 |
| (b) TF：树根出生 txg 严格小于 R⁻.txg、子树里诞生代号都不大于它 | 「那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）」（`.claude/kb/invariants.md:146`） | 第 558 行 `root.root_pointer_birth_txg < previous_txg`、第 564 行 `.all(\|birth_txg\| birth_txg <= root.root_pointer_birth_txg)`；根非全零且种类在 extent、inode、`deleted_inodes` 里（种类常量第 380 行，`tree_table_entry_of_the_walk` 第 409 行） | 第 471、493、521 行用例 | 2251、2252、2253 现跑红（2252 实现员没单独证，我现跑抓到） |
| β 再次出现也判 | 「R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判」（`.claude/kb/invariants.md:146`） | 第 569 行：只跳过 R⁻ 组引用的单元 | 第 555 行用例 | 2254 现跑红 |
| 头读不出不判 | 「b 自己的头读不出时这一个落点不判」（`.claude/kb/invariants.md:146`） | 第 577 行 `let Some(birth_txg) = birth_txg_of(unit) else {` | 无专门用例；`instance_table_chain_read.rs` 的 `chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant` 间接钉（我改成读不出当 0，它红，`logs/headunreadable-bh-red.out`） | 一致；有间接钉 |
| gap：txg 不相邻 | 「R.txg ≠ R⁻.txg + 1 时这一对之间有洞」（`.claude/kb/invariants.md:146`） | 第 462 行 `if previous_txg.checked_add(1) == Some(version_txg) {` | 第 612、654 行用例 | 2255、2256 现跑红 |
| gap-row | 「最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）」（`.claude/kb/invariants.md:146`） | 第 455–497 行 `hole_in_the_candidate_set_between`：R 的实例大于 R⁻ 的（第 466 行）、有行 (R⁻ 实例, T = R⁻.txg)（第 471 行）、中间每个实例都有一行（第 479 行）、行都是 (0, 0)（第 482 行） | 第 682 行用例（正例） | 2257 现跑红；**四个子条件里只有「R 的实例大于 R⁻ 的」被钉，其余三个无钉**（见 2.2） |
| N3 | 「不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 birth(b) > R⁻.txg」（`.claude/kb/invariants.md:146`） | 第 572 行：洞接不上且单元在用户可见树里才 `continue`；别的单元走 (a)；因为 (b) 只对有标签的单元成立，洞接不上时非用户可见单元只剩 (a)，与条款相同 | 第 612、654 行用例 | 2255、2256 现跑红 |
| 不适用 | 「R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由」（`.claude/kb/invariants.md:146`） | 第 532、544 行跳过引用不全的一对；第 597 行 `if units_judged == 0 {` 整条不适用 | 整条不适用：`writable_mount_of_a_formatted_pool.rs` 各步（见 K6）；**引用不全的一对跳过：无用例、无变异行** | 一致；跳过无钉 |
| 走读修正一：谱系树、墓碑树容器的落点记在根节点下面 | （走读，不是条款句） | 第 2000 行 `walk_container_tree_root` | `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（`checker_known_bad_images.rs`） | 2260 现跑红 |
| 走读修正二：树表共享时映射根照样并进 | 同上 | 第 1409 行 `None if tree_table_walked_by_an_earlier_version => {` 之后的映射根那一段 | 第 773 行用例 | 2259 现跑红 |

注：表里「第 N 行」指冻结副本 walk.rs。「现跑」= 我在冻结副本拷贝上 `prove-red.sh` 跑，日志 `logs/prove-k3k4-harness.out`，末行：

```text
✓ 点名 17 条：跑了 17 条，跳过 0 条，跑的都抓到了
```

### 2.2 gap-row 四个子条件与「引用不全跳过」没有钉：我的探针

在副本里每次改 walk.rs 一处，跑 `checker_judges_reference_intervals_and_every_new_root_segment_field`、`checker_known_bad_images`（后两条探针另加 `checker_narrow_invariants_and_abandoned_roots`）：

| 探针 | 改了什么 | 结果 | 日志 |
|---|---|---|---|
| ba | 去掉「R 的实例大于 R⁻ 的」（第 466 行） | 红 2 条（第 612、654 行用例） | `logs/gaprow-ba-red.out` |
| bb | 去掉「有行 (R⁻ 实例, T = R⁻.txg)」整个条件 | 15 过 + 69 过，0 红 | `logs/gaprow-bb-survives.out` |
| bc | 去掉「中间每个实例都有一行」 | 同上 0 红 | `logs/gaprow-bc-survives.out` |
| bd | 去掉「中间的行都是 (0, 0)」 | 同上 0 红 | `logs/gaprow-bd-survives.out` |
| be | 只去掉「T = R⁻.txg」（行的实例对就行） | 同上 0 红 | `logs/gaprow-be-survives.out` |
| bf | 前一版引用不全不再跳过这一对 | 15 + 69 + 20 过，0 红 | `logs/pairskip-bf-survives.out` |
| bg | 后一版引用不全不再跳过这一对 | 同上 0 红 | `logs/pairskip-bg-survives.out` |

意思：gap-row 在「实例表行说接得上」这一侧只有一条正例用例（相邻实例、有行、T 对），没有「T 对不上」「中间缺行」「中间行不是 (0, 0)」的反例，所以 8-2 自取读法 ③ 与条款里「T = R⁻.txg」那半句，改松了没有红。这是钉的缺口，不是判法错：这几处放松之后只会多判（把本该 N3 放过的单元也照 (a) 判），在我造的合法历史上都不红（见 2.4）。**线索**。什么会推翻：补上三条反例用例后这几条探针转红，缺口就闭了；或找到一段合法历史上放松之后会红，那条件就是承重的。

### 2.3 8-2 自取的读法（报告第六节 S4）各自在哪段合法历史上误红、在哪段坏镜像上漏红

| 读法 | 合法历史上误红？ | 坏镜像上漏红？ | 钉 |
|---|---|---|---|
| ① I-3.5 只判出现规则，③ ④（deadlist）没实现 | 没判，谈不上；注意 kb 行状态列还写着「未实现（实现进下一批）」（`.claude/kb/invariants.md:146`），代码里 `IMPLEMENTED_INVARIANTS` 已有它，写回批要改 | ③ ④ 那一整类（deadlist 归属区间错、活头引用的块出现在 deadlist 里）判不出，已知没做 | — |
| ② R⁻ 侧几条并成一组时 gap-row 取组里任一实例接得上 | 同 txg 两条合法候选根造不出（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 8 行引 `recovery.rs:1289`），走不到 | 没有：任一接得上就照常判，是更严的一侧 | 无（2.1 表第一行） |
| ③ 中间每个实例都要有一行、都是 (0, 0)，缺行算接不上 | 没找到。写行那一步 `instance_rows_to_write`（`crates/singlefs-core/src/mount.rs:3512`）给 [max(上一个实例, 1), 要取的号) 每个实例各一行：上一个实例 (i, T, W)，中间 (j, 0, 0)；表逐次累加，最新根的表里中间实例都有行（推的）。表链读不全时行不全，判成接不上，只会少判 | 缺行 → 接不上 → 用户可见树里的单元不判，就是条款对「接不上」的处置，不算漏 | 无（bb–bd） |
| ④ 引用不全的一对不判，一对都判不到才整条不适用 | 没找到 | 引用不全的那一对不判，与条款「整对报不适用」同义 | 无（bf、bg） |
| ⑤（我补的）「引用集合数不全」怎么数：这一版走读期间 `walk_failures` 增加了几条 | 冻结 walk.rs 第 1002 行 `references_are_complete: self.walk_failures.len() == walk_failures_before,`。共享子树第一次走时的失败记在先走的那一版，后走的版本经 visited 表并进、不重记，于是后走的版本被当成「引用完整」而其实经共享子树少了一段。它与先走的那一版少的是同一段，同一对里两边一起少；两边不一起少要一个版本走另一条路径到同一批单元，我没造出，也没找到合法历史（推的） | 同左 | 无 |

条款对「数不全」只写了「走读读不出，或走到没有条款的形状」（`.claude/kb/invariants.md:146`），没说怎么数：规则没说。

### 2.4 `hole_in_the_candidate_set_between` 三态，对 8-3 报告第十节那张「窗口里每个 txg 住着什么」的表

三态函数的输入只有 (R⁻.txg、R⁻ 的实例集、R.txg、R 的实例、最新根的实例表行)，不读根槽（冻结 walk.rs 第 455 行起），所以那张表里「槽里是什么」只决定候选集里有哪几版，不进三态。逐状态对：

| 8-3 的状态（R⁻ 与 R 之间的 txg） | 三态 | 对不对 |
|---|---|---|
| 根在位、被抛弃（R⁻ 之后的实例 i 的根被新实例弃掉） | R 的实例大于 R⁻ 的、行 (i, T = R⁻.txg) 在、中间行 (j, 0, 0)：HoleBridged | 对：R 直接建在 R⁻ 上，新单元的诞生代号都大于 R⁻.txg |
| 由记录施加 | 它自己是候选集里的一版（`versions_applied_only_by_records`，冻结 walk.rs 第 6508 行），成为相邻的 R⁻ 或 R，不在两者之间 | 对 |
| 取号跳过 | 行证据齐 → HoleBridged；缺 → HoleNotBridged（N3 放过用户可见树里的单元，别的单元仍照 (a)） | 对，与条款 gap、gap-row 两句一致 |
| 槽里是上一圈的根、槽坏 | 同实例内的洞：`version_instance <= previous_instance` → HoleNotBridged（探针 ba 钉住这一支）；换了实例且行证据齐 → HoleBridged | 对（R 建在被选中的那一版上，行证明它） |
| 同实例内跳号（根槽写失败重发跳一格） | 同上第一支 | 对 |

注：8-3 报告第十节说 I-3.1 在洞上要的是「t 那一版存在过、却既不是根在位也不是由记录施加」那一列；三态函数不区分被抛弃、取号跳过、槽坏，给不出那一列，8-2 报告写明「留给 I-3.1 那一处接」，进第 9 批，不是这一轮的错。

### 2.5 合法历史上有没有误红：我的跑数

harness 档，冻结副本拷贝上，原样结果行（日志在 `logs/`）：

```text
$ SINGLEFS_RANDOM_HISTORY_FIRST_SEED=5000 SINGLEFS_RANDOM_HISTORY_SEEDS=80 SINGLEFS_RANDOM_HISTORY_OPERATIONS=24 SINGLEFS_RANDOM_HISTORY_SHRINK=none bash research/scripts/capped.sh 15 …run-with-memory-cap.sh 6G cargo test -p singlefs-harness --test random_histories random_histories_large_tier_seeds_and_length_from_the_environment -- --ignored --nocapture
历史 80 段：跑完 80、以已知红收尾 {}、新发现 0；根环转过一圈的 27 段；最高 txg 35；一版里最多 664 条分配记录
  I-3.5：判绿 1072 次、不适用 101 次
$ …… rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms -- --ignored
历史 48 段：跑完 48、以已知红收尾 {}、新发现 0；根环转过一圈的 39 段；最高 txg 38；一版里最多 706 条分配记录
  I-3.5：判绿 936 次、不适用 13 次
$ …… reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms -- --ignored
历史 48 段：跑完 48、以已知红收尾 {}、新发现 0；根环转过一圈的 47 段；最高 txg 51；一版里最多 856 条分配记录
  I-3.5：判绿 1094 次、不适用 14 次
```

（`logs/random-seeds5000-80x24.log` 第 10、117 行，`logs/random-rollback-heavy.log` 第 9、110 行，`logs/random-reuse-heavy.log` 第 8、112 行。）三档合 176 段历史，I-3.5 判绿 3102 次、从没判红。

再造「写行那次崩在根槽之前」的合法洞（`tests/probe_k3_i35_on_legit_hole_histories.rs`：覆盖写到 txg 8、11、14、17、20，留 0–3 个洞，之后挂载、空发布与覆盖写交替 14 步，逐版 `check_pool_image`）：

```text
K3PROBE_TOTAL images=330 i35_red=0 any_red=0
```

330 张镜像 I-3.5 全是「成立」，别的不变量一条没红；再加一份回退（`tests/probe_k3_i35_rollback_after_a_hole.rs`，洞之后回退到洞之前的根，再发 6 版）：`K3RB_TOTAL judged=84 rollbacks_done=12 i35_red=0 any_red=0`。

**这两组的局限，要写明**：我把 gap 整条去掉（`if previous_txg.checked_add(1) == Some(version_txg) || true {` 恒返回 NoHole，`probes/gap`），同样两组镜像照样 0 红（`logs/probe-k3-holes-nogap.log`、`logs/probe-k3-rollback-hole-nogap.log`）：这些历史里洞右边新出现的单元诞生代号都大于 R⁻.txg，gap、gap-row、N3 在它们上面没被用到。所以这一格「合法历史上不红」的依据只有第二、三轮判决里的抽象模型数与攻方的历史段数（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 3、6 行），我复核不了；我这边的数说明出现规则里 (a)、TF 在这些合法历史上不红：把 TF 的 (b) 去掉（`probes/tf`），rollback_heavy 那一档 48 段里 42 段新发现 `CheckerViolations { invariants: ["I-3.5"] }`（`logs/random-rollback-heavy-without-tf.log` 第 10、112 行），可见这些历史确实用到了 TF；gap 一支没被用到。

### 2.6 走读两处修正有没有改别的不变量的读数

- 修正一（容器落点记在根节点下面，冻结 walk.rs 第 2000 行）改的是「这一版引用了哪些落点」那张每版的表（`placements_referenced_below_a_walked_unit`），读它的是 I-5.1 的「同一版第二次引用同一个落点」、I-4.9 并入共享单元时逐个判诞生代号、I-3.5。I-3.1 用的 `references`（冻结 walk.rs 第 957 行 `note_reference` 往里记）是全局的、第一次走到时就记，不读这张表；I-3.10、I-5.4 读分配记录与单元头，不读它。⇒ 读数可能变的只有 I-5.1、I-4.9、I-3.5，I-3.1、I-3.10、I-5.4 不变（推的）。
- 修正二（树表共享时映射根照样并进，第 1409 行）：已走过的映射根节点 `read_index_node` 返回 None，`walk_code_two_subtree` 交回 `AlreadyWalkedFromAnotherRoot`，叶条目不再收集（冻结 walk.rs 第 1606 行起），所以 `walk_central_mapping_entries` 不会重复计同一批映射条目，I-3.1 的引用并集不变；变的只是这一版的落点集合多了映射树那几个单元。
- 实测侧：`checker_known_bad_images`（69 过）、`checker_narrow_invariants_and_abandoned_roots`（20 过）、`one_revolution_window_after_a_hole_in_the_root_ring`、`raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic`、`rollback_by_a_forward_publish`、`reuse_after_raising_the_floor`、`writable_mount_of_a_formatted_pool`、上面三档随机历史，全绿（前七个二进制的原样结果行在 `logs/legit-binaries-clean.out`：69、20、1、2、3（13 忽略）、1（22 忽略）、15 过，0 红）。

### 2.7 K3 判定

| 子问 | 判 | 什么会推翻 |
|---|---|---|
| 新判法与 I-3.5 一行原文、第二三轮判决逐要素 | 一致（2.1 表）；同 txg 并组、引用不全跳过、gap-row 的三个子条件无钉 | 补用例后探针转红即闭；或找到一个与表不同的行为 |
| 8-2 自取四处读法 | 合法历史上误红：没找到（推 + 上面 176 段与 414 张镜像）；漏红：③④（deadlist）整类，其余读法只会多判；「数不全」怎么数条款没说 | 一段合法历史上红 I-3.5 |
| 洞判法三态对 8-3 表 | 一致（2.4 表）；子条件无钉 | 同上 |
| 走读两处修正对别的不变量 | I-3.1、I-3.10、I-5.4 读数不变（推的），I-5.1、I-4.9 的每版集合变完整；实测相关二进制全绿 | 一张镜像上这三条之一的判定因这两处修正而变 |

## 三、K4　I-8.10 七段、RR1、RR3 与 GPU 两侧

### 3.1 七段各比的是哪一段，CPU 与 GPU 逐段同口径

条款：新根段 364 字节「已定项 15 的新根段 364 字节（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4）」（`.claude/kb/decisions/23-journal的角色与格式.md:155`）；根记录偏移「flags 20 / 实例代号 24 / checkpoint_txg 28 / 树表单元指针 36 / 树 ID 水位 122 / 回退下界 F 130 / 自证校验和 138 / 实例表单元指针 170 / 中央映射树根指针 256 / 分配记录树根指针 342」（`.claude/kb/decisions/22-单元原子性怎么合成.md:151`）。用户 2026-10-03 定 I-8.10 从四段扩到七段：「I-8.10 扩到七段（CPU 与 GPU 两侧）」（`records/2026-09-30-里程碑三开工第一轮.md:481`）。kb 的 I-8.10 行还写着四段：「J 的新根段四段与 R 的同名四段逐字节相同」（`.claude/kb/invariants.md:99`），写回批要改（不是这一轮的错）。

| 段 | 记录新根段（偏移，宽） | 同代根（偏移，宽） | CPU 冻结 walk.rs 第 6389–6444 行 | GPU `crash_judge_checker.wgsl` 第 699–709 行 | 判 |
|---|---|---|---|---|---|
| 树表指针 | 0，86 | 36，86 | `NEW_ROOT_SEGMENT_TREE_TABLE_POINTER` 0..86 ↔ `ROOT_RECORD_TREE_TABLE_POINTER` 36..122 | `bytes_equal(r.base, segment, root.base, 36u, 86u)` | 同 |
| 映射根指针 | 86，86 | 256，86 | 86..172 ↔ 256..342 | `segment + 86u, … 256u, 86u` | 同 |
| 树 ID 水位 | 172，8 | 122，8 | 172..180 ↔ 122..130 | `segment + 172u, … 122u, 8u` | 同 |
| F | 180，8 | 130，8 | 180..188 ↔ 130..138 | `segment + 180u, … 130u, 8u` | 同 |
| 实例表指针 | 188，86 | 170，86 | 188..274 ↔ 170..256 | `segment + 188u, … 170u, 86u` | 同 |
| 分配记录树根指针 | 274，86 | 342，86 | 274..360 ↔ 342..428 | `segment + 274u, … 342u, 86u` | 同 |
| 卸载记号 | 360，4 | 20，4（flags 整 4 字节） | 360..364 ↔ 20..24 | 「&& bytes_equal(r.base, segment + 360u, root.base, 20u, 4u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:709`） | 同 |

两侧的分组口径也一样：同 (实例代号, txg) 带末条标志的记录恰一条才判，与根环里同 (实例, txg) 的每条自证过的根逐条比，一对都没比到报不适用（CPU 冻结 walk.rs 第 6645–6700 行，GPU 同文件第 674–716 行）。卸载记号那 4 字节整段比，根 flags 位 0 之外非 0 的根自证不过进不了（`crates/singlefs-checker/src/lib.rs:340` 的 `if flags & !ROOT_FLAG_UNMOUNT_MARKER != 0 {`），记录一侧位 0 之外非 0 的记录被 RR1 挡掉，所以比的实际是位 0。

钉：CPU 七段里新加三段各有变异行 2261、2262、2263，现跑都红（`logs/prove-k3k4-harness.out`）；F 一段没有 CPU 变异行，我现拿掉那一对（探针 `probes/f`），`each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant` 红，所以它被用例钉住、只是缺一行变异。GPU 侧：`mutations.tsv` 里 `crash_judge_checker.wgsl` 的 I-8.10 行只有一条（第 1931 行，「第 5 批 GPU 判器：I-8.10 新根段与同代根不比」，整条不比），新加的三段与 RR1 在 GPU 侧**没有变异行**（我用 awk 按文件名与名字筛过）；checker 档我没跑，GPU 侧有没有用例红它们复核不了。

### 3.2 RR1 卸载记号其余位：三处口径

| 处 | 判法 | 位置 |
|---|---|---|
| core 读者 | 位 0 之外有位为 1 ⇒ `None`（记录当损坏） | `crates/singlefs-core/src/root_record.rs:44`「if field & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {」；journal 解析 `crates/singlefs-core/src/journal.rs:346` |
| CPU checker | 同一个掩码，返回 `Verdict::UnmountMarkerBitsOtherThanBitZero` | 「if read_u32(record, JOURNAL_UNMOUNT_MARKER_OFFSET) & !JOURNAL_UNMOUNT_MARKER_BIT != 0 {」（`crates/singlefs-checker/src/lib.rs:802`） |
| GPU checker | 掩码 `0xFFFFFFFE` | 「if ((rd32(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 360u) & 0xFFFFFFFEu) != 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:306`） |

三处同口径。钉：core 单测 `record_whose_new_root_segment_unmount_marker_sets_any_bit_other_than_bit_zero_is_refused_by_the_parser`（journal.rs:531）；CPU：`a_record_whose_unmount_marker_has_bits_other_than_bit_zero_is_dropped_by_the_checker_as_by_the_recovery`（`crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs:970`）拿恢复扫环收不收与 checker 的 I-4.1 判法对着比，取值 1、2、0x8000_0000；变异行 2264 现跑红。GPU：没有变异行。

**线索 L-K4-1（GPU 恢复内核没有 RR1）**：GPU 的恢复内核有自己的记录解析 `parse_record`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:956`），它判 magic、类型、校验和、标志、长度、提交标记、序号、新根段两条指针头部、fsid，不判卸载记号其余位（该函数体里没有对 `JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 360u` 的读）。core 的读者拒这样的记录（journal.rs:346），所以 GPU 恢复内核与 core 在「校验和对得上、卸载记号其余位非 0」的记录上会分叉：core 丢记录，GPU 恢复内核收下。合法崩溃历史走不到（要校验和对得上的外来字节），影响的是 CPU/GPU 对拍在构造的状态上。8-2 报告 RR1 只改了 GPU checker 读记录那一处，没提这一处。推的；要 checker 档对拍才能量。

### 3.3 GPU 内核换了格式常量之后，有没有一处还按旧值算

按 `.claude/rules/format-evolution.md`「改一个格式常量」第 1 步逐类搜（命令与全部命中在 `logs/format-const-grep.out`）：

```text
$ cd crates && grep -rnE '\b(287|311|196|67|188)\b' --include=*.rs --include=*.wgsl singlefs-checker-tier/src singlefs-checker/src singlefs-core/src singlefs-format/src | wc -l
20
```

20 条命中，逐条看过：除 e171 第 3926 行与 e165 第 9091 行两处「311 字节头」注释（已知没做，body 第二节点名）与 `write_request_split.rs:201` 的 67 档位名（已知，只在名字注释里）外，都不是指旧头宽（别的数据单元偏移 67、journal.rs 里「[287, 373)」是新布局下实例表指针的偏移、walk.rs 第 6392–6393 行是新根段七段常量）。wgsl 里的 `JOURNAL_HEADER_BYTES`、`JOURNAL_NEW_ROOT_SEGMENT_*`、fsid 偏移 463 都走常量（`crash_judge_common.wgsl:91-97`），`records_parse_equal` 比的宽是 `JOURNAL_NEW_ROOT_SEGMENT_BYTES + 8u`（新根段加 fsid，原来写死的 196）。点名项宽 56 不变，每条记录点名项上限 64（`singlefs-format` 里 `JOURNAL_NAMED_ENTRIES_PER_RECORD`，由 (4096 − 487) ÷ 56 推出）。

**线索 L-K4-2（GPU 走读内核对「由记录施加出来的那一版」还按新根段只有四段的时代取指针）**：

- 条款：施加一条记录是「施加一条记录 = 把所选根的这七个字段换成记录里的（实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根）」（`.claude/kb/decisions/23-journal的角色与格式.md:451`）。
- CPU 走读：由记录施加的那一版，实例表链与分配记录树根都从记录的新根段取（冻结 walk.rs 第 1344–1356 行 `walk_version_applied_only_by_records`，用 `NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER`、`NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER`）。
- GPU 走读内核：`w_walk_one` 对 `WALK_KIND_VERSION` 把 `carrier_base` 设成「它施加在其上的那条根」（冻结 crash_judge_checker.wgsl 第 2602–2610 行），实例表链取 `parse_node_pointer(carrier_base, 170u)`、分配记录树根取 `342u`（第 2614–2616 行）；注释还写着「实例表与分配记录树取自它施加在其上的那条根」（第 2594 行）。这是新根段只有树表、映射根、水位、F 四段（188 字节）那个时代的形态。
- 后果（推的）：写行那次发布的记录已落、根槽没落的状态上，记录里的实例表指针与分配记录树根与所选根的不同（写行发布 COW 了实例表、写了新的分配记录树），GPU 走的是所选根的旧实例表与旧分配记录树，CPU 走的是记录里的新的，两边引用集合不同，I-3.1、I-4.1 一类判定可能分叉。要 CPU/GPU 对拍才能坐实，checker 档没跑，复核不了。8-2 报告 S7 点名的是 GPU **恢复**内核拼新根时照抄所选根（`crash_judge_common.wgsl:1286` 一带，已知没做），这里是走读内核，是另一处。
- 什么会推翻它：`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 在写行记录已落、根槽没落的状态上 CPU 与 GPU 违例位逐位相同。

### 3.4 K4 判定

| 子问 | 判 | 什么会推翻 |
|---|---|---|
| 七段 CPU 与 GPU 逐段同口径 | 一致（3.1 表，七行逐行对） | GPU 对拍在某个状态上 I-8.10 两侧不同 |
| RR1 三处（core、CPU checker、GPU checker） | 一致；GPU 侧无变异行；GPU 恢复内核没有这一判（线索 L-K4-1） | core 与 GPU 恢复内核在同一条记录上收放一致 |
| GPU 内核有没有旧值：头 311、点名项 67、新根段 188 | 常量与偏移都已换新；走读内核对「由记录施加的那一版」仍按四段时代取实例表与分配记录树根（线索 L-K4-2）；e171、e165 两处注释已知 | 对拍写行记录状态上两侧相同 |
| RR3（写行记录两项新字段） | 一致：写行那次发布的记录（`prepare_the_row_publish_on_a_version_without_file`）里「new_instance_table: root.instance_table,」（`crates/singlefs-core/src/transaction.rs:1934`）与分配记录树根都取自这次写的根；变异行 2265、2266 现跑红，用例 `the_row_publish_record_carries_every_new_root_segment_field_of_its_root`（`crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs:887`） | 写行发布的记录与它的根在盘上不逐字节相同 |

## 四、K5　读者与拒法

### 4.1 七类走读读者判了哪几段，各有几处钉

共享的判法是 `PointerHeadAsRead::judged`（`crates/singlefs-core/src/pointer.rs`：第 211 行 MAC / nonce，第 214 行算法类型，第 221 行压缩算法码，第 228 行压后长度，之后 extent 偏移）。七类读者都改成走它（冻结副本 grep 全部调用点）：

| 类 | 读者（core） | 失败怎么报 | 变异行（现跑红，`logs/prove-k5-harness.out`） |
|---|---|---|---|
| 1 树表条目的根指针 | `records.rs:449` `TreeTableEntry::parse` | 返回 `None`（整条条目不可用） | 2243（MAC 非 0） |
| 2 inode 内部条目的子指针 | `records.rs:503` | `PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion` | 2244（nonce 非 0） |
| 3 extent 叶记录的数据指针 | `records.rs:523` | 同上 | 2245（压缩算法码不认识） |
| 4 extent 上段叶条目（下段根指针、内联数据指针） | `extent_tree.rs:363`、`:370` | `ExtentUpperLeafEntryMalformed::PointerHeadFieldOutsideTheFirstVersion` | 2246（内联数据指针，算法类型） |
| 5 extent 树内部条目的子指针 | `extent_tree.rs:735` | `RecoveryFailure::UnitMalformed` | 2247（码 0 而压后长度非 0） |
| 6 分配记录树内部条目的子指针 | `allocation_record_tree.rs:777` | `RecoveryFailure::UnitMalformed` | 2248（MAC 非 0） |
| 7 实例表链指针 | `instance_table.rs:199`、`:224`、`:252`；`recovery.rs:2241` | 链记录 `parse` 返回 `None` | 2249（算法类型） |

补的：多层码 2 树内部条目（`code_two_tree.rs:154`）与 journal 新根段指针（`journal.rs:324–341`）在这一批之前就判。没判的：`root_record.rs:143–150` 四条指针只判 extent 偏移（`read_judging_the_extent_offset_from`），body 第二节点名「`root_record.rs` 那一类读者没判 MAC 等」，已知没做。

四种加密、压缩字段的检查各自被钉：我在副本里每次把 `judged()` 里一个检查拿掉（`probes/j1`–`j4`，每次跑 `pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader` 一个二进制）：

```text
j1 去掉算法类型：test result: FAILED. 5 passed; 2 failed   （logs/judged-j1-red.out）
j2 去掉压缩算法码：test result: FAILED. 6 passed; 1 failed   （logs/judged-j2-red.out）
j3 去掉压后长度：test result: FAILED. 6 passed; 1 failed   （logs/judged-j3-red.out）
j4 去掉 MAC / nonce：test result: FAILED. 4 passed; 3 failed   （logs/judged-j4-red.out）
```

### 4.2 处置与 D19 已定项 3、D9 已定项 14 原文逐字相同吗

| 字段 | 条款原文 | 实现 | 判 |
|---|---|---|---|
| MAC / nonce 非 0 | `.claude/kb/decisions/19-块指针的结构与宽度预算.md@已定项 3：定宽，不加密时留位`：「加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏」 | `MacOrNonceNotZero`，调用方把那条条目或那一片当损坏 | 一致 |
| extent 偏移非 0 | 「extent 偏移那 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与加密开不开无关」（同一节） | `ExtentOffsetNotZero`，同一个处置 | 一致 |
| 压后长度，码 0 | `.claude/kb/decisions/09-加密.md@已定项 14：压缩算法码的登记表`：「码 0 时它恒 0，非 0 判该指针损坏」 | `CompressedLengthNotZeroWithoutCompression`，先判码、码不为 0 时先报码 | 一致 |
| 压缩算法码不认识 | `.claude/kb/decisions/09-加密.md@已定项 14：压缩算法码的登记表`：「读者遇到不认识的码一律判该指针无法解引用」（同一句后面还跟括注：与算法类型那个字段同一条政策，D22 已定项 17） | `CompressionAlgorithmCodeNotRecognized`，与别的字段**同一个处置**：该指针所在的结构损坏（`PointerHeadFieldGroup` 的注释「两组的处置相同（判该指针所在的结构损坏，不拿这条指针去解引用）」（`crates/singlefs-core/src/pointer.rs:128`）；8-1 报告第五节第 4 点自报没分） | 条款说「无法解引用」、实现统一成「结构损坏」：行为同类（都不拿这条指针去解引用），措辞不同；两词要不要分开条款没说：规则没说 |
| 算法类型不是 0 | `.claude/kb/decisions/09-加密.md@已定项 3：运行时按 CPU 特性选，格式里不钉死`：「读者遇到未登记的码显式失败，不按别的算法校验」 | 任何非 0（含登记过的 1、2）都拒 | 登记过的 1、2 在没有加密实现的第一版怎么读条款没说（D9 已定项 10 只说加密不进第一版）：规则没说，实现取拒 |

另两处 kb 与代码对不上，写回批要改（不是这一轮的错）：D9 已定项 14 的射程一段末句（转述）还写着读侧两条判定今天没有代码在判（`.claude/kb/decisions/09-加密.md` 已定项 14 一节），实际已有；D19 已定项 3 写「core 每个读盘上指针头部的读者都判」，而 `root_record.rs` 四条指针只判 extent 偏移（上表之后），这一句今天对根记录不成立，已知没做。

### 4.3 `mounted_read.rs` 两种拒法的分界

`parse_inode_internal_entry_or_refusal`（`crates/singlefs-core/src/records.rs:489`）先判条目宽：「if bytes.len() < usize::try_from(INODE_INTERNAL_ENTRY).expect("120") {」（`crates/singlefs-core/src/records.rs:492`）⇒ `NarrowerThanItsFieldTable`；够宽才解子指针头部，不是第一版取值 ⇒ `PointerHeadFieldOutsideTheFirstVersion`。只读挂载把前者映成 `RecordMalformed`（`crates/singlefs-core/src/mounted_read.rs:467`），后者映成 `PointerInTheRecordCannotBeDereferenced`（`crates/singlefs-core/src/mounted_read.rs:473`）。输入按条目字节数分三类：窄于 120、够宽而头部有一段不是第一版取值、够宽而头部好；三类各落一个成员（含「又窄又带坏头」落窄：窄的读不到完整头部，先判宽）。没有一种输入落到错的成员。两条变异行 2241、2242 现跑红（`logs/prove-k5-harness.out`）。

注：这个分界只做在 inode 内部条目这一处（只读挂载读它时）；别的带指针的条目在只读挂载里走各自的失败类型（extent 树、树表），不在这一格。

### 4.4 影子账两读者改回不判指针头部之后

两处：`crates/singlefs-core/src/allocation_record_tree.rs:896`（`node_pointers_as_far_as_readable`）与 `crates/singlefs-core/src/recovery.rs:2261`（实例表链各片），都传 `PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger`。第一轮判决要的是多隔离、不漏隔离。

- 合法历史上：写者恒写 0（`crates/singlefs-core/src/pointer.rs:255` 起 `PointerHead::write_to` 对 MAC、nonce、算法类型、压缩码、压后长度、extent 偏移写 0），五个字段在合法镜像上恒为第一版取值，判与不判读出同样的指针，影子账收下的节点与别的读者收下的相同；两种读者除头部之外判的东西（层级、条目宽、key 次序）也相同（对照 `allocation_record_tree.rs` 的 `read_node_and_its_subtree` 与 `node_pointers_as_far_as_readable`，推的）。⇒ 没找到一段合法历史让影子账收下 core 别的读者拒的节点。
- 坏镜像上：收下是设计（头部一段不是第一版取值的节点，位置条目照样认，多隔离一个单元）。
- 钉：变异行 2239、2240 把这两处改成判，对应用例红（现跑，`logs/prove-k5-harness.out`），说明「不判」被钉住。
- 什么会推翻：一段合法历史里某个指针头部被写成非第一版取值（写者改了、或加密接入之后），那时这两处就是漏认的口子，要重判。

### 4.5 K5 判定

| 子问 | 判 | 什么会推翻 |
|---|---|---|
| 七类读者的字段与处置对 D19 已定项 3、D9 已定项 14 | 字段一致；「无法解引用」与「损坏」实现不分、算法类型的登记码读法条款没说：规则没说 | 条款要求两种说法分成两个成员 |
| 两种拒法的分界 | 没找到落错成员的输入 | 一条按字节数分类下落错的输入 |
| 影子账收下 core 别的读者拒的节点 | 没找到合法历史 | 写者写出非第一版取值 |
| 根记录读者没判 MAC 等 | 与 D19 已定项 3 那一句冲突，已知没做 | — |

## 五、K6　改钉

### 5.1 I-3.5 进两张不适用表的那两步

8-2 写的不适用条件（冻结 walk.rs 第 597 行 `if units_judged == 0 {`）：一个新出现、头读得出的单元都没判到——候选集只剩一版、或相邻两版有一版走读记过失败、或缺口那一对新出现的只有用户可见树里的单元。三种原因同一句说明，表上看不出是哪一种。我在副本里的 `judge_the_appearance_rule_of_adjacent_candidate_versions` 末尾加一行打印（`probes/dbg`，不改判定），跑改钉后的两个用例，原样输出（`logs/k6-dbg-1.log`、`logs/k6-dbg-2.log`）：

```text
test writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold ... I35DBG versions=3 groups=1 pairs=0 pairs_skipped_incomplete=0 units_judged=0
I35DBG versions=3 groups=3 pairs=2 pairs_skipped_incomplete=0 units_judged=0
I35DBG versions=4 groups=4 pairs=3 pairs_skipped_incomplete=0 units_judged=12
test a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold ... I35DBG versions=5 groups=5 pairs=4 pairs_skipped_incomplete=0 units_judged=6
I35DBG versions=6 groups=6 pairs=5 pairs_skipped_incomplete=0 units_judged=18
```

| 步 | 表 | 打印 | 原因是 | 判 |
|---|---|---|---|---|
| mkfs 之后 | `NOT_APPLICABLE_RIGHT_AFTER_MKFS` | 3 个版本并成 1 组，0 对 | mkfs 把第 0 代根种进三个区域各自槽 0（「第 0 代根记录种进三个区域各自槽 0」，`crates/singlefs-core/src/make_filesystem.rs:5`），同 txg 并成一组，候选集只剩一组 | 是 8-2 写的条件之一（候选集只剩一版）；注释写「只剩 mkfs 的第 0 代根一版」口径对 |
| 第一次可写挂载之后（无文件） | `NOT_APPLICABLE_WITHOUT_FILE` | 3 组、2 对、0 对被跳过、判到 0 个 | 写行与暖机两次零单元发布，树表与实例表照抄上一版、映射根全零，后一版没有新出现的单元 | 是；不是引用不全 |
| 写行那次发布之后（无文件） | **不在表里** | 5 组、4 对、判到 6 个 | 写行把实例表 COW 到新落点、写了新的分配记录树，新单元诞生代号 3 大于前一版的 2 | 对：这一步真判到且成立，没有被登成不适用 |
| 第一个文件版本之后 | 不在不适用表里 | 判到 12、18 个 | — | 对 |

没有把本该成立的一步登成不适用。什么会推翻：这两步里某一步 `pairs_skipped_incomplete` 不为 0（即不适用其实是走读失败造成的）。

### 5.2 五行变异的新原文改坏的是不是名字说的行为

先确认锚点：我用脚本对冻结副本逐行核了 `crates/mutations.tsv` 第 4 行起每一行的原文各命中源码恰好一次，0 行不是一次（含这五行和第 2229–2266 行）。每行现跑 `prove-red.sh`（`logs/prove-k6-harness.out`）末行：

```text
✓ 点名 5 条：跑了 5 条，跳过 0 条，跑的都抓到了
```

| 行 | 名字说的 | 新原文改坏的 | 红在哪条用例 | 判 |
|---|---|---|---|---|
| 477 | 发布取完落点之后不记根 | `publish_admitted`（冻结 transaction.rs 第 7751 行）里那次「先重读再记根」调用（原文后面紧跟 `let slot_of = …`，只命中这一处）整个拿掉，不转环、不回收 | `rolling_back_while_mounted_to_the_oldest_ring_root_on_a_small_device_and_overwriting_again_all_succeed`（`random_histories`，慢档，`--include-ignored`） | 对；它只管 `publish_admitted` 这一处，另三处记根调用点没有行（K1 1.8） |
| 1767 | I-8.10 判定恒真 | `differing_fields.is_empty(),` 换成 `true,` | `each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant` | 对 |
| 1916 | 树表指针不比 | 七对表里那一对整对删掉 | 同上 | 对 |
| 1917 | 映射根指针不比 | 同上，映射根那一对 | 同上 | 对 |
| 1918 | 树 ID 水位不比 | 同上，水位那一对 | 同上 | 对 |

### 5.3 K6 判定

| 子问 | 判 | 什么会推翻 |
|---|---|---|
| 两张不适用表的两步 | 一致（5.1），依据是 8-2 写的「一个单元都没判到」，不是引用不全 | 见 5.1 |
| 五行变异新原文 | 一致，各自红在名字说的行为上，锚点各命中一次 | 某行改回旧原文仍命中、或红在别的行为上 |

## 六、判定一览

线索编号：L-K1-1 E2 三种结局与「按哪张表」无钉；L-K1-2 记根四处调用点只有一处有变异行；L-K1-3 F 越过读不出的有效根与 I-7.9 口径；L-K3-1 gap-row 子条件、引用不全跳过、同 txg 并组无钉；L-K4-1 GPU 恢复内核没有 RR1，GPU 侧 RR1 与 I-8.10 后三段无变异行；L-K4-2 GPU 走读内核对记录施加版仍按四段时代取实例表与分配记录树根。线索都不是「实现错」的实测，判决里照「线索」处理，由主 agent 现查坐实。

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 三句条款与代码 | 一致 | 逐句对上；「读得出而自证不过」「被抛弃」两种结局条款没点名，实现取当没有根（规则没说） |
| K1 记下 txg 高于真住的根 | 没找到 | 引理枚举 17399790 点 0 违例；前提是该槽真住的根 txg ≤ first_txg − 1 |
| K1 E2 选的实例表，写行前后 | 一致（推的） | 挂载时用最新根的表，写行之前换成写行那一张，换表与第一次记根之间没有真记根；无钉（L-K1-1） |
| K1 B2 上限、F 越过读不出的有效根 | 条款字面一致；与 I-7.9 按镜像判的口径合不起来（线索 L-K1-3） | X 读不出 F = 22、读得出 F = 21；窗口改法后的 I-7.9 会判前者红（推的） |
| K1 RR4 与「抬到上限」那一形 | 一致；该形无用例无变异行，且推得门槛恒等于 F、占位无区分力 | RR4 补漏只对 F 低于窗口里读得出的最旧有效根的显式抬 F 有区分力 |
| K1 钉（9 行变异现跑） | 都红；三种重读结局与三处记根点无钉（L-K1-1、L-K1-2） | 我的探针 p1/p2/p3/q1/q2/q3 在 harness 档全绿 |
| K3 新判法逐要素 | 一致 | 同 txg 并组、引用不全跳过、gap-row 三个子条件无钉（L-K3-1） |
| K3 8-2 自取读法 | 合法历史误红：没找到；漏红：③④ deadlist 整类，别的只会多判 | 176 段随机历史 I-3.5 判绿 3102 次；330 + 84 张洞历史镜像 0 红；gap 一支这些历史没用到 |
| K3 洞判法三态 | 一致（对 8-3 表逐状态） | 同实例内跳号走 HoleNotBridged，探针 ba 钉住；其余子条件无钉 |
| K3 走读两处修正 | I-3.1、I-3.10、I-5.4 读数不变（推的） | 变的只是每版落点集合（I-5.1、I-4.9、I-3.5 读它） |
| K4 七段 CPU 与 GPU | 一致 | 七行偏移与宽逐行对上；CPU 三个新段变异行现跑红，GPU 后三段无变异行 |
| K4 RR1 三处 | 一致；GPU 恢复内核没有（线索 L-K4-1） | core、CPU checker、GPU checker 同一掩码 |
| K4 GPU 旧值（头 311、点名 67、新根段 188） | 常量与偏移已换新；走读内核对记录施加版取指针的形态还是旧的（线索 L-K4-2） | e171、e165 两处注释已知 |
| K4 RR3 | 一致 | 变异行 2265、2266 现跑红 |
| K5 七类读者的字段与处置 | 字段一致；「无法解引用」与「损坏」实现不分（规则没说） | 四个加密压缩检查各有至少一条钉（探针 j1–j4 红）；根记录读者没判，已知没做 |
| K5 两种拒法的分界 | 没找到落错成员的输入 | 按条目字节数分三类各落一个成员 |
| K5 影子账读者 | 没找到合法历史 | 写者恒写 0；变异行 2239、2240 现跑红 |
| K6 两张不适用表 | 一致 | 依据是「一个单元都没判到」（打印：pairs_skipped_incomplete = 0），没有把本该成立的一步登成不适用 |
| K6 五行变异 | 一致 | 锚点各命中一次，现跑 5 条都抓到 |

## 七、没做什么

- 没跑 checker 档（`singlefs-checker-tier`）的任何东西：GPU 对拍、层 0、crates 变异整表、门禁都没碰；L-K4-1、L-K4-2 因此只是推的。
- K2 不归我，没看。第一轮站住的格没重推。
- 没出判决，没替主 agent 采纳哪条线索。
- gap、gap-row、N3 在合法历史上不红，我这边没有证据（见 2.5：我造的历史没用到那一支），依据只有第二、三轮判决里的数，我复核不了。「回退发布崩在根槽之前、记录又被短 journal 环盖掉」那一形（第二轮 J1-a）我没造。
- 没量「整条命都读不出」那一形（第二轮攻）；没量 I-7.9 改成只取窗口之后在 X 读不出那一格上的真实判红（I-7.9 还没改，推的）。
- harness 默认快档 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation` 在 238 秒内没跑完，被截断，没有结果可引（没入报告任何结论）。
- 没引主工作区里与冻结副本不同的 walk.rs 等文件（见第零节），这一轮的行号都对冻结副本。

## 八、要主 agent 跑的命令

L-K4-1、L-K4-2 要 GPU 对拍才能坐实。这台机器上 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 在没有 NVIDIA 显卡时打 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」，那就不是通过。

```text
工作目录：仓根；环境：SINGLEFS_HEAVY_TESTS=commit；内存上限 16G；线程 15（或照主 agent 当下的分配）
SINGLEFS_HEAVY_TESTS=commit bash research/scripts/capped.sh 15 bash research/scripts/run-with-memory-cap.sh 16G cargo test -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
跑完看：`test result:` 行是否 `ok`、有没有 `NOT_RUN_NO_GPU_ADAPTER`；有红时看违例位里 I-3.1、I-4.1、I-8.10（位 i 对应 IMPLEMENTED_INVARIANTS[i]）在哪个状态上 CPU 与 GPU 不同
```

我没查这六条流里有没有「写行记录已落、根槽没落」的状态；没有的话 L-K4-2 要另造一个这样的状态（harness 里 `common_root_ring_holes` 的 `crash_a_mount_before_its_row_root` 造的就是它）再交 GPU 判器。

## 九、草稿、模型与清理

- 模型与脚本：`research/prompts/m3-root-ring-code-r2-sonnet-model/`（`e2_window_lemma.py` 与它的输出、三份探针测试 `tests/`、变异探针旧串新串 `probes/`、`scripts/prove.sh`、`scripts/patch2.py`、`scripts/runt3.sh`、全部日志 `logs/`）。探针测试要拷进一份冻结副本的 `crates/singlefs-harness/tests/` 下跑；`README.md` 写了跑法。
- 草稿目录 `/tmp/claude-1000/m3-root-ring-code-r2-sonnet/`：`draft/`（本报告各段草稿）、`logs/`、`probes/`、脚本，留着；与模型目录同内容，主 agent 要删可整个删。
- 我建的编译目录与仓副本 `/tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/`（含 `work/tree/` 冻结副本的拷贝与 `work/tree/target/` 编译目录）：交回前删掉；为什么不留：全部探针已拷进模型目录，副本里除探针外与冻结副本逐字节相同（每次探针后 `cmp` 过），留着只占盘。

## 十、交回前的引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-root-ring-code-r2-sonnet-output.md`（默认根）末行：

```text
  ✓ 核了 37 处引文，对上 37 处，没判 0 处（逐处列在上面）
```

用 `--root /tmp/claude-1000/m3-root-ring-code-r2/tree --root <仓根>`（冻结副本加主仓）跑，结果相同。我删掉了第九节说的 `work/`（7.0G）。
