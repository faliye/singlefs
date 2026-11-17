# d3-fallback-placement-r1 攻方腿（第一轮：过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱）

日期：2026-11-09。这一段交回时模型一次都没跑：下面**每个新数都是「推的，待主 agent 跑」**；标「r3 旧数」的是任务 11382 的副本产物（`~/.local/share/singlefs/work/m4-close-fixes-code-r3-opus/rerun-1/observations.txt`），这一腿只读没复跑，当线索。

## 复跑

`bash research/prompts/d3-fallback-placement-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新目录>`（整条交服务，见末尾「要主 agent 跑的命令」）。

```
a14ac31ef0c298236dcf4c5bc7f1aa17915bda22cc107dbaead7a4f12212b4d3  research/prompts/d3-fallback-placement-r1-opus-model/tools/apply_fallback_arms.py
77238a85ad52db61b067df9adb91faf7a3a018699a494a9474e318be1bb3ca9f  research/prompts/d3-fallback-placement-r1-opus-model/tools/rerun.sh
2dd86ab06f648f260196f4d7d84c992dd8b51db79afdab6d569436f3a68e24c8  research/prompts/d3-fallback-placement-r1-opus-model/tests/attack_d3_fallback_arms.rs
```

## 各格判定一览

| 格 | 判定 | 依据状态 |
|---|---|---|
| 「优先重写集、不够退最低空槽」修不修得了 `did_not_settle_limit_64` | **打中：修不了**，那几探上逐探等于今天 | 推的（下面有证明与 r3 旧数的拟合），待主 agent 跑 |
| 修得了的改法要改哪 | 只有「不够时退到哪」那一半改了才修得了；「优先重写集」这一半本身不起作用 | 推的，待跑 |
| 变差形状：只用用户操作老化后删光一段再写 | **打中候选**：提交内生块回到正在腾的段，全空段数比今天少，前台写先被合取 7 拒，撞 D3 已定项 9 第 2 条 | 推的，待跑；跑前写明量不到就撤回 |
| 多盘：每块盘各答 | **打中**：两盘差不到一段时今天不拒的地方拒；只有池级定义过得去 | 推的，待跑 |
| D3 已定项 8 第 2 条 | 字面要改（用户已定，不算打破）；「该设备内」三字与第 5 条末段要合写 | 现查条文 |
| D3 已定项 8 第 1 条、已定项 5、已定项 10 | 没打破：候选集不变，用户数据不走这条路，① 开段次序不动 | 现查条文与代码 |
| D26 已定项 3 底额 | 条文没打破，**行为**上被上面那条变差形状打到 | 推的，待跑 |
| D26 已定项 1 第 3 条（`R` 排他） | 没打破：排除仍在候选判断里 | 现查代码，模型补丁照带 |
| checker 不变量 | 不用动：落点仍在今天的候选集内，`crates/singlefs-checker/src/` 里 grep 回落零命中；I-5.5 只管用户数据与从 mkfs 水位建树那一版 | 现查；老化段每条序列末尾判池级 checker 待跑；**崩溃状态没枚举** |
| 固定点内部 | 代码注释「走得完」那条前提不再成立，要重写；振荡没构造出来 | 现查加推的 |
| 最脆弱的一条 | 「改落点就关掉 64 那一格」这条因果 | 见第三节 |

## 一、过度外推与越位

### 打中 A：优先重写集这一半，在 5 次探的形状上不改变任何一个落点

被判的句子：题面第二节把 5 次探列成要修的格，第三节第 3 问问它们还撞不撞。

**证明（推的）**。今天的回落取最低空槽：`crates/singlefs-core/src/allocator.rs:2570`：「最低的全空段（开新段），没有全空段就回落到槽号最小的空槽」。造法（`research/prompts/m4-close-fixes-code-r3-opus-model/tests/attack_fixes_r3_settling_fallback.rs:76` 的 `forge_the_free_map`）让第 12 片叶之前一个空槽都没有，之后每片 F 个。

1. 上一版的单元都在第 12 片叶之前（四次覆盖写在空盘上），释放链走到的叶里没有空槽，所以重写集里的空槽全部来自上一轮取落点碰到的叶。
2. 上一轮取走的是最低的 `L` 个空槽，碰到的叶是一段连续的叶号前缀，除最后一片外都取空了。
3. 这一轮要的数 `L'` ≥ `L`（重写集只增不减）。按叶升序先取重写集的空槽，正好是最低空槽串接着往下取；不够时退最低空槽，还是同一串。
4. 所以这一串落点与今天逐个相同，往下每一轮的重写集也相同。归纳完。

换任何子集形式的偏好（按段、只取部分叶）结论不变：取走的集合仍是最低 `L` 个，只是次序变；只要「不够时退最低空槽」不改，下一轮碰到的叶就不变。

**动力学（推的，拿 r3 旧数对过一遍）**。一片叶对一块盘一个节点，两块盘同槽：碰到一对叶多要 2 个落点，给 F 个。F = 2 净赚 0，缺口永远不缩，每轮往后多走 `缺口 / 2` 对叶。于是轮数约等于 `F × 片数 / 缺口`，缺口 = 基础角色数 − 稠密前缀。r3 旧数（前缀扫描，每片 2 空槽 120 片）：

```
prefix=8 S rounds=33
prefix=10 S rounds=43
prefix=12 S rounds=63
prefix=14 DNS rounds=64
prefix=16 S rounds=3
```

缺口 8、6、4 分别对 33、43、63，约 252 / 缺口；前缀到 16 缺口归零、3 轮定下。F = 3 净赚 1，每轮衰减，旧数 5 轮；F = 1 净亏，旧数 8 到 10 轮但每轮翻倍。**这 5 次探是这条线撞到 64 这个数，不是某个局部形状的毛病**：片数够多、F = 2、缺口小于片数乘 2 除以 62 就撞。

**推翻条件**：模型 `forged_summary arm=prefer_then_lowest` 的 `did_not_settle` 少于 `arm=today`，或任何一探的 `rounds`、`roles` 与 today 不同，这一格撤回。跑前估的数：today 这批形状 8 探撞（前缀 14、15 两探加两组形状各 3 探，形状组里最差前缀就是 14，推的）。

### 打中四问（打中 A）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨。`prefer_then_lowest`、`prefer_then_lowest_on_each_device`、`..._sparing_...` 与今天同中；`prefer_then_richest_leaf`、`prefer_then_lowest_leaf_with_more_than_two_free` 预期不中 |
| 系统看不看得到判别它的东西 | 看得到：重写集在 `settle_the_allocation_record_tree` 手里，每片叶的空槽数分配器扫得出 |
| 满足判据的哪一分句 | 题面第三节第 3 问「`did_not_settle_limit_64` 那 5 次探在新政策下还撞不撞」的「撞」 |
| 每个改法在这几格上还中不中 | 见第四节的表 |

### 越位：把锻造盘上的动力学当自然盘的

造法不写分配记录：`crates/singlefs-verification/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs:10`：「不写分配记录」。所以锻造盘上新碰到的叶是**新节点**，没有旧落点，释放链不延长。自然盘上每片叶都有记录，碰一片就要重写它、释放它的旧落点、弄脏旧落点所在的叶。净赚的门槛因此不是 F > 2，要看闭包带来几片叶、那几片叶有多少空槽。r3 的「F ≥ 3 就衰减」这句不能搬到自然盘（推的）。

r3 旧数的自然段：`natural_summary` 三条 `most_rounds=3`，`fell_back_so_far=28`，全程不重挂。**这段历史没有重挂**：写到被拒时 `empty_segments=15`，多出来的空槽都关在这次挂载开过的段里（`.claude/kb/decisions/03-空间分配.md:168`：「重开之后（bump 指针与「这次挂载开过哪些段」都只住内存，挂载后新开一段，已定项 10 ①），上一次挂载开过、装着提交内生块的段对用户数据开放，混放允许」）。重挂之后那些段对数据开放，写得进去，才走得到题面要的稀疏形状。所以题面「`natural` 那一段没走到这个形状」不能读成「用户操作走不到」。老化段（模型第四段）补这一格。

准入这一头也把形状往那里推：落地判要的提交内生块槽数按整棵树算（`crates/singlefs-core/src/admission.rs:1014`：「拿 ckpt_cost 的最坏数」），两块盘每片叶一对节点，所以写到被拒时没挡的槽数大约只剩**每对叶 2 个多一点**（推的）。平均 F ≈ 2 正是净赚 0 的那条线，这不是巧合，是准入的上界决定的。

## 二、内部矛盾与循环依赖

### 打中 B：「每块盘各答」与已定项 8 第 5 条末段不能同真

条文两句：`.claude/kb/decisions/03-空间分配.md:168`：「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」；`.claude/kb/decisions/03-空间分配.md:173`：「第一版盘不等大时，各盘落点答案必须相同」。今天这两句同真靠一件事：最低空槽在两盘同样被占时天然同号。优先重写集之后这件事不再成立。

**历史**（盘是锻造的，用户操作走到它没跑，这一格写明）：

1. 两块盘差 1 到 63 槽（现实里同型号盘差这么一点是常见的，推的）。`crates/singlefs-core/src/allocator.rs:1006` 的段数向下取整，多出来的几槽凑不成段，`lowest_empty_segment` 两盘都答没有，走回落。今天两盘都答最低共同空槽，一致。
2. 小盘最后一片叶在重写集里（那片叶里有上一版的单元），大盘多出来的槽正落在这片叶的盘号里。
3. 各答的臂：大盘答它多出来的槽，小盘这片叶没空槽、退最低空槽，两盘不同，走 `crates/singlefs-core/src/allocator.rs:2608` 的 `AnswersDiffer` 拒成 `CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined`。**今天不拒的地方拒了**，而且是在无空段时，正是最不能拒的时候。

模型第三段（`devices_differing_by_less_than_a_segment_under_each_arm`）是分配器级的：多 1、33、63 槽。预期：`today` 与池级各臂答 `lowest_common_free`，`prefer_then_lowest_on_each_device` 三格都 `Err`。**推翻条件**：这一臂给 `Ok`。

⇒ 第 1 问的答案只能是池级定义：**每块盘都在重写集里的叶，每块盘都空的槽**。这一定义让今天「两盘最低空槽不同就拒」那条路在回落上走不到，`crates/mutations.tsv:74` 那行变异（各盘给提交内生块的去处不同也不拒）要重看：拒绝分支只剩开段与回落混答、开的段不同两种。这是定义的连带，不是副作用，要写进条文。

### 没打破但要改的一句：固定点终止论证的前提

`crates/singlefs-core/src/transaction.rs:13812`：「多一个角色只把这一串拉长，有记录的节点只多不少，重写集也只增不减，都以池里的节点数为上界」。今天落点不依赖重写集，所以「一串拉长」成立。新政策下 **落点 = f(重写集)**，这是一条新边：重写集 → 落点 → 记录 → 重写集。一个 `P` 里新进来的较低的叶，会把原来落在较高叶 `M` 的单元吸走，`M` 内容不再变而留在重写集里（重写集只增不减）：**多罩一片叶**。合法（同一段注释「多罩的节点内容没变、照样重写一份（合法的 COW）」），但今天这条路上没有。量：`rich_release_leaf` 与老化段的 `roles_sum` 对比 today。

终止本身还在：重写集仍只增，节点数有界。`holding_records` 不再单调是新的，见「没打中的形状」。

真发那一半也要带上同一个偏好集：`crates/singlefs-core/src/transaction.rs:13808`：「真发时走的是同一串释放与取落点，得出同一份记录」。今天不用传东西，新政策下**要传**：`publish_admitted`、写行发布两处真发，加两个固定点，一共四个调用点；传漏一处，预演与真发落点不同。`crates/singlefs-core/src/recovery.rs:6577` 那个排演不走固定点，只在单元区开头两段里排演，走不到回落（推的），可不动。模型补丁的四处在 `tools/apply_fallback_arms.py`。

## 三、哪一条最脆弱

### 最脆弱：「改落点政策就关掉 64 那一格」这条因果（打中 A）

它只有在「重写集的叶里有最低空槽串之外的空槽」时才起作用，而 5 次探的形状恰好没有。真正关掉这一格的不是「优先」，是**不够时退到哪**（第 1 问的第二小问）：往净赚的叶退，缺口一轮就补上。

上界 64 本身也不是落点问题：落地判已经按整棵树扣过槽（`crates/singlefs-core/src/admission.rs:1186`：「分配记录树那一项不读这一版有哪些节点」），所以走得到固定点的发布，整棵树重写都放得下；撞 64 的发布不是没有解，是走得慢（推的）。这句只说明白撞的是什么，要不要改用户已定，这里不提。

### 打中 C（候选）：优先重写集把提交内生块放回正在腾的那一段

被判的句子：`.claude/kb/decisions/03-空间分配.md:197`：「删掉一个 s 字节的对象之后，同样大小的写必须在有界步数内成功」；`.claude/kb/decisions/26-后台整理与放置回收.md:104`：「形态是准入的**第二个合取式** `(可用 ≥ 需求) ∧ (全空聚簇段数 ≥ N₀)`」。

**历史**（只用用户操作，模型第四段；每步许可它的那一句）：

1. mkfs，一个单元的小文件写到前台被拒，进程退出重挂，再写，几趟。重挂让上次开过的段对数据开放（`.claude/kb/decisions/03-空间分配.md:168`：「上一次挂载开过、装着提交内生块的段对用户数据开放，混放允许」）。
2. 走到按记录全空段数 0、没有可开的段。前台写被合取 7 拒（`crates/singlefs-core/src/transaction.rs:13320`：「只拦前台写入类发布」）。
3. 用户删光**高处一段** `S` 里的文件。删除不被合取 7 拦，无空段走回落。今天：提交内生块落低处的单槽空洞。
4. `S` 的文件一个个释放。释放弄脏 `S` 所在的叶，那片叶进重写集。回收之后 `S` 里有空槽。
5. **优先重写集的臂**：提交内生块落回 `S`（那片叶里别的段装满数据，只有 `S` 有空）。今天的臂不碰 `S`（低处有空洞时）。
6. `S` 里因此总有未释放记录（`crates/singlefs-core/src/allocator.rs:906`：「一段里只剩 defer 槽时这里算它全空」说的是全空的口径，有未释放记录就不算）。根与中央映射这类每次发布都重写的单元落下一轮又在 `S`：`S` 的旧落点释放、回收、再被取回，它成了提交内生块的家。
7. 全空段数停在 0，前台写一直被合取 7 拒。**用户删了一整段，写不进去。**今天：`S` 按记录全空，合取 7 过，回收之后还能开段。

**只在**低处有空洞时分辨臂；低处没有空洞时今天也落回 `S`，两臂同中。

**反面也要量，这是两条臂都该认的**：今天的臂把 `S` 开成聚簇段之后，`S` 在这次挂载里不给数据（第 2 条），新写的数据单元可能因此没有段外的槽对；优先重写集的臂 `S` 没开段，`S` 的槽对反而给数据。所以「写」这一头可能反过来是新政策赢。模型 `aged_delete_summary` 的 `written` 与 `first_refusal` 就是看这个，全空段数与写成次数分开报。

**推翻条件（跑前写下）**：`arm=prefer_then_lowest` 的 `after_the_unlinks` 与 `aged_delete_summary` 上 `empty_segments` 不比 `arm=today` 少，或 `written` 不比 today 少：这一格撤回，交回写明「变差那一格没量到」。扫的用户动作：删哪里（最低、中间、最高）× 删几段（1、4）；另两条散删加随机覆盖写（隔 8、64 个删一个，400 步）。

### 打中四问（打中 C，待跑才算数）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 预期分辨今天与所有先取重写集的臂；`..._sparing_...` 只防段已按记录全空之后，防不住腾的过程，预期部分起作用 |
| 系统看不看得到 | 看得到：`unreleased_per_segment` 在分配器里；「这一段正在被腾」看不到，只看得到「这一段现在没有未释放记录」，这是 `sparing` 臂防不全的原因 |
| 满足判据的哪一分句 | 第 3 问「有没有反过来变差的形状」；条文上是已定项 9 第 2 条的「有界步数内成功」，不是第 1 条的假性 ENOSPC：`df` 那一半要另核 |
| 每个改法还中不中 | 见第四节的表 |

## 四、改法与代价

第 1 问的候选写成臂，都只在我的模型上，**被攻过零轮**。

| 臂 | 定义（都排除 `R`、排除三种挡位） | 64 那几探 | 重写集叶自己富 | 盘差不到一段 | 删光一段再写 |
|---|---|---|---|---|---|
| `today` | 每盘最低空槽 | 撞（r3 旧数） | 走稀疏叶（推的，待主 agent 跑） | 不拒（推的，待主 agent 跑） | 基线 |
| `prefer_then_lowest_on_each_device` | 每盘各答：本盘重写集叶，再本盘最低 | 撞（推的，待主 agent 跑） | 省（推的，待主 agent 跑） | **拒**（推的，待主 agent 跑） | 差（推的，待主 agent 跑） |
| `prefer_then_lowest` | 池级：重写集叶按升序，再最低共同空槽 | **撞**（推的，待主 agent 跑） | 省（推的，待主 agent 跑） | 不拒（推的，待主 agent 跑） | 差（推的，待主 agent 跑） |
| `prefer_then_richest_leaf` | 池级：重写集叶，再共同空槽最多的叶 | 不撞（推的，待主 agent 跑） | 省（推的，待主 agent 跑） | 不拒（推的，待主 agent 跑） | 差，且专吃最空的叶（推的，待主 agent 跑） |
| `prefer_then_lowest_leaf_with_more_than_two_free` | 池级：重写集叶，再共同空槽多于 2 的最低叶，都没有再最低 | 不撞（推的，待主 agent 跑） | 省（推的，待主 agent 跑） | 不拒（推的，待主 agent 跑） | 差（推的，待主 agent 跑） |
| `prefer_then_lowest_sparing_record_empty_segments` | 同 `prefer_then_lowest`，两步先绕开按记录全空的段 | 撞（推的，待主 agent 跑） | 省（推的，待主 agent 跑） | 不拒（推的，待主 agent 跑） | 部分（推的，待主 agent 跑） |

「多于 2」这个门槛按锻造盘的账定：一对叶要 2 个落点。自然盘上闭包带更多叶，门槛可能不对（第一节「越位」），老化段量。

**改法在所有候选定义下都不成立吗**：不是。「优先重写集」作为一条独立政策站得住（重写集叶富的那一格省步数，推的）；不成立的是「**只**改优先、不改退路就关掉 64」。

**代价（第 4 问）**：步数、写放大的绝对值看 `forged_summary`、`rich_release_leaf`、`aged_*_summary` 的 `rounds`、`roles`、`roles_sum`，待跑。模型扫了 F、片数、前缀、家叶位置、删除位置与个数。没量真发布的挂钟；每次回落多扫重写集的叶，上界是叶数乘 812（现查 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`，推的，今天最坏是扫全盘）。

**改动面（现查 grep）**：

- 两个固定点、两处真发，加分配器的回落函数。
- 钉死今天答案的：`crates/singlefs-verification/tests/unequal_devices.rs:388`、`crates/singlefs-verification/src/model_comparison.rs:1774`、`crates/singlefs-verification/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs:166` 那一条；`crates/mutations.tsv:73`、`crates/mutations.tsv:74`、`crates/mutations.tsv:77`、`crates/mutations.tsv:78`、`crates/mutations.tsv:79`、`crates/mutations.tsv:80` 这几行的旧串要跟着改。
- 计数器名 `commit_generated_placements_that_fell_back_to_the_lowest_free_slot`、`FallBackToLowestFreeSlot` 名不副实，新政策下要改（`.claude/kb/term-renames.md` 登记）。
- 条文：已定项 8 第 2 条、C146 第 ② 条的「已还」、`crates/singlefs-core/src/transaction.rs:13811` 那段注释。

## 没打中的形状

- **固定点振荡**（试过，纸上）：要振荡，得有一片叶不在重写集时被用、进了重写集之后不被用。`prefer_then_lowest`：不在集里时被用，说明集里的空槽不够；它进集之后要的又多 2 个，集里其余的叶照样不够，它照样被用。退路换最富的叶同理。所以这几条臂上没构造出振荡，`holding_records` 不单调只在「叶里没有任何记录」时才可能（这样的叶每段都被隔离或扣住才不开段，罕见）。这是一次纸上推，没打中不算数，轮数上界照留。
- **落点违反 checker**：候选集没变大：偏好只在 `is_blocked_for_commit_generated` 与 `R` 排除都过的槽里挑，所以 I-5.1、I-5.4、I-5.6 这些只看槽占没占的不变量不受影响（推的）。`crates/singlefs-checker/src/` 里 grep `回落`、`fell_back`、`lowest_free_slot` 零命中（量的，命令见第四节那次 grep；范围只是这个目录下的 `.rs`）。I-5.5 的回落假定只管从 mkfs 水位建树那一版，那一族上回落计数是 0（看门用例 `crates/singlefs-verification/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs:1`）。
- **出生序号与落点次序**：已定项 10 ⑤ 的次序是取落点的**角色**次序（`.claude/kb/decisions/03-空间分配.md:212`：「这个次序同时就是出生序号的发号次序」），不要求槽号单调；新政策下一次发布里槽号不再单调，`crates/singlefs-checker/src/` 里按槽升序的判都是自己排序之后判（只扫了 grep「升序」的命中，没逐条读完）。
- **用户数据**：数据单元排在提交内生块之前取落点（`crates/singlefs-core/src/admission.rs:1049`：「数据单元在这次的角色里排最前、先取落点」），所以回落政策不改数据落点，已定项 8 第 1 条不受影响。**整理的搬迁例外**：搬迁目的地在角色之前取，其中提交内生块那几个走回落；D26 这一路没进模型。

## 这条腿自己的限度

- **模型一次都没编译**：`tools/apply_fallback_arms.py` 的替换锚点与 `tests/attack_d3_fallback_arms.rs` 都没过编译器。可能要接手的腿修编译错。
- 老化段的历史规模：单元区 2¹⁴ 槽（约 20 对叶）。撞 64 要 `F × 片数 / 缺口 > 62`，这个规模上撞不到，老化段答的是「形状走不走得到」「变差不变差」，不答「自然盘撞不撞 64」。估出来有余量再放大到 2¹⁵。缩的是历史规模，理由是挂钟，模型里 `aged_estimate` 打估时，超 40 分钟自停。
- 第三段（盘差不到一段）只到分配器一层，盘是锻造的，没走真发布。
- 第一、二段是锻造盘，不写分配记录，自然盘的闭包不在里面（第一节「越位」）。
- **崩溃状态没枚举**：新政策下放进重写集叶的新节点这一格只有「候选集没变大」的静态论证和老化段每条序列末尾的池级 checker（只判末态镜像）。原型没写，判决要引这一格就要接手的腿写（照定义第 3b 步另建一个 bin，不放验证包）。
- 整理（D26）这一路只有静态的话：偏好集含 `R` 所在的叶，`R` 本身被排除，新节点落 `R` 的邻段，下一次 greedy 选段（活槽最少）可能因此改选。没模型。
- 打中 C 的第 5、6 步（提交内生块回到 `S`、成了家）是读代码推的，要看模型 `aged_delete_trace` 的全空段数轨迹才算数。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务，判决要引的数没复跑。
- 没读 `research/prompts/d3-fallback-placement-r1-sonnet-model/`（别的腿的模型）。
- 崩溃枚举原型没写（见限度）。
- 模型 `apply_fallback_arms.py` 第一段一次写了 203 行，超了分段上限 150 行（量的 `wc -l`），第二次起按上限写。

## 要主 agent 跑的命令

1. 模型整份复跑（编译加跑，一个测试目标，四个测试函数，单线程）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_d3_fallback_arms; 范围=1 个测试目标; 为什么=攻方腿比回落政策几条臂的轮数与变差形状' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d3-fallback-placement-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/d3-fallback-placement-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/d3-fallback-placement-r1-opus/copy --work /home/fy5090/.local/share/singlefs/work/d3-fallback-placement-r1-opus/rerun-1`
   - 在哪起：仓根（`--cwd` 已写）。冻结副本 `/home/fy5090/.local/share/singlefs/work/d3-fallback-placement-r1-opus/copy` 是开工时 `scripts/hosts/minimal-copy.py` 建的（52M，`allocator.rs`、`transaction.rs` 的 sha256 与仓里相同，已在 `.deliverables` 写成交接）。如果范围声明闸拒了这个声明，照 r3 那一条（`research/prompts/m4-close-fixes-code-r3-opus-output.md:253`）不带声明再交，声明写在脚本里的 cargo 那一条上。
   - 可重入：每次尝试写 `<--work>.attempts/attempt.*`，跑完才拷进 `--work`；服务撞上限重起同一条不因旧输出退 2（账本二百零三节）。
   - 估时：锻造段约 500 探，r3 同形每探不到 1 秒；老化段打 `aged_estimate`，超 40 分钟自停，估算式在测试里，全量约 20 到 30 分钟（推的）。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`（0 是目标绿，绿不代表臂好坏）；`task-server/task log <号> --all` 里 `name=attack_d3_fallback_arms_target rc=` 与 `name=attack_d3_fallback_arms_summary`；观测在 `rerun-1/observations.txt`，看 `forged_summary`、`rich_release_leaf`、`unequal`、`aged_fill`、`aged_start`、`aged_probe`、`aged_delete_summary`、`aged_churn_summary`。编译不过时看 `rerun-1/arms.log` 的 `error` 行（`rc` 非 0 也会拷进 `--work`）。
   - 跑完新派同立场的腿，派发提示写「承接：research/prompts/d3-fallback-placement-r1-opus-output.md」并指到任务号与 `rerun-1/`。

## 等结果才能往下做的

打中 A、B、C 都要等第 1 条命令的数定稿（四问里标待跑的那几格）；接手的腿修编译错（如果有）、回填第四节的表（改成「量过」并贴原样输出）、撤回或坐实打中 C、按需要写崩溃枚举原型。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/d3-fallback-placement-r1-opus-output.md` 末行（写进报告之前跑的）：

```
  ✓ 核了 15 处引文、18 处裸行号，对上 15 处，没判 18 处（其中裸行号没有引文可比 18 处；逐处列在上面）
```
