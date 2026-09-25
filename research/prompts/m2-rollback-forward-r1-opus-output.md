# m2-rollback-forward-r1 云端攻方腿（Opus）报告

轮名 m2-rollback-forward-r1（设计轮第一轮）；攻击面：F1、F2 的失败面（历史里会不会落到错的版本、块会不会被复用），F3 里的「可以删」，F4 的代价数。写于 2026-09-25（UTC 09:00–11:30 前后，JST +9）。

## 〇、复跑命令与文件

模型目录 `research/prompts/m2-rollback-forward-r1-opus-model/`：`prototype.patch`（冻结副本上 `mount.rs`、`allocator.rs` 的改动）、`rbf_attack.rs`（harness 用例，18 条）、`rerun.sh`、`SHA256SUMS`（本报告末段贴原样）。

```
bash research/prompts/m2-rollback-forward-r1-opus-model/rerun.sh /tmp/claude-1000/m2-rollback-forward-r1/tree <草稿目录> [full]
```

它把冻结副本拷到草稿目录、打补丁、放进用例，经 `run-with-memory-cap.sh 20G` + `capped.sh 12` 跑 `cargo test --release -p singlefs-harness --test rbf_attack`；`full` 另跑两条全量崩溃枚举。产出 `rerun-fast.out`、`rerun-full-unmount.out`、`rerun-full-rollback.out`（只留 `RBF` 行与 `test result` 行）。**所有数都是副本上的，不是入库装置上的；主 agent 要引，按规则在入库装置上重做。**

## 各格判定一览

「量过」= 这条腿在冻结副本拷贝上的原型里跑出来的（贴的是 `rerun-fast.out` / 全量两份的原样行）；「推的」= 按代码推、没实现没跑。**所有数都是副本上的数，不是入库装置上的数。** 下文「原型」指第二节那份最小规格。

| 编号 | 格 | 攻什么 | 判定 | 量过 / 推的 |
|---|---|---|---|---|
| H1 | F2 | B（卸载时抬 F 到最新根）之后 F 回落：盘 1 上带新 F 的根坏 2–3 条 ⇒ F_生效 回到 0 ⇒ 候选集重新收进 F 之下、块已被复用的根 ⇒ 回退被接受 | **打中，分辨臂**：放开「重开后写几次 × 回退目标」15 格，做 B 12/15 格打中（其中 5 格回退之后的最新根读不出文件），不做 B 0/15 | 量过（t6s） |
| H2 | F2 | B 按字面越过 D16 已定项 1 的抬 F 上限 | 打中（判据要改）：卸载那一串全量 327690 个崩溃状态里 I-7.9 红 262151 个；oracle 违例 0 | 量过（t5a 全量） |
| H3 | F1 | 回退那次发布只「释放被丢掉那几代独有的块」、不把 R_old 引用而之后已释放的块改回已分配 | 打中（零故障、主路径）：回退之后 I-3.9、I-3.11 当场红，下一次覆盖写被发布路径拒（`ReleaseTargetAlreadyReleased`） | 量过（t1 ReleaseOnlyNoRevive 臂） |
| H4 | F1 / F4 | 按诞生代号剪枝走最新的树找「独有的块」，在「刚回退又回退」上 | 打中（泄漏方向，零故障）：第二次回退漏 2–4 个落点，I-3.11 红；精确差集那一臂同一段历史不红 | 量过（t3） |
| H5 | F1 | 记账树「按这次发布现算」时 inode 号水位跟着 R_old 回退 | 打中（零故障）：回退跨过建 inode 的代之后 I-9.6 红；号会被重发 | 红是量过的（t3、t9 W2）；改法推的 |
| H6 | F3 | 「影子账是为回退抛弃更新的代才有的，可以删」 | **打穿「可以删」**：一段没有任何管理员回退的历史（崩溃恢复因暂时读错落到旧根、实例表抛弃较新的根，之后那条根又读得出），关影子账，C 的数据单元被复用，改坏 7 条根后恢复落在 C 读不出；开影子账读回 C | 量过（t7c） |
| H7 | F3 | 「候选集按实例表判仍然有效那一条可以删」 | 未打穿到读错：删掉之后原型自带的「同槽同分配代」核拒了；两道都删的那一格没造 | 量过一半（t8b），另一半推的 |
| H8 | F1 | 回退那次挂载每一个崩溃点，另加「最新 1–3 条根读不出」 | 没打中：全量 524301 个状态 oracle 违例 0；每 200 个抽 1 个（2622 个）再改坏最新 1–3 条根，0 格读错 | 量过（t4 全量） |
| H9 | F1 | 回退之后最新根读不出（改坏最新 1..k 条根，k 到环里只剩 mkfs 根） | 没打中：每一深度都落在一版读得对的内容上 | 量过（t5b） |
| H10 | F1 | 崩溃恢复落到更旧的根之后再回退、再写两次、再改坏比 C 新的 7 条根 | 没打中：落在 C、读回 C，落点表无坏格 | 量过（t11） |
| H11 | F1 | 原型造出来的镜像上 I-3.9 红 | **判据 / 原型构造，不算打中**：R_old 的固定点单元被复活后在同一次发布里换下，释放代 = 回退那次的 txg，落在 I-3.9 的区间 (最后引用, 最早不引用] 之外；方向是晚放（安全侧） | 红是量过的；归因推的 |
| H12 | F3 | 回退见证：向前发布之后不再有被管理员回退抛弃的根，见证可删 | 没打穿「可以删」；另见 t7：崩溃恢复抛弃的根暂时读不出时影子账算不到它们（隔离 0），之后那几条根回来、新实例的根全坏，恢复落在被抛弃的根上读不出——**两臂一样中、今天也中**，见证从来不罩这一形 | 量过（t7，不分辨臂） |
| F4 | F4 | 回退那次发布的代价、卸载那一串的代价、一个节点读不出 | 数见第五节；原型里「独有的块」从两棵分配记录树求差（挂载时整棵读进内存），不走用户可见的树 | 量过（t9、t10、t12） |

## 二、原型的最小规格（这条腿自己定的，不是条款，被攻过零轮）

改动全在 `prototype.patch`（冻结副本 `crates/singlefs-core/src/mount.rs`、`allocator.rs` 两个文件）；产品路径的函数一个都没改行为，新加的都以 `prototype_` 打头。

**P-FWD：挂载时做的向前发布回退**（`prototype_mount_rollback_forward`）

1. 照可写挂载恢复：择根、施加前缀，得最新那一版 E（实例 i_E、txg T_E），从 E 的账重建分配器（影子账照开，`extra_abandoned` 恒假——这次回退不抛弃任何根）。
2. 候选集照今天：R_old 在环里、txg ≥ F_生效、按 E 那一版的实例表仍然有效、不被见证抛弃。
3. 从盘上重建 R_old 那一版，作「上一版」发写行那次发布：树表条目、inode / extent 树、映射树照 R_old；实例表取 **E 那一版的行** 加这次挂载的普通恢复行 (i_E, T_E, W)（不写回退行、不写见证条目）；txg = 环里 max + 1、计数器接最大 jsn、树 ID 水位取环里 max，同可写挂载；之后暖机同可写挂载。
4. 分配器从 E 的账起按两棵账逐条调整（第一版两盘同槽，按盘 0 的记录认落点）：
   - E 里仍分配、R_old 里没有同槽同分配代的仍分配记录 ⇒ **释放**，释放代 = 回退那次发布的 txg；
   - E 里已释放、R_old 里仍分配 ⇒ **复活**：改回仍分配，分配代写回 R_old 那条记录的分配代（I-3.10）；
   - R_old 仍分配的落点在 E 的账里没有记录、或被别的分配代占着 ⇒ 拒（`TargetUnitNotProtected`，这一道条款里没有，是原型加的）。
5. 臂：`Full`（上面全做）、`ReleaseOnlyNoRevive`（只做释放；为了发得出去，只复活这次发布要换下的 R_old 固定点单元）、`BirthPrunedRelease`（释放只取 E 里分配代 > T_old 的，= 按诞生代号剪枝走最新的树）、`FullWithoutProtectionCheck`（去掉第 4 步第三条，只靠候选集挡）、`FullWithoutTableValidityCheck`（候选集去掉按实例表判有效那一条）。

**B：卸载时抬 F 到最新根**（`prototype_unmount_raising_the_floor_to_the_newest_root`）：调今天的 `raise_rollback_floor` 那一整段（整串预演、回收先扣住、推空发布直到每块盘都有带新 F 的根、再放开），新 F = 现行那一版的 txg，**不判抬 F 的上限**（`prototype_raise_rollback_floor` 的 `bypass_ceiling`）。

另加一个只供原型的挂载入口 `prototype_mount_writable_with_shadow_ledger`（可写挂载，影子账那一臂由调用方给），F3 的「影子账可以删」用它量。

**这份规格的已知毛病（报告里各处都照它读）**：以 R_old 那一版为「上一版」发布，发布路径会换下 R_old 的固定点单元（树表、实例表、记账树、映射树、分配记录树那几片），所以第 4 步先把它们复活、再由这次发布释放——这是 H11 那条 I-3.9 红的来源；记账树按 R_old 那一版现算，inode 号水位跟着回到 R_old 的值——这是 H5。

## 三、F2：B（卸载时抬 F 到最新根）的失败面

### H1　B 之后 F 回落，回退候选集收进块已被复用的根（打中，分辨臂）

**历史**（`t6s_sweep_user_steps`；前缀与故障写死，用户那几步放开扫）：

1. 实例 1：A（txg 3）、B（4）、C（5）。
2. 卸载：B 臂抬 F 到 5，推空发布 txg 6（盘 0）、7（盘 1），两块盘各一条带 F = 5 的根 ⇒ 生效；释放代 ≤ 5 的落点回收。对照臂不做这一步。
3. 重开可写挂载（实例 2，写行与暖机），**用户写 0..4 次覆盖写**（放开扫）。新根都带 F = 5（新实例的根带的 F = 恢复后生效的 F）。重开之后的第一次覆盖写就把回收的槽发出去（`t5b` 那一行 B 臂 `D_data_slot=50178`：那是 A 的树表单元的槽，对照臂是 50186）。
4. 故障：盘 1 上 txg ≥ 6 的根全坏（1–3 条根槽；B 臂里带新 F 的根只有它们住在盘 1）。
5. **管理员回退到 A / B / C**（放开扫），按原型 `FullWithoutProtectionCheck` 臂：候选集照条款——按实例表有效 ∧ txg ≥ F_生效 ∧ 不被见证抛弃。

每一步许可它的条款（kb 快照行号）：

- 第 2 步：`16-发布语义.md:37` 整行 `| 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |`；回收按 `16-发布语义.md:33` 整行 `| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |`。B 本身越过 `16-发布语义.md:36` 那条上限，见 H2。
- 第 4 步之后：盘 1 上一条带 F = 5 的根都没有 ⇒ 按第 37 行的「各幸存盘所带 F 最大值的最小值」F_生效 回到 0（`t6` 那几行 `effective_floor_after=0`）。
- 第 5 步：候选集按 `16-发布语义.md:38` 整行 `| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |`，A、B、C 的 txg ≥ 0 ⇒ 都在候选集。

**结果**（副本，`rerun-fast.out` 原样行在第四节之后的「原样行」小节）：B 臂 15 格里 12 格打中（写 0 次的 3 格不中——还没有东西复用），其中 5 格回退之后新根下读不出文件（`read_ok=false`），另 7 格读得对但 checker 判 I-7.4 / I-4.8 / I-2.1 / I-5.1 / I-3.1 红（候选集里别的根引用的块已被复用；新根的账把被复用的槽记成 R_old 那一版的分配代，I-3.10 红）；对照臂 15 格 0 格打中。`t6s-summary b=false hits=0/15`、`t6s-summary b=true hits=12/15`。

**四句**：

1. 分辨臂：分辨「做 B / 不做 B」——同一段前缀、同一个故障，只差第 2 步；用户那两步（写几次、回退到哪）放开扫过，不是装置写死的后缀造出来的（写 0 次那一行就是后缀选坏了会看到的「不中」）。
2. 被判系统看不看得到判别它的东西：**看不到**。回退那一刻盘上剩下的是：盘 0 上几条带 F = 5 的根、盘 1 上一条都没有。这与「抬 F 那一串崩在盘 1 那一次之前」（F 从没生效、什么都没回收）在盘上逐条相同，而两种情形要的候选集相反。这正是 `16-发布语义.md:52` 那条「「生效」那一行已被用户打回重议」写的回落机理（C419），B 把它从「准入不够才抬 F」的角落变成每一次正常卸载都走的主路径。
3. 满足的是判据字面哪一句：正文第一节共用问句「那一版引用的块有没有被复用」——回退落到的那一版（A）引用的块被复用；I-7.4 按字面「回退候选集里每一个根……所引用的块」（`invariants.md:54`，只抄行首定位，整行见快照）在回退之后的镜像上判红。
4. 改法在打中的格上还中不中：
   - 原型自带的「R_old 仍分配的落点在最新那一版的账里是同槽同分配代」核（`Full` 臂）：**量过，只挡一半**——`t6` 回退到 A 被拒（`TargetUnitNotProtected { slot: 50180 }`），回退到 B 被接受、新根读得对，但 checker 仍判 I-7.4 红（「候选根 txg 0 引用的单元已被复用」）：它护得住 R_old，护不住候选集里别的根。
   - F 另在系统配置里记一份（每盘两槽，随轮换写）：推的，没实现。能让「盘 1 上带新 F 的根全坏」这一格不回落；代价是系统配置多 8 字节、D22 已定项 9 字段表动。
   - 恢复后的生效值改取「各幸存盘所带 F 的最大值」（不取「最大值的最小值」）：推的，没实现。在 H1 的格上 F_生效 = 5，A、B、C 都不在候选集，回退被拒——修得到这一格。它在「抬 F 那一串崩在盘 1 那一次之前」那一格上让 F 提前生效：那时回收的槽还扣着没发，提前生效只是让盘 1 上 F 之下的根退出候选集，不致读错。第一版两块盘、掉一块只许只读挂载，「带新 F 的那块盘整块没了」走不到回退。没在副本上量。
   - 不做 B（照今天的上限抬）：等于不采纳 B，不算改法。

### H2　B 按字面越过抬 F 的上限，I-7.9 在全部崩溃状态上红（判据要改）

`16-发布语义.md:36` 整行 `| 抬 F 的上限 | min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根) |`。B 要的 F = 最新根，而最新根与「第 4 新的非空根」只在环里非空根不足 4 个且最新根就是最旧有效根时才相等——第一版池一次写都不到位。

卸载那一串（A B C 之后，txg 6、7 两次空发布，各写 8 个单元、21 次写调用、279040 字节）全量 327690 个崩溃状态：oracle 违例 0、恢复失败 0；I-7.9 判红 262151 个（`rerun-full-unmount.out`）。不做 B 的对照臂 checker 全绿（`t5b rollback=false b=false checker_reds=[]`）。

四句：分辨臂（只有 B 臂红）；系统看得到（F 与上限都在盘上）；满足的是 I-7.9 的字面（F 高于上限），不是任何数据错——oracle 0 违例；这是「判据自己写着今天的规则」，B 要落地就得同时改 D16 已定项 1 的上限一行与 I-7.9，不是 B 的缺陷。改法：给卸载那一串另立上限「每块盘上最新的持久有效根」（推的，没实现；checker 那一侧要分得出「卸载那一串」与「准入抬 F」，今天盘上没有字段分）。

### H2b　B 之后崩溃兜底能退几步（量过，不是打中，给本地腿对数用）

`t5b`：A B C →（B 臂抬 F 到 5，txg 6、7）→ 重开（实例 2，txg 8–10）→ D（11）E（12）。对每一条环里的根 X，把比它新的根全改坏：

| 改坏最新几条 | 不做 B | 做 B |
|---|---|---|
| 0–4 | 落 (2, 最新) 读对 | 同 |
| 5–9 | 落实例 1 的根、施加记录追到 C 那一版，读对 | 落实例 1 的根、追到 (1, 7)，读对 |
| 10 起 | 退到 mkfs / 暖机根，报「没有文件」（合法：那几条根下本来没有文件） | 退到 txg ≤ 2 的根，**走读失败** `UnitUnreadable { slot: 50178 }`（那个槽已被 D 复用） |

B 之后退到 F 之下的根是「读出错」而不是「读出错的内容」：校验和挡住了，没有静默错数据。这是 B 的设计后果（用户原话「只保证快照永久」），不算打中；记下来是因为 H1 那一格的区别就在这里——H1 是**管理员主动回退**到 F 之下的根，那一路今天没有校验和之外的第二道。

## 四、F1：向前发布回退的失败面

### H3　只释放「被丢掉那几代独有的块」、不复活（打中，零故障）

正文第一节 F4 那一行写的是「回退那次发布要释放『被丢掉那几代独有的块』」。只做这一半时：R_old 引用、而在它之后被换下的块（A 的数据单元、A 的 inode 叶与根、extent 根……）在最新那一版的账里是「已释放 + 释放代」，新根又引用它们。

历史（`t1` 的 `ReleaseOnlyNoRevive` 臂）：A（3）B（4）C（5）→ 重开回退到 A（txg 6、7）→ 覆盖写两次。结果：回退之后当场 I-3.9 红（「盘 1 槽 50180 的分配记录带已释放标志（释放代 4），而候选集里每一条有效根都还引用它」）、I-3.11 红；下一次覆盖写被发布路径拒（`ReleaseTargetAlreadyReleased { unit: Data(..), slot: 50180 }`）。若发布路径不拒，释放代 4 ≤ 回收门槛那一刻起 50180 就会被发出去，而新根还指着它——零故障的复用。`Full` 臂同一段历史 checker 只剩 H11 那一条。

四句：分辨臂（Full / 只释放）；系统看得到（两棵账都在盘上）；满足共用问句「那一版引用的块有没有被复用」的前一步（块在账里是可回收的）与 I-3.9 字面；改法「复活：R_old 仍分配、最新那一版已释放的落点改回仍分配、分配代写回 R_old 记录里的」在这一格上量过不中。

### H4　按诞生代号剪枝走最新的树，在「刚回退又回退」上漏释放（打中，泄漏方向）

D5 已定项 14 的引用区间 `birth(b) ≤ S.txg < death(b)` 自己在射程里限定了只对单向线性历史成立（`05-快照-空间记账机制.md:328`，整行见快照，这里不摘句）。向前发布回退之后同一个块可以「死在 T_a、在回退那次复活」：A 的单元诞生 3、在 B（4）被换下、回退根（6）又引用它。再回退到 B 时，最新那一版里诞生 3 ≤ 4 的节点被当成「B 也引用」剪掉，实际 B 不引用 ⇒ 漏释放。

历史（`t3`）：A B C → 回退到 A（6、7）→ 中间一步放开扫（覆盖写 / 建 5 个 inode / 什么都不做）→ 关掉 → 第二次回退到 B（1,4）、C（1,5）、或第一次回退的根（2,7）。

| 中间一步 | 第二次回退目标 | 精确差集臂释放 | 剪枝臂释放 | 剪枝漏掉 | 剪枝臂额外红 |
|---|---|---|---|---|---|
| 覆盖写 | B / C / (2,7) | 13 / 13 / 12 | 同 | 0 | 无 |
| 建 5 个 inode | B / C | 13 / 13 | 11 / 11 | 2 | I-3.11 |
| 什么都不做 | B / C | 13 / 13 | 9 / 9 | 4 | I-3.11 |

「覆盖写」那一行全不中：第一版的覆盖写把文件那一支整条 COW，最新那一版里没有诞生 ≤ T_old 还活着的节点——**装置把后缀写成覆盖写就会报「剪枝没问题」**，中间一步放开之后才露出来。方向是泄漏（漏释放的槽之后永远是「已分配、不被最新根引用」，I-3.11 当场红），不是复用；剪枝臂从不会把目标引用的块释放（诞生 > T 的块不可能被 T 引用）。

四句：分辨臂（剪枝 / 精确差集）；系统看得到（精确差集用的就是盘上两棵分配记录树）；满足 I-3.11 字面；改法：原型的精确差集（最新那一版的账挂载时整棵已在内存，另读 R_old 那一版的分配记录树 5–7 个节点，见第六节）量过不中；「剪枝时把阈值换成『最近一次向前回退的 txg』与 T 取小」推的，没实现。

### H5　记账按 R_old 现算时 inode 号水位跟着退回去（打中；一半是 checker 的读法）

`t3d`：回退到 B（(1,4)，只有 inode 1）之后新根的记账行 inode 号水位 = 2，而中间那一版（仍是有效根、仍可恢复、仍可回退到）用过 2..6。`t3`「建 5 个 inode」那几行 I-9.6 红：「记账里的 inode 号水位 2 不大于 inode 树内最大 key 6」。

判据那一半：checker 走完最新根之后还走候选集里每一条根（`crates/singlefs-checker/src/walk.rs:4682` 的 `walk.walk_root(&root.record_bytes, false);`），「inode 树内最大 key」跨根累加，而 kb `invariants.md:276` 按「该树」判（整行见快照）。按 kb 字面，最新根那棵 inode 树里最大 key 是 1，水位 2 > 1 成立——**打中归错了判据的一半**。但实情不因判据写法消失：今天的回退把中间实例整段抛弃，I-9.6 那一行（`invariants.md:276`）因此把「回退之后号会重发」当成允许的；向前发布之后中间几代是**有效的早先状态**，号重发之后同一个 inode 号在两条仍可恢复的根下指两个不同对象（码 3 容器身份里 inode 叶的容器号取 inode 号，`18-块里携带什么信息.md:279` 容器号那一行，整行见快照）。

改法：水位取「根环里全部根的 max」，与树 ID 水位同法（`tree_identifier_watermark_of_the_ring`）——推的，没实现。

### H8　回退那次挂载的每一个崩溃点，外加最新 1–3 条根读不出（没打中）

流：A B C 之后进程退出，重开走 P-FWD 回退到 A（写行 txg 6、暖机 7；46 次写、8 段）。全量 524301 个崩溃状态（等于闭式数），oracle 违例 0、恢复失败 0；I-3.9 红 262151 个（H11）。每 200 个状态抽 1 个（共 2622 个）另把最新的 1、2、3 条可读根改坏再恢复：读错 0 / 0 / 0（`rerun-full-rollback.out`）。探针取样是缩过的：不抽样要 ×200，按单状态实测估超 40 分钟。

### H9　回退之后最新的根读不出（没打中）

`t5b` 不做 B 的回退臂：A B C → 回退到 A（6、7）→ 重开（8–10）→ D E（11、12）。改坏最新 k 条根，k = 0..12：k ≤ 4 落实例 3 最新；5–6 落 (2,7)（回退根，读回 A）；7–11 落实例 1 的根并施加到 C，读回 C；12 退到 mkfs 根报没有文件（合法）。没有一格读错、没有一格读到被复用的块。回退之后更早的几代仍是合法的早先状态，这是向前发布相对今天最直接的收益：今天同一形要靠见证才不落到被抛弃的根上。

### H10　崩溃恢复落到更旧的根之后再回退（没打中）

`t11`：C 的根与数据单元暂时读不出 ⇒ 恢复落到 B、实例 2 写行抛弃 C；撤故障（C 又读得出）；再重开向前回退到 A，覆盖写两次；把比 C 新的 7 条根改坏。恢复落 C、读回 C；落点表每一深度无坏格。C 的 14 个槽由影子账隔离着（`isolated=[(0,14),(1,14)]`）——这一格不中靠的正是影子账，见 H6。

### H11　原型镜像上的 I-3.9 红（判据 / 原型构造，不算打中）

`Full` 臂回退之后恒有一条「盘 1 槽 50245 的分配记录释放代 6 不在 (3, 4] 里」。50245 是 A 那一版的固定点单元：原型以 R_old 为「上一版」发布，先复活再在同一次发布里换下，释放代 = 6；按 I-3.9 应在 4。晚放是安全侧（多钉住两代），不致复用。它是这份原型的构造毛病（第二节末段），正推腿若把回退发布的「上一版」取成最新那一版、只把树表条目换成 R_old 的，就不会有这一条——推的。

## 五、F3：对「可以删」造要红的历史

这条腿看不到正推腿那张表（禁读），下面按正文第二节列的机制逐样攻「可以删」这一种判法；每行写删了之后哪段历史红、量没量。

| 机制（冻结副本） | 「可以删」打不打得穿 | 历史 | 量过 / 推的 |
|---|---|---|---|
| 影子账（`mount.rs` `isolate_slots_referenced_only_by_abandoned_roots`、`ShadowLedger`；D28 已定项 1 第九项） | **打穿** | H6 | 量过 |
| 候选集里「按实例表判仍然有效」（`abandoned_by_table`，`mount.rs:576`） | 部分：删了之后被原型自带的同槽同分配代核拒；两道都删没造 | H7 | 一半量过 |
| 回退见证（`rollback_witness.rs`、系统配置偏移 481 起 753 字节、`choose_root` 跳过被见证抛弃的根、I-7.10 / I-7.11） | 打不穿：P-FWD 不写见证，全部用例里见证表恒空，没有一格因此读错 | H8–H10、H12 | 量过（只在原型上；旧镜像带非空见证表那一格没造） |
| 回退行（实例表 flags bit0）与「重放遇回退行截断到 W」（`rollback_high_water_of_root`） | 没攻：P-FWD 不写回退行，截断那一支在原型里走不到 | — | 推的 |
| 抬 F 的上限（`rollback_floor_ceiling`）与 I-7.9 | 不是「删」而是「B 要改它」：见 H2 | H2 | 量过 |
| 新实例首个 txg = max(环里根, 环里记录) + 1（`first_txg_of_new_instance`） | 没攻（P-FWD 照用） | — | — |
| 树 ID 水位取环里 max（`tree_identifier_watermark_of_the_ring`） | 没攻；H5 说明同一类「水位要取环里 max」对 inode 号也要 | H5 | 推的 |

### H6　影子账：一段没有任何管理员回退的历史，删了之后块被复用（打穿「可以删」）

影子账的条款主句写在管理员回退那一段里（`23-journal的角色与格式.md:378`，整行见快照），所以容易读成「回退专用」。但冻结副本的**普通可写挂载**同样开着它：`mount.rs:2423` 那一行 `        ShadowLedger::On,`（`mount_writable_with_space_admission` 调 `rebuilt_allocator` 的那一处），「被抛弃」按最新根的实例表判（`abandoned_by_table`）——崩溃恢复写的普通行同样抛弃根。

历史（`t7c`，两臂只差第 4 步的影子账开关）：

1. 实例 1：A（3）B（4）C（5）。
2. 进程退出；重开时 C 的根槽（盘 1）与 C 的数据单元（两盘槽 50184）**暂时读不出**。择根落 B；C 那条记录施加前验点名单元失败 ⇒ 不施加；实例 2 写行 (1, 4, W)、暖机（txg 6、7——首个 txg 按环里读得出的记录取 max + 1，没盖 C 的根槽）。许可：`23-journal的角色与格式.md:378` 所在已定项 14 的前缀六条（施加前逐项验证点名单元），与 D18 已定项 11 的「恢复写的行」。
3. 撤故障：C 的根与数据单元又读得出。C 按实例 2 那张表 (1, 4, W) 被抛弃。
4. 再重开（实例 3，txg 8–10）：影子账开 ⇒ 隔离 C 独占的 14 个槽；关 ⇒ 0。
5. 覆盖写 D、E。关影子账时 D 的数据单元落 **50184**（C 的数据槽），开时落 50186。
6. 比 C 新的 7 条根全坏（实例 2、3 的根）。恢复择 C（按见证不抛弃、按剩下的根里 txg 最大），施加不到任何记录。

结果：开影子账 `under_C_intact=Ok(true)`、落 (1,5) 读回 C；关影子账 `under_C_intact=Err("MappingStillUnreadable { slot: SlotNumber(50184) }")`、恢复落 (1,5) 走读失败。

四句：分辨臂（开 / 关）；被判系统看得到（第 4 步 C 的根读得出，实例表判它被抛弃）；满足共用问句「那一版引用的块有没有被复用」字面（C 引用的 50184 被 D 复用）；改法：删影子账在这一格上中，留着不中。**所以 F3 里影子账不是「可以删」，是「崩溃恢复（与实例切换）还要它」**。向前发布只让管理员回退不再产出被抛弃的根，崩溃恢复落到较旧的根照样产出。

### H7　候选集按实例表判有效

`t8b`：H6 那段历史走到第 3 步（C 被实例表抛弃、又读得出），向前回退到 C。`Full` 臂按实例表拒（`OnAbandonedTimeline`）；删掉这一条的臂被原型的「R_old 仍分配的落点在最新那一版的账里同槽同分配代」核拒（`TargetUnitNotProtected { slot: 50184 }`——C 的数据单元在 B 那一版的账里没有记录，只被影子账在内存里隔离）。两道都删的那一臂没造：推的后果是新根引用一个账里空闲的槽，下一次分配就复用——零故障。结论：这一条与原型那道核互为备份，至少留一道；条款里只有前一道。

### H12　见证删掉之后：一格两臂同中的越格线索（不分辨臂，不拿它判）

`t7`：实例 2 的四条根在重开时全部**暂时读不出**（不是只坏一条）⇒ 影子账读不到它们的账（`isolated=[(0,0),(1,0)]`，两臂一样），实例 3 的 D、E 复用了它们的槽；撤故障后再把实例 3 的根全改坏，恢复落在 (2,8) 走读失败（`UnitUnreadable { slot: 50304 }`）。影子账开 / 关两臂逐字相同，冻结副本同样中：`mount.rs:604` 整行 `/// 树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载；候选根读不出就当它什么都不豁免（隔离只会多不会少）。`（这里的「读不出」连根槽读不出也在内：根读不出就根本不在被抛弃根的清单里）。它不分辨任何一条臂，病根在「被抛弃的根暂时读不出时它引用什么无从得知」这一共用前提，按「判据自己也会写错」那一节另立一笔账，不拿它判见证删不删。

## 六、F4：代价（原型上量的，副本数）

几何：两块 4 GiB 盘、R = 3、S = 8（`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM`）、分配记录树高 3。起点 A = 第一个文件 3000 字节（txg 3）。回退目标恒为 A。「释放」「复活」按一块盘上的落点条数 / 槽数计（两盘各一份，第一版同槽）。

**回退那次发布**（`t9`；负载 W1 = 每代覆盖写那个单单元文件，W2 = 每代新建 500 个 inode，W3 = 每代顺序写一个 20 个数据单元的文件）：

| 负载 | k（跨几个非空状态） | 释放 条 / 槽 | 复活 条 / 槽 | 剪枝要读的节点（非数据） | 其中用户可见树的节点 | 剪枝释放的数据单元 | 写行那次写的单元 | 暖机次数 |
|---|---|---|---|---|---|---|---|---|
| W1 | 1 / 2 / 3 / 4 | 12 / 14（四档同） | 12 / 14 | 11 | 3（extent 根、inode 叶、inode 根） | 1 | 9 | 2 / 1 / 1 / 2 |
| W2 | 1 / 2 / 3 / 4 | 12/15、14/19、16/23、18/27 | 10 / 11 | 12 / 14 / 16 / 18 | 4 / 6 / 8 / 10（inode 叶 3/5/7/9 + 根） | 0 | 9 / 9 / 9 / 11 | 2 / 1 / 1 / 2 |
| W3 | 1 / 2 / 3 / 4 | 32/53、32/53、34/55、34/55 | 12 / 14 | 12 / 12 / 14 / 14 | 4（extent 下段 1、extent 根、inode 叶、inode 根） | 20 | 9 / 11 / 11 / 11 | 2 / 1 / 1 / 2 |

读法：

- 释放量跟的是**最新那一版与 R_old 的差**，不跟 k：中间几代被换下的块早已是「已释放 + 释放代」，回退一条都不碰。W1、W3 每代把整个文件 COW 一遍，k 从 1 到 4 数不变；W2 每代往 inode 树里加叶，差随 k 线性长（每代 +2 条、+4 槽）。
- 「剪枝要读的节点」包括四样固定点（树表、记账树、映射树、分配记录树 5–7 片）——它们本来每次发布都换，读它们不是剪枝的代价；只算用户可见的两棵树是第六列。
- 原型不走树：它比两棵分配记录树。最新那一版的账挂载时已整棵读进内存（`08-核心索引结构.md:387`「挂载怎么读」那一条），多读的只有 R_old 那一版的分配记录树：**5 个节点**（W3 k ≥ 3 时最新那一版 7 个，R_old 恒 5）。与 H4 合起来：比账比走树便宜，且不受「刚回退又回退」的非线性影响。
- 写行那次发布多写的是分配记录树里变了的叶与祖先：W1 9 个单元（与普通写行同数：实例表 1 + 分配记录树 5 + 记账 1 + 映射 1 + 树表 1）；W3 k ≥ 2 时 11 个。差集跨更多叶时这一数跟着长，而写行那次的元数据按条款走切换预留（`16-发布语义.md:40` 准入那一行，整行见快照）——预留按「一次切换的最坏量」算，没算回退那次要改多少片分配记录树的叶。准入紧张时的回退这一格**没造**（见第八节）。

**一个节点读不出**（`t12`，A 之后建 300 个 inode、再覆盖写一次，然后回退到 A）：

| 读不出的是 | 向前回退 | 普通可写挂载 |
|---|---|---|
| R_old 那一版分配记录树的一片叶（两份都坏） | 拒（`MappingStillUnreadable { slot: 50245 }`，重建 R_old 那一版时） | 照常挂上（实例 2） |
| 最新那一版的一片 inode 叶容器 | 拒（恢复重建最新那一版时） | 同样拒 |
| 最新那一版分配记录树的一片叶 | 拒 | 同样拒 |

漏掉的槽 = 0：原型先把两版整个重建出来，读不出就在任何写之前拒。走树剪枝的规格在同一格上漏掉的是那个节点整棵子树里诞生 > T 的块（推的，没实现），方向是泄漏。

**卸载那一串（B）**（`t10`、`t5a`）：

| 负载（4 代之后卸载） | 空发布次数 | 每次写的单元 | 每次写调用 / 字节 | 盘 0 的 defer 槽 前 → 后 |
|---|---|---|---|---|
| W1 | 3（txg 8、9、10） | 8 | 21 / 279040 | 57 → 24 |
| W2 | 3 | 8 | 21 / 279040 | 45 → 24 |
| W3 | 3 | 8 | 21 / 279040 | 176 → 24 |
| A B C 之后（`t5a`） | 2（txg 6、7） | 8 | 21 / 279040 | — |

次数由根环落点定：现行 txg 5 ⇒ 6（区域 0，盘 0）、7（区域 1，盘 1）两次；现行 7 ⇒ 8（区域 2，盘 0）、9（区域 0，盘 0）、10（区域 1，盘 1）三次——区域 0、2 都在盘 0，碰上就白推一次（与 D16 已定项 8 写的「白推」同一机理）。卸载之后 defer 里剩的 24 槽是这一串空发布自己换下的固定点（释放代 > 新 F），每次 8 槽 × 3。

## 七、没打中的形状与取样范围

- 回退那次挂载的每个崩溃点：全量 524301 个状态（闭式数，8 段全展开），oracle 0、失败 0；另 2622 个状态（每 200 个抽 1）×「改坏最新 1、2、3 条根」至多 7866 次恢复（环里根不够时那一档跳过），读错 0。只有一条流：A B C → 回退到 A；回退到 B、C、中间实例根的崩溃流没枚举。
- 卸载那一串的每个崩溃点：全量 327690 个状态，oracle 0、失败 0（I-7.9 的红见 H2）；没做「改坏最新几条根」的探针（`RBF_NOPROBE`）。
- 回退之后最新根读不出：`t5b` 四臂 × 改坏 0..k 条（k 到环里只剩 mkfs 根），不做 B 的两臂每一深度读对。
- 崩溃恢复落到更旧的根之后再回退：`t11` 一段历史，读对。
- 刚回退又回退：`t3` 3 种中间步 × 2–3 个目标 × 2 臂，精确差集臂只剩 H11 那条 I-3.9（与 H5 的 I-9.6），没有读错、没有复用。
- 把回退 / 卸载之后的镜像跑 checker：除 H2、H5、H11 列的那几条之外全绿。
- 单次观测：原型与装置都是确定性的，同一份 `rerun.sh` 从冻结副本从头起跑了三遍（两遍 fast、一遍 full），`RBF` 行逐行相同（差别只在我改用例之后新加的行）；「没打中」按规则仍只算这一条腿的一次观测。

## 八、这条腿自己的限度

- 原型只做了**挂载时**回退（新实例）；「挂着的时候回退」（同一实例、记录链不断）没做，那一路上「崩在回退之后、新根只落一块盘，再从更旧的根施加记录追上来」会不会把回退那次记录重放回来（推的：会，而且那是对的），没量。
- 原型的回退发布以 R_old 为「上一版」，带出 H11 与 H5 两条构造毛病；正推腿若取「以最新那一版为上一版、只换树表条目」，那两条会变，本报告的数不能直接搬。
- 准入紧张时的回退与卸载没造：原型的写行那次发布走的是今天的准入（取号之前 0 需求 + 预演），回退要改的分配记录树叶多于普通写行时预留够不够，这一格本该造、没造。B 那一串在准入紧张时被预演拒、卸载怎么办也没造（今天的整串预演会整串不发，推的：卸载报错而池可用）。
- 实例号撞号（S6）、E158 的择根四岔路、快照都没碰。
- 负载只有三档、盘只有两块 4 GiB、S = 8；k 只到 4。
- H1 的两条改法（F 进系统配置、生效取最大值）都是推的，被攻过零轮；原型的「同槽同分配代」核只在我的模型上量过、被攻过零轮。

## 九、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的数都没在入库装置上重做。
- 没读禁读清单里的文件（`m2-rollback-forward-r1-sonnet-*`、`m2-rollback-forward-r1-local-*`）。
- 没跑门禁、层 0、QEMU、全量 cargo test；只跑了自己加的测试二进制 `rbf_attack`（在草稿目录的副本里）。
- 草稿目录 `/tmp/claude-1000/m2-rollback-forward-r1-opus/`（副本、各次日志、复跑目录）没入库：产物已由 `rerun.sh` 的三份输出放进模型目录，副本本身能由 `rerun.sh` 从冻结副本重建。

## 十、复跑原样行与 SHA256SUMS

补写于 2026-09-25 12:10 UTC（21:10 JST），由接手的 agent 写；上面各节是前任写的，没改。

**这一遍复跑怎么来的。** 前任最后那次全量复跑（草稿目录 `rerun4/`）在「回退那条流」那一步被会话中断打断：`rerun-fast.out`、`rerun-full-unmount.out` 跑完了，`rerun-full-rollback.out` 是空文件，也没有进程在跑。接手后从冻结副本整份重跑了一遍（11:55–12:08 UTC，exit 0）：

```
bash research/prompts/m2-rollback-forward-r1-opus-model/rerun.sh /tmp/claude-1000/m2-rollback-forward-r1/tree /tmp/claude-1000/m2-rollback-forward-r1-opus/rerun5 full
```

跑之前核过：冻结副本的 `crates/singlefs-core/src/mount.rs` 的 sha256 与 `research/prompts/m2-rollback-forward-r1-snapshot/crates-sha256.txt` 里那一行相同（`c9f7a2b0…2cf3`）；复跑目录里那份 `rbf_attack.rs` 与模型目录里的逐字节相同（`cmp`）。模型目录里的三份 `.out` 已换成这一遍（`rerun5/`）的输出，`SHA256SUMS` 重新生成了。原因是 11:03 UTC 那份 `SHA256SUMS` 生成之后 `rerun.sh` 的过滤改过（按「RBF 」切行首），`rerun.sh` 那一行已经对不上。

**和前几遍比。**

- `rerun-fast.out`：旧过滤那一遍（`rerun3/`，11:03 放进模型目录的那份）143 行，这一遍 161 行。旧的每一行这一遍都有；多出的 18 行是每条用例输出的第一行。libtest 把「test 名 ...」印在这一行的开头，旧过滤 `^RBF` 把它们漏掉了，其中有 `t6s-summary` 所在用例的第一行、`t4`/`t5a` 的计数行、`t1b` 的 `ReleaseTargetAlreadyReleased` 行等。`rerun4/` 与 `rerun5/` 的 `rerun-fast.out` 去掉 `test result` 那一行之后逐行相同。
- 两份全量：`RBF` 行与 `rerun3/` 逐行相同，只有 `finished in` 的耗时不同（卸载那一串 202.88 s → 191.97 s，回退那条流 492.52 s → 488.10 s）。
- 第七节末那条写的是「从头起跑了三遍（两遍 fast、一遍 full）」。现在还要加上两遍：`rerun4`（fast 与卸载那一串跑完，回退那条流被打断）与 `rerun5`（full，跑完）。`RBF` 行的结论不变，而且这仍只算这一条腿的一次观测。

**报告里引的数与复跑逐条对。** 下面每一格在 `rerun5/` 的输出里都找得到原样行，对得上：

- H1：`t6s-summary b=false hits=0/15`、`b=true hits=12/15`；做 B 那一臂打中的 12 格里有 5 格 `read_ok=false`；写 0 次的 3 格是 `verdict=clean`；盘 1 上改坏的根槽不做 B 时 1 个，做 B 时 2–3 个。`t6 arm=Full target=(1, 3)` 被拒，原因是 `TargetUnitNotProtected { slot: SlotNumber(50180) }`；`target=(1, 4)` 被接受，`read=Ok(true)`，I-7.4「候选根 txg 0 引用的单元已被复用或抹头」；`effective_floor_after=0`。
- H2：`I-7.9=262151`，`states=327690 violations=0 failed=0`。
- H3：`t1 arm=ReleaseOnlyNoRevive` 的 I-3.9（槽 50180、释放代 4）与 I-3.11；`t1b` 两次覆盖写都得到 `ReleaseTargetAlreadyReleased { … slot: SlotNumber(50180) }`。
- H4：`t3` 那张表的释放数、漏数（13/13/12；11、漏 2；9、漏 4）。
- H5：`t3d cur wm=2`；「建 5 个 inode」那 4 行都有 I-9.6「记账里的 inode 号水位 2 不大于 inode 树内最大 key 6」。
- H6：`t7c` 开影子账时隔离两盘各 14 个槽，D 落在 50186，`under_C_intact=Ok(true)`；关影子账时 D 落在 50184，`MappingStillUnreadable { slot: SlotNumber(50184) }`。
- H7：`t8b` 两行（`OnAbandonedTimeline`；`TargetUnitNotProtected { slot: SlotNumber(50184) }`）。
- H8：见下面的原样行。
- H10：`t11` 隔离两盘各 14 个槽，`kill_newer_than_C(7)=Ok((1, 5))`。
- H12：`t7` 开 / 关影子账两行都落在 (2, 8)，`UnitUnreadable { slot: SlotNumber(50304) }`。
- 第六节 `t9` 表每一格（释放、复活、剪枝节点数、inode 叶 3/5/7/9、写的单元数、暖机次数 = `rollback_txgs` 长度 − 1）、`t10`（57/45/176 → 24）、`t12` 三行都对得上。
- 文中引的 kb 快照行（`16-发布语义.md` 第 33、36、37、38、40、52 行，`invariants.md` 第 54、276 行，`23-journal的角色与格式.md:378`，`05-快照-空间记账机制.md:328`，`18-块里携带什么信息.md:279`，`08-核心索引结构.md:387`）与冻结副本行（`mount.rs` 第 576、604、2423 行，`walk.rs:4682`）都按行号取出来看过，落在文中说的那一行上。

**有一处对不上：H2b 表的「不做 B」那一列。** 这张表写的是「0–4 落 (2, 最新)、5–9 落实例 1 的根、10 起退到 mkfs」，但这一列错了一格。不做 B 那一臂环里只有 10 条根（`newest=9`），复跑的实际分界是：改坏 0–3 条落在 (2, 9)，4–8 条落在 (1, 5)，第 9 条起落在 (0, 0)、读出「没有文件」。「做 B」那一列（0–4 / 5–9 / 10 起读失败 `UnitUnreadable { slot: SlotNumber(50178) }`）是对的。这处错不影响 H2b 的结论：不做 B 的每一深度都读对，做 B 的在 F 之下读失败，没有静默错数据。原样行见下。

### 原样行（`rerun5/`，副本上的数）

`rerun-full-rollback.out`（整份）：

```
RBF t4 txgs=[6, 7] counts=PrototypeForwardCounts { released_placements: 12, released_slots: 14, revived_placements: 12, revived_slots: 14, birth_pruned_would_miss: 0, birth_pruned_nodes_read: 11, birth_pruned_data_units: 1, target_allocation_record_tree_nodes: 5, newest_allocation_record_tree_nodes: 5, newest_born_after_target_by_family: {"accounting_tree": 1, "allocation_tree": 5, "data": 1, "extent_root": 1, "inode_leaf": 1, "inode_root": 1, "mapping_tree": 1, "tree_table": 1} }
RBF crash name=forward-rollback closed_form_full=524301 segment_lengths=[2, 18, 2, 1, 18, 2, 1, 2] full=true
RBF crash name=forward-rollback writes=46 states=524301 violations=0 failed=0 first_violation=none probe_states=2622 probe_bad(kill1,kill2,kill3)=[0, 0, 0] probe_first_bad=[None, None, None]
RBF crash name=forward-rollback checker_violated=["I-3.9=262151:盘 1 槽 50245 的分配记录释放代 6 不在 (3, 4] 里：还引用它的最新一条有效根是 txg 3，最早不再引用它的是 txg 4"]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 488.10s
```

`rerun-full-unmount.out`（整份）：

```
RBF t5a chain_txgs=[6, 7] units_written=[8, 8] writes=[(21, 279040), (21, 279040)]
RBF crash name=unmount-b closed_form_full=327690 segment_lengths=[16, 2, 1, 18, 2, 1, 2] full=true
RBF crash name=unmount-b writes=42 states=327690 violations=0 failed=0 first_violation=none probe_states=0 probe_bad(kill1,kill2,kill3)=[0, 0, 0] probe_first_bad=[None, None, None]
RBF crash name=unmount-b checker_violated=["I-7.9=262151:实例 1 txg 6 那条根把回退下界 F 从 0 抬到 5，高于它之前的根算出的抬 F 上限 0：每块盘上最新的有效根 {0: 5, 1: 4}，非空有效根 [3, 4, 5]，空不空判不了的有效根 []，最旧有效根 txg 0"]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 191.97s
```

`rerun-fast.out` 里 H1、H2b、H3 引的那几行，还有它的 `test result` 行（整份 161 行在模型目录）：

```
RBF t1b arm=Full overwrites=Ok(8),Ok(9) recover=Ok((2, 9)) under_B_ok=Ok(true) under_C_ok=Ok(true) checker_reds=["I-3.9:盘 1 槽 50245 的分配记录释放代 6 不在 (3, 4] 里：还引用它的最新一条有效根是 txg 3，最早不再引用它的是 txg 4"]
RBF t1b arm=ReleaseOnlyNoRevive overwrites=Err("ReleaseTargetAlreadyReleased { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(0), slot: SlotNumber(50180) }"),Err("ReleaseTargetAlreadyReleased { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(0), slot: SlotNumber(50180) }") recover=Ok((2, 7)) under_B_ok=Ok(true) under_C_ok=Ok(true) checker_reds=["I-3.9:盘 1 槽 50180 的分配记录带已释放标志（释放代 4），而候选集里每一条有效根都还引用它", "I-3.11:盘 0：记账的已分配 Some(1032192) 减 defer 待释放 Some(868352)，不等于从最新根（txg 7）走读到的 262144（其中隔离豁免 0）"]
RBF t1b arm=BirthPrunedRelease overwrites=Ok(8),Ok(9) recover=Ok((2, 9)) under_B_ok=Ok(true) under_C_ok=Ok(true) checker_reds=["I-3.9:盘 1 槽 50245 的分配记录释放代 6 不在 (3, 4] 里：还引用它的最新一条有效根是 txg 3，最早不再引用它的是 txg 4"]
RBF t5b rollback=false b=false chain=[] D_data_slot=50186 E_data_slot=50188 newest=9 checker_reds=[] first_bad=none
RBF t5b-row rollback=false b=false killed=0 land_txg=9 verdict=Ok((2, 9))
RBF t5b-row rollback=false b=false killed=1 land_txg=8 verdict=Ok((2, 9))
RBF t5b-row rollback=false b=false killed=2 land_txg=7 verdict=Ok((2, 9))
RBF t5b-row rollback=false b=false killed=3 land_txg=6 verdict=Ok((2, 9))
RBF t5b-row rollback=false b=false killed=4 land_txg=5 verdict=Ok((1, 5))
RBF t5b-row rollback=false b=false killed=5 land_txg=4 verdict=Ok((1, 5))
RBF t5b-row rollback=false b=false killed=6 land_txg=3 verdict=Ok((1, 5))
RBF t5b-row rollback=false b=false killed=7 land_txg=2 verdict=Ok((1, 5))
RBF t5b-row rollback=false b=false killed=8 land_txg=1 verdict=Ok((1, 5))
RBF t5b-row rollback=false b=false killed=9 land_txg=0 verdict=Ok((0, 0))
RBF t6 arm=Full target=(1, 3) chain=[6, 7] floor_roots=[(6, 5), (7, 5), (8, 5), (9, 5), (10, 5), (11, 5), (12, 5)] corrupted_on_dev1=[7, 10] effective_floor_after=0 recover=Ok((2, 12)) rollback=refused TargetUnitNotProtected { slot: SlotNumber(50180) }
RBF t6 arm=Full target=(1, 4) chain=[6, 7] floor_roots=[(6, 5), (7, 5), (8, 5), (9, 5), (10, 5), (11, 5), (12, 5)] corrupted_on_dev1=[7, 10] effective_floor_after=0 recover=Ok((2, 12)) rollback=accepted newest=(3, 14) read=Ok(true) checker_reds=["I-2.1:树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上", "I-3.1:盘 0：记账的已分配 Some(2195456)，遍历全部有效根得到 2211840（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 14、环里自证过的根槽 13 个、最老的自证过的根 txg 0、遍历的候选根槽 13 个、并进遍历的由记录施加出来的版本 2 个、被实例表判抛弃的根槽 0 个、回退下界 F 0", "I-3.10:盘 1 槽 50180 的分配记录（未释放、跨 2 槽）分配代 3，它罩住的单元头里的诞生代号是 12", "I-4.8:候选根 txg 0 出发的遍历有单元对不上或读不出", "I-5.1:盘 0：树表单元（槽 50178 跨 1）与 数据单元（槽 50178）重叠", "I-7.4:候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）", "I-7.9:实例 1 txg 6 那条根把回退下界 F 从 0 抬到 5，高于它之前的根算出的抬 F 上限 2：每块盘上最新的有效根 {0: 5, 1: 4}，非空有效根 [4, 5]，空不空判不了的有效根 [0, 1, 2, 3]，最旧有效根 txg 0"]
RBF t6 arm=FullWithoutProtectionCheck target=(1, 3) chain=[6, 7] floor_roots=[(6, 5), (7, 5), (8, 5), (9, 5), (10, 5), (11, 5), (12, 5)] corrupted_on_dev1=[7, 10] effective_floor_after=0 recover=Ok((2, 12)) rollback=accepted newest=(3, 14) read=Err(MappingStillUnreadable { slot: SlotNumber(50180) }) checker_reds=["I-2.1:数据单元 在盘 0 槽 50180 的那一份与位置条目里的校验和对不上", "I-3.1:盘 0：记账的已分配 Some(2195456)，遍历全部有效根得到 2211840（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 14、环里自证过的根槽 13 个、最老的自证过的根 txg 0、遍历的候选根槽 13 个、并进遍历的由记录施加出来的版本 2 个、被实例表判抛弃的根槽 0 个、回退下界 F 0", "I-3.10:盘 1 槽 50180 的分配记录（未释放、跨 2 槽）分配代 3，它罩住的单元头里的诞生代号是 12", "I-4.8:最新根（txg 14）出发的遍历有单元对不上或读不出", "I-5.1:盘 0：树表单元（槽 50178 跨 1）与 数据单元（槽 50178）重叠", "I-7.2:最新的根走不完：数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的", "I-7.4:最新根（txg 14）引用的单元已被复用或抹头（校验和对不上或头用不了）", "I-7.9:实例 1 txg 6 那条根把回退下界 F 从 0 抬到 5，高于它之前的根算出的抬 F 上限 2：每块盘上最新的有效根 {0: 5, 1: 4}，非空有效根 [4, 5]，空不空判不了的有效根 [0, 1, 2, 3]，最旧有效根 txg 0"]
RBF t6 arm=FullWithoutProtectionCheck target=(1, 4) chain=[6, 7] floor_roots=[(6, 5), (7, 5), (8, 5), (9, 5), (10, 5), (11, 5), (12, 5)] corrupted_on_dev1=[7, 10] effective_floor_after=0 recover=Ok((2, 12)) rollback=accepted newest=(3, 14) read=Ok(true) checker_reds=["I-2.1:树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上", "I-3.1:盘 0：记账的已分配 Some(2195456)，遍历全部有效根得到 2211840（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 14、环里自证过的根槽 13 个、最老的自证过的根 txg 0、遍历的候选根槽 13 个、并进遍历的由记录施加出来的版本 2 个、被实例表判抛弃的根槽 0 个、回退下界 F 0", "I-3.10:盘 1 槽 50180 的分配记录（未释放、跨 2 槽）分配代 3，它罩住的单元头里的诞生代号是 12", "I-4.8:候选根 txg 0 出发的遍历有单元对不上或读不出", "I-5.1:盘 0：树表单元（槽 50178 跨 1）与 数据单元（槽 50178）重叠", "I-7.4:候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）", "I-7.9:实例 1 txg 6 那条根把回退下界 F 从 0 抬到 5，高于它之前的根算出的抬 F 上限 2：每块盘上最新的有效根 {0: 5, 1: 4}，非空有效根 [4, 5]，空不空判不了的有效根 [0, 1, 2, 3]，最旧有效根 txg 0"]
RBF t6s b=false writes=0 target=(1, 3) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=0 target=(1, 4) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=0 target=(1, 5) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=1 target=(1, 3) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=1 target=(1, 4) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=1 target=(1, 5) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=2 target=(1, 3) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=2 target=(1, 4) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=2 target=(1, 5) corrupted_dev1=1 verdict=clean
RBF t6s b=false writes=3 target=(1, 3) corrupted_dev1=2 verdict=clean
RBF t6s b=false writes=3 target=(1, 4) corrupted_dev1=2 verdict=clean
RBF t6s b=false writes=3 target=(1, 5) corrupted_dev1=2 verdict=clean
RBF t6s b=false writes=4 target=(1, 3) corrupted_dev1=2 verdict=clean
RBF t6s b=false writes=4 target=(1, 4) corrupted_dev1=2 verdict=clean
RBF t6s b=false writes=4 target=(1, 5) corrupted_dev1=2 verdict=clean
RBF t6s b=true writes=0 target=(1, 3) corrupted_dev1=2 verdict=clean
RBF t6s b=true writes=0 target=(1, 4) corrupted_dev1=2 verdict=clean
RBF t6s b=true writes=0 target=(1, 5) corrupted_dev1=2 verdict=clean
RBF t6s b=true writes=1 target=(1, 3) corrupted_dev1=2 verdict=HIT read_ok=true reds=["I-2.1", "I-3.1", "I-4.8", "I-5.1", "I-7.4"]
RBF t6s b=true writes=1 target=(1, 4) corrupted_dev1=2 verdict=HIT read_ok=true reds=["I-2.1", "I-3.1", "I-4.8", "I-5.1", "I-7.4"]
RBF t6s b=true writes=1 target=(1, 5) corrupted_dev1=2 verdict=HIT read_ok=true reds=["I-2.1", "I-3.1", "I-4.8", "I-5.1", "I-7.4"]
RBF t6s b=true writes=2 target=(1, 3) corrupted_dev1=2 verdict=HIT read_ok=false reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.2", "I-7.4"]
RBF t6s b=true writes=2 target=(1, 4) corrupted_dev1=2 verdict=HIT read_ok=true reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.4"]
RBF t6s b=true writes=2 target=(1, 5) corrupted_dev1=2 verdict=HIT read_ok=true reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.4"]
RBF t6s b=true writes=3 target=(1, 3) corrupted_dev1=3 verdict=HIT read_ok=false reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.2", "I-7.4"]
RBF t6s b=true writes=3 target=(1, 4) corrupted_dev1=3 verdict=HIT read_ok=false reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.2", "I-7.4"]
RBF t6s b=true writes=3 target=(1, 5) corrupted_dev1=3 verdict=HIT read_ok=true reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.4"]
RBF t6s b=true writes=4 target=(1, 3) corrupted_dev1=3 verdict=HIT read_ok=false reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.2", "I-7.4"]
RBF t6s b=true writes=4 target=(1, 4) corrupted_dev1=3 verdict=HIT read_ok=false reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.2", "I-7.4"]
RBF t6s b=true writes=4 target=(1, 5) corrupted_dev1=3 verdict=HIT read_ok=true reds=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-5.1", "I-7.4"]
RBF t6s-summary b=false hits=0/15
RBF t6s-summary b=true hits=12/15
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.49s
```

### SHA256SUMS（模型目录，原样）

```
874a5302003927fedee92f5cea28cd785bf672fa288dee7dccae2c23c102c5b4  prototype.patch
3ff7ac680202adcf379bd0fa4c411ab53596f7a58cd83fe8db4f90b86f50b38f  rbf_attack.rs
a4108f65e8602854815a8f89cb8dea957a89cbda8d8c86fb8c39873ecde51249  rerun.sh
629d82f6c4a08557f8c74615b4cdf550030be810b475203e92d6abc83ca0565b  rerun-fast.out
b701069787e3120e6f5644ad44476337154892342796dea7d7d981039aa2f7c1  rerun-full-unmount.out
deba842d53e4d2eb1557cbf9d475336b2068508df81360a1764d463f2b026c7c  rerun-full-rollback.out
```
