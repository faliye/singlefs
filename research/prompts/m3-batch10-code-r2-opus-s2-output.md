# m3-batch10-code-r2 攻方第二次抽样（opus-s2）报告（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 Q3 Q4 rr2 G3 G4 -->

攻击面：正文第三节 K1、K2、K4，重点 K2。代码行号引冻结副本 `/tmp/claude-1000/m3-batch10-code-r2/tree/`（写成绝对路径）。副本上的数是攻方副本上的数，不进 kb。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-batch10-code-r2-opus-s2-model/`。复跑（工作目录随意，草稿目录要不存在或为空，跑 5 分钟上下）：

```
bash scripts/run-with-memory-cap.sh 4000M bash research/prompts/m3-batch10-code-r2-opus-s2-model/rerun.sh <草稿目录> /tmp/claude-1000/m3-batch10-code-r2/tree
```

末行报「比了 13 份，N 份不同」，逐份比每行 `name=`。它把冻结副本的 `Cargo.toml`、`Cargo.lock`、`crates/` 拷进草稿目录，打 `mount-rs-attack-arms.patch`（只在 `rebuilt_allocator` 里加一个读环境变量 `ATTACK_RR2_ARM` 的臂开关：空或 `rr2` 照冻结副本，`before` 回到 rr2 之前「整个环一条都没有才重读」，`merge` 是 rr2 加「重读那一遍按槽并两遍读数」），放进原型测试，release 编译，跑 12 次 grid 与一次新池那一格。这份复跑脚本本身这一轮没有整份跑过（前台时限装不下，见「要主 agent 跑的命令」）；`out/` 里的产物由同一份原型、同一份补丁在草稿副本上跑出，补丁打到冻结副本上与草稿副本的 `mount.rs` 逐字节相同（`cmp` 过）。

| 文件 | sha256 |
|---|---|
| `k2_attack_reread_replaces_the_first_pass.rs` | 89a630b6b59193603360325fff12acea669d239ee654226e37baaa221c030a09 |
| `mount-rs-attack-arms.patch` | a38d9a7dde468f5327d67ca5d00cc2b7cb684fa568cfe8ef57a70ef56645ddb8 |
| `rerun.sh` | fda78a294b968cd611f279982574b057acc0cf7611066a54e89b8db7bf651f41 |
| `summarize.sh` | bef87991f6fa8901414f6cc006abbea4df5ad7afdbff8989ebbfb33747c84eff |
| `out/grid-384-1-before.log` | cb5635687d6f024f0d4d847b42aa988d35bed932912868aaeb9bcb87217167b6 |
| `out/grid-384-1-merge.log` | f159620b112d6b22b2ff635eb0b4d58f60aa0c8b6d7ae77615a6c67239218127 |
| `out/grid-384-1-rr2.log` | 0e58538cd43a6b773773a2553ecf3591ee57896f12dff753725f077bdd2c2e6d |
| `out/grid-384-3-before.log` | 7ad24fae3d5c1078382f57cf997cd24f8ebf629f14dee570c5c63ac502da2f81 |
| `out/grid-384-3-merge.log` | 6b5c0dcf1c1e20764573d0eabbbefb9417cb17a943fadce50642703b3dbbe178 |
| `out/grid-384-3-rr2.log` | 64fdd601e4c6d7a70d43b5f5d5e4c8593b189b1dcff8e5b6325cf4189577022b |
| `out/grid-4g-1-before.log` | a27d983a9ff05dacfdb6b1944b5e83833432a98d1e1a4c9954f5844d22c36e65 |
| `out/grid-4g-1-merge.log` | d3bba0f1f077cc0bb9b88dd5c439f7cd5323b63a5c1ef241d7b6366298dce7ec |
| `out/grid-4g-1-rr2.log` | e1a71e28391fa8050fb0448675f580fa19f147e68b483a7a5b5cb2157f071001 |
| `out/grid-4g-3-before.log` | 63250feffde6311ba47c4092edacc9d634212acbdc20524b21434e224035a128 |
| `out/grid-4g-3-merge.log` | a3332b11a7b04ade90c1352113f4b80acf8102027b3cfc93374a751533efe53b |
| `out/grid-4g-3-rr2.log` | aeccf1539bd4bf70d6a7c54cfde97e6030f88354336ba07f099c243ff2331c7c |
| `out/pivot.txt` | d4e3b879c6aa0e185507ca789266f0464cd44b09cbb9d0389fa83728ca5259ae |
| `out/summary.txt` | b389dc840acddb5cf74b1389d320ec19399b3b1c03bd60ad26a2ae471263363e |
| `out/young.log` | 36eb6c96bcffc76fcd7cd6672444070c3740ecd3dae07f0e9ed0fdc44b7e2c1b |

`out/summary.txt` 与 `out/pivot.txt` 是从 12 份 grid 日志用 awk 抽的（命令在本报告 K2-2 节），不是原型自己写的。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2-1 第一遍「每块盘都读出」与当场放开取 m 是不是同一处定义 | 没打中 | 两处都调 `newest_valid_root_txg_on_every_device`，同一张表、同一个 `device_of_txg` |
| K2-2 重读比第一遍坏、整份换成重读的读数：影子账 | **打中** | 第一遍读出被抛弃根 C、重读那一遍只把 C 的槽读坏，C 掉出影子账、隔离 0，第一次覆盖写就复用 C 的单元，池级 checker 红 I-7.4；rr2 之前那一臂与「按槽并两遍」那一臂在同一段历史上都不丢 |
| K2-3 重读比第一遍坏：回收门槛、抬 F 上限越过有效根 | 没打中 | 重读把盘 0 除最新根与 C 之外的槽读坏（报错、全零、乱字节三种），三臂 checker 都不红、F 相同 |
| K2-4 多拒（实现员读法 ②） | 与写的相同 | rr2 臂 4/4 报 Q3 拒，rr2 之前那一臂与并两遍那一臂 4/4 照常挂上 |
| K2-5 新池每次可写挂载都重读（读法 ①） | 与写的相同，射程量过 | 只有 mkfs 之后第一次可写挂载、和它在第 1–5 个写（共 12 个）上掉电之后的下一次挂载重读；每个根环槽多读 1 次、钩子多调 1 次 |
| K2-6 有没有用例因此钉错 | 没打中（一处名字不符） | `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread` 钉的是 `OnTheOneReread`，名字里的 does_not_reread 只对读阶段成立 |
| K1-1 实例表读不出时「不按表滤」（读法 ③） | 没打中（推的） | 那一支在每条走得到 Q3 的路上都碰不到，翻不了任何一格判定 |
| K1-2 复算判据与实现判拒是不是同一个定义 | 没打中（推的） | 被抛弃判法逐字同式；读根环同走 `read_root_ring_slot` 那一类读法；没读故障时 Q3 拒走不到 |
| K4-1 第三截 Q3 计数与报红与第二截同不同口径 | 口径不同（线索，推的，没跑 checker 档） | 第二截凡「不合法」都报红；第三截只报「判据复算为真」那一种，判据复算为假的 Q3 拒只计数——与 G4、N-配置 在第三截的口径相同 |
| K4-2 读序号后挪一遍根环 | 没打中（推的，没跑 checker 档） | 重读只在第一遍缺盘时发生、只读根环槽；E158 按落点数读序号的计划点名的是实例表片，不挪 |

## K2　rr2 与它的两个后果

### K2-1　第一遍的判法与当场放开取 m 同一处定义（没打中）

第一遍判「每块盘都读出了不被抛弃的自证有效根」走 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1940`：「newest_valid_root_txg_on_every_device(」，当场放开到哪儿走 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:2068`：「let floor_released_immediately = match newest_valid_root_txg_on_every_device(」。两处交的是同一张 `newest_table`（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1872`：「let (newest_table, instance_table_of_the_newest_root) =」）、同一个落点公式闭包、同一个 `all_devices`；前一处交第一遍的根，后一处交选中那一遍的根。重读那一遍仍缺一块盘时窗口锚照取（只要整个环有一条），之后 m 取不到走 `None => oldest_valid_root_in_the_ring` 那一支。原型里「第一遍盘 1 全坏、重读盘 1 仍全坏」那一格（`pass1:dev1-all-e reread:dev1-all-e`）三臂 12/12 照常挂上、checker 不红（见 K2-2 的总表）。

### K2-2　重读比第一遍坏时，第一遍读出的被抛弃根掉出影子账（打中）

**历史**（每一步在被判对象里的依据）：

1. 两块内存稀疏盘上 mkfs、取号 1、暖机、第一个文件，覆盖写 1 次或 3 次（实例 1）；崩了再挂（实例 2），覆盖写一次得 C = (2, 8)。盘宽两档：4 GiB（`HistoryDeviceWidth::FourGibibytes`）与单元区 384 槽。每区 8 槽、环长 24（原型里盘 1 那一区的 8 个槽恰好各坏一次，`fired=8`）。
2. 照 `tests/common` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 那一形抛弃 C（藏 C 的根槽与数据单元、系统配置轮换停在 C 之前，可写挂载落到 C 之前那一条，实例 3 写行与暖机，再写回）。这是 D23（journal 的角色与格式） 已定项 14 射程里「影子账与按实例表判抛弃留着，理由是崩溃恢复」那一形，影子账要隔离 C 独占的单元（不注故障的再挂隔离每盘 14 槽，384 槽盘每盘 12 槽）。
3. 进程退出，再可写挂载（实例 4，准入判着、影子账开，经 `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`）。读故障按每个根环槽自己被读的次数点名：重建分配器读根环那一遍是每个槽的第 6 次读（原型扫 k = 1..10 现定，与 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:65`：「const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD: u64 = 6;」同数），重读是第 7 次。
   - 第 6 次读：盘 1 那一区（区域 1）每个槽报读错，于是盘 1 一条有效根都没读出，rr2 触发重读（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1959`：「.filter(|_| a_valid_root_was_read_on_every_device(&first_readings));」）。
   - 第 7 次读：只有 C 的根槽读坏（报错、读回全 0、读回乱字节三种）。盘 1 这一遍读得出。
   - 被判对象许可整份换读数：`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1951`：「// 根环表、回收门槛与影子账都用重读的读数（重读那一遍仍有一块盘没读出的，照 m 取不到那一支往下走）；」，代码 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1969`：「let readings_of_the_reread = read_every_root_ring_slot_once();」之后 `ring_slot_readings` 只取重读那一份（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1984`：「let roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = ring_slot_readings」），影子账拿它算（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:2088`：「ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(」，被抛弃根只从这份根里取：`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1508`：「for root in roots.iter().filter(|root| is_abandoned(root)) {」）。第一遍读出的 C 被丢掉。
4. 挂上之后接着覆盖写，至多 40 次，每次核 C 写出的单元在盘上还是不是 C 的字节；停下时在盘面上跑池级 checker。

**结果**（`out/pivot.txt` 原样，每格 4 段 = 2 盘宽 × 2 前置覆盖写；`red` 是 checker 有违例的段数，`iso0` 是两块盘隔离都为 0 的段数，`refused` 是可写挂载被拒的段数）：

```
pass1:dev1-all-e reread:c-Error | rr2: n=4 red=4 iso0=4 refused=0 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
pass1:dev1-all-e reread:c-Garbage | rr2: n=4 red=4 iso0=4 refused=0 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
pass1:dev1-all-e reread:c-Zeros | rr2: n=4 red=4 iso0=4 refused=0 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
pass1:dev1-all-e+c-e reread:- | rr2: n=4 red=0 iso0=0 refused=0 | before: n=4 red=4 iso0=4 refused=0 | merge: n=4 red=0 iso0=0 refused=0
pass1:c-e (C554 control) | rr2: n=4 red=4 iso0=4 refused=0 | before: n=4 red=4 iso0=4 refused=0 | merge: n=4 red=4 iso0=4 refused=0
no-fault | rr2: n=4 red=0 iso0=0 refused=0 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
```

打中那几格的单段原样（`out/summary.txt`）：

```
4g-1-rr2|pass1:dev1-all-e reread:c-Error|result=Ok|ring=OnTheOneReread|[((0),0),((1),0)]|24|reused_after_publishes=1 txg=14|floor_after=0|checker_red={"I-7.4"}
4g-1-before|pass1:dev1-all-e reread:c-Error|result=Ok|ring=OnTheFirstRead|[((0),14),((1),14)]|0|reused_after_publishes=20 txg=33|floor_after=0|checker_red={}
4g-1-merge|pass1:dev1-all-e reread:c-Error|result=Ok|ring=OnTheOneReread|[((0),14),((1),14)]|0|reused_after_publishes=20 txg=33|floor_after=0|checker_red={}
384-3-rr2|pass1:dev1-all-e reread:c-Zeros|result=Ok|ring=OnTheOneReread|[((0),0),((1),0)]|20|reused_after_publishes=1 txg=14|floor_after=0|checker_red={"I-7.4"}
```

列依次是：段（盘宽-前置覆盖写-臂）、格、挂载结局、重建那一遍在哪一遍读出、两块盘隔离槽数、挂上之后 C 的落点在分配器里空闲的个数、C 的单元第一次被改写在第几次发布（`txg` 是那一刻现行那一版）、F、checker 红的不变量。rr2 臂挂上那一刻 C 的 26 个落点里 24 个空闲（384 槽盘 22 个里 20 个），第一次覆盖写（txg 14）就改写 C 的数据单元，I-7.4（近 K 代块未被复用）红；rr2 之前那一臂与并两遍那一臂隔离 14 / 14、C 的落点 0 个空闲，C 的单元直到 C 的根槽被 txg 32 以后的发布盖掉才被复用，checker 不红。

抽表命令是模型目录里的 `summarize.sh`（`bash summarize.sh <放 grid-*.log 的目录>`，sha256 bef87991f6fa8901414f6cc006abbea4df5ad7afdbff8989ebbfb33747c84eff）；拿 `out/` 里 12 份日志重抽，两份与 `out/` 里的逐字节相同（`cmp` 过）。被拒的段没有 checker 字段，pivot 的 red 列把它们也数进去，读被拒那一格看 refused 列。

**打中之后的四句**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）：

1. 分不分辨臂：分辨。同一段历史、同一份读故障，rr2 臂 12/12 段（三种读坏 × 4 段）丢隔离、checker 红；rr2 之前那一臂与并两遍那一臂 0/12。反方向也有一格分辨：第一遍盘 1 全坏且 C 的槽也坏、重读读得出（`pass1:dev1-all-e+c-e reread:-`），rr2 与并两遍 0/4 红，rr2 之前那一臂 4/4 红——rr2 在这一格比之前好，在 K2-2 那几格比之前坏，并两遍两边都不红。
2. 被判的系统当时看不看得到判别它的东西：看得到。丢掉 C 的那一刻 `first_readings` 还在 `rebuilt_allocator` 的栈上，里面那一槽是自证过的 C；换成重读那一份是代码的选择，不是读不到。
3. 满足的是判据字面的哪一个分句：正文第三节 K2 第二句「重读比第一遍坏时整份换成重读的读数：有没有一段合法历史因此多拒，或让回收门槛越过一条盘上真住着、可回退到的有效根（不安全方向）」。按字面它两个分句都不满足：C 是被抛弃的根，不是「可回退到的有效根」，回收门槛也没越过谁（K2-3）。它落在 K2 第一句「回收门槛、影子账、根环表三样用的是不是同一遍读数」那一问的后果上——三样确实用同一遍，同一遍丢了第一遍读到的东西——违反的条款是 `.claude/kb/decisions/23-journal的角色与格式.md:390`：「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」。判它算不算这一轮的打中，由主 agent 照正文第五节第三条核：正文第二节「已知没做」列的是「影子账抛弃判定与择根不是同一次读」「进程内重读读出被抛弃的根不隔离就丢」，都不是这一形；最近的一笔是 C554，它记的进入法是 `.claude/kb/checks-owed.md:427`：「同一形在被抛弃的根所在的槽挂载那一刻读不出、可写挂载里重读也读不出时照样出现」，那是 C 的槽在两遍里都坏；这里 C 的槽第一遍读得出，只坏重读那一遍（另一块盘第一遍坏是触发重读的那个故障），在 rr2 之前的代码上同一段历史不丢。所以我把它报成 rr2 新开的一个进入法（C554 同族），不是已知项的重报。
4. 跑前条款给的改法在打中的格上还中不中：正文第五节没有给改法（「打中 ⇒ 主 agent 现查坐实，改法进下一批」）。岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行 Q4 的两个候选里，「照今天只在两块盘都没读出时重读」在这几格上不中（量过，before 臂 0/12），但它在 `pass1:dev1-all-e+c-e reread:-` 那一格上中（量过，4/4）；rr2 在这几格上中。Q4 那一行的翻面观测本身写了「或在重读之后让 checker 红」，这几格是 rr2 一侧的这种观测。

**用户动作有没有写死**：挂上之后的那几步（接着覆盖写）是用户动作。判 before、merge 两臂「不中」用的不是后缀上的结局，是挂上那一刻分配器的状态：两臂 C 的落点 0 个空闲、隔离 14 / 14（384 槽盘 12 / 12），任何后缀在 C 的根离开根环之前都拿不到这些槽；rr2 臂挂上那一刻 C 的落点 24 个（384 槽盘 20 个）已在空闲池里，第一次分配就取到。后缀只扫了「覆盖写 2999 字节」一种，别的用户动作（建 inode、顺序写、卸载）没扫；它们改的是 rr2 臂红在第几次发布，改不了挂上那一刻的空闲集。

**改法**（只在我的模型上量过、被攻过零轮）：

| 改法 | 做什么 | 修哪一格 | 证据 |
|---|---|---|---|
| merge：重读那一遍按槽并两遍 | 重读之后逐槽取：任一遍自证过的根（两遍都自证过取重读那一条）＞任一遍读坏不是全零＞全零；根环表、回收门槛与影子账都用并出来的那一份 | K2-2 的 12 格；也保住 `pass1:dev1-all-e+c-e reread:-` 那 4 格 | 量过：`out/pivot.txt` merge 列，见上面原样 |
| merge 对 Q3 那一拒的影响 | 并出来的一份里只要第一遍有一条根，Q3 就不拒；Q3 只剩「两遍都一条都没有」 | K2-4 那 4 格从「拒」变回「挂上」 | 量过：`pass1:dev1-all-e reread:all-e` 那一格 merge 4/4 挂上；这改了用户 2026-10-05 定的「重读那一遍整个环仍一条都没有就拒」的字面，要交用户 |
| merge 对回收门槛、抬 F 上限 | 并出来的读坏集只会更小、自证根只会更多：门槛照读坏槽钉住的那几项少了，F 上限按读坏槽并的那几项少了 | K2-3 那几格 | 推的：K2-3 九格 merge 与另两臂同数（量过），但「门槛不越过」只在这一组历史上量过，没有证明 |

merge 的安全性理由（推的）：自证过的根不会是读错造出来的（整槽校验和过且 fsid 相同），挂载期间没有别的写者，第一遍读出的那一条在重读时仍在盘上；所以并进来的根都是真住在那一槽的根。

### K2-3　重读比第一遍坏：回收门槛与抬 F 上限（没打中）

同一组起点，第 6 次读盘 1 全坏，第 7 次读把盘 0（区域 0 与 2）上除最新那条根与 C 之外的每个根环槽读坏（报错、全零、乱字节三种）。这几槽在第一遍都读得出、住着有效根。`out/pivot.txt` 原样：

```
pass1:dev1-all-e reread:dev0-but-newest-and-c-Error | rr2: n=4 red=0 iso0=0 refused=0 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
pass1:dev1-all-e reread:dev0-but-newest-and-c-Garbage | rr2: n=4 red=0 iso0=0 refused=0 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
pass1:dev1-all-e reread:dev0-but-newest-and-c-Zeros | rr2: n=4 red=0 iso0=0 refused=0 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
```

三臂 36 段 checker 都不红，F 相同（4 GiB 盘上 40 次覆盖写不抬 F，`floor_after=0`；384 槽盘上准入把 F 抬到 22 / 21，三臂同数）。读坏、全零的槽在回收门槛里钉在它可能住过的最旧 txg（`txgs_ring_slots_bad_at_mount_can_hold`），门槛只会更低。把 C 的槽也算进去的那三格（`dev0-but-newest-*`）rr2 臂照 K2-2 红，原因同 K2-2，不另算。

取样限度：只在「C 被抛弃」这一个起点族上扫了读坏的槽集，F 只在 384 槽盘上被准入抬过一次；Q2 那一形（全零槽不并上限、F 越过住在全零槽里的有效根）要抬 F 的那一刻正好有全零槽，这组历史没摆出来。rr2 让「第一遍读得出、重读读回全零」的槽进到上限的计算里，是 Q2 代价的一条新进入法（推的，没量出 F 越过谁）。

### K2-4　多拒（实现员读法 ②：与写的相同）

第 6 次读盘 1 全坏，第 7 次读整环全坏（`pass1:dev1-all-e reread:all-e`）：

```
pass1:dev1-all-e reread:all-e | rr2: n=4 red=4 iso0=0 refused=4 | before: n=4 red=0 iso0=0 refused=0 | merge: n=4 red=0 iso0=0 refused=0
```

（这一格 rr2 列的 red=4 是 summarize 的计法：被拒的段没跑 checker，看 refused 列。）rr2 臂 4/4 拒成 `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，rr2 之前那一臂照第一遍走 m 取不到那一支、4/4 挂上，并两遍那一臂 4/4 挂上。代价就是读法 ② 写的那么大：这一次可写挂载被拒、只读照常（这组历史在 4 GiB 与 384 槽两档盘上都不是窄盘，rr2 之前那一臂挂得上）。

### K2-5　新池与年轻的环（实现员读法 ①：与写的相同，射程量过）

`out/young.log`（rr2 臂那一份 grid 日志里同样的行）原样：

```
name=k2-young mount=M1-after-mkfs ring=OnTheOneReread hooks=1 reads_per_ring_slot={15} effective_root=(0,0)
name=k2-young mount=M2-after-a-clean-M1 ring=OnTheFirstRead hooks=0 reads_per_ring_slot={14} effective_root=(1,2)
name=k2-young mount=M3 ring=OnTheFirstRead hooks=0 reads_per_ring_slot={16} effective_root=(2,4)
name=k2-young-sweep-total crash_points=12 m2_rereading=5
```

mkfs 之后第一次可写挂载重读一次：钩子多调 1 次、每个根环槽多读 1 次（15 次对 14 次）。干净做完那一次之后（暖机在 txg 1 落了盘 1 的根）不再重读。第一次可写挂载在它第 n 个写上掉电（n = 1..12 扫完），下一次可写挂载在 n = 1..5 上重读、n = 6..12 不重读。rr2 之前那一臂同一组 0 次重读（`grid-4g-1-before.log` 里 `m2_rereading=0`）。所以「每次可写挂载都重读」只到第一条根落到盘 1 为止。读序号的后挪只发生在这几次挂载里，之后的挂载读序号与 rr2 之前相同。

### K2-6　有没有用例因此钉错（没打中，一处名字与钉值不符）

`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:497`：「fn the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread() {」与 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:518`：「root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,」。钉值对（K2-5 量的就是这一次），名字里的 does_not_reread 只对读阶段成立，读的人会以为这次挂载什么都不重读。不是行为问题，改名进下一批即可。

用了钩子的 harness 用例（`BeforeTheOneReread::CallTheTestOnlyHookFirst`，冻结副本 `crates/singlefs-harness/tests/` 下 6 份文件）里，钩子会在新池那一次挂载上多被调一次的，只有起点是「只做过 mkfs」或「第一次挂载在第 1–5 个写上掉电」的；读过 `writable_mount_of_a_formatted_pool.rs` 第 1752 行起那一段（第三次挂载之前盘 1 已有 txg 1 的根），钩子只在读阶段那次被调。checker 档包里没有一处用 `BeforeTheOneReread` 的钩子（`grep -rln BeforeTheOneReread crates/` 只列 core 与 harness），崩溃注入恒走 `RereadImmediately`。

## K1　Q3 拒因与它的复算

只读码，没跑模型（K1 归正推腿主攻，这一条腿只看与 K2 相连的两处）。

### K1-1　实例表读不出时「不按表滤」那一读法（读法 ③：没打中，推的）

判官在实例表读不出时交假：`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-harness/src/mount_transparency.rs:2544`：「let Some(table_of_the_newest_root) = instance_table_of_root(crash_image, &newest) else {」，下一行 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-harness/src/mount_transparency.rs:2545`：「return false;」。实现这一侧在读根环之前先读这张表，两遍都读不出就拒成另一个成员（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1441`：「None => Err(MountError::NewerStateStillUnreadableAfterOneReread(」），走不到 Q3。所以挂载报 Q3 时，那次挂载读出过最新那条根的实例表：

- 被拒那次挂载不带故障层（崩溃注入第二、三截、E171 装置，`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2585`：「ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults;」）：挂载读的就是镜像，镜像上那张表读得出，「不按表滤」那一支走不到。
- 被拒那次挂载经故障层：镜像上 `choose_root` 可能择到挂载时读坏了的更新那条根，它的表在镜像上可以读不出，于是那一支交假；但这一档判官把「判据复算为假」那一条剔掉（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-harness/src/mount_transparency.rs:2785`：「conditions_not_met.retain(|condition| {」），交真交假判定相同。

两种情形里它都翻不了任何一格判定。推翻条件：找到一条不带故障层的挂载，读出了最新根的实例表、报了 Q3，而同一份镜像上 `instance_table_of_root` 读不出那张表。

### K1-2　复算判据与实现判拒是不是同一个定义（没打中，推的）

被抛弃的判法：判官 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-harness/src/mount_transparency.rs:2559`：「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」，实现 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/recovery.rs:1540`：「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」，逐字同式。读根环：判官走 `readable_roots`，实现走 `read_root_ring_slot_telling_never_written_apart`；两边都把读不出、全零、自证不过当「没有根」，自证过的才算。全零与读坏在 Q3 判据里都不算根，两边同。没读故障时实现的两遍读数相同，Q3 只在镜像上整环真没有不被抛弃的自证根时报；而那时读阶段择到的根按它自己的表不被抛弃（表里只有更早实例的行），所以这种镜像上读阶段本身就择不到根、报的是 `Recovery(NoValidRoot)`，不是 Q3。这是推的，没造反例。

## K4　checker 档装置

只读码，checker 档一条都没跑（子 agent 不跑重型测试）。

### K4-1　第三截 Q3 的计数与报红，与第二截不同口径（线索，推的）

第二截：判官判「不合法」的每一次都报红（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2650`：「tally.writable_mount_refusals_on_the_d23_list_illegal += 1;」，下一句交红）。第三截：只在「崩溃态没注读故障、判据复算为真」时报红，其余只计数（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3806`：「criterion_true_without_read_faults.then_some((reason, conditions_not_met))」；函数文档 `/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3497`：「条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。再挂被拒时 I-8.4 照旧不判，另判 C670 与改法 G4 那两条」）。

后果：第三截那次再挂不带故障层，Q3 在那里只要报出来就一定是错的——要么镜像上判据为真（报红，按设计），要么判据为假（实现无故拒，U6 不合法）。后一种在第三截只进 `remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator` 与 `TheRemountWasRefused` 两个计数，不报红；同一种拒在第二截报红。G4、N-配置 在第三截是同一个口径，Q3 照抄了它（第 10g 批的改动只加了这一支，没改口径）。我没造出一条让实现在第三截无故报 Q3 的历史（不带故障层时两遍读数相同，见 K1-2），所以只是线索：一条只在第三截才走到的 Q3 误拒，今天的崩溃注入看不见。改法（推的、零轮）：第三截里 Q3 与 G4 那两种拒，判官交「不合法」的一律报红，不只「判据复算为真」那一条。

### K4-2　读序号后挪一遍根环（没打中，推的）

rr2 的重读只在第一遍缺盘时发生，没读故障时只在 K2-5 量到的那几次挂载里发生；重读只读根环槽。按每个落点自己被读的次数点名故障的装置里，E158 那一族点名的是最新根的实例表片（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:15131`：「instance_table_page_faults_of_the_newest_root(&base, duration)」），实例表在读根环之前读（`/tmp/claude-1000/m3-batch10-code-r2/tree/crates/singlefs-core/src/mount.rs:1872`：「let (newest_table, instance_table_of_the_newest_root) =」），它的读序号不随根环重读挪。按整池调用序号点名的（`FaultSchedule::the_nth_call_across_the_pool`）在新池那一次挂载里会后挪一整环（24 次读），落到别的读上；哪几条钉值因此换了槽，要 checker 档跑过才知道，正文已写由主 agent 跑。harness 档 `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 按「每个槽第 6 次读」点名，我在被抛弃根那组起点上独立扫出同一个 6（K2-2 第 3 步）。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 重读那一遍仍缺盘 1（m 取不到那一支） | 被抛弃根起点 × 2 盘宽 × 2 前置覆盖写 × 3 臂 = 12 段 | 都挂上、checker 不红 |
| 重读把盘 0 上第一遍读得出的有效根读坏（报错 / 全零 / 乱字节），不碰 C | 同上 × 3 种 = 36 段 | 三臂都不红，F 同数 |
| 只坏重建那一遍（盘 1 全坏），重读读得出 | 12 段 | 三臂都不红；rr2 与并两遍重读，rr2 之前那一臂照第一遍走 |
| 不注故障 | 12 段 | 都不红，隔离 14 / 14（384 槽 12 / 12） |
| 第一遍判法与 m 的定义不一致 | 读码 | 同一个函数（K2-1） |
| Q3 复算与实现判拒的定义不一致 | 读码 | 没找到（K1-2，推的） |
| 读阶段、实例表、影子账三处重读的钩子在新池上互相顶替 | 读了 6 份用钩子的 harness 用例 | 只有「只做过 mkfs」与「第一次挂载在前 5 个写上掉电」的起点多调一次，没找到依赖钩子次数而钉错的 |

层 0 枚举这一轮没跑：攻的是挂载那一刻的读故障与之后的分配，不是崩溃点；原型里没有调 `enumerate_layer0` 的地方，状态数 0。

## 这条腿自己的限度

- 起点只有一族：实例 2 的 C 被崩溃恢复抛弃（`tests/common` 那一形）、盘宽两档、前置覆盖写两档。窄盘起点（192、216 槽）、删除续做、卸载半途崩溃的起点都没跑；K2-2 的机理不依赖起点（第一遍读到、重读丢掉），但「哪些起点上被抛弃根与触发重读的故障同时出现」没量。
- 故障形状写死：触发重读的永远是「盘 1 那一区第 6 次读全坏」，丢 C 的永远是「C 的槽第 7 次读坏」。盘 0 缺根触发、两块盘各坏一部分触发没扫。
- 挂上之后的用户动作只扫了覆盖写一种（判「不中」用的是挂上那一刻的空闲集，见 K2-2）。
- merge 是我在副本上写的改法，只在这一个模型上量过、被攻过零轮；它改了 Q3 那一拒的字面（K2-4 那 4 格不再拒），要交用户。
- K1、K4 全是读码推的；checker 档（崩溃注入第三截、E158、E171）一条没跑。
- 一次抽样；这一条是第二次抽样（s2），与第一次抽样怎么对齐由主 agent 做。

## 没做什么

- 没跑 checker 档、层 0、变异；没在入库装置上重做 K2-2（副本上的数不进 kb，要入库得主 agent 在入库装置上新跑一次）。
- 模型目录的 `rerun.sh` 没整份跑过：前台时限 240 秒装不下 12 次 grid（这一轮 grid 那一次被工具自动挪到后台跑完，约 5 分钟）。它的两个组成部分各核过：补丁打到冻结副本上与跑出产物的那份 `mount.rs` 逐字节相同，`summarize.sh` 拿 `out/` 的日志重抽出的两份与 `out/` 里的逐字节相同。
- K3、K5、K6 不归这条腿，没看。
- 不替主 agent 判 K2-2 算不算这一轮的打中（它与 C554 的关系见 K2-2 第 3 句）。

## 要主 agent 跑的命令

1. 复跑模型（工作目录随意；内存上限 4000M；线程不需要设，原型单线程）：
   `bash scripts/run-with-memory-cap.sh 4000M bash research/prompts/m3-batch10-code-r2-opus-s2-model/rerun.sh /tmp/claude-1000/<主 agent 草稿>/opus-s2-rerun /tmp/claude-1000/m3-batch10-code-r2/tree`
   看末行「比了 13 份，0 份不同」。

## 过程里的几件事

- 收到主 agent 两条消息：读完 `rebuilt_allocator` 那一段时线程上限改成 15，搭原型的时候改回 31；之后起的编译与跑都照新的，原型本身单线程。
- 内存包装拒过三次（退出码 251，命令一行都没跑）：派发提示推的 8G、之后的 4G 都超过峰值表里那条编译命令实测峰值 2041M 的 2 倍；跑测试二进制那一次峰值表里是只过滤出 0 个测试那一次的 2M。改成 4000M，后一种带了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`。
- 负载：开工时 `ps` 看到两条 cargo test（`-p singlefs-harness -- --ignored` 与另一条多包的），没有性能测量在跑；没等锁。

## 草稿与副本

草稿目录 `/tmp/claude-1000/m3-batch10-code-r2-opus-s2/`：仓副本 `tree/`（只有 `Cargo.toml`、`Cargo.lock`、`crates/`，`mount.rs` 打了臂开关、加了原型）、编译目录 `target/`、四轮 grid 产物 `grid1/`–`grid4/`、`young.log`。交回前删掉 `tree/` 与 `target/`，留下日志供主 agent 核。
