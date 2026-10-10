# 第 10z 批第十、十一块代码轮第一轮 攻方 s2 报告（2026-10-08）

同一题面独立第二次抽样（s2）。只攻分到的 K1–K6，不判正推 / 辩方那几格。腿自己量的数在
`~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-opus-s2/` 里量，副本上的数不算入库装置上的数。
模型目录 `research/prompts/m3-batch10z-jk-code-r1-opus-s2-model/`（sha256 见该目录 `SHA256SUMS`，复跑 `rerun.sh`）。

**崩溃状态枚举没跑**：两条残余用例全量层 0 超过 2.6×10⁸ 状态（题面第二节「已知没做」），本腿与第十块实现员同，没跑；
本腿的证据是 harness 档随机历史 + 每一步根落之前那一态与之后在池级 checker（`checker_violations_on`）上判 I-7.4 / I-4.8，
不是层 0 全量，也没调 `enumerate_layer0*`（因此不占原型枚举的 10⁶ / 10⁷ 状态预算）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 丙并中间罩不罩得住 | 没打中（本轮取样内） | 补法臂在 b1/b3/b5/b2、384+1536、cfgq,qroot,qlast,racct 组合上 0 红；多根取丙、超一圈的边界没造出反例 |
| K1-2 丙并中间放的时机 | 没打中 | 补法臂 `iso_end=0`（全放）、`refused_steps=0`（无白拒）；放早/放不掉未见 |
| K2-1 / K2-2 回退复活罩不罩得住 | 没打中（含嵌套回退） | 补法臂 racct 全绿、`revived_in_c_r` 满额；R_old 自己也是回退时仍 0 红 |
| K2-3 认不出回退的可达面 | **量到**（题面要求量，不单算打中） | 见下表：rall / racct,oldroot / racct,oldacct / racct,rlin 四组走得到，各红满额；给出一条不改格式的退路臂 s2fb |
| K2-4 多护那一截何时放 | 没打中 | 补法臂多护的 24 槽（384）随 C 同一时机放，`iso_end=0`，无白拒 |
| K3-1 列卡后记的那笔会不会被销 | 没打中（读码+单调号） | 借据号 `AtomicU64` 单调 `fetch_add`、永不复用；`listed` 是快照、只销列出来且仍在的 |
| K3-2 三台不同一台时多销/销不掉 | 没打中（读码） | `assert` 卡同一张卡 + 按等的那一台过滤，跨台不销 |
| K4-1 两把锁死锁 / 双起 | 没打中（压测） | 10 线程 × 5000 轮无死锁、无 panic；锁序只 starting→started |
| K4-2 借之前碰重起不等 | 没打中（压测+读码） | `started_now` 不拿 `starting`，压测跨线程阻塞 0 毫秒 |
| K4-3 丢旧设备压别的线程 | 没打中（本轮取样内） | 丢在放开 `started` 之后、`starting` 之内：只挡同卡再起，`started_now` 不挡（量到 0 毫秒） |
| K5 判别力 | 部分核（见该节） | 第十块库单测与 harness 用例的红由本腿的 today/chain-off 阳性对照独立印证 |
| K6 只防 OOM 不压并发 | 没打中（本轮取样内） | 压测里账臂恒成功、无拒；未见账给大于真能用或整趟弃卡 |

## K2-3　认不出回退的可达面（题面重点，在入库代码上造出来量）

补法认回退靠 R 读得出的记录点名的谱系片里 `rollback_txg = R.txg` 的那条谱系记录
（冻结副本 `crates/singlefs-core/src/mount.rs:2169`「`.find(|lineage| lineage.rollback_txg == checkpoint_txg_of_the_publish)`」），
记录结构见 `.claude/kb/decisions/18-块里携带什么信息.md@已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录`：
「`记录 (T_rb 8, T_old 8) = 16 字节（T_rb 是回退那次发布的 txg，T_old 是 R_old 的 txg，读作「T_rb 的父是 T_old」）`」。
认回退那条取 `None`、或根环里没有 txg = T_old 的有效根、或读它的账失败，补法交空（`crates/singlefs-core/src/mount.rs:2118`
「`fn placements_a_rollback_publish_revived_released_in_the_current_account<Device: BlockDevice>(`」），照丙原样，复活的单元照旧会发出去。

本腿在入库代码（冻结副本）上把这一形造出来量。量法：探针 `opus_s2_probe.rs`（起点取自第十块实现员 `probe_10zj.rs`，
加前缀、按 txg 取回退目标、R_old 一侧的新故障）；臂只看环境变量，`today` = 丙原样、`implemented` = 补丁。
故障组：`racct`（只 R 的账读回全零，认得出回退）、`rall`（R 这次发布写的全部单元读回全零，连谱系片一起）、
`racct,rlin`（R 的账 + R 写的谱系片）、`racct,oldroot`（R 的账 + R_old 的根槽）、`racct,oldacct`（R 的账 + R_old 的账两棵树）。
每组 384 与 1536 两档、b1/b3/b5/b2 四种回退目标，各 4 段随机 `onsFb` 20–34 步，每一步根落之前那一态与之后判 checker。
`agg.py` 汇总（`runs/k23a`、`runs/k23b`）：

| 故障组 | 认出回退？ | 补法臂 hit（红段/总段，384+1536 合计） | `revived_in_c_r` | 挂载那一刻 iso 峰值 |
|---|---|---|---|---|
| racct | 认得出 | 0 / 40 | 满额（8/8 每组） | 42 |
| racct,rlin（谱系片也坏） | 认不出 | 40 / 40 | 0 | 18 |
| rall（R 全部单元坏） | 认不出 | 40 / 40 | 0 | 18 |
| racct,oldroot（R_old 根槽坏） | 认不出→R_old 不在环 | 40 / 40 | 0 | 18 |
| racct,oldacct（R_old 账坏） | 认出回退但读 R_old 账失败 | 40 / 40 | 0 | 18 |

读法：**认回退要同时读得出 R 的谱系片与 R_old 那一版的账**，这两处任一读不出，补法就交空、退回丙原样会红。
`racct,rlin` 与 `rall` 打在「谱系片读不出」上；`racct,oldroot`、`racct,oldacct` 打在「R_old 那一版读不出」上。
四组各在两个几何上红满额，`refused_steps=0`（认不出时不拒、照发），正是题面第二节「交用户的残余」那一形的可达面。
这一形的病根与 `racct` 下补法能认出回退是同一套读盘：R 的账坏（丙的触发条件）与谱系片/R_old 坏往往同源（同一次崩溃抹掉 R 这次发布的更多单元，或连带抹掉 R_old 的槽）。
阳性对照：`today` 臂在 `racct` 组 8/8、4/4 全红（`runs/posctl`），`implemented` 同组 0 红——探针的 oracle 是活的。

### 一条不改格式能让它认得出的退路（s2fb 臂，被攻过零轮，只在我的副本上量过）

题面 K2-3 问「有没有一条读得出的信息（不改格式）能让它认得出」。补法今天只在**认出回退**（谱系片 + R_old 账都读得出）时护。
但 R 读得出的记录头带 `transaction`（冻结副本 `crates/singlefs-core/src/journal.rs` 的 `JournalRecord.transaction`），
回退那次发布与空发布一样写事务号 0（`mount.rs` `roll_back_by_a_forward_publish` 注释「回退那次发布的事务号写 0」）。
于是不改任何盘上格式，就能从「R 记录齐、账读不出、R 的记录事务号为 0」判出「R 是一次零单元发布（回退或空发布）」，
在认不出精确回退目标时退到一个更宽的护法。我在副本里加了 s2fb 臂（`model-arms-on-mount.diff`，被攻过零轮）：
认不出回退或找不到 R_old 账、而 R 的记录里有事务号 0 的，退到「现行账里释放代晚于 R 的 F 下界的全部已释放落点」。

量（`runs/k23fb`，补法认不出的那四组，384+1536 各 4 段 × 2 回退目标 = 共 50 段每组）：**四组全部 hit 归零、`refused_steps=0`**，
代价是挂载那一刻隔离从 18 涨到 90–164 槽（视回退目标多深），R_old 不在环（b 很深）时甚至 610+ 槽。
这与第四块判决第二节 K2-2 的乙 / 正推对照同一类「不滤已释放」的量级（多护约 100–600 槽），不是免费的。
**结论**：这条退路可达、不改格式、把 K2-3 的红全堵上；代价是在零单元发布上一律多护一大截（空发布也会误护，因为事务号 0 两者共用）。
交用户在三条里选：留补法原样（认不出照发、残余）、退到 s2fb（不改格式、堵红、多护一截）、或拒可写（要新错误成员）。
**这个数只在我的副本上量过、被攻过零轮**（表里 iso 峰值标「量过」，s2fb 空发布误护标「推的」：没单造空发布历史验）。

## K1　丙并中间（没打中，本轮取样内）

丙并中间把 (所选那一版, R) 之间每个 txg 在根环里对应的槽记着的落点并进 R 那一槽
（冻结副本 `crates/singlefs-core/src/mount.rs:2064`「`fn with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(`」），
中间 txg 至多取一整圈（`crates/singlefs-core/src/mount.rs:2078`「`.take(ring_length_in_txgs)`」，环长 3S = 24）。
补法臂在攻方 b1/b3/b5/b2 四种、racct 组、两档几何上 0 红（K2-3 表 racct 行与 `runs/k23*` 一并量），
K5 阳性对照里撤掉丙并中间（chain-off/today）在那几形上照红——补法确实堵住了那一格。

本轮没造出的两形（读码推、没做成反例）：
1. **多条根同时走丙那一支**（`txgs_of_the_roots_taking_the_current_account` 不止一个）：要两条较新根各自「记录齐、账读不出」，
   我的故障模型只对 R 一条打 racct，没造出第二条同时走丙的历史；读码看 `mount.rs:2064` 的循环对每条根各并一次、各读原始 `named_by_ring_slot`，
   未见互相污染，但没有反例坐实。
2. **(所选那一版, R) 相差超过一整圈**：`.take(ring_length_in_txgs)` 只取最近一圈的 txg，注释称「超过一圈时环里每一槽都已取到」；
   这依赖「那一圈里每个槽都写过」，我没造出一段相差 > 24 txg 又让某个更早 txg 的单元漏护的历史。
两形都归「本轮取样内没打中」，不是「打不中」。

## K2-2 / K2-4　回退复活罩不罩得住、多护那一截何时放（没打中）

补法只并「R_old 账里仍分配、P 的账里已释放」那一截。本腿在 racct 组的所有 b1/b3/b5/b2 段上量到补法臂 `revived_in_c_r` 满额、0 红
（K2-3 表 racct 行）。又另造了 R_old 自己也是一次回退的嵌套历史（前缀 `oooob1o`/`ooob1oo`/`oob2oo`，R 再回退一次，`runs/nest.log`）：
补法臂 9 段 + 9 段全 0 红、`revived_in_c_r` 满额、`refused_steps=0`——R_old 是回退时复活那一截仍罩住。

K2-4 多护那一截（R_old 自己的固定点单元）的放的时机：补法臂整段 `iso_end=0`（会话跑完全部解开）、`iso_max` 随抬 F 升到 46/70 又回落、
`refused_steps=0`（没有一步因多护而白拒）。本轮没见「一直不放、把空间占满而准入白拒」。
第九项计数：补法把 R_old（比所选那一版旧）账里的落点并进来，而 `.claude/kb/decisions/28-挂载期承诺量.md@已定项 1：准入不等式的权威形式`
字面只写到「`水位之上没施加的 journal 记录点名的落点、那次发布的账里的落点与根环里比所选那一版新、按实例表有效、读得出的根的账里的落点`」，
没说到「比所选那一版旧的 R_old 账」——这是 kb 字面要随补法定形写回的一处（题面已交代写回归主 agent，本腿只记这笔）。

## K3　列卡带借据号（没打中）

K3-1：列卡之后才记成没等成的那一笔会不会被销。冻结副本 `crates/singlefs-harness/src/gpu_memory_ledger.rs:904`
「`fn return_the_listed_leases_kept_after_a_failed_wait_on(`」只对 `listed`（列卡那一刻的快照）里、等的是这一台、且此刻还 `remove(...).is_some()` 的借据销账；
列卡之后新记的借据不在 `listed` 里，不销。借据号复用这条攻法走不通：`crates/singlefs-harness/src/gpu_memory_ledger.rs:997`「`next_lease_identifier: AtomicU64::new(0),`」+
`:1056`「`let lease_identifier = self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);`」，借据号单调、永不复用，`listed` 里的旧号被销后新借据拿的是更大的号，`remove(旧号)` 取 `None`、跳过。
K3-2：三台不是同一台。`crates/singlefs-harness/src/gpu_memory_ledger.rs:1612`「`listed.card_position, self.card_position,`」的 `assert` 卡「列的卡 = poll 的设备所在卡」，
`return_the_listed...` 里又按「等的那一台 == 被 poll 的那一台」过滤，跨台的借据不销（不多销）；它要等的那一台真做完活、或那一台丢了/放掉时另有路径销（不会永远销不掉）。
读码没打中；没为它写并发压测（账的失败等待路径要整套 `GpuMemoryLedger` 场景，本轮把并发预算放在 K4 两把锁上）。

## K4　两把锁（没打中）

K4-1 死锁 / 双起、K4-2 借之前碰重起不等、K4-3 丢旧设备压别的线程，一起用并发压测 `opus_s2_device_lock_stress.rs` 攻
（`DeviceStartedOnDemand<BlockingOnDrop>`，丢设备时按 `S2_DROP_MS` 阻塞，模拟 wgpu 等设备空）。锁序读码：
`started_or_start` 先无锁克隆（`gpu_memory_ledger.rs` 的 `clone_of_the_device_kept`），再拿 `starting`（`:1710`
「`let _starting = self.starting.lock().unwrap_or_else(PoisonError::into_inner);`」），只在换设备那一句临时拿 `started`，
`started_now`（`:1740`）与 `is_started` 只拿 `started`——`started` 绝不在持 `starting` 之外被长期持有，锁序只有 starting→started，不构成环。

压测原样（`runs` 外的 `lockfix.log`、`lock-*.log`）：
```
S2LOCK threads=10 rounds=5000 drop_ms=0 starts=8335 drops_seen=75001 started_now_blocked_max_ms=0
S2LOCK threads=10 rounds=800 drop_ms=3 starts=1334 drops_seen=11999 started_now_blocked_max_ms=0
```
10 线程 × 数千轮、约四分之一轮把留着的判成丢了逼重起，**无死锁、无 panic、跑完留着一台**（K4-1）。
`started_now_blocked_max_ms=0` 即便丢设备阻塞 3 毫秒：`started_now` 跨线程一次都没被挡（K4-2、K4-3 的 `started_now` 那一半）。
K4-3 丢旧设备在放开 `started`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1724`「`drop(replaced);`」在 `started` 锁的临时 guard 落了之后）、
但仍在 `starting` 之内：只挡「同一张卡上再起设备」的那条线程，不挡取留着设备的借用、不挡别的卡。这是设计内的同卡重起串行，本轮没打中。
**修正记录**：第一版压测把 `started_now` 交回那台设备的 Drop 算进了计时，误显示 2 毫秒；移出计时后为 0（`opus_s2_device_lock_stress.rs` 里 `drop(taken)` 在 elapsed 之后）。

## K5　判别力（部分核）与 K6（没打中）

K5：本腿没重跑第十块 10 行与第十一块 4 行变异（checker 档 / 整表复跑不归子 agent），但用 today/chain-off 两个阳性对照独立印证了
回退复活与丙并中间的用例确有判别力：撤掉补法（today）在 racct 组 8/8、4/4 全红（`runs/posctl`），补法原样 0 红——
与第十块实现员「撤掉补法红在第 20 次覆盖写根落之前」同向。第十一块 K5 的 checker 4 行（X1/X2/X4/X4b）本腿未碰（checker 档不跑）。
`a_second_start_on_the_same_card_waits...` 那条用例要第二条线程 500 毫秒内排上才红——本腿的压测用的是 mpsc 同步点、不靠时限，
同向印证「第二条等第一条起完、不另起」（`starts_by_the_second = 0` 那一断言的并发版：压测里双起从未发生、无 panic）。

K6（只防 OOM 不压并发）：压测里账臂（`started_or_start`）恒成功、`refused_steps` 概念在账侧未触发整趟弃卡；
未造出「账给的量大于此刻真能用」或「卡放得下却判红弃卡 / 并发被压住」。本轮取样内没打中；账的量化要整套 `GpuMemoryLedger` + `nvidia-smi` 读数场景，未搭。

## 没打中的形状（试过哪些、取样多大）

- K1 多根同走丙、相差超一圈：故障模型只对 R 一条打 racct，没造出第二条同走丙或 >24 txg 漏护的历史。取样：b1/b3/b5/b2 × {384,1536} × 五故障组 × 每组 4 段 ≈ 各 160 段，中间发布 0–1 次。
- K2-2 回退到更早版、R_old 本身回退：嵌套只造到两层（`runs/nest.log`，9+9 段），更深未造。
- K3 账的失败等待路径并发：没搭 `GpuMemoryLedger` 多卡场景压测，只读码 + 借据号单调性。
- K4-3 丢旧设备压同卡再起：量到只挡同卡再起（设计内），没把「压住提交」做成可观测的红。
- K6：账的 OOM 边界没搭量化场景。

## 这条腿自己的限度

- 全是 harness 档随机历史 + 池级 checker 逐步判，不是层 0 全量；崩溃状态没枚举（>2.6×10⁸，未跑），因此 K1/K2 的「没打中」只在本轮取样内成立，不是「造不出」。
- s2fb 退路臂与它的 iso 代价只在我的草稿副本上量过、被攻过零轮；副本上的数不进 kb，要入库须主 agent 在入库装置上重做（题面与规则）。
- 探针 oracle 只看 I-7.4 / I-4.8；别的不变量没纳入判红。
- s2fb 对空发布误护那一句是「推的」（事务号 0 回退与空发布共用），没单造空发布历史验。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；不跑任何门禁阶段、不跑 checker 档测试、不碰 GPU 设备。
- 不重跑第十、十一块的变异整表（归整表复跑 / 主 agent）。
- 草稿与副本（含 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-opus-s2/{repo,target,runs,src}`）交回时删，模型源已拷进模型目录；删除见交回正文。
