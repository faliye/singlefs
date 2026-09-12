# D16（发布语义） 未定项 1 第四轮 · 反推腿产出

**立场**：假定材料的三个倾向都是错的，去找能推翻它们的观测。不投票、不总结材料，只交攻击与结果。

**口径**：2026-09-12。全部动手的攻击跑在仓外副本
`/tmp/claude-1000/-home-fy5090-code-singlefs/86b2ac0e-7e65-418c-97a7-f3d755730fe6/scratchpad/r4-opus/research/`
（`rsync -a --exclude target/` 从 `research/` 拷出）。仓里除这份文件之外一个字都没改（`git status --porcelain research/` 里没有一行是这一侧写的）。
副本里的基线先复跑过：`./target/release/e139_tightened_floor` 与
`research/results/e139-tightened-floor-2026-09-12.out` **逐字节一致**（`diff` 无输出），本机 release 65 秒。
四个探针二进制 `probe_a` / `probe_b` / `probe_c` / `probe_d` 都是 `e139_tightened_floor.rs` 前 1600 行原样 + 自写的 `main`，
改动逐条列在文末「附：探针的 diff」。探针产出整行抄在各节里，全量在 `all-probes.out`（180 行 `E7RESULT`）。

**结论先摆**（逐格的理由在下面各节）：

| 判据 | 打中 / 没打中 | 一句话 |
|---|---|---|
| 4-1 正确性 | **没打中** | 连续 6 次撕裂 + 丢盘、162 个组合，假候选 0、不可恢复 0、择根落到 F 之下 0 次 |
| 4-2 第 1 条 | **打中（有条件）** | 挂载时为实例切换扣住而 `df` 没扣的量一旦 > 60 块，G2R 当场出 24 次假性 ENOSPC；那个量就是 C283（准入失败时不先推发布就报 ENOSPC） 旁边的 C126（切换预留的最坏量没有口径），全仓没有数，而 C126 逐字说「按最坏情况直译是整棵记账树」 |
| 4-3 第 2 条 | **没打中**（对 G2R 按 E139 的世界）；但见 4-9：「G2R 每一格都过」靠一处因 G2R 输了才做的更正撑着 |
| **4-4 不能同真** | **打中** | 给出 G3A（锚根 + 按生命区间判可再分配）：候选集**任何时候 4 个不同状态**、这次删的块 ≤ 6 次发布放回、不靠用户再改状态、滞后量 0、假候选 0。倾向 ② 按字面**错**。同时给出严格一族里它为什么是定理，以及材料那句理由（「比 t 旧的根都引用释放代 t 的块」）**是假的** |
| 4-5 `df` 口径 | **打中** | `df_S` 的滞后量**没有上界**：实测随「一次删多少」线性涨，一次删 256 块时滞后量 1044 块。「平均 34 / 35、最大 44 / 52」是 8 块对象这一个取样点的产物 |
| 4-8 界与环深 | **打中两处** | ① `crit5_bound_exceeded_cells=0` 是恒真的——装置自己把处置砍在 B − 1 次，那一格从来没有判别力；② 环深式子量出来了：**S ≥ k + 2**，而 **S = 2 / 3 在 k_tol = 2 之内就违反最少保留**（184 / 224 次），E139 的故障世界只扫了 S ∈ {4, 16} |
| 4-9 跑前写死 | **打中两处** | ① 两次产物的 diff 是 **8 行**，不是「五格 + 汇总两行」；② 撤掉「写装置时的更正 #3」，**G2R 在 2 个格上删了再写失败 24 / 24 个种子**，而那处更正正是因为 G2R 输了才做的 |
| 材料自己的错 | **4 处** | 见最后一节；其中一处是 E139 正文自陈「按状态数读**任何时候**不少于 4 个」，而它自己的产物里 disk_loss 那几格 `min_states=1` |

---

## 判据 4-1：正确性 —— 没打中

### 4-1-A 连续多次撕裂 + 丢盘（E139 只建 1 次撕裂）

E139 的 `torn_newest` 世界把 `tear_next_publication` 置一次就崩溃恢复，所以「连崩、每次都崩在根槽写上」这一格没建。
副本里把它建起来：连续 tears ∈ {1..6} 次「写根槽时撕裂 → 崩溃 → 恢复」，再叠丢盘 0 / 丢盘 1 / 不丢盘，
三条臂（G2R / G2S / E138（按盘回退下界与推空的空间要求） 的臂 G）× S ∈ {1, 4, 16} × 8 个种子，
每次撕裂前先 `forced_pressure()` 把 F 抬到上限并推到生效。**162 行，一行都没红**：

```text
E7RESULT name=multi_tear arm=G2R_tightened_roots s=16 tears=6 lost_disk=none fakes=0 unrecoverable=0 halted=0 chosen_below_floor=0 min_states=4 min_candidates=6
E7RESULT name=multi_tear arm=G2R_tightened_roots s=16 tears=6 lost_disk=0 fakes=0 unrecoverable=0 halted=0 chosen_below_floor=0 min_states=1 min_candidates=2
E7RESULT name=multi_tear arm=G2S_tightened_states s=16 tears=6 lost_disk=1 fakes=0 unrecoverable=0 halted=0 chosen_below_floor=0 min_states=3 min_candidates=4
```

**机制（核过，不是推的）**：`fourth_newest_valid_txg()` / `fourth_newest_nonempty_valid_txg()` 取的是
`valid_roots()`，而 `visible_roots()` 那一层已经把 `is_self_verified == false` 的撕裂根滤掉了
（`e139_tightened_floor.rs:527-533`）。所以撕裂的根**不占那 4 个名额**：撕坏几个，上限就往回退几格，
F 跟着退，`newest_valid_root()` 永远 ≥ F。副本里加的 `chosen_below_floor` 探针正是数「择中的根落在 F 之下」，
162 行全是 0。⇒ **F 与择根之间没有缝**，这条攻击面按字面关死。

### 4-1-B journal 重放窗口（E139 它答不了的第 3 条）—— 没打中，但理由要写下来

材料点名要攻这一条，攻不动，原因是它归约到 4-1-A：

- D16（发布语义） 连带新规则 2 第 2 条逐字的危险形态是「块在 journal 位置 p 被释放、q > p 被重新分配，
  重放到 `p < r < q` 的前缀得到一棵指向该地址的树，而盘上字节已经是 q 的内容」。
- 可再分配要 `释放代 t ≤ F_生效`，而 F 的上限逐字是 `min(每块幸存盘上最新的持久有效根, 第 4 新的…)`
  ⇒ **F ≥ t ⇒ 最新有效根 ≥ t**。D23（journal 的角色与格式） 已定项 14 逐字「恢复只施加
  `(实例代号, checkpoint_txg)` **严格大于**所选根的记录」，而恢复择的是最新有效根（≥ t）
  ⇒ 代号 ≤ t 的记录一条都不施加 ⇒ 那棵「指向已被复用地址」的树重放不出来。
- 剩下的一格是**在飞窗口**：t + 1 的记录被前缀切开时，重放出来的树**根本不引用**那块（它在 t 就释放了），
  不是悬空指针，是「块还空着」。

⇒ 这条在 F 与择根不脱钩的前提下自洽。**它真正的缺口已经登记在案**：回退之后第一次发布那一格，
即 C281（回退后第一次发布会复用被抛弃时间线还引用的块），E139 产物里八条臂各 72 次
（`crit9_c281 … G2R_tightened_roots=72 G2S_tightened_states=72`），与取哪条臂无关。

### 4-1-C 一个必须点名的观测：两条臂在故障世界里都给不出 4 个状态

不是我构造的，是 E139 自己的产物（整行抄自 `research/results/e139-tightened-floor-2026-09-12.out`）：

```text
E7RESULT name=cell arm=G2S_tightened_states world=disk_loss s=16 lost_disk=0 fakes=0 unrecoverable=0 retention_violations=0 checks=4418 min_candidates=2 avg_candidates_x100=4670 avg_nonempty_x100=4664 min_states=1 avg_states_x100=4077 stalls=0
E7RESULT name=cell arm=G2S_tightened_states world=torn_newest s=16 lost_disk=0 fakes=0 unrecoverable=0 retention_violations=0 checks=9654 min_candidates=5 avg_candidates_x100=4554 avg_nonempty_x100=4527 min_states=2 avg_states_x100=4259 stalls=0
E7RESULT name=cell arm=G2S_tightened_states world=admin_rollback s=16 lost_disk=0 fakes=0 unrecoverable=0 retention_violations=0 checks=9677 min_candidates=1 avg_candidates_x100=4446 avg_nonempty_x100=4442 min_states=1 avg_states_x100=4174 stalls=0
```

G2S 在 `steady` / `torn_newest` / `slot_write_fail` 全部 `min_states=2`，在 `disk_loss`（丢盘 0）/
`admin_rollback` / `rollback_first_publish` 全部 `min_states=1`，S = 4 与 S = 16 一样。
而这六格的 `retention_violations` 全是 0 —— 因为 `is_retention_violated()`
（`e139_tightened_floor.rs:763-776`）判的是「F ≤ 上限」和「丢盘后幸存盘上最新的有效根 ≥ F」，
**不判「候选集里有没有 4 个不同状态」**。⇒ 最少保留这条判据的判别力，罩不到用户那句话本身。
（这一条同时是下面「材料自己的错」的第 2 条。）

---

## 判据 4-2：D3（空间分配） 已定项 9 第 1 条 —— 打中一处（有条件），另两处没打中

### 4-2-A 实例切换的挂载时预留 —— **打中**

E139 它答不了的第 4 条逐字：「实例切换：只建「txg 推进一格、写失败的槽留旧根」这一半；
它自己的挂载时预留没有口径，`df` 没扣它。」

副本里把这一半建起来：加一个 `instance_switch_hold`，它**只从准入里扣**（`user_allocatable`），
**不从 `df` 里扣**——这正是「`df` 没扣它」的字面形态。近满盘 400 个窗口 × 24 个种子，S = 16、(c₀, c_max) = (5, 10)：

```text
E7RESULT name=switch_hold arm=G2R_tightened_roots s=16 c0=5 c_max=10 deletes=1 meta_after=5 grow_at=18446744073709551615 switch_hold=60 max_lag=0 avg_lag_x100=0 write_after_delete_failed=0 false_enospc_arm=0 false_enospc_raw=24 true_enospc=24 stalls=0 max_publications=7 bound=11 max_txgs=7 min_states=1 halted=0
E7RESULT name=switch_hold arm=G2R_tightened_roots s=16 c0=5 c_max=10 deletes=1 meta_after=5 grow_at=18446744073709551615 switch_hold=80 max_lag=0 avg_lag_x100=0 write_after_delete_failed=0 false_enospc_arm=24 false_enospc_raw=24 true_enospc=0 stalls=0 max_publications=7 bound=11 max_txgs=7 min_states=1 halted=0
E7RESULT name=switch_hold arm=G2S_tightened_states s=16 c0=5 c_max=10 deletes=1 meta_after=5 grow_at=18446744073709551615 switch_hold=40 max_lag=52 avg_lag_x100=3817 write_after_delete_failed=96 false_enospc_arm=120 false_enospc_raw=120 true_enospc=0 stalls=0 max_publications=4 bound=8 max_txgs=4 min_states=4 halted=0
```

**G2R 的余量是 60 块**：扣住 0 / 40 / 45 / 50 / 60 块时假性 ENOSPC 都是 0，扣到 **80 块时 24 次**
（24 个种子各一次，`true_enospc` 同时掉到 0 —— 也就是这 24 次从「真 ENOSPC」翻成了「`df` 说有空间、写不进」）。
**G2S 的余量只有 40 块以下**：扣 40 块就出 120 次。

**它为什么是打中而不是「随便调一个参数就红」**：那个量不是我编的，它有名字、有落点、而且被登记成欠账。
C126（切换预留的最坏量没有口径）逐字（整行抄自 `.claude/kb/checks-owed.md:133`）：

> **可写挂载的准入合取里有一条「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13、D23（journal 的角色与格式） 已定项 14），而它要的那个量——「一次固定点重做的最坏量」——全仓没有任何数。**一条会把文件系统判成只读的准入条件，它的阈值不存在；按「最坏情况」直译是整棵记账树，那么高填充率的池永远挂不成可写。预留量还要按 N_switch 倍算（实例表链每切一次长一行、写行 COW 重写整条链，第 k 次比第 1 次贵）。

⇒ **「按最坏情况直译是整棵记账树」** 与 **「G2R 的余量是 60 块」** 放在一起，结论是清楚的：
G2R 在 D3（空间分配） 已定项 9 第 1 条上**输不输，取决于一个全仓没有的数**。
E139 的 `crit4_false_enospc_cells=0` 是在这个量等于 0 的前提下量的。
这不是「超过 c_max / 超过 k_tol」那两格，E139 没把它写进「不判」的名单，所以不记射程外。

⚠️ **口径**：60 / 80 这两个数是**容量 4000 块的模型里**量的。式子里 `reserve`、`residue`、`hidden_blocks`
都是绝对块数、与容量无关，所以余量大概率也是绝对量而不是比例——但这一步是**推的**，没量过别的容量。

### 4-2-B 非空发布的元数据随树高变 —— 没打中

E139 把非空发布的元数据固定在 5 块（`METADATA_BLOCKS_PER_PUBLICATION`），而保留池式子里的
`2 × 5` 那一项就是按它算的。副本里让**实际**写的元数据在第 100 个窗口涨到 6 / 7 / 8 / 10 / 13 块，
式子里那个 5 不动（也就是「挂载时算好了，树后来长高了」）。S ∈ {4, 16}、(c₀, c_max) = (5, 10)：

```text
E7RESULT name=meta_growth arm=G2R_tightened_roots s=16 c0=5 c_max=10 deletes=1 meta_after=13 grow_at=100 write_after_delete_failed=0 false_enospc_arm=0 false_enospc_raw=24 true_enospc=24 stalls=0 max_publications=7 bound=11 max_txgs=7 halted=0
E7RESULT name=meta_growth arm=G2S_tightened_states s=16 c0=5 c_max=10 deletes=1 meta_after=13 grow_at=100 write_after_delete_failed=96 false_enospc_arm=0 false_enospc_raw=120 true_enospc=120 stalls=0 max_publications=4 bound=8 max_txgs=4 halted=0
```

元数据涨到 2.6 倍，G2R 的假性 ENOSPC、删了再写、卡死**全是 0**；G2S 的删了再写从 72 涨到 96，
但按它自己的 `df` 全部是真 ENOSPC（`false_enospc_arm=0`）。
**机制**：`df_arm` 里的 `df_raw()` 是物理量，元数据一变大它自己就跟着缩，而准入被 `df_arm` 封顶
（`user_allocatable` 里的 `.min(self.df_arm())`）⇒ 式子里那个 5 变陈旧只让保留池少 2 × 8 = 16 块，
而 G2R 的余量（4-2-A 量出的 60 块）吃得下。⇒ **没打中**，而且它顺带说明 4-2-A 那 60 块余量是真的在起作用。

### 4-2-C 开销超过 c_max、超过 k_tol —— 射程外，但要点名一句

判据 4-2 逐字：「若只在 E139 明写「不判」的格（超过 c_max、超过 k_tol）里成立，记「射程外」」。
两格都成立，都记射程外，不记输：

- 开销超 c_max：E139 自己的产物 `cost_growth arm=G2S_tightened_states s=16 c0=1 c_max=5 grown_to=8 within_declared=0 false_enospc_arm=32`，G2R 同格 0。
- 超 k_tol：副本全枚举，G2R 在 k = 3 / 4 / 5 上出假性 ENOSPC（下面 4-8 有数）。

⚠️ **但有一条不能当射程外放过**：`c_max` 的口径欠在 C83（提交固定点没人回答） 上，
而 `df_R` 少报的常数逐字是 `保留池 + 残留 = (2 × 5 + 10 c_max) + (5 + 9 c_max) = 15 + 19 c_max`
（核过：c_max = 5 ⇒ 110 ✓、c_max = 10 ⇒ 205 ✓，与产物 `hidden_blocks` 逐格相等）。
⇒ **`df` 少报的那个「常数」对 c_max 的斜率是 19**。c_max 没有口径 ⇒ 这个常数也没有上界。
这一条落在判据 4-5 里，见下。

---

## 判据 4-3：D3（空间分配） 已定项 9 第 2 条 —— 没打中（但见 4-9）

按 E139 的世界（近满盘、推空期间 ≤ 2 次写失败、坏槽、开销升到 c_max），
副本里重跑 G2R 的近满盘一族，`write_after_delete_failed` 逐格 0，与产物一致。
4-2-B 的元数据涨到 13 块、4-2-A 的切换预留扣到 60 块，两格也都是 0。**没打中。**

⚠️ **这句「每一格都过」有一处必须带着的限定**：它依赖「写装置时的更正 #3」。
撤掉那处更正，G2R 在两个格上 24 / 24 个种子删了再写失败。数与判决在 4-9-B。

---

## 判据 4-4：「不能同真」是不是定理 —— **打中，倾向 ② 按字面错**

这一问分三层答：材料那句**理由**是假的；严格一族里**结论**是定理；而一族之外一小步就有反例，且反例跑得出来。

### 4-4-A 材料那句理由是假的 —— 核过，靠 E139 自己的源码

材料 二.2 与 五 逐字：「要复用这次删的块，引用它的根都得出候选集，而**比它旧的根都引用它**」。

后半句不成立。引用一块的根是 **txg ∈ [分配代, 释放代) 的那些**，不是「所有比 t 旧的」。
E139 自己就是这么记的 —— `allocate()` 里复用一块时把哪些根记成损坏（整行抄自
`research/e7-index-bench/src/bin/e139_tightened_floor.rs:704-708`）：

```rust
        for &index in &chosen {
            if let BlockState::Freed { allocated_at, freed_at } = self.pool.blocks[index] {
                for damaged_txg in allocated_at.max(oldest_visible_txg)..freed_at {
                    self.damaged_txgs.insert(damaged_txg);
                }
            }
```

区间的左端是 `allocated_at`，不是 0。跑前登记「口径」那一节也逐字写着
「txg 为 T 的根引用块 b ⟺ **分配代 ≤ T** < 释放代」。
⇒ **分配代晚于某个候选根的块，那个候选根根本不引用它**，材料那半句在装置里就是假的。
这不是文字游戏：4-4-B 的反例整个建在这半句的缝上。

### 4-4-B 严格一族里它是定理（补上材料没给的证明）

材料把「按盘回退下界一族」定义成：一个 F 写在根记录里、**候选集 = txg ≥ F 的有效根**、
可再分配 ⟺ 释放代 ≤ max(F_生效, 环里最旧有效根)。在这个定义里倾向 ② 成立，证明两行：

设这次删除发布成 txg t（t 是当时最新的 txg），要让释放代为 t 的块可再分配，就要 `t ≤ max(F_生效, m)`，
m = 环里最旧有效根。
- 若 **m ≥ t**：环里每个有效根的 txg 都 ≥ t，而 ≥ t 的 txg 当时只有 t 一个 ⇒ 候选集只有一个 txg ⇒ ≤ 1 个状态。
- 否则 **F_生效 ≥ t** ⇒ 候选集 ⊆ {txg ≥ t}。那一刻 ≥ t 的根只有 t 自己和处置期间发的空发布根，
  而用户 2026-09-12 的澄清逐字「推空与抬 F 产生的空发布根不算」⇒ 候选集里**恰好 1 个**用户可见状态。

⇒ 在严格一族里，「这次删的块立刻回来」与「候选集里 4 个状态」**不能同真**。∎
材料的结论方向对，理由错，而且射程说小了——见 4-4-D。

### 4-4-C 一族之外一小步：G3A，**构造成立，跑出来了**

**G3A 的定义**（跑前写死在副本源码里，跑完没改）：

| 项 | 取什么 |
|---|---|
| 格式 | 根记录里的 F（8 字节）**加三个锚根 txg**（24 字节），**加三个不参与轮转的锚根槽**（住在环之外，`txg mod R` 那套轮转键不动） |
| 候选集 | `{有效根 R : R.txg ≥ F} ∪ {三个锚根}`（**不连续**） |
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) ∧ **没有锚根落在 [分配代, 释放代) 里** |
| 抬 F 的上限 | `min(每块幸存盘上最新的持久有效根)` —— 锚已经给了 3 个状态，环上只需再留 1 个 |
| `df` | `没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 锚钉住的已释放块` |
| 锚取在哪 | 挂载之初三个各建一个对象的窗口（txg 1 / 2 / 3），此后所有数据的分配代都 > 3 |

**结果**（24 个种子 × 400 个删建窗口 × 4 格，整行抄自 `all-probes.out`）：

```text
E7RESULT name=anchored_protected arm=G2R_tightened_roots anchored=1 credits=1 s=16 c0=1 c_max=5 write_after_delete_failed=0 first_failure_window=0 last_failure_window=0 false_enospc_arm=0 false_enospc_raw=24 true_enospc=24 max_publications=6 bound=11 stalls=0 fakes=0 unrecoverable=0 retention=0 min_states=4 avg_states_x100=1128 min_candidates=6 forced_per_window_x100=96 max_lag=0 avg_lag_x100=0 anchor_pinned_max=15 fallback_hits=0
E7RESULT name=anchored_protected arm=G2S_tightened_states anchored=1 credits=1 s=4 c0=5 c_max=10 write_after_delete_failed=0 first_failure_window=0 last_failure_window=0 false_enospc_arm=0 false_enospc_raw=24 true_enospc=24 max_publications=6 bound=8 stalls=0 fakes=0 unrecoverable=0 retention=0 min_states=4 avg_states_x100=689 min_candidates=7 forced_per_window_x100=151 max_lag=0 avg_lag_x100=0 anchor_pinned_max=15 fallback_hits=48
```

八格（两套式子 × S ∈ {4, 16} × 两档开销）全部：

- **`min_states=4`**：候选集任何时候 4 个不同的用户可见状态（`retention=0`，而这里的最少保留检查
  已经换成**直接数候选集里的不同状态数 < 4**，比 E139 那条严）。
- **`write_after_delete_failed=0`**：删了立刻能写，**一格都没失败**（G2S 基线同参数是 48 / 72）。
- **`max_lag=0 avg_lag_x100=0`**：没有滞后量那一截。
- **`max_publications=6`**，两条式子的界（11 / 8）都没碰到。
- `fakes=0 unrecoverable=0 stalls=0 false_enospc_arm=0`。
- `anchor_pinned_max=15`：锚永久钉住 15 块（三个锚窗口各自的 5 块元数据），**常数**。

⇒ **判据 4-4 的失败条款按字面触发**：「给出这一族里的一种形态：候选集任何时候保留 4 个不同状态，
且这次删的块在有界次发布内放回，不靠用户再改状态」—— 给出来了，而且跑出来了。
按失败条款逐字：**倾向 ② 作废，那一形立实验量，这一轮不交用户定案。**

### 4-4-D G3A 的代价，以及它逼出来的**真正**的定理

不把代价写小。同一份探针里，**锚根引用的那三个对象也让用户删**（`anchored`，不加 `protected`）：

```text
E7RESULT name=anchored arm=G2R_tightened_roots anchored=1 credits=1 s=16 c0=1 c_max=5 write_after_delete_failed=32 first_failure_window=11 last_failure_window=391 false_enospc_arm=0 false_enospc_raw=56 true_enospc=56 max_publications=6 bound=11 stalls=0 fakes=0 unrecoverable=0 retention=0 min_states=4 avg_states_x100=1122 min_candidates=6 forced_per_window_x100=97 max_lag=0 avg_lag_x100=0 anchor_pinned_max=39 fallback_hits=0
```

`anchor_pinned_max` 从 15 涨到 **39**（15 + 3 × 8），删了再写失败 26–43 次、**散在第 4 到第 400 个窗口之间**。
机制：那三个对象被删的那一刻，它们的 8 块**一块也不回来**——它们属于一个被保留的状态。
`false_enospc_arm=0`，`df` 没说谎（`df` 扣掉了锚钉住的量）；输的是 D3（空间分配） 已定项 9 **第 2 条**。

⇒ **真正的定理**（G3A 与 G2R / G2S 都是它的特例）：

> 释放代为 t、分配代为 β 的块能立刻放回 ⟺ **被保留的那几个状态里，没有一个落在 [β, t) 里**。
> 换句话说：**一个被保留的状态，钉住的正是「在它那一刻活着、后来才死」的全部块。**

三条臂就是这条定理上的三个点：

| 臂 | 保留哪 4 个状态 | 钉住什么 | 撞上的频率 |
|---|---|---|---|
| G2R | 只保留 1 个（推空之后） | 一块不钉 | — |
| G2S | 最近 4 个 | 最近 3 个窗口的释放量 —— **每次删都撞上**，且**没有上界**（见 4-5） | 每次删 |
| G3A | 3 个远古锚 + 当前 | 取锚那一刻活着、后来才死的块 —— **有上界 = 取锚时的活块数** | 只在删到锚里的数据时 |

⇒ **材料把这一问问窄了**。用户要定的不是「按根数读还是按状态数读」，是 **「哪 4 个状态」**，
而每种选法的代价是那 4 个状态的块集合之并。
⇒ **还有一条要写给用户看**：P2 的「最少保留 4 个可退到的不同状态」这条规格，
**不指定「多近」的时候，可以被三个远古锚根几乎零成本地满足**——G3A 给的 4 个状态是
「三个开机初期的状态 + 当前」，形式上完全满足用户 2026-09-12 的澄清，实用价值却不是用户想要的那个。
**规格该补的是「最旧的那个保留状态不许比现在旧过 X」或者「4 个状态要覆盖最近 X 次发布」**，
不是「4」这个数。

### 4-4-E G3A 还差什么（不许当成已经可以交）

1. **只跑了近满盘一族**，没进 E139 的六个故障世界、没进推空写失败与坏槽。`fakes` / `unrecoverable` 的 0 只在近满盘里成立。
2. **要三个不参与轮转的根槽**，也就是动 D22（单元原子性怎么合成） 已定项 2 的轮转键那一层，格式代价比 G2R / G2S 的 8 字节大得多。
3. **可再分配的谓词要「分配代」**。今天释放侧有没有这个数没核过（D5（快照 / 空间记账机制） 的 birth txg 在指针里，
   而 deadlist 条目带不带它，是 D18（块里携带什么信息） 未定项 15 那一带的事）。**这是推的，没现查。**
4. 副本里两处口径是我自己定的，不是仓里定的：`df` 扣锚钉住的量（不扣就出 24 次假性 ENOSPC，第一版实测过）、
   空根按「不晚于它的最后一个非空根」折算状态（不折算，候选集全是空根时会误报只有 3 个状态）。

---

## 判据 4-5：`df` 口径合不合用户原话 —— **打中，两半都打中**

### 4-5-A `df_S` 的滞后量**没有上界**，随一次删多少线性涨

E139 报的「近满盘平均约 32–35 块、最大 44 / 52」是**一个取样点**的产物：装置里一个对象恒 8 块
（`USER_BLOCKS_PER_OBJECT = 8`）。副本里只把「一个窗口删几个对象」从 1 扫到 32（别的一个字没动），
S = 16、(c₀, c_max) = (1, 5)、24 个种子：

```text
E7RESULT name=lag_scale arm=G2S_tightened_states s=16 c0=1 c_max=5 deletes=1 meta_after=5 grow_at=18446744073709551615 switch_hold=0 max_lag=44 avg_lag_x100=3380 write_after_delete_failed=48 false_enospc_arm=0 false_enospc_raw=72 true_enospc=72 stalls=0 max_publications=4 bound=8 max_txgs=4 min_states=4 halted=0
E7RESULT name=lag_scale arm=G2S_tightened_states s=16 c0=1 c_max=5 deletes=4 meta_after=5 grow_at=18446744073709551615 switch_hold=0 max_lag=148 avg_lag_x100=7157 write_after_delete_failed=24 false_enospc_arm=0 false_enospc_raw=48 true_enospc=48 stalls=0 max_publications=4 bound=8 max_txgs=4 min_states=4 halted=0
E7RESULT name=lag_scale arm=G2S_tightened_states s=16 c0=1 c_max=5 deletes=32 meta_after=5 grow_at=18446744073709551615 switch_hold=0 max_lag=1044 avg_lag_x100=7005 write_after_delete_failed=24 false_enospc_arm=0 false_enospc_raw=48 true_enospc=48 stalls=0 max_publications=4 bound=8 max_txgs=4 min_states=4 halted=0
```

| 一个窗口删掉的块数 | 8 | 16 | 32 | 64 | 128 | 256 |
|---|---|---|---|---|---|---|
| `max_lag`（块） | 44 | 84 | 148 | 276 | 532 | **1044** |

拟合（算过）：`max_lag = 4 × (一个窗口释放的块数)`，也就是 **4 × (删掉的量 + 一次发布的元数据)**，
D ≥ 2 时逐点相等（4 × 21 = 84、4 × 37 = 148、4 × 69 = 276、4 × 133 = 532、4 × 261 = 1044）。
「4」是「最近 3 个改状态的窗口 + 正在攒的那个窗口」。G2R 同参数全是 `max_lag=0`。

⇒ **判据 4-5 的判红条件逐字「少报的量没有上界、随盘上内容涨」，两条都中**：
`df_S` 少报的量 = 80 或 145 块（常数）+ **4 × 最近四个窗口的释放量**，后一项只受盘容量限制。
具体到用户看得见的行为：**删掉 s 字节，`df` 一个字节都不涨**（释放进「已释放的」，同额被滞后量扣回去），
要再改三次用户可见状态才涨回来。这与 pitfalls.md 第 1 条逐字「`df` 有空间但写不进、**删文件也报没空间**」
是同一个症状的后一半，而 D3（空间分配） 已定项 9 的依据逐字是用户那句
「我们的目标是 避免 btrfs ENOSPC， 我们绝对不能接受 ENOSPC」。

⇒ 按判据 4-5 的处置：**记「需用户确认」，写在交用户的表最前面**。
⚠️ 而且**不许写成「平均约 34 / 35 块，很小」**——那个数是 8 块对象量出来的，
判据 4-5 要问的正是它随什么涨。

### 4-5-B `df_R` 少报的 110 / 205 也不是常数：它对 c_max 的斜率是 19

核过的算术：`hidden = 保留池 + 残留 = (2 × 5 + (B_R − 1) c_max) + (5 + (B_R − 2) c_max)`，B_R = 11
⇒ `hidden_R = 15 + 19 c_max`。c_max = 5 ⇒ 110 ✓、c_max = 10 ⇒ 205 ✓（与产物 `hidden_blocks` 逐格相等）。
G2S：`hidden_S = 15 + 13 c_max`（c_max = 5 ⇒ 80 ✓、10 ⇒ 145 ✓）。

⇒ **`df_R` 少报的「常数」= 15 + 19 c_max，而 c_max 本工程没有口径**（E139 它答不了的第 5 条逐字
「c_max 是声明的上界，本工程没有它的口径（C83（提交固定点没人回答） 欠着运行时读数）」）。
c_max 声明得越保守，`df` 就少报得越多，**斜率 19**。
⇒ 「G2R 的 `df` 只少报一个与 S 无关的常数」这句话要带上限定：**与 S 无关，但与 c_max 线性相关，
而 c_max 今天是个没人算得出的数**。这不是滞后量那种「随负载变」，但它同样没有上界。

⚠️ **一处顺带的算术错**：E139 正文第 96 行与材料 P6 都写「110 / 205 块（1.7 / 3.3 MiB）」，
而 205 × 16384 = 3 358 720 字节 = **3.203 MiB**，产物那一行自己写着 `hidden_bytes=3358720`。
应为 **3.2 MiB**。110 块那个 1.7 MiB 是对的（1.719）。

---

## 判据 4-8：界与环深 —— **打中两处**

### 4-8-A 「发布数 ≤ B」这一格**恒真**，从来没有判别力 —— 打中

`maximum_empty_publications()`（整行抄自 `e139_tightened_floor.rs:957-966`）对收严臂返回
`self.arm.publication_bound(self.ring_depth) - 1`，而 `push_after_commit` 的循环写死
`for _ in 0..maximum_empty_publications`，`publications = 1 + <循环次数>`。
⇒ **一次准入的发布数在装置里永远 ≤ B**，`crit5_bound_exceeded_cells` 这一格**不可能非 0**，
对臂 A 同理（上限 `ring_depth − 1`，界 `ring_depth`）。

⇒ E139 判据 8 逐字「G2R 容忍范围之内实测最大 11 = B_R，「每次失败加 2」是紧的」——
那个 11 是天花板贴出来的，**不管「每次失败加 2」对不对，它都会读成 11**。
处置做不完的时候，症状不是「越界」，是被砍掉之后报的假性 ENOSPC / 删了再写失败。

**证据**：副本里把天花板放到 200，同一批组合重跑（`PROBE_UNCAP=1`）：

```text
E7RESULT name=bound_full arm=G2R_tightened_roots s=4 c0=5 c_max=10 failures=3 offset_limit=11 patterns=220 cases=1760 max_publications=12 formula_bound=13 exceeds=0 worst_pattern=2/5/8 false_enospc_arm=0 write_after_delete_failed=0 stalls=0 fakes=0 unrecoverable=0 retention_violations=0
E7RESULT name=bound_full arm=G2R_tightened_roots s=4 c0=5 c_max=10 failures=4 offset_limit=11 patterns=495 cases=3960 max_publications=14 formula_bound=15 exceeds=0 worst_pattern=2/5/8/11 false_enospc_arm=0 write_after_delete_failed=0 stalls=0 fakes=0 unrecoverable=0 retention_violations=0
```

天花板砍着的时候（E139 的形态），同一格 k = 3 是 `false_enospc_arm=32 write_after_delete_failed=32`；
天花板放开之后**两个都是 0**，发布数 12 < 13。⇒ **G2R 在 k = 3..5 上的失败，全部是装置把处置砍在 B − 1 次造成的，
不是机制的毛病**。这条对怎么定 k_tol 有直接后果：B = 7 + 2k 是**够用的预算**，
真正要定的是「愿意在一次准入里花几次发布」。

### 4-8-B 界的式子本身：全枚举没越过，但 E139 报的那两个数不是最坏值

E139 它答不了的第 8 条逐字「3 次失败的组合只取了前 20 种（位置靠前），那一格的最大发布数不是 3 次失败的最坏值」。
副本里按 E139 自己的失败位置区间（G2R 0..11、G2S 0..8）**全枚举** k = 1..5，8 个种子：

| 臂 | k = 1 | k = 2 | k = 3 | k = 4 | k = 5 |
|---|---|---|---|---|---|
| G2R 实测最大发布数（砍着 / 放开） | 9 / 9 | 11 / 11 | 10 / 10–12 | 11 / 12–14 | 11 / 11–13 |
| 式子 7 + 2k | 9 | 11 | 13 | 15 | 17 |
| G2S 实测（砍着 / 放开） | 4–6 / 4–5 | 6–8 / 6–7 | 8 / 7–9 | 8 / 6–9 | 8 / 5–9 |
| 式子 4 + 2k | 6 | 8 | 10 | 12 | 14 |

⇒ **式子一格都没被越过**（`exceeds=0` 全部 40 行）。判据 4-8 前半 **没打中**，式子站得住。
⇒ 但 E139 产物里的 `beyond_tolerance_max_publications=9`（G2R）/ `=7`（G2S）**不是最坏值**：
全枚举放开天花板之后是 12–14 / 7–9。E139 自己把这一条写在「它答不了的」里，所以不记作错，只补数。

### 4-8-C 环深式子 —— **打中，而且 S = 2 / 3 在容忍之内就违反最少保留**

E139 它答不了的第 11 条逐字「环深与状态数的关系（S = 4 上 3 次失败那一格）只有一个观测，没有式子」。
副本里扫 S ∈ {1, 2, 3, 4, 5, 6, 8, 16} × 失败次数 0..4（失败位置全枚举），G2S、(c₀, c_max) = (5, 10)、8 个种子：

```text
E7RESULT name=ring_depth arm=G2S_tightened_states s=2 ring_depth=6 failures=2 cases=840 retention_violations=184 min_states=2 max_publications=5 txgs_needed=11 stalls=0 write_after_delete_failed=104 false_enospc_arm=0
E7RESULT name=ring_depth arm=G2S_tightened_states s=3 ring_depth=9 failures=2 cases=840 retention_violations=224 min_states=2 max_publications=7 txgs_needed=13 stalls=0 write_after_delete_failed=112 false_enospc_arm=0
E7RESULT name=ring_depth arm=G2S_tightened_states s=4 ring_depth=12 failures=2 cases=840 retention_violations=0 min_states=4 max_publications=7 txgs_needed=13 stalls=0 write_after_delete_failed=0 false_enospc_arm=0
E7RESULT name=ring_depth arm=G2S_tightened_states s=4 ring_depth=12 failures=3 cases=3640 retention_violations=264 min_states=2 max_publications=8 txgs_needed=15 stalls=0 write_after_delete_failed=224 false_enospc_arm=0
E7RESULT name=ring_depth arm=G2S_tightened_states s=5 ring_depth=15 failures=3 cases=3640 retention_violations=0 min_states=4 max_publications=8 txgs_needed=15 stalls=0 write_after_delete_failed=0 false_enospc_arm=0
E7RESULT name=ring_depth arm=G2S_tightened_states s=5 ring_depth=15 failures=4 cases=10920 retention_violations=308 min_states=2 max_publications=8 txgs_needed=16 stalls=0 write_after_delete_failed=218 false_enospc_arm=74
E7RESULT name=ring_depth arm=G2S_tightened_states s=6 ring_depth=18 failures=4 cases=10920 retention_violations=0 min_states=4 max_publications=8 txgs_needed=16 stalls=0 write_after_delete_failed=64 false_enospc_arm=64
```

违反最少保留的阈值（`retention_violations` 第一次非 0 的那一档）：

| 环深 N（= 3S） | k = 1 | k = 2 | k = 3 | k = 4 |
|---|---|---|---|---|
| 6（S = 2） | 0 | **184** | 2640 | 15800 |
| 9（S = 3） | 0 | **224** | 2312 | 11736 |
| 12（S = 4） | 0 | 0 | **264** | 2520 |
| 15（S = 5） | 0 | 0 | 0 | **308** |
| 18（S = 6） | 0 | 0 | 0 | 0 |

⇒ **式子**：要容忍 k 次根槽写失败而不违反最少保留，需要 **S ≥ k + 2**，即 **N ≥ 3(k + 2)**
（k = 2 ⇒ N ≥ 12 ✓、k = 3 ⇒ N ≥ 15 ✓、k = 4 ⇒ N ≥ 18 ✓）。
⚠️ 这是**实测拟合**，不是推导出来的；只在 (c₀, c_max) = (5, 10)、8 个种子上扫过。

⇒ **打中的是这一句**：**S = 2 与 S = 3 在 k_tol = 2（容忍之内）就违反最少保留**，184 / 224 次，
`min_states` 掉到 2，同时 104 / 112 次删了再写失败。
E139 的故障世界只扫了 `FAULT_SLOTS_SWEEP = [4, 16]`，所以这两档一次都没跑过。
材料 P5 逐字「G2S 在容忍范围之内 0」与 E139 判据 7 逐字「G2S 在容忍范围之内 0」，
**射程只到 S ∈ {4, 16}**，不到「按状态数读这条臂」。
⇒ 交用户时，按状态数读**必须同时交一条环深下限**（S ≥ k_tol + 2），否则这条臂在浅环上按它自己的判据就是输的。

⚠️ 另一处：**S = 1（环深 3）上 G2S 根本给不出 4 个状态**。产物逐行
`near_full arm=G2S_tightened_states s=1 c0=1 c_max=5 … min_states=1 avg_states_x100=207 … retention_violations=0`
—— 状态数 1，最少保留检查报 0。原因同 4-1-C：那条检查判的是 F 与上限，不是状态数。

⚠️ 顺带核实一条 E139 的细节：判据 7 逐字「一次带 3 次失败的处置用掉 11 个 txg」。
S = 4、k = 3 那一格 `max_publications=8`，加上 3 次失败各烧一个 txg ⇒ 11 ✓。**对。**

---

## 判据 4-9：跑前写死的纪律 —— **打中两处**

### 4-9-A 两次产物的 diff 是 8 行，不是「五格 + 由它们汇总的两行」

`diff research/results/e139-tightened-floor-2026-09-12-run1.out research/results/e139-tightened-floor-2026-09-12.out`
改了 **8 行**，行号 62 / 92 / 93 / 134 / 158 / 686 / 694 / 898：

| 行 | 是什么 | 变了什么 |
|---|---|---|
| 62、92、93、134、158 | `name=cell` G2S s=1 的五个故障世界格 | `retention_violations` 18 / 6 / 6 / 6 / 5 → 全 0 |
| **686、694** | **`name=near_full` G2S s=1 的两格**（c₀ = 1 与 c₀ = 5） | `avg_lag_x100` 1354 → **1426** 与 1354 → **1728** |
| 898 | `name=verdict_candidate` G2S | `crit7_retention` 65 → 24（65 − 24 = 41 = 18 + 6 + 6 + 6 + 5 ✓，确实是那五格的汇总） |

⇒ E139 历史版本逐字「与第二次只差下面五格与**由它们汇总的两行**」、跑前登记末节逐字
「别的判据没有一格不同……**别的格不受影响**」，两句都对不上：
**汇总行只有一行**（898），另外两行（686、694）是**独立的 near_full 测量格**，
而且它们变的字段是 `avg_lag_x100`，与 `retention` 一点关系都没有。

**按判据 4-9 的两半分开判**：
- **触发观测**逐字「两次产物的 diff **超出** G2S 在 S = 1 的五格与由它们汇总的两行」⇒ **按字面触发**（8 行 > 7 行）。
- **判红条件**逐字「改变了任何判据、臂的谓词或 **S ≥ 4 的任何一格**」⇒ **不触发**：8 行全部 `s=1`，
  没有一行 S ≥ 4；判据一条没改；臂的谓词只动了「非空有效根不足 4 个」那条分支。
- 我这一腿的判定：**不判 E139 整轮作废**，判**记账那一句写错了，要改**——
  历史版本那一句应写成「五个故障世界格 + 两个 near_full 格 + 一个汇总行，共 8 行，全部在 S = 1」。

**顺带核过跑前登记的另一句**：「S ≥ 4 时非空有效根总是够 4 个，这条分支走不到」。
副本里在填满之后计数那条分支走了几次，24 个种子 × 400 个窗口：

```text
E7RESULT name=fallback_after_fill arm=G2S_tightened_states s=1 c0=1 c_max=5 fallback_hits=38400 checks=38400
E7RESULT name=fallback_after_fill arm=G2S_tightened_states s=4 c0=1 c_max=5 fallback_hits=0 checks=24000
E7RESULT name=fallback_after_fill arm=G2S_tightened_states s=16 c0=5 c_max=10 fallback_hits=0 checks=21720
```

S = 4 / 16 在近满盘删建循环里 **0 次**，S = 1 **每次都走**。⇒ 这句**是对的**（射程：近满盘那一族；故障世界没量）。

### 4-9-B 「G2R 每一格都过」靠一处**因为 G2R 输了才做**的更正撑着 —— 打中

跑前登记「写装置时的更正」第 3 条逐字：收严臂的可分配量与 `df` 都把**当前活着的元数据**记回保留池，
理由一半是式子（「保留池 2 × 5 + (B − 1) × c_max 正是给这些块留的」），
另一半逐字是「**写单测时 G2R 在 c₀ = 5 那一格删了再写失败一次（df 报 6，物理上可分配 76）**」。

副本里把这处更正撤掉（`credits_live_metadata = false`，别的一个字不动），近满盘 24 个种子 × 400 窗口：

```text
E7RESULT name=nocredit arm=G2R_tightened_roots anchored=0 credits=0 s=4 c0=5 c_max=10 write_after_delete_failed=24 first_failure_window=6 last_failure_window=6 false_enospc_arm=0 false_enospc_raw=48 true_enospc=48 stalls=0 max_publications=6 bound=11 … min_states=1 …
E7RESULT name=nocredit arm=G2R_tightened_roots anchored=0 credits=0 s=16 c0=5 c_max=10 write_after_delete_failed=24 first_failure_window=6 last_failure_window=6 false_enospc_arm=0 false_enospc_raw=48 true_enospc=48 stalls=0 max_publications=7 bound=11 … min_states=1 …
```

⇒ **G2R 在 2 个格上删了再写失败，24 / 24 个种子，全部在第 6 个计数窗口**。
（G2S 那一侧同时受影响：失败窗口区间从「第 1..3」变成「第 1..5」。）

**判定**：这处更正在第一次运行之前、写进了跑前登记，形式上合规。
但按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在跑前写死之列」那三步：
- 第 1 步「记一次输」**做了**（登记里逐字写了 G2R 在那一格失败、`df` 报 6 / 物理 76）；
- 第 3 步「写明收严在哪」**做了**（逐字「这是记账的口径，不改臂的谓词、上限、生效与界」）；
- 第 2 步「**只许收严**」**没做到**：这处更正同时抬高 `df` 与准入，对判据 4（假性 ENOSPC）是收严，
  对**判据 5（删了再写）是放宽**——它把 48 次失败变成 0。

⇒ **不判 E139 作废**（改动在第一次运行之前、有独立的式子理由、且 `false_enospc_arm` 两边都是 0），
但 **「G2R 每一格都过」这句话交用户时必须带上这个限定**：它依赖「当前活着的元数据算保留池占用」这条记账口径，
而那条口径是在 G2R 于 (S, c₀) = (4 或 16, 5) 上输了之后补的。**这句话不该写成「G2R 无条件全过」。**

---

## 材料自己的错（逐条带行号）

| # | 位置 | 材料 / 原文怎么写 | 实际是什么 |
|---|---|---|---|
| 1 | 材料 P5（第 21 行）逐字「候选集**任何时候**不少于 4 个状态」；同一句在 E139 正文第 78 行逐字「按状态数读**任何时候**不少于 4 个（`min_states=4`）」 | 「任何时候」 | **只在 near_full 那一族成立**。同一份产物里 `world=steady` / `torn_newest` / `slot_write_fail` 全是 `min_states=2`，`world=disk_loss`（丢盘 0）/ `admin_rollback` / `rollback_first_publish` 全是 `min_states=1`，S = 4 与 S = 16 都一样；`near_full … s=1` 也是 `min_states=1`。这是「限定词没了」那一型：引的每个数都是真的，错的是结论比产物支持的范围宽 |
| 2 | 材料 P5 与 E139 判据 7 逐字「G2S 在容忍范围之内 0」「只在 S = 4、c₀ = 5、推空期间 3 次写失败（超过容忍的格）里 24 次」 | 违反最少保留只在超过容忍的那一格 | **S = 2 / 3 在 k = 2（容忍之内）就违反**，184 / 224 次（4-8-C）。另外**同一个 S = 4 的 c₀ = 1 那一格**，三元组全枚举（84 种，E139 只取前 20 种）下是 160 次违反 + 32 次删了再写失败，E139 那一格报的是 0 |
| 3 | 材料 P6（第 22 行）与 E139 正文第 96 行逐字「110 / 205 块（**1.7 / 3.3 MiB**）」 | 3.3 MiB | **3.2 MiB**。205 × 16384 = 3 358 720 字节 = 3.203 MiB，产物同一行自己写着 `hidden_bytes=3358720` |
| 4 | 材料 P7（第 23 行）逐字「两次只差 G2S 在 S = 1 的五格」；E139 历史版本逐字「只差下面五格与由它们汇总的两行」；跑前登记末节逐字「别的格不受影响」 | 5 格 + 2 行汇总 = 7 行 | **8 行**：5 个故障世界格 + **2 个 near_full 格**（`avg_lag_x100` 1354 → 1426 / 1728）+ 1 个汇总行。那两个 near_full 格不是汇总，是独立的测量格（4-9-A） |

⚠️ **第 5 条不算「错」，但要连着引**：材料 二.2 与 五 那句「比 t 旧的根都引用释放代 t 的块」是整个倾向 ②
的理由，而它在 E139 自己的装置里就是假的（4-4-A）。它不是笔误——**4-4-C 的反例正是从这半句的缝里长出来的**。

---

## 给主 agent 的三句

1. **判据 4-4 的失败条款按字面触发**（形态给出来了、跑出来了、八格全过）⇒ 照材料自己写的失败条款，
   倾向 ② 作废、那一形要立实验量、**这一轮不交用户定案**。要不要照办由主 agent 判，
   但如果照办，立的实验该问的是 4-4-D 那条定理——**「哪 4 个状态」各要付多少**，不是「4 还是 1」。
2. **判据 4-9 不判 E139 作废**（8 行全在 S = 1，判据与 S ≥ 4 一格没动），但两处记账要改：
   历史版本那句「五格 + 汇总两行」，以及「G2R 每一格都过」要带上「依赖写装置时的更正 #3」这个限定。
3. **交用户的表最前面要写的不是取舍，是三个没有数的量**：`c_max`（C83（提交固定点没人回答），
   `df_R` 少报 = 15 + 19 c_max）、实例切换的挂载时预留（C126（切换预留的最坏量没有口径），
   G2R 的余量实测 60 块）、`df_S` 的滞后量（无上界，= 4 × 最近四个窗口的释放量）。
   三个里有两个直接决定 D3（空间分配） 已定项 9 第 1 条兑不兑现。

---

## 附：探针的 diff 与复跑

四个探针都是 `research/e7-index-bench/src/bin/e139_tightened_floor.rs` 的**前 1600 行原样**
（到 `fn main()` 之前）加上自写的 `main`，放在仓外副本里。复跑：

```bash
SP=/tmp/claude-1000/-home-fy5090-code-singlefs/86b2ac0e-7e65-418c-97a7-f3d755730fe6/scratchpad/r4-opus
cd "$SP/research"
cargo build --release --bin probe_a --bin probe_b --bin probe_c --bin probe_d
./target/release/probe_a anchor          # 判据 4-4
./target/release/probe_a credit          # 判据 4-9-B
PROBE_UNCAP=1 PROBE_EXTRA=0 ./target/release/probe_b bound   # 判据 4-8-A/B
PROBE_EXTRA=0 ./target/release/probe_b ring                  # 判据 4-8-C
./target/release/probe_c lag             # 判据 4-5-A
./target/release/probe_c switch          # 判据 4-2-A
./target/release/probe_c meta            # 判据 4-2-B
./target/release/probe_d tear            # 判据 4-1-A
./target/release/probe_d fallback        # 判据 4-9-A 后半
```

`probe_a`（G3A 与更正 #3 的开关）对 `Simulation` 的改动，整段抄自副本源码：

```rust
    /// G3A 探针：钉住的锚根（住在环之外的专用槽里，不参与轮转）。空 ⇒ 与 E139 原样一致。
    anchor_roots: Vec<RootRecord>,
    anchor_mode: bool,
    /// 更正 #3 的开关：false ⇒ 活着的元数据不记回保留池（登记里的原始口径）。
    credits_live_metadata: bool,

    /// 有没有一个锚根落在这块的生命区间 [分配代, 释放代) 里。
    fn anchor_covers(&self, allocated_at: u64, freed_at: u64) -> bool {
        self.anchor_roots.iter().any(|root| root.txg >= allocated_at && root.txg < freed_at)
    }
```

改掉的四处谓词（`-` 是 E139 原样，`+` 是 G3A）：

```diff
     fn floor_upper_limit(&self) -> u64 {
+        if self.anchor_mode {
+            return self.newest_valid_txg_on_every_surviving_disk();
+        }

     fn reusable_count(&self) -> u64 {
-            BlockState::Freed { freed_at, .. } => *freed_at <= bound,
+            BlockState::Freed { allocated_at, freed_at } => *freed_at <= bound && !self.anchor_covers(*allocated_at, *freed_at),

     fn df_arm(&self) -> u64 {
-        (self.df_raw() + self.live_metadata_credit()).saturating_sub(self.reserve_blocks + self.push_residue_blocks + self.lag_blocks())
+        (self.df_raw() + self.live_metadata_credit()).saturating_sub(self.reserve_blocks + self.push_residue_blocks + self.lag_blocks() + self.anchor_pinned_total())

     fn count_fake_candidates(&self) -> (u64, u64, u64, u64) {
-        let candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).collect();
+        let mut candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).collect();
+        for anchor in &self.anchor_roots {
+            if !candidates.iter().any(|root| root.txg == anchor.txg) { candidates.push(*anchor); }
+        }
```

`probe_b` / `probe_c` 只加了三个不影响 `anchor_mode = false` 路径的开关：
失败组合全枚举（`all_patterns_of_size`）、`PROBE_UNCAP` 把 `maximum_empty_publications()` 的
`B − 1` 换成 200、以及 `deletes_per_window` / `actual_metadata_cost` / `instance_switch_hold` 三个字段。
`probe_d` 只加了两个计数探针（`fallback_hits`、`chosen_below_floor`），一行行为都没改。

**对照证明探针没把基线弄坏**：`probe_a` 的 `baseline`（`anchored=0 credits=1`）行与
`research/results/e139-tightened-floor-2026-09-12.out` 的 `near_full` 行逐字段相同，例如
`write_after_delete_failed=48`、`max_lag=44`、`avg_lag_x100=3380`、`min_states=4`、
`forced_per_window_x100=114`（G2S，S = 16，c₀ = 1）与 `max_publications=7 min_states=1 forced_per_window_x100=131`（G2R，同格）。
