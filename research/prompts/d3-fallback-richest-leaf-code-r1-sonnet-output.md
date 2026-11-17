# d3-fallback-richest-leaf-code-r1 正推腿报告（2026-11-12）

立场：正推。从用户 2026-11-11 的选项原文「改成先取重写集、不够取空槽最多的叶」和 `crates/` 今天的实现推到代码，核代码做的是不是这句话。
行号都是主工作区今天现读的（`crates/singlefs-core/src/allocator.rs`、`transaction.rs`），不是 diff 里的行号。没跑任何项目代码，没往任务服务交任务；「量的」只指我读到的既有任务日志或产物（读了 `task-server/task log` 13428、13429、13430、13432、13862），「推的」是读码与手推。
没读攻方腿这一轮的产出（禁读清单）。

## 结论（先给判定）

1. 攻击面 1（逐句对）：政策每一句都能落到行，没有越出用户那句话；有四处是实现员自己定的取法，条款与用户原话都没写（1.2）。一处与 kb 末段字面有张力：各盘自答不同、池里有共同空槽时现在分配成功（1.3）。
2. 攻击面 2（固定点终止）：演练与真发读同一串重写角色、同一个重写集——成立（六个调用点逐个核了，2.1）。走满 64 轮的输入我推得出一类（每片叶只有一个共同空槽、无全空段），新旧政策同样走满；「来回换」没构造出来也没证（2.2）。`recovery.rs` 预演走空重写集：只在单元区小于两个整段的几何上两边才可能取到不同的叶，边角，推的，没核（2.3）。
3. 攻击面 3（每叶没挡槽数）：三种挡位的置位清位全部经 `set_blocking_bit`，三张位图字段全文件只有这一处写（第三节表）；没找到绕开它的路径；上界永不低估，所以取到的叶不会因上界错而错。
4. 攻击面 4（旧政策钉值）：补丁没带出、我判为推不出而多半会红的有 5 处（T1–T5），另有 T6、T7 风险低；最实的三处是 `a_floor_raise_refused_for_space_counts_as_short_of_space.rs` 的 19 / 18 次覆盖写、`admission_checkpoint_cost_per_device_paths.rs` 里「至少一次改多于两片叶」的前提断言、`crash_enumeration_last_fitting_…` 的 `FULL_STATE_COUNT`（第四节表）。背景材料列的任务里没有跑到它们的。
5. 检查单八格（第二节）：成立 5（走不到的理由、不兼容旧代码、不新建判定库、多机可用、测试多线程）；不成立 1（类型与名字）；推不出 2（`--item` 列得全不全、新崩溃流多罩了什么）。

## 第一节 攻击面 1 与 2：从用户那句话正推到代码

### 1.1 用户那句话拆成八句，各落在哪几行

用户选项原文（背景材料第一节，不引行号）：「改成先取重写集、不够取空槽最多的叶」，展开为：每块盘都答回落时，池里先取这次发布分配记录树重写集里每块盘都重写的叶中、每块盘都空的最低落点；没有再取每块盘都空的槽最多的那片叶里的最低落点；两步都排除活整理意图的 R；固定点的轮数上限 64 不变。

| 句 | 代码里落在哪 | 判 |
|---|---|---|
| S1 只有每块盘都答回落才进池级政策 | 「let every_device_falls_back = answer_per_device.iter().all(|(_, answer)| {」（`crates/singlefs-core/src/allocator.rs:2859`），`.then(...)` 守门在 `allocator.rs:2866` | 落实 |
| S2 一块开段一块回落，照旧在动任何状态之前拒（已定项 8 末段） | 没换成池级答案时走 `agreement_across_devices`（`allocator.rs:2888`），「DeviceAgreement::AnswersDiffer(answer_per_device) => {」（`crates/singlefs-core/src/allocator.rs:2898`）与 `SomeDevicesWithoutAnswer`（`allocator.rs:2893`）都在 `return Err`；改状态的 `self.open_segment = …`、`record` 在 2908–2925，拒绝都在它们之前 | 落实 |
| S3 重写集 = 每块盘都重写的叶 | 取交集：「pub fn present_on_every_device(leaf_indices_per_device: &[BTreeSet<u64>]) -> Self {」（`crates/singlefs-core/src/allocator.rs:217`）；每块盘各自的层级 0 节点序号：`crates/singlefs-core/src/transaction.rs:2518-2543` | 落实 |
| S4 先取重写集的叶里每块盘都空的最低落点 | 「.find_map(|leaf_index| {」（`crates/singlefs-core/src/allocator.rs:2945`）按 `BTreeSet` 升序，叶不相交且升序 ⇒ 第一片有落点的叶里的最低落点就是并集里的最低落点；逐叶取最低 `allocator.rs:3037-3061`；整跨度在每块盘都空 `allocator.rs:732-747` | 落实 |
| S5 没有再取每块盘都空的槽最多的叶 | 「.or_else(|| {」（`crates/singlefs-core/src/allocator.rs:2952`）→ `allocator.rs:2964-3012`；每块盘都空的单槽数逐槽数 `allocator.rs:3015-3034`；一样多取叶序号小的：升序走叶加严格大于才换，「if open_slots <= most_open_slots_so_far {」（`crates/singlefs-core/src/allocator.rs:2999`）`continue` | 落实 |
| S6 两步都排除活 R | 「let excluded_region = self.compaction.region_excluded_from_placement();」（`crates/singlefs-core/src/allocator.rs:2845`）一次读出，经 2869 传进两步；判点 `allocator.rs:732-747` 里每一槽都问 `contains_slot`；`slots_open_on_every_device_in_the_leaf` 也走同一判点（`allocator.rs:3025`） | 落实 |
| S7 64 轮不变 | 「pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;」（`crates/singlefs-core/src/transaction.rs:14165`），两条固定点循环 `transaction.rs:2006`、`transaction.rs:14030` | 落实 |
| S8 用户数据落点一行没动 | 「PlacementRule::UserData => allocator.try_allocate_user_data(txg),」（`crates/singlefs-core/src/transaction.rs:2501`）；`try_allocate_user_data` 在 `allocator.rs:2757`，主工作区对 HEAD 的 diff 里没有一行动它或 `lowest_user_data_slot`（命令与输出见 1.5） | 落实；注意 `set_blocking_bit`（用户数据也经它）多维护了叶计数，只加账、不改分支 |

### 1.2 实现员自己定的取法（条款与用户原话都没写）

| 取法 | 在哪 | 说明 |
|---|---|---|
| 一样多取叶序号小的 | `allocator.rs:2987`（升序）与 2999（严格大于才换） | 用户原话「槽最多的那片叶」没给并列规则；符合「最低落点」的方向，不越位，但是新定的 |
| 两槽单元：最多的叶里放不下一对就让给下一片 | `allocator.rs:3002-3009`（只有找到落点才更新 `most_open_slots_so_far`） | 用户原话没说两槽单元怎么办；按单槽数排「最多」，按能放下跨度取。另一种读法（按偶数起点对数排）会选出不同的叶 |
| 「槽最多」按单槽数，不按能放下的跨度数 | `allocator.rs:3015-3034` 用 `UnitFootprint::OneSlot` 数 | 同上 |
| 搬迁目的地里的提交内生块也带重写集 | `transaction.rs:13795-13817`（`allocate_relocation_destination`），调用点 14098、15763 | 用户那句话讲提交内生块的回落；整理搬迁目的地走同一入口，所以一起换了。D26 已定项 5 第 2 条把目的地政策定成「开放段」那一支，回落时是否也该先取重写集，条款没写，我没读到用户裁定 |

### 1.3 越位问题：各盘自答不同、池里有共同空槽时现在分配成功

读法（推的，读码）：两块盘都答回落、各自按自己空闲图取到的最低空槽不同（例如影子账只在一块盘上隔离了某槽，`isolate_abandoned` 按盘隔离：`allocator.rs:3224-3231`），改前走 `AnswersDiffer` 拒；现在 S1 成立（两块盘都答回落），池级取落点（S4、S5）取到一个每块盘都空的槽，把每块盘的答案换成它（`allocator.rs:2875-2886`），`Agreed` 之后两块盘各记一条同槽的记录。

- 对用户那句话：成立。用户句子的前提就是「每块盘都答回落时」，池里取的是「每块盘都空」的落点，各盘自答不同时池里有共同空槽取它，是句子的字面推论，不是实现员加的臂。
- 对 kb：有张力。`.claude/kb/decisions/03-空间分配.md:173` 末段写「第一版盘不等大时，各盘落点答案必须相同」。若「答案」读作各盘自己按自己空闲图取的落点，新行为放宽了它；若读作最终发出去的落点，新行为不违反。用户的选项原文没有裁这一点，所以这是推出来的，不是用户原话。写回 kb 时要在已定项 8 里写明「答案」指池里取的那个落点，否则这句字面与实现对不上。
- 反方向（一块开段一块回落）：不进池级，`allocator.rs:2859` 的 `all` 为假 ⇒ 沿用原答案 ⇒ `AnswersDiffer` 拒（`crates/singlefs-verification/tests/unequal_devices.rs:388` 钉着，读码判仍对）。
- 池级取不到（没有每块盘都空的落点）时 `None => answer_per_device`（`allocator.rs:2886`）：此时各盘自答不会一致（若一致，那个槽就是每块盘都空，池级必取得到），所以这一支只会产出原来的 `AnswersDiffer`，不会产出 `Agreed`（推的，上面一句是证）。

推翻 1.3 的现象：构造出一个池，每块盘自答相同（`Agreed`）而池级却返回 `None`，或池级返回 `Some` 但那个槽在某块盘上被挡。后者我读 `is_whole_span_open_for_commit_generated`（`allocator.rs:732-747`）逐盘判，不会发生。

### 1.4 小结（攻击面 1）

八句都落了行，S2 的「动任何状态之前拒」落实。没有用户没说的臂；四处自定取法（1.2）里「搬迁目的地也带重写集」我读不出用户定过，判决时要主 agent 裁一下是不是用户那句话的射程。

### 1.5 命令与原样输出

```
$ git diff HEAD -- crates/singlefs-core/src/allocator.rs > <草稿目录>/alloc-head.diff   （草稿目录 /home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-forward）
$ grep -cE '^[-+].*(try_allocate_user_data|lowest_user_data_slot)' <草稿目录>/alloc-head.diff
0
```

（量的：0 行。这份 diff 含主工作区上别的补丁，所以「0」比「只看本轮 diff」更严。）

## 第一节续 攻击面 2：固定点终止

### 2.1 演练与真发读的是不是同一串角色、同一个集合

六个调用点逐个核（`allocate_placement_for_role` 与 `allocate_relocation_destination` 都要求传 `&RewriteSetLeaves`，漏传编不过，所以不是靠人数）：

| 调用点 | 重写集从哪来 | 用在哪 |
|---|---|---|
| `transaction.rs:2054` 与 2056 | 本轮 `rewritten_roles`（树表 0 条那版的固定点演练） | 在 `allocator.clone()` 上取 |
| `transaction.rs:2210` 与 2213 | `settled.rewritten_roles`（演练返回的那一串） | 真发，在真分配器上取 |
| `transaction.rs:14095-14101`（搬迁 14098、角色 14101） | `resolved.rewritten_roles`（带文件那版的演练） | 在 `allocator.clone()` 上取 |
| `transaction.rs:15758`（搬迁 15763、角色 15846） | `resolved.rewritten_roles`（`publish_admitted` 的 `rewritten`，见 `transaction.rs:15695`） | 真发 |

演练里「定下来那一轮」返回的 `resolved` 就是真发用的 `resolved`（`SettledPublish { resolved, … }`，`transaction.rs:14107-14113`），所以同一串角色、同一个集合。顺序也同：先释放、再搬迁目的地、再各角色（演练 14076-14101，真发 15732-15846）。只有一处两边本来就不同、本轮没动：读盘核对不上的那一份在真发时留在已分配、演练里照常释放（`mount.rs` 抬 F 那一串的注释已写这一格，`crates/singlefs-core/src/mount.rs:5681` 的 `assert_eq!` 在有隔离时跳过比较）。成立。

### 2.2 走满 64 轮或来回换的输入

- 走满 64 轮：构造得出一类（推的，手推，没跑）。盘上没有全空段，每片叶恰好只有一个每块盘都空的槽。设某轮计划里有 X 片叶、需要取落点的角色数是 X + A（A ≥ 1：根、记账树等不在叶里的角色，以及叶之外要落点的提交内生块）。第一步在重写集的 X 片叶里只有 X 个槽，不够；多出来的 ≥ A 个落到第二步的「空槽最多的叶」，也就是新叶（全是并列的一个槽，取叶序号小的）。新叶装进记录，下一轮并进计划，X 增加 ≥ A，角色数又加 ≥ A。每一轮都超出，直到叶用光（交 `PlacementRefused`）或走满 64 轮（交 `AllocationRecordTreeRewriteSetDidNotSettle`）。叶多于 64 / A 时先到 64。旧政策（槽号最小的空槽）在同一形状上同样往新叶走，所以这一形新旧同样走满，不是新政策带来的；实现员的「两空槽一叶」锻造盘（`allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf.rs:35` 的 4 轮）正是新政策能收敛、旧政策不收敛的那一形（量的：任务 13429 退 0）。
- 来回换（重写集不变而这一版的节点不单调）：我没构造出来，也没证。推演：重写集里的叶取光后溢出到的新叶，下一轮进计划、进「先取」的集合，之后先取它；落点集中于计划里的叶，装着记录的节点集 ⊆ 上一轮的节点集 ∪ 溢出的新叶。未用到的叶会让「装着记录的节点 ≠ 这一版的节点」而收缩，收缩后角色数同时减少，放得下就不再溢出；溢出方向与收缩方向各自单调，我没找到把两者接成环的输入。这是推的，不是证；可靠的只有实现员量到的最多 5 轮（41 探）、6 轮（212 格）。
- 实现员写进 `transaction.rs:13975-13984` 的「推的、没证」那一段，我读了：说法与上面一致，没有写过头（它只说回落那一支，开段与 bump 与重写集无关）。

推翻 2.2：任何一组输入走满 64 轮而 `rewrite_set_leaves` 不再变大，或在 `settle_the_allocation_record_tree` 里把每轮的 `nodes_after` 打印出来出现重复（`N_a → N_b → N_a`）。

### 2.3 `recovery.rs` 预演走空重写集，真发走带重写集

`crates/singlefs-core/src/recovery.rs:6617` 的预演调 `allocate_commit_generated`（空集），拿到的是每块盘的落点条数（`PlacementsOnEachDevice`，一个下限），不是落点位置。两边只有在回落发生时才可能取到不同的叶。预演的分配器只建到单元区起点后两段（`recovery.rs:6586`），新池新建文件时提交内生块开第二段，到不了回落，除非单元区小于两个整段。该几何上两边可以落到不同的叶，节点数可差一两个，方向没证；`recovery.rs:6677` 的 8 轮上界（注释说「几轮就该停」，是推的，没量）也没被新政策重量过。边角，推的，没核。

推翻：一个单元区 < 128 槽、跨两片叶的几何上，`placements_on_each_device_after_the_new_pool_file_creation` 的返回值大于真发第一个文件之后每块盘的记录条数（恢复会拿它当下限误判）。

## 第二节 检查单八格

判法：每格先照背景材料第四节那格的问法，再给证据（文件:行）；「推不出」算打中。

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 新加的 `expect` 三条（两个函数）：`allocator.rs:693`「位图下标装得进 u64」（usize 到 u64 在 64 位上不会失败）、`allocator.rs:698`「叶下标装得进 usize」（位置 ≤ 单元区槽数，而槽数早在 `allocator.rs:582` 过了 `usize::try_from(…).expect`）、`allocator.rs:3033`「一片叶的槽数装得进 u64」（一片叶 ≤ 812）。diff 里非测试行 grep `todo!/unreachable!/assert!/Undecided/Unsupported/OutsideTheFirstVersion` 零命中。实现员没点到、我追了的 panic 面：`allocator.rs:668`、`allocator.rs:672` 的下标与 `-= 1`（`Cargo.toml:24` 开着 `overflow-checks`），靠「叶计数恒等于该叶没挡槽数」不变量，第三节表论证成立；新分支里 `allocator.rs:2977-2978` 的「有盘单元区为空」两臂与 `allocator.rs:2983` 的 `?` 从唯一调用点走不到（调用点要求每块盘都答了回落 ⇒ 每块盘单元区非空），但返回的是 `None` 不是 panic，无害。 |
| 类型与名字 | 不成立 | ① 名字不是实话：变体 `FallBackToLowestFreeSlot`（`allocator.rs:195`）在池级换答案时被用来装池里取的落点（`allocator.rs:2880-2881`），计数 `commit_generated_placements_that_fell_back_to_the_lowest_free_slot` 在回落分支加一（`allocator.rs:2920`），而池里真取的落点不再是「最低空槽」。实现员报告「交主 agent 的问题」第 1 条自己点了；改名要动 `unequal_devices.rs:388`、`model_comparison.rs:1774`、变异表 77 / 78 / 2267 行原文，没改。② 同一个底层类型 `u64` 装叶序号、槽数、裸槽号：`allocator.rs:704`（`leaf_index: u64 -> u64` 返回槽数）、`allocator.rs:719`（`Option<(u64, u64)>` 一对叶序号）、`allocator.rs:3037-3048`（`leaf_index`、裸 `candidate` 槽号）；与既有的 `index_in_device: u64`（`allocation_record_tree.rs:50`）同形，没引入更多混用，但按规则字面（同一个量不同语义不共用裸整数）是打中。③ 三条 `expect` 的话只写「装得进」，没写靠哪条不变量保证（对照 `allocator.rs:625-628` 的长写法）。没有缩写、没有 `_ =>`（新 `match` 都列全：`transaction.rs:2528-2538`、`allocator.rs:2973-2982`）。 |
| 不兼容旧代码与旧格式 | 成立 | 命令 `grep -nE 'legacy\|old_format\|migrat' research/prompts/_d3-fallback-richest-leaf-code-r1-diff.md; echo "rc=$?"` 原样输出 `rc=1`（零命中）。`try_allocate_commit_generated`（`allocator.rs:2815`）留着当空重写集入口：非测试的调用者是 `allocate_commit_generated`（`allocator.rs:2801`），后者在 `recovery.rs:6617` 的新池新建文件预演里用（产品路径，空重写集是有意的）；`crates/` 里除 `allocator.rs` 之外直接调它的只有测试（10 处，`grep -rn 'try_allocate_commit_generated(' crates --include=*.rs` 现跑）。所以它是「没有重写集可先取」的入口，不是为旧调用方留的兼容层。旧的逐盘取法 `lowest_commit_generated_fallback_slot`（`allocator.rs:1262-1289`）留作各盘自答，也被只供测试的目的地政策用（`allocator.rs:2374`），不是兼容分支。 |
| 不新建判定库 | 成立 | 补丁的 `diff --git` 只有四个文件：`allocator.rs`、`transaction.rs`、两个验证包测试（`crates.patch` 第 1、741、941、1107 行）；新测试文件现 grep `tsv\|judg\|verdict\|Command::new\|std::fs` 只命中第 83 行文档里提到 `mutations.tsv`（说判别力在哪一行变异）。变异行追加进既有的 `crates/mutations.tsv`，不是新判定库。 |
| 所有任务多机可用 | 成立 | 新验证包测试 `allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf.rs` 在进程里造盘（现 grep `CARGO_MANIFEST_DIR\|std::env\|read_to_string\|/home/\|/tmp\|hostname` 零命中）；5 条分配器单测同。日志 13432 里有「第二台开跑」。 |
| 测试优先多线程 | 成立 | 新测试都是一条用例一个场景，由测试框架并行；`allocator.rs:4745` 有一个两项的 `for (footprint, expected)`（一槽、两槽各一回），可拆但不是切片并行该做的活；新验证包测试起的那一条线程（`…two_free_slots_per_leaf.rs:88`）只为给 64 MiB 栈（`:39` 的常量），不是并行也没写线程数。 |
| 带 --item 的命令与项 | 推不出 | 实现员报告「受影响的崩溃枚举用例与崩溃点重放项」列的 8 项，名字都在 `.claude/gate.d/crash-point-replay-items.tsv` 第一列里（52、53、60、70、76、79、80、18 行，现 grep 过）；命令写全了没核（那一节我读了，是 `--item <项>` 的整条）。但这 8 项是「按源码里出现 `384_slot_small_devices`、`fill_free`、『没有全空段』这类字挑的」，报告自己写了「不是按量的」。没点到而会走回落的候选：`crash-injection-default-batch`、`random-histories-and-fault-injection-given-states`、`read-faulted-states-never-hold-the-witness-criterion`（都用 384 槽小盘的随机历史，`history.rs:137-148` 说那一档盘「回收之前几十次发布就写满」），第四节 4.2 表里 T1、T4、T6 那类靠探针调出来的墙边用例是普通测试目标，不在崩溃点重放项里。列得全不全，我推不出。 |
| 新崩溃流多罩了什么 | 推不出 | 本轮没加崩溃流。题问的是「回落进空槽最多的叶之后，那片叶头一次被重写的那一版」，已有流罩不罩得到。结构上它与「bump 开新段跨进新叶」同形（已有 `position-addressed-tree-streams`、`large-file-overwritten-across-lower-leaves-stream` 罩着新叶出生）；但没有证据说明哪条已登记的流真走到回落的第二步（空槽最多的叶）：近满池两条流（`truncate-to-zero-on-a-nearly-full-pool`、`last-fitting-unit-…`）只是造盘写法像，没量走没走。主 agent 缩深度 5 跑过的 89 态（背景材料，任务 13626）只证没红，不证走到。所以「已有流罩得到」是推的，没有量，算推不出。 |

## 第三节 攻击面 3：三种挡位的置位清位路径（`DeviceFreeMap` 的每叶没挡槽数）

先给全局事实（量的，命令在主工作区现跑）：

```
$ grep -nE 'allocated\[[^]]*\]\s*=[^=]|isolated\[[^]]*\]\s*=[^=]|held_until_floor_takes_effect\[[^]]*\]\s*=[^=]' crates/singlefs-core/src/allocator.rs
658:            SlotBlockingBit::Allocated => self.allocated[index] = is_set,
659:            SlotBlockingBit::IsolatedByTheShadowLedger => self.isolated[index] = is_set,
661:                self.held_until_floor_takes_effect[index] = is_set;
$ grep -nE 'unblocked_slots_per_allocation_record_tree_leaf' crates/singlefs-core/src/allocator.rs | grep -vE '^\s*[0-9]+:\s*//' | grep -vE '///'
419:    unblocked_slots_per_allocation_record_tree_leaf: Vec<u64>,
562:        let unblocked_slots_per_allocation_record_tree_leaf: Vec<u64> = if unit_area_slots == 0 {
611:            unblocked_slots_per_allocation_record_tree_leaf,
668:                self.unblocked_slots_per_allocation_record_tree_leaf[leaf] -= 1;
672:                self.unblocked_slots_per_allocation_record_tree_leaf[leaf] += 1;
710:                self.unblocked_slots_per_allocation_record_tree_leaf
4873:    fn unblocked_slots_per_allocation_record_tree_leaf_are_maintained_incrementally_and_match_a_scan(
```

三张位图的下标赋值全文件只有 658、659、661 三行，都在 `set_blocking_bit`（`allocator.rs:654-688`）里；叶计数的写只有 668 与 672，紧跟在同一个函数里算出的「这一槽改前改后是否没挡」之后：「let leaf = self.allocation_record_tree_leaf_position_of_index(index);」（`crates/singlefs-core/src/allocator.rs:664`）。三个字段都是私有，别的文件改不了；`allocator.rs` 里别处（含 `PoolAllocator`，同模块可见私有字段）没有直接写这三张图的代码。起步值（`allocator.rs:562-576`）对着「整个单元区都没挡」算，每片叶是它与单元区相交那一截的槽数。

| 挡位 | 置位 / 清位 | 路径（文件:行） | 经 `set_blocking_bit` |
|---|---|---|---|
| 已分配 | 置 | `mark_allocated`（`allocator.rs:1157`）→ 1174。调用：`PoolAllocator::record`（`allocator.rs:2616`，所有分配含整理搬迁目的地、mkfs 两个单元 `mark_format_time_units` `allocator.rs:2539-2543`）；`rebuild_from_records`（`allocator.rs:2514`，挂载从盘上建空闲图：`mount.rs:3248`、`mount.rs:3762`）；只供测试的开关 `take_every_free_slot_away`（`instance_table_sweep.rs:886`） | 经 |
| 已分配 | 清 | `mark_reclaimed`（`allocator.rs:1068`）→ 1080，调用在 `reclaim_released_records_up_to`（`allocator.rs:3186`），入口 `reclaim_released_up_to`（3111）与 `reclaim_released_up_to_isolating_what_the_isolation_still_protects`（3131） | 经 |
| 已分配 | 不动位的相邻动作 | `mark_released`（`allocator.rs:903`，释放进 defer）与 `mark_resurrected_out_of_the_defer_queue`（回退复活）只改 defer / 记录口径的计数，位图上分配位一直置着（两处都先 `assert!` 分配位置着） | 不碰这三种位，不用经 |
| 影子账隔离 | 置 | `isolate_slot_at_index`（`allocator.rs:971`）→ 973，来自 `isolate`（940）与 `isolate_the_slots_not_allocated`（956）；上面的入口：`isolate_abandoned`（`allocator.rs:3224`，终点 3230；挂载 `mount.rs:1841`）、`isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records`（终点 3219）、`…_referenced_by_an_abandoned_root`（终点 3250；挂载 `mount.rs:1835`、1864、1890）、抬 F 回收后补隔离 `isolate_reclaimed_slots_the_isolation_still_protects`（终点 3572） | 经 |
| 影子账隔离 | 清 | `clear_isolation_of_slot`（`allocator.rs:1012`）→ 1015，唯一调用 `clear_isolation_nothing_still_protected_backs`（`allocator.rs:3548`） | 经 |
| 抬 F 扣住 | 置 | `hold_until_floor_takes_effect`（`allocator.rs:1023`）→ 1029，唯一调用在回收时 `ReclaimedReuse::HeldUntilFloorTakesEffect` 一支（`allocator.rs:3191`） | 经 |
| 抬 F 扣住 | 清 | `release_holds`（`allocator.rs:1048`）→ 1059，调用 `release_reclaim_holds`（`allocator.rs:3200-3203`），挂载里 `mount.rs:5686`、`mount.rs:8785` | 经 |

题里点的几条路径：

- 挂载从盘上建空闲图：`with_unit_area_start`（`allocator.rs:545`）起步全没挡 + `rebuild_from_records`（`allocator.rs:2502`）逐条 `mark_allocated`，再按 mount.rs 里的隔离入口补隔离。全经。
- 回收：上表「已分配 清」。经。
- 整理搬走：目的地走 `record` → `mark_allocated`（经）；源落点走 `release`（只进 defer，`mark_released` 不碰位），之后同「回收」。活 R 不是一种位，不进叶计数（`is_whole_span_open_for_commit_generated` 另判，`allocator.rs:732-747`），所以上界不含 R，只会偏高不会偏低。
- 抬 F：回收 + 扣住 + 补隔离，上表三行。经。
- 回退：复活走 `mark_resurrected_out_of_the_defer_queue`（不动三种位）；回退之后释放 / 回收同上。经或不碰。
- `PoolAllocator` 整个拷贝或换回（失败的发布换回旧分配器）：`DeviceFreeMap` 是 `#[derive(Clone)]`（`allocator.rs:354`），叶计数随拷贝，不会与位图分家。

结论：没找到绕开 `set_blocking_bit` 直接改位的路径。叶计数与位图在同一个函数里同一次转移里改，不变量「叶计数 = 该叶（与单元区相交那一截）里三种位都没置的槽数」逐转移保持：改前改后没挡与否不同才加减，`(true, true) | (false, false)` 不动（`allocator.rs:674`）。`overflow-checks` 开着（`Cargo.toml:24`），万一某条未来的路径让它漂移，会 panic 而不是悄悄选错叶。

上界与逐槽数不一致时取到的叶对不对：上界 = 各盘该叶没挡槽数的最小值（`allocator.rs:2988-2993`），逐槽数的「每块盘都空」是各盘没挡 ∩ 在单元区内 ∩ 不在 R 里的交集的大小，必 ≤ 上界，所以上界永不低估，`allocator.rs:2994` 的跳过不会漏掉更好的叶；一样多取叶序号小的不受影响（升序 + 严格大于）。上界偏高只多做一次逐槽数。最终落点和「空槽数」都是现场逐槽数的（`allocator.rs:2997-3009`），不读计数，计数只管剪枝。所以计数漂移最坏是选不到最优叶（偏低时）而不是选出不合规的槽。

新单测 `allocator::tests::unblocked_slots_per_allocation_record_tree_leaf_are_maintained_incrementally_and_match_a_scan`（`allocator.rs:4873`）覆盖的转移：分配（置分配位）、隔离、扣住、释放后回收（清分配位）；没覆盖：`release_holds`（清扣住位）、`clear_isolation_of_slot`（清隔离位）、`rebuild_from_records` 起步。后两条清位与已覆盖的「清分配位」走 `set_blocking_bit` 里同一条 `(false, true)` 分支，只是 `bit` 不同（`allocator.rs:670-673`），所以结构上同一份代码，但没有一条测试直接盯这两个入口（推的）。

推翻第三节：任何一条路径让叶计数与逐槽数出来的不等——例如在 `set_blocking_bit` 之外给 `allocated` / `isolated` / `held_until_floor_takes_effect` 赋值，或 `DeviceFreeMap` 的字段在别的构造路径上被换掉；现象是新单测里的 `assert_eq!` 红，或运行里 `allocator.rs:668` 的 `-= 1` 下溢 panic。

## 第四节 攻击面 4：钉着旧回落政策推出的落点、单元数、轮数的测试、装置、模型上界与注释

范围：我搜的是 `crates/` 下全部 `.rs` / `.tsv`、`.claude/gate.d/`、`scripts/experiment-apparatus/`（排除 `target/`）。搜法：回落相关词（`FallBackToLowestFreeSlot`、`fell_back_to_the_lowest_free_slot`、`回落`、`最低空槽`、`槽号最小`、`lowest_commit_generated_fallback_slot`）一遍；造「没有全空段」池的词（`384_slot`、`fill_free`、`nearly_full`、`take_every_free_slot_away`、`没有全空段`、`逼满`）一遍；`try_allocate_commit_generated(` 的全部调用点一遍；`*_COUNT: u64 =`、探针调出来的数（`草稿探针`、`现跑`、`量到`）一遍。没有逐个文件通读：384 槽小盘一族有 31 个文件用到（`grep -rlE '384_slot' crates --include=*.rs`），我读了其中带固定数的那几个，别的按「只判通过 / 拒绝、不钉数」归类，没逐个核。

「量的 / 推的」那一列：量的 = 数出自既有的跑出来的产物或任务日志（写明是哪个）；推的 = 读码或手推。「新政策下还对不对」只回「对」「不对」「推不出」。

### 4.1 补丁已改过的钉值（背景材料列出，我核了它们现在的样子）

| 编号 | 文件:行 | 钉的是什么 | 新政策下 | 量 / 推 |
|---|---|---|---|---|
| A1 | `crates/singlefs-verification/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs:170`（文档 165 行「回落到叶 61 里最低的空槽 50179」） | 没有全空段时每块盘的落点序列（50179、50182、50184、50186–50194，实现员报告：钉值未改，只改名与文档） | 对（叶 61 恰是重写集的叶，落点逐个相同） | 量的：任务 13428 该目标「8 passed」（实现员副本上） |
| A2 | 同文件 `:252` | 越过影子账隔离的槽取 50182，入口改带叶 61 重写集 | 对 | 量的：13428 |
| A3 | 同文件 `:586` | 抬 F 三次空发布各取叶 62 里两块盘都空的 8 个槽，共 24 个 | 对 | 手算 + 量的：13428 |
| A4 | 同文件 `:767`、`:783` | 只留 10 / 18 个最低共同空槽（都在叶 61），第 2 / 3 次空发布取不到落点被预演拒 | 对 | 量的：13428 |
| A5 | `crates/singlefs-verification/tests/common_images_for_the_given_states_engine/admission_raises_the_floor_before_refusing.rs:369` 「卸载那一串两次空发布要 12 个」 | 卸载那一串空发布要的单元数 14 变 12 | 对 | 量的：A/B 13860（现行政策读 12）、13861（还原读 14），整目标 13862「26 passed」 |
| A6 | `crates/singlefs-verification/tests/allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf.rs:35` 「const ROUNDS_OF_THE_SETTLING_ON_THE_FORGED_DISK: usize = 4;」 | 锻造盘上固定点 4 轮 | 对 | 量的：探针读数 4、任务 13429「1 passed」 |

### 4.2 没改、没跑到、数是旧政策下量出来或调出来的（会红的候选，按风险排）

| 编号 | 文件:行 | 钉的是什么 | 新政策下 | 量 / 推 | 风险 |
|---|---|---|---|---|---|
| T1 | `crates/singlefs-verification/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs:49`「覆盖写 19 次之后第 20 次…那一串三次空发布在预演里第 1 次就取不到落点」与 `:117`「覆盖写 18 次之后崩了再挂（空间准入判着）」 | 384 槽小盘（6 MiB 环，单元区 [1408, …)）上覆盖写撞墙的确切步数：19 次后第 20 次被拒、18 次后挂载的抬 F 那一串第 1 次就拒；文档写「草稿探针在新几何上逐次覆盖写现跑的数」，并写明换几何时这些数整体挪过（17→18、18→19） | 推不出，多半挪：撞墙的步数由每次发布重写的节点数决定，回落落点换了就换节点数（同一页文档 `:51` 自己说过「每次发布重写的节点跟着变，撞墙的步数与那一串的次数都变」） | 量的（旧政策）；新政策下没量 | 高 |
| T2 | `crates/singlefs-verification/tests/admission_checkpoint_cost_per_device_paths.rs:1153`「more_than_two_leaves_on_one_device > 0,」，文档 `:1099` 「K1 下 144 次空发布里一块盘改的叶多于两片的有 4 次（实测）」 | 历史 `unit-area-2600-filled-seed-13` 这一段（没有全空段、回落一个单元一处）必须出现至少一次「一块盘改多于两片叶」，否则前提断言红；文档里的 4 次、12/8、第 22/58/137/154 步都是旧政策实测 | 多半不对：新政策的目的就是减少一块盘改的叶数（先取重写集的叶），这条前提断言要的恰是旧政策的坏形；整体上界断言（`fits_in_the_whole_tree_checkpoint_cost`，取几何整棵树）不受影响 | 量的（旧政策）；新政策推不出 | 高 |
| T3 | `crates/singlefs-verification/tests/crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream.rs:78`「const FULL_STATE_COUNT: u64 = 16_920_659;」，文档 `:77`「126 次写、26 段」，测试 `the_recorded_stream_counts_its_crash_states`（`:998-1011`，`assert_eq!(states, FULL_STATE_COUNT, …)`），同数还写在 `.claude/gate.d/crash-point-replay-items.tsv:76` 的说明里 | 贴边池上最后一个单元 + 拒写 + 卸载这条流的写次数、段数、全量崩溃状态数 | 推不出：这条流在 384 槽小盘贴边处，提交内生块走回落；落点换了，每次发布重写的叶数可能变，写次数与状态数跟着变。红的话只有这一条断言红，不涉及崩溃判定 | 量的（旧政策下录制流闭式计数，第二台录的） | 中高 |
| T4 | `crates/singlefs-verification/tests/admission_raises_the_floor_for_the_deletion_of_another_inode.rs:19-20`「const OVERWRITES_BEFORE_THE_UNLINK_WHOSE_DELETION_BATCH_IS_REFUSED: usize = 12;」「…_TO_BE_REFUSED: usize = 30;」 | 384 槽小盘上覆盖写 12 次、每块盘留 30 个没挡的槽，才是「意图放行、删除批被落得下那一判拒、推抬 F 之后做成」那一形；文档写明靠探针在 28–34 槽带上现跑，带宽窄（覆盖写 8、11 次时只在 34 槽） | 推不出：带的位置由每次发布占的槽数定，回落政策变了带会挪 | 量的（旧政策，草稿探针） | 中高 |
| T5 | `crates/singlefs-verification/tests/random_histories.rs:827`「(32, 0, 0, 1141),」（测试 `:787` `unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_…`） | 256 槽小盘、32 段 × 150 步逼近单元区墙、空间准入关着：全跑完、落点拒绝 0 次、池级 checker 跑 1141 次（「2026-10-03 量」） | 推不出：1141 随每步发布是否成功而变，新政策让原来被拒的发布成功时数会变；0 次落点拒绝也可能因 `DidNotSettle` 之类新拒绝而不再是 0 | 量的（2026-10-03，旧政策） | 中 |
| T6 | `crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:56`「const UNBLOCKED_SLOTS_LEFT_FOR_THE_FLOOR_RAISE: u64 = 400;」与 `:65`「const OVERWRITES_BEFORE_THE_ACCEPT: usize = 14;」 | 2048 / 8192 槽盘上覆盖写 14 次、每块盘留 400 槽，造抬 F 的空间场景 | 推不出（我没读它们的断言是否贴着这些数的边） | 调出来的参数；没读到出处 | 中低 |
| T7 | `.claude/gate.d/crash-point-replay-items.tsv:70`（`truncate-to-zero-on-a-nearly-full-pool` 说明里「12 段」） | 贴边池截到 0 这条流分 12 段 | 推不出，随写流形状；测试文件里没有写死这个数（`crash_enumeration_truncate_to_zero_on_a_nearly_full_pool_stream.rs` 对比的是闭式计数 `closed_form`，自洽） | 量的（旧）；只是说明文字 | 低 |

### 4.3 没改，读下来仍对

| 编号 | 文件:行 | 钉的是什么 | 新政策下 | 量 / 推 |
|---|---|---|---|---|
| C1 | `crates/singlefs-verification/tests/unequal_devices.rs:388` 与 `crates/singlefs-verification/src/model_comparison.rs:1774`「CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(SlotNumber(196_638))」 | 一块盘开段、一块盘回落时的拒绝里各盘自答的去处 | 对：一开段一回落不进池级（`allocator.rs:2859` 的 `all` 为假），拒绝带的仍是各盘自答 | 推（读码）；任务 13430 `unequal_devices`「3 passed」是量的（实现员副本） |
| C2 | `crates/singlefs-verification/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs`（计数 0 与 >0 两类断言） | 重复挂载加第一个文件那一族上回落计数为 0；盘上不剩全空段时计数 >0 | 对：新政策只换「回落落在哪」，不换「何时回落」；计数在 `allocator.rs:2920` 仍加一 | 推 |
| C3 | `crates/singlefs-verification/tests/core_review_unit_area_start_and_publish_limits.rs:453`、`:469` | 每盘 `lowest_commit_generated_fallback_slot` 从环末尾的下一槽起 | 对：该函数没改（`allocator.rs:1262-1289`）；名字里的「回落」指各盘自答 | 推 |
| C4 | `crates/mutations.tsv:77`、`:78`、`:2267`（`增补 2：提交内生块段耗尽不回落…`、`26-3 分配器回落计数不加`） | 把每盘回落答案改成 `None` / 计数不加，对应测试要红 | 对：改成 `None => None` 之后每盘答不出 ⇒ 不进池级 ⇒ 拒；计数行同 | 推；背景材料说十一行变异分两片抓到 5 + 6 条（量的，任务 13806），我没核 |
| C5 | `compaction_user_change_clears_the_last_completed_intent.rs:162`、`compaction_off_the_calling_thread_and_admission.rs:648`、`compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs:168` 与 `:281`、`compaction_rollback_in_the_session_clears_the_last_completed_intent.rs:126`、`compaction_plan_chooses_regions_and_batches_by_the_clauses.rs:895` | 直接调 `try_allocate_commit_generated(OneSlot…)` 当填充 / 开段 | 对：填充在开放段里 bump，开段是 R 外最低全空段，都不走回落 | 推（读了循环条件） |
| C6 | `crates/singlefs-core/src/allocator.rs` 测试模块里旧的 `allocate_commit_generated` 回落测试（`:3601`、`:3713-3733`、`:3825`、`:3947`、`:4066`） | 回落与开段的老用例 | 对 | 量的：任务 13432 在核心库里跑了 `allocator::tests`，「49 passed」、377 条被过滤掉（实现员副本，补丁后的树；只是那一个模块，不是整个核心库） |

### 4.4 模型上界、注释、装置

| 编号 | 文件:行 | 钉的是什么 | 新政策下 | 量 / 推 |
|---|---|---|---|---|
| D1 | `crates/singlefs-verification/src/model.rs:4210`「落点按 D3（空间分配） 已定项 10 从最低处取（开新段取最低的全空段，回落取最低的空槽）」与 `:4211`「落点都在单元区起点往后 `64 × P` 个槽里」；用到上界的数：`model.rs:10501`（9）、`model.rs:10496`（529）、`model.rs:11515`（123）、`model.rs:5488` 占槽上界 | 分配记录树一次发布至多重写几个节点：假设落点都在起点往后 64×P 个槽里 | 数不变（这几个测试在新池上算，没有回落）。论证在新政策下没证全：回落发生 ⇒ 每个整段都有占用 ⇒ P ≥ 整段数 ⇒ 64×P 罩满整段区，这一步成立；但缺口两处：① 单元区尾部不满 64 槽的那一截不算整段（`allocator.rs:1239` 的 `full_segments = self.allocated.len() / per_segment`），可以全空而不挡「没有全空段」，新政策取空槽最多的叶可能落进去，旧政策要等前面全满才到；② 「整段都有占用」可以由隔离或扣住造成，不算进模型的 P。实现员报告「交主 agent 的问题」第 4 条也说「这一角没推完」。注释字面「回落取最低的空槽」已不是实话 | 推的；模型是独立模型（不与实现共用代码），按 `implementation-first.md`「规矩」第 4 条两边都该查，我没跑 |
| D2 | `crates/singlefs-core/src/admission.rs:981`「回落到最低空槽挡的都只是这三种位」；`:1188`「没有全空段回落到最低空槽」与 `:1189`「量到一块盘改 3–4 片叶」 | 第一处说挡位口径（三种位）；第二处说分配记录树那一项为什么取几何整棵树当上界（少扣三种情形，量到一块盘改 3–4 片叶） | 第一处：挡位口径仍对，「最低空槽」字面过时。第二处：判据（取整棵树）不受影响，因为取的是几何上限，不依赖落点；「3–4 片叶」是旧政策下 A4c 的实测，新政策下只会更少，注释数字不再是现值 | 3–4 是量的（旧政策）；其余推的 |
| D3 | `crates/singlefs-core/src/compaction_plan.rs:475`「（提交内生块回落那一个函数）」 | 只供测试的目的地政策取「起点最小、整跨度没挡」的落点，注释称它是提交内生块回落那一个函数 | 字面过时：产品的回落已是池级政策，这个只供测试的政策与之不同，用它的整理测试钉的是最低槽，不受影响 | 推 |
| D4 | `crates/singlefs-verification/src/history.rs:140`「回收之前几十次发布就写满」 | 384 槽小盘一段历史里写满的发布次数级 | 「几十次」是级，不钉数；回落阶段的每次占槽数变了也大概率仍是几十次。字面不用改 | 推 |
| D5 | `crates/singlefs-core/src/transaction.rs:13982`「这几个点都有全空段、走不到回落」，指 `.claude/kb/experiments/206-发布固定点轮数的代价.md:126` 的「16 个点 p2a 最大 5 轮」 | E206 第一段 16 个取样点的轮数上界 5 | 与新政策无关（这些点不走回落）；「都有全空段」是实现员写在注释里的，我没去 E206 产物里逐点核 | 5 轮是量的（E206 产物）；「都有全空段」是推的，没核 |
| D6 | `crates/singlefs-core/src/recovery.rs:6677`「const ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS: usize = 8;」 | 新池新建文件预演节点数调几轮 | 对（预演走空重写集，有全空段时走不到回落）；注释写明「没量过会不会到第三轮」 | 推的 |
| D7 | `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py:789`（`s29_fallback_scan`，锚在 `pub fn lowest_commit_generated_fallback_slot_outside_the_region(` 那一整个方法，该签名今天仍在 `allocator.rs:1271`）、`:70`（计数名 `fallback_slot_linear_scan`）；`scripts/experiment-apparatus/e206_fixed_point_cost.rs:298`、`:417`、`:801`、`:1067`「fallback_finds: calls_now("fallback_slot_linear_scan").min(round.records_changed),」 | 回落的代价按「每盘自答的线性扫描次数」计数，E206 把它乘进每轮代价模型 | 计数点仍会被调到（每次回落每块盘一次），所以装置还能打补丁、能出数；但新政策多出的池级叶扫描（`allocator.rs:2987-3011` 的 O(叶数) 加若干片叶的逐槽数）不在计数里，代价模型对新政策不全。已有产物是旧政策下的量，留存不回改 | 量的（旧）；新政策下的完整性是推的 |
| D8 | `scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs:451`、`:520`「allocator.commit_generated_placements_that_fell_back_to_the_lowest_free_slot()」 | 装置直接用真分配器，读回落次数 | 重跑读数会变（落点不同，后续是否再回落也变）；已有产物是旧政策下的 | 量的（旧） |
| D9 | `scripts/experiment-apparatus/e151_arrival_and_container_arms.rs`、`e93_aging_placement.rs`、`e10_aging_compaction.rs`（`:1086`「回落到槽号最小的空闲槽」）、`e187_quarantine_mark_cost.rs`（`:911`）、`e142_new_pool_file_creation_dry_run.rs` 等 | 各自模型里的「最低空槽」回落 | 这些装置不共用实现代码（`grep -c singlefs_core` 在 e206、e142、e187、e10、e151、e93 上都是 0），是历史实验的独立模型；它们的回落不再等于实现的回落，按 `implementation-first.md`「规矩」第 4 条是「装置与实现对不上，两边都查」，但历史产物属于冻结证据，不回改 | 推 |

### 4.5 小结

补丁带出的钉值只改了 6 处（4.1）；我认为还有 T1–T5 这五处（4.2）旧政策下量出来、调出来的数在新政策下推不出，多半会红，背景材料列的任务里没有跑到它们的；T6、T7 风险低。4.3 的六类读下来仍对。模型上界的论证有两个缺口（D1）。

推翻 4.2 的现象：在补丁打上之后的树上跑 T1–T5 对应目标全绿（那时「多半会红」的推测作废，这些数在新政策下碰巧不变）。要跑的命令见下一节。

## 要主 agent 跑的命令

我没跑、没交任何任务。下面六条是 4.2 的 T1–T6 各一条（那几格「新政策下」写的是推不出，待主 agent 跑）。每一条：在仓根 `/home/fy5090/code/singlefs` 起；跑完看 `task-server/task log <号>` 里的 `test result:` 行与 `TASK_SERVER_RESULT` 那一行的 `exit=`；全绿（`exit=0`、`ok`）= 那一格「碰巧不变」，T 行的推测作废；红了照红的断言名回 4.2 对应行，数是新政策下的实际值，由实现员手算核、不直接抄进钉值。

1. T1：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 a_floor_raise_refused_for_space_counts_as_short_of_space; 范围=1 个测试目标; 为什么=回落换了落点政策之后核旧政策下探针调出的撞墙步数 19 / 18 次还对不对（正推腿报告 4.2 T1）' task-server/task submit -- cargo test -p singlefs-verification --test a_floor_raise_refused_for_space_counts_as_short_of_space`
2. T2：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 admission_checkpoint_cost_per_device_paths; 范围=1 个测试目标; 为什么=核「至少一次一块盘改多于两片叶」的前提断言在新回落政策下还成不成立（正推腿报告 4.2 T2）' task-server/task submit -- cargo test -p singlefs-verification --test admission_checkpoint_cost_per_device_paths`
   （红的话看是不是 empty_publishes_of_the_histories_where_one_device_rewrites_more_than_two_leaves_fit_in_the_whole_tree_checkpoint_cost 红在「至少一次空发布一块盘改的叶多于两片」。）
3. T3：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream（只跑只录流的一条）; 范围=1 个测试目标; 为什么=核录制流闭式计数 16_920_659 在新回落政策下变没变（正推腿报告 4.2 T3）' task-server/task submit -- cargo test -p singlefs-verification --test crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream -- the_recorded_stream_counts_its_crash_states`
   （输出里 CRASH_ENUMERATION_STATE_COUNT 那一行给出 states 与 writes 的现值。）
4. T4：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 admission_raises_the_floor_for_the_deletion_of_another_inode; 范围=1 个测试目标; 为什么=核覆盖写 12 次 + 留 30 槽那一形在新回落政策下还在不在（正推腿报告 4.2 T4）' task-server/task submit -- cargo test -p singlefs-verification --test admission_raises_the_floor_for_the_deletion_of_another_inode`
5. T5：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 random_histories（一条）; 范围=1 个测试目标; 为什么=核 (32, 0, 0, 1141) 在新回落政策下变没变（正推腿报告 4.2 T5）' task-server/task submit -- cargo test -p singlefs-verification --test random_histories -- unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall`
6. T6：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 accept_raises_the_floor_for_space; 范围=1 个测试目标; 为什么=核 400 槽 / 14 次覆盖写那组调出来的参数在新回落政策下还成立（正推腿报告 4.2 T6）' task-server/task submit -- cargo test -p singlefs-verification --test accept_raises_the_floor_for_space`

模型：这一轮我没写模型，没有 `rerun.sh`、没有 sha256 要报。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 攻击面 1 逐句对 | 一致 | 八句政策都落了行（1.1），无越位；四处自定取法（1.2）和「各盘自答不同时现在成功」与 kb 末段字面的张力（1.3）交主 agent 裁 |
| 攻击面 2 固定点终止 | 一致（演练 = 真发）/ 规则没说（终止） | 六个调用点同一串角色同一个集合；走满 64 轮的一类输入推得出（新旧同），来回换没构造出也没证；预演走空集的边角没核 |
| 攻击面 3 每叶计数 | 一致 | 三张位图只在 `set_blocking_bit` 写，叶计数同函数同转移里改，没有绕开的路径；上界不低估 |
| 攻击面 4 旧钉值 | 冲突（推） | 补丁改了 6 处钉值（4.1）；T1–T5 五处推不出而多半红，T6、T7 低，D1 模型上界有两个缺口（4.2、4.4） |
| 检查单 走不到的理由 | 成立 | 三条 `expect` 的范围成立；漏列的下标 / 减法 panic 面靠不变量成立 |
| 检查单 类型与名字 | 不成立 | `FallBackToLowestFreeSlot` 与回落计数名已不是实话；叶序号 / 槽数 / 槽号共用 `u64`；`expect` 没写靠哪条不变量 |
| 检查单 不兼容旧代码 | 成立 | `legacy\|old_format\|migrat` 零命中；空重写集入口有产品路径调用者，不是兼容层 |
| 检查单 不新建判定库 | 成立 | 补丁只动四个文件，变异行进既有 `crates/mutations.tsv` |
| 检查单 多机可用 | 成立 | 新测试在进程里造盘，无本机独有状态 |
| 检查单 测试多线程 | 成立 | 一条用例一个场景；唯一线程只为 64 MiB 栈 |
| 检查单 带 --item | 推不出 | 8 项名字都在合表里，但是按源码字面挑的，不是量走没走回落；384 槽随机历史那几条没点 |
| 检查单 新崩溃流 | 推不出 | 没加流；「已有流罩得到回落第二步」是推的，没有哪条登记流被量过走到 |

## 没做什么

- 没跑任何项目代码、没交任务、没写模型；T1–T6 都待主 agent 跑。
- 384 槽小盘一族 31 个文件我只读了带固定数的几个（4.2 前言写明），别的按「不钉数」归类，没逐个核。
- 没核 E206 产物里每个取样点是否都有全空段（D5），没核 `crates/mutations.tsv` 新增 7 行与替换 3 行是否真抓到（背景材料说任务 13806 两片抓到 5 + 6 条，我没读那份日志）。
- 没核 `mount.rs` 以外的恢复路径里别的"预演 = 真发"断言（只核了 `mount.rs:5681` 那一处与 `recovery.rs:6617`）。
- 来回换（2.2）没构造出输入，也没证不存在。
- 不判别：不采纳、不出判决；kb 写回哪几句（已定项 8 第 2 条与末段）是主 agent 的事。
