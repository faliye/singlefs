# m3-batch10z-lm-code-r1 正推腿报告（第 10z 批第十二、十三块，代码轮第一轮）

日期 2026-10-09。立场：正推。核「代码做的是不是条款、判决与用户定案说的」，判据格 K1（1–3）、K2、K3（1–3）、K4、K5（1、2）。
腿的模型与日志在 `research/prompts/m3-batch10z-lm-code-r1-sonnet-model/`（下称「模型目录」，说明见同目录各子目录文件名）；草稿目录 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet/`（量法副本、编译目录，见「没做什么」后的清理说明）。

## 0　做法与核查

| 项 | 做了什么 |
|---|---|
| 冻结副本 | 在 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree/` 里对 `research/prompts/m3-batch10z-lm-code-r1-snapshot/crates-sha256.txt` 跑 `sha256sum -c`，336 个文件全 `OK`（`grep -c ': OK'` 数得 336，非 OK 行 0）。冻结副本只读，量法与变异都在拷贝上 |
| 第十二块的参照臂 | 用户定的判法出自 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`。我没有用实现员量法副本里的 tri 臂，而是把这份旧 diff 的 tri 逻辑逐支重写成一个参照函数 `triref`（加在冻结副本拷贝的 `mount.rs` 上，`src/mount-triref-and-diff-arm.diff`），再加一个 `SINGLEFS_PROBE_ARM=diff` 臂：同一次调用里先跑入库实现、再跑 `triref`，把两边并进 R 那一槽的落点集合（按盘、槽、跨度去重排序）比，不同就打 `equal=false` 与两边差集 |
| 探针 | 上一轮正推腿的 `probe_jk.rs`（原样，`src/probe_jk.rs.txt`）与实现员量法副本里的 s2 探针（`src/s2_probe.rs.txt`，原件是上一轮攻方 s2 探针；我加了 `S2_ACTIONS` 显式动作串、故障 `rlin1`、`rlin2`、`qracct`、`qlin`、`qall`，差异在 `src/s2_probe-additions.diff`） |
| 第十三块 | 账层（`GpuMemoryLedger`）全在 harness 档，不要卡：在冻结副本拷贝上跑 `-p singlefs-harness --lib -- gpu_memory_ledger`；checker 档两条扫源码用例不跑（重型），把它们逐字搬进 `scan-model/`（模型目录）单独喂源文件；`cargo check -p singlefs-checker-tier --features gpu --lib` 只编不跑 |
| 没起 GPU 设备 | 全程没起 wgpu 设备，没跑 checker 档测试、没跑任何 `.claude/gate.d/` 阶段 |

基线（冻结副本拷贝上、未改）：
- harness 档 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 整份：`test result: ok. 19 passed`（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/baseline-harness.log`）。
- harness 档 `gpu_memory_ledger` 模块：`test result: ok. 117 passed`（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/baseline-gpu-lib.log`）。
- 扫源码模型在冻结副本原件上：`GREEN`、`GREEN`（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/scanmut.txt` 头两行）。

两条行号约定：引代码写冻结副本里的路径与行号（`crates/…`，cite-check 的 `--root` 先找冻结副本）；引 kb 写 kb 文件名加那份文件自己的行号。

## 1　K1　第十二块 tri 做到没有

### 1.1　三态读盘与三种结局（K1-1）

代码今天的样子（冻结副本 `crates/singlefs-core/src/mount.rs`）：
- 读盘一层 `packed_units_named_by_the_records_of_a_publish_as_read`（`crates/singlefs-core/src/mount.rs:2203`）把每个点名的打包单元读成三态：读不出或校验和对不上（`crates/singlefs-core/src/mount.rs:2214` 「Err(_unreadable) => PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,」）、读得出解不开（`crates/singlefs-core/src/mount.rs:2216` 「Err(_unparsable) => PackedUnitNamedByAPublishAsRead::ReadButUnparsable,」）、解开了（带打包记录类型与逐条记录字节）。
- 纯函数一层 `rollback_recognized_among_the_packed_units_named_by_a_publish`（`crates/singlefs-core/src/mount.rs:2240`）交三种结局（枚举在 `crates/singlefs-core/src/mount.rs:2229` 「enum RollbackRecognizedAmongThePackedUnitsNamedByAPublish {」）：认出回退、肯定不是、不知道。
- 落到 R 那一槽的在 `placements_a_rollback_publish_revived_released_in_the_current_account`（`crates/singlefs-core/src/mount.rs:2125`）：肯定不是交空（`crates/singlefs-core/src/mount.rs:2151`），不知道交现行账里全部已释放的落点（`crates/singlefs-core/src/mount.rs:2152` 起），认出回退而根环里没有 R_old 或 R_old 的账读不出也交全部已释放的（`crates/singlefs-core/src/mount.rs:2181` 「Some(_) | None => all_released_in_the_current_account(),」）。

与参照臂逐支对：

| 支 | 参照臂（旧 diff 行） | 入库实现 | 同义？ |
|---|---|---|---|
| 点名的打包单元读不出 | `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff:76` 「let Ok(bytes) = read_unit_via_locations(devices, &named.locations, packed_unit_bytes) else {」置不知道 | 同 | 同 |
| 读得出解不开 | 同文件第 80 行 「let Ok(packed) = crate::unit::parse_packed_unit(&bytes) else {」置不知道 | 同 | 同 |
| 谱系片里一条记录解不开（长度不是 16） | 同文件第 88 行 「if let Some(lineage) = LineageRecord::parse(&record_bytes) {」：解不开的记录被跳过，不置不知道 | `crates/singlefs-core/src/mount.rs:2266` 「None => some_named_packed_unit_unknown = true,」置不知道 | **实现更护**（参照臂当肯定不是） |
| 认出回退 | 取第一条 rollback_txg 等于这条根 txg 的 | 同（遇到就返回，不管别的几片读不读得出） | 同 |
| 没认出且有读不出的 | 参照臂的返回 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff:38` 「return if tri && some_named_packed_unit_unreadable {」宽护 | 宽护 | 同 |
| R_old 候选 | 同文件第 49 行 「.find_map(|rollback_target| {」取根环里 txg 等于 T_old、按实例表有效的**第一条读得出账的**；第 52 行 「if r_old_account.is_none() {」没有就宽护 | `crates/singlefs-core/src/mount.rs:2165` 「Some(accounts) if !accounts.is_empty() => {」：同 txg 的**每一条**都读得出才窄取，任何一条读不出或一条都没有都宽护 | 同 txg 只有一条根时同；有多条时**实现更护** |

实测（造反例而不是读码）：
- 命令形：`SINGLEFS_PROBE_ARM=diff` 经 `bash scripts/run-with-memory-cap.sh 3G bash scripts/capped.sh 1 <探针二进制> --nocapture s2_sweep`（`scripts/piece.sh`，模型目录），12 件并行（≤ 线程上限 15）。
- 扫的形（`scripts/jobs-1.txt`、`jobs-2.txt`、`jobs-6.txt`、`jobs-10.txt`、`jobs-11.txt`）：R 取 b1–b4、b6、b12、b20（回退）、d（删除，含意图崩在半途）、n、o、s；前缀 oooo、oooon、ooonn、oooons、40 次覆盖写；中间一步 Q 取无、o、n、b1；盘 384 与 1536 槽；故障组 racct、racct+rlin、racct+rall、racct+rtomb、racct+oldroot、racct+oldacct、racct+rlin+oldroot、rall、rtomb、racct+rlin1、racct+rlin2（后两个只坏谱系片两份副本里的一份）。
- 入库实现与参照臂并进 R 那一槽的落点集合逐次比：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/tridiff-summary.txt` 末几行 「合计 TRIDIFF=1272」「合计 equal=false 0」，即 1272 次挂载里 0 次不同；diff 臂的 1309 行历史里 `hit=true` 为 0。
- 这 1272 次里结局分布：肯定不是（落点 0）、认出回退窄取（20、24 槽一级）、不知道宽护（82–130、608–726 槽一级，384 槽盘上 40 次覆盖写后到 720）；单份副本坏（`rlin1`、`rlin2`）读另一份认得出，走窄取，与参照臂同（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b10/p1.log`）。

没造出来的形：
- 同一次挂载里两条根同走宽护（R 之后又一条账读不出的 S）：探针里账读不出的根不止一条时可写挂载就拒（`mount=Err:Recovery(UnitUnreadable…)`，`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b7/k1.log`、`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b9/n2.log`），上一轮正推腿也没造出（`research/prompts/m3-batch10z-jk-code-r1-sonnet-model/logs/summary-post-attempts.txt` 的 `mount_err=8`）。这一形只能靠读码：每条根各自调一次本函数，各自并上同一份全部已释放的落点（`crates/singlefs-core/src/mount.rs:2016` 「txgs_of_the_roots_taking_the_current_account.push(root.checkpoint_txg);」之后每条根各调一次），丙并中间照 R 之后那几槽并（E7 用例钉着）。
- R_old 已不在环里：所有 r_old_in_ring 都是 true（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b11/q1.log` 里 b6、b12、b20 的 r_old_in_ring）；b30 在 384 与 1536 槽盘上回退做不成。该支用「R_old 的根槽读回全零」（`oldroot`）代替：根环里读不出 R_old，走同一个 `Some(_) | None`（`crates/singlefs-core/src/mount.rs:2181`），与参照臂同。

判：**一致（逐支同义，两处实现更护）**。
- 更护的两处：谱系片里一条记录解不开算不知道；同 txg 的 R_old 候选要每条都读得出才窄取。前一处要「整单元校验和对得上、解得开、记录宽却不是 16」才走到，零填充类故障造不出；后一处要根环里同 txg 两条根（一个 txg 在环里只有一个槽，`target_for_publish`），我没造出。两处都只会多护、不会少护。
- 推翻条件：任一次挂载里入库实现并进 R 那一槽的集合缺参照臂的某个落点（`missing_in_impl` 非空）；或造出一个「点名的打包单元读不出/解不开/记录解不开」而实现判成肯定不是的形（纯函数用例 `an_unreadable_named_packed_unit_without_a_recognized_rollback_is_unknown` 等钉着这几支，见 K2）。

### 1.2　宽护罩不罩得住（K1-2）

推：R 引用的单元只有四处来历：R 自己写的（记录点名，C 里有）、所选那一版 P 账里仍分配的（丙原有的 `placements_still_allocated_in_the_account` 一截）、P 与 R 之间别的发布写的（丙并中间，按 txg 并 C 里中间那几槽）、回退目标 R_old 引用而 P 账里已释放的（回退复活）。宽护一支把现行账里**全部**已释放的落点并进 R 那一槽，是复活那一截的超集，R_old 账里属于 P 账已释放的都在里面。

测：
- 宽护各形下 `hit=true` 为 0、`first_red` 全空（1309 行 diff 臂历史，含抬 F 与回退发布混着的动作串）；`revived_in_c_r` 恒等于 `revived`（宽护形 2/2），见 `research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b1a/` 各 `.log` 的 `revived_in_c_r=2`。
- 撤掉宽护的变异（K2 里 core[00]–core[03]）在这些形上红 I-7.4，说明这些形是真会复用的形，宽护不是摆设。
- 现行账自己读不出：可写挂载择根要读所选那一版的账，读不出就不会选它（上一小节的 R 落到 P 即此），`current_records` 是读阶段所选那一版交出来的分配记录：分配器由所选那一版的记录重建（`crates/singlefs-core/src/mount.rs:2736` 「PoolAllocator::rebuild_from_records(device_maps, output.allocation_records.clone())」），记录取自它（`crates/singlefs-core/src/mount.rs:2742` 「let current_records = allocator.records().to_vec();」）。P 的账读不出时读阶段不会选它（上面 `n2` 那样两条账都读不出就是挂载 `Err`），不进这个函数。
- R 之后还有较新的根也走丙：见上，没造出（挂载拒）。

判：**一致**（在能造出的形里罩得住；两条根同走宽护那一形没造出，靠读码）。
推翻条件：造出一条 R 引用、P 账里记成已释放而宽护集合里没有的单元（要它的落点跨度被改小：变异 X1 把宽护的跨度一律记 1，已被现有用例拦下，红）。

### 1.3　放的时机与准入（K1-3）

放：宽护落点和别的 C 落点同一槽、同一时机——这个进程写掉 R 那一槽。测了：`iso_end=0` 且 `r_left_at` 有数的历史里隔离数回到 0（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b4/f1-impl-o-d-rall.log` 的 `iso_end 0`、`r_left 14` 一类，b4 的 24 件里 impl 臂 12 件 `refused 0`、`iso_end 0`）。

D28 第九项的计数：第九项的值是分配器实际隔离的槽数，读法在 `crates/singlefs-core/src/admission.rs:586` 「BytesOnOneDevice::of_slots(device_map.isolated_slots()),」，隔离数在 `crates/singlefs-core/src/allocator.rs:601` 「pub fn isolated_slots(&self) -> u64 {」；宽护落点走的是 C 的同一条隔离路径，所以第九项自动含它。kb 字面（`.claude/kb/decisions/28-挂载期承诺量.md:27` 「只除去现行账里仍分配着的槽」）写的是「比所选那一版新、按实例表有效、读得出的根的账里的落点……不论它们有没有被回退候选集里的根引用，只除去现行账里仍分配着的槽」，宽护落点是现行账里已释放的、不在被除去的那一截里，**数得进去**；但字面没说到「认不出回退时并现行账里全部已释放的落点」这一支（判决第五节已说写回等第十二块交回之后一起写，`.claude/kb/decisions/16-发布语义.md:37` 的「根槽这一次读坏」那一行末句止于「改成丙」）。这一格判：**规则没说**（kb 待写回，不是代码错）。

一直不放、把空间占满而准入白拒——**造出来了，窄**（玩具盘，且与参照臂逐字同，是 tri 判法本身的代价，不是实现偏离）：
- 384 槽盘（两块各 384 个单元区槽）、前缀 40 次覆盖写、R 是回退（b6）、R 的账与谱系片读回全零：挂载那一刻隔离 720 槽（两块盘合计 768 槽），之后的用户写全被拒，`NoSpaceAfterRaisingTheFloor`，段末隔离仍 720：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b11/q2.log:11` 「iso_mount=720 iso_after_first_raise=- iso_max=720 iso_end=720 refused_steps=2」（动作串 `oo`，2 步 2 拒）、同文件第 14 行 「iso_mount=720 iso_after_first_raise=- iso_max=720 iso_end=720 refused_steps=3」（动作串 `oFoFo`，5 步 3 拒，其余是 F）。该文件里 `diff` 臂同次的 `TRIDIFF` 为 `equal=true`（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b11/q2.log` 头部 608 一级的那几行）。
- 一直不放的机理：宽护落点随 R 那一槽解开，要这个进程**写掉 R 那一槽**；写被准入拒了就不发布、R 那一槽不动，隔离不退。384 槽盘上那一串动作里只有回退发布（不申请新空间）出得来：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b5/g1-impl.log:4` 「iso_end=234 refused_steps=23 first_refused=11o:NoSpaceAfterRaisingTheFloor」是同一前缀的另一形（删除发布 R、R 的全部单元读回全零，随机动作串第 11 步起 23 步里 23 拒、R 一直留在环里）；末尾换成连续回退（动作串 `nbsbFFbFobboFobooFbbbbbbbbbbbbbbbb`）R 才离开环，`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b5/g1-impl.log:7` 「r_left_at=23」（该件 4 步被拒后连续回退把 R 写出环）。同样动作串在窄取法（`narrow` 臂，pre-tri）下 0 拒：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b5/g1-narrow.log:4` 「iso_end=0 refused_steps=0」。
- 规模：宽护集合是现行账里全部已释放的落点，随前缀长度饱和，不随盘大小涨。6144 槽盘上前缀 10、20、40、80、160 次覆盖写，R 是回退（b4）、racct+rlin：挂载隔离 260、524、838、968、968 槽（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b12/w10.log:4` 「iso_mount=260 iso_after_first_raise=- iso_max=260 iso_end=260 refused_steps=0」、`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b12/w20.log:4` 「iso_mount=524」……，`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b12/w80.log:4` 「iso_mount=968 iso_after_first_raise=- iso_max=968 iso_end=968 refused_steps=0」、`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b12/w160.log:4` 同 968），窄取法同形 18–22 槽（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b12/x160.log:4` 的 `iso_mount=18`，前缀 20 次时 22）；1536 槽盘同形 840 槽（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b11/q1.log`），0 拒。所以拒写只在隔离量接近整盘的玩具盘出现；真盘上这一截占盘的比例随盘变大趋近 0，但被拒的玩具盘形和"不释放要靠发布"这个机理是 tri 的固有代价，用户 2026-10-08 弹窗定案时的数字（384 槽盘挂载 116 槽对 42 槽）没带这一形。
- 推翻条件：把盘放大到 6144 槽、前缀 80 以上，若宽护集合随盘大小线性涨（而不是饱和在 968 一级），本条的"规模"一句就错；或在 384 槽盘上造出同一形而不拒写。

判：K1-3 **打中（窄，玩具盘，tri 判法固有）**。D28 第九项计数**一致**（走同一条隔离路径）。放的时机**一致**（写掉 R 那一槽时放；写不出去时不放）。

## 2　K2　第十二块 K5 判别力

做法：把实现员交的第十二块 22 行变异（`_m3-batch10z-lm-code-r1-diff.md` 里 mutations-append 18 行 + mutations-replacements 4 行，模型目录 `mut/core-rows.json`）逐行套到**冻结副本的新拷贝**上，各跑它点名的测试目标（harness 档 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 整份 19 个，或 `-p singlefs-core --lib` 纯函数那几条），每个变异一份独立的编译目录；再加我自己写的 12 行（`mut/extra-core.py`）。驱动 `mut/drv.py`，3 个工人 × 4 线程，全经 `run-with-memory-cap.sh` 与 `capped.sh`。汇总 `research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/mutation-summary.txt`（core 34 行：红 30、照绿 4）。

- 实现员交的 22 行：**22 行全红**（该汇总文件 core[00]–core[21] 全 `RED`），E1（core[08]）、E2（core[09]，纯函数用例 4 个红）、E3（core[07]）、E7（core[17]，纯函数 `every_root_taking_the_current_account_takes_what_the_publishes_between…`）都能拦住改回去。
- 三态的每一支，改回去有没有照绿：

| 支 | 改回去 | 拦住它的用例 | 结果 |
|---|---|---|---|
| 不知道 → 宽护（谱系片读不出） | 交空 | `units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount`（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:2130`） | 红（core[00]） |
| 不知道 → 宽护（R 写的全部单元读不出） | 交空 | `units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount`（同文件第 2152 行） | 红（core[01]） |
| 读盘一层：读不出 | 当读得出的非谱系片 | 上面两条 | 红（core[02]） |
| 读盘一层：**读得出而解不开** | 当读得出的非谱系片（旧 `.ok()` 吞掉的写法） | **没有** | **照绿（X18，core[28]：`19 passed`）** |
| 读盘一层：**只看第一条记录点名的打包单元 / 只看第一个打包单元** | 加 `.take(1)` | **没有** | **照绿（X4、X5，core[24]、core[25]：`19 passed`）** |
| 读盘一层：不筛类标签 3 / 类型记 0 / 记录丢空 | | 非回退那条与复活那几条 | 红（X6、X20、X21） |
| 纯函数：读不出、解不开、记录解不开各算不知道；认出后别片读不出不改；一片都没点名算肯定不是；认回退比 rollback_txg；交回 T_old | 各改回去 | 纯函数 8 个用例 | 红（core[09]–core[16]、X7、X19） |
| 肯定不是 → 交空 | 也宽护 | `a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount`（同文件第 2308 行，隔离数钉 18：`const ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK: u64 =` 在第 2374 行） | 红（core[06]） |
| 认出回退、R_old 根槽读不出 / R_old 账读不出 | 交空 | `…its_account_and_the_root_slot_of_its_target_read_back_zero…`、`…its_account_and_the_account_of_its_target_read_back_zero…` | 红（core[03]、[04]、[05]、X23） |
| 宽护集合：跨度记 1 / 取仍分配的不取已释放的 | | 复活那几条（`free_slots_covered_by` 看整段跨度） | 红（X1、X2） |

照绿的三行（X4、X5、X18）的原因，用 `TRIDESC`（我在拷贝的 diff 臂里加的打印）坐实：
- 回退发布只点名**一个**打包单元，就是谱系片：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b6/h6.log:3` 「records=1 packed=["rec0:packed:type5"]」；建 inode 的 R 点名一个类型 2 的：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b6/h3.log:3` 「records=1 packed=["rec0:packed:type2"]」；顺序写的 R 三条记录、只第三条点名一个类型 2 的：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b6/h5.log:3` 「records=3 packed=["rec2:packed:type2"]」。所以「只看第一条记录 / 第一个单元」在这几个形状上与全看同义，现有用例分不出。
- 入库的 harness 档测试文件里没有删除发布（墓碑片）的用例：在冻结副本该文件里 `grep -c 'tomb'` 数得 0。而实现员报告的「删除发布的墓碑片读不出也会触发多护」我用 diff 臂造出来了（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/runs/b2/` 里 `faults=racct,rtomb` 的行里 `iso_mount` 取过 136、142、166，另有 18 的那几行是 R 不带墓碑片的前缀；对照 `faults=racct` 的 18、26），即这一支是对的但**没有入库用例钉着**。X4/X5 若被「顺手优化」成只看首片，删除发布墓碑片读不出这一形会悄悄退回窄取（该形的复活集为空，没有 I-7.4 风险，所以不红；但与用户 2026-10-08 定案时交代的「删除发布的墓碑打包单元读不出时也会触发多护」不符）。
- X18 的读盘半边无法在 harness 档造：要「整单元校验和对得上而 `parse_packed_unit` 解不开」，零填充类故障做不到，需要同时改单元字节与位置条目里的校验和；纯函数那一半（`ReadButUnparsable` 算不知道）有用例钉着。
- X22（`.take(1)` 只取第一个同 txg 的 R_old 候选）照绿：根环里一个 txg 只有一个槽，等价变异，不算漏。

判：K2 **打中（窄）**：入库的 22 行变异全红，E1、E2、E3、E7 与三态大部分支能拦；但读盘一层「遍历全部记录与全部点名单元」「解不开算不知道」这两处改回去照绿，且「删除发布墓碑片读不出 → 宽护」没有入库用例。
推翻条件：能给出一个入库用例，在 X4 / X5 / X18 之一上红（例如一个点名两个打包单元、不可读的那个排第二的回退发布）；或证明回退发布的 `named` 表恒只有一个打包单元（则 X4/X5 是等价变异，只剩删除发布那一条缺口）。
补用例的建议（不替主 agent采纳）：删除发布 R + 墓碑片读回全零 + R 的账读回全零，断言 C 里 R 那一槽含现行账里全部已释放的落点；我的探针形状在模型目录 `scripts/jobs-2.txt`（`faults=racct,rtomb`、`S2_RS=d`）。

## 3　K3　第十三块 借之前早记「丢了」

代码（冻结副本 `crates/singlefs-harness/src/gpu_memory_ledger.rs`）：`take_in_the_devices_kept_before_booking`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1443` 「pub fn take_in_the_devices_kept_before_booking<Device>(」）对列出来的每张卡取卡池那一格留着的设备，丢了就调 `take_in_that_the_device_was_found_lost`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1673`），后者只在 `crates/singlefs-harness/src/gpu_memory_ledger.rs:1676` 「if books_of_the_card.device_in_use == Some(self.device) {」成立时才调 `take_back_the_driver_share_of_a_lost_device`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:866`）——这一个函数就是旧重起路径（`prepare_the_driver_share_before_starting_the_device`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1320`）里记丢了的那一步，早记没有写第二套记法。

### 3.1　多给（K3-1）

推：记成丢了之后驱动那一份是 `DeviceLostNotStartedAgain`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:867` 「self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;」），这个状态下实际占着的里算作「属于许出去的」的部分交 0（`crates/singlefs-harness/src/gpu_memory_ledger.rs:215` 「| DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,」），所以此刻这个进程能占的 M 只等于现读的空闲 F，能借的是 `容量(F) − 活着的借据`，容量里已减驱动那一份 D，活着的借据 ≥ 还没建的：不会超过 `F − D − 还没建的`。没等成的借据在记丢了时出账，它们的缓冲在丢了的那台设备上、仍占着卡，已经在现读的 F 之外，不会被再借一次。旧设备还有克隆没丢时，那一台进 `lost_devices_not_released`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:869` 「self.lost_devices_not_released.insert(lost_device);」），重起好之后状态是丢了的还有没放掉的那一支（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1393` 起），占着的仍交 0，直到最后一个克隆放掉。

测：
- 入库用例钉了两条：缓冲仍占着卡 `a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:5371`，同文件第 5396 行断言 `room <= room_truly_left_on_the_card`）；重起之后新设备没丢 `a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:5409`）。
- **随机操作串**（我加的 T5，`src/lm-gpu-ledger-tests-T1-T5.diff`）：一张假卡（沿用入库的 `CardUnderTheLedger` 与 `room_truly_left`），4000 条随机串、每串 14 步，操作含借（1000/4000/9000/15000）、建缓冲、丢借据（等成/没等成）、设备丢了（缓冲与驱动那一份转成「丢了的留在卡上的」）、借之前那一步、重起、放掉旧设备。凡是「账已经知道丢了」（借之前那一步列出了这张卡，或重起的 prepare 走过）或根本没丢的状态，断言 `room_now ≤ room_truly_left`：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/gpu-lm-tests-baseline-t1-t5.txt` 「LM_T5 checked=35556 violations=0 lost_but_unmarked_overgive_states=13208」，35556 个断言点 0 违例。
- T5 会红的自证：把新增的几处改坏再跑 T5（`mut/gpu-lm-t5.py`，输出 `research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/gpu-lm-t5-on-mutants.txt`）：借之前看到丢了不告诉账 130 违例；告诉账只销那几笔不照重起记 1468；Z3（丢了的改成销列出来的借据）1468；Z6（在用的那台不进 lost_devices_not_released）312；Z5（不销没等成的，更保守）0。T5 能区分。

判：**一致（没造出多给）**。推翻条件：T5 加一种操作（例如 poll 交回队列空了同时转移缓冲）后出现 `violations>0` 且违例串里的最后一次账变化来自 `take_in_the_devices_kept_before_booking`；或真卡上丢失标记变真之后读数里丢了的设备的占用比 `left_by_lost` 模型小（则账少扣，多给）——真卡没造，复核不了。

### 3.2　只在在用的那一台时才改账（K3-2）

代码按设备号比，不按「是不是丢了」比（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1676`）。三种交错：
- 卡池取到的是重起之前留着的那台（旧）、新的一台已记在账上：旧设备号 ≠ 在用的新设备号，什么都不改。入库用例 `a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone`（第 5409 行）钉着：新设备上没等成的那一笔照旧记着，账不动。变异 `if true`（把设备号条件换成恒真）红。
- 重起正在进行：线程 B 已经 prepare（旧设备记成丢了，在用的是 `None`）、新设备还没记：线程 A 的早记见 `device_in_use == None`，不改；B 随后 `record_the_device_started_on_the_card` 断言在用的是 `None`（`crates/singlefs-harness/src/gpu_memory_ledger.rs` 里的 `assert_eq!(books_of_the_card.device_in_use, None,` 一句）仍成立。我没有造线程交错的用例，靠账的锁（每一步在 `books_locked()` 里改）与设备号比；T5 里重起前后穿插借之前那一步 35556 个断言点无违例。
- 两条线程同时借同一张卡：两条都见到旧设备丢了，第一条改账，第二条见在用的是 `None`，不改；幂等。
- 「丢了的永远记不上」：早记只对列出来的卡做，列不出（没有没等成的借据）的卡仍到重起那一步记（3.3）。

照绿的改回去：
- Z1「取留着的设备缺了的那张卡就 `break`（原是 `continue`）」照绿（`mut/` 汇总 gpu[11]）：`crates/singlefs-harness/src/gpu_memory_ledger.rs:1451` 「let Some(kept) = device_kept_on_the_card(listed.card_position()) else {」之后的 `continue`；第一张卡的设备正在起（`started_now` 交 `None`）时，后面的卡这一轮列不到。后果只是别的卡的没等成借据这一轮没被早记/销，更保守，不多给；但入库用例里没有「第一张卡取不到设备、第二张卡有」的串。我补的 T3 能拦（`src/lm-gpu-ledger-tests-T1-T5.diff`，baseline 绿、Z1 红：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/gpu-lm-tests-on-mutants.txt` 的 Z1 段）。
- Z6：在用的那一台记成丢了时不进 `lost_devices_not_released`，入库 117 个用例全绿；后果是重起之后驱动那一份回到 `DeviceStarted`、丢了的那台连它的缓冲还占着卡却被算成「属于许出去的」，多给。T5 的随机串在 Z6 上红（312 违例），我补的 T2 红（同一份日志的 Z6 段）。这是「早记之后接重起」没有入库用例钉着。
- Z8：借之前没丢的那一支 poll 之后销「该卡此刻全部没等成的」（不只列出来的）照绿；后果是 poll 的当口另一笔刚记成没等成的借据（它的活可以是 poll 之后才提交的）被一起销，多给。入库的钉子只在直接调 `take_in_that_the_work_of_the_listed_leases_finished` 那一层（`crates/singlefs-harness/src/gpu_memory_ledger.rs:906` 一带），新函数这一层没有。我补的 T1（在 poll 闭包里记一笔没等成的借据）baseline 绿、Z8 红。

判：K3-2 **一致（代码按设备号比，没造出错记）；改回去照绿三处（Z1、Z6、Z8）**，归 K4。
推翻条件：造出一串线程交错使 `device_in_use` 先被新设备占、旧设备的早记仍改了账——代码上 `device_in_use == Some(self.device)` 是在锁里比，不会；或造出一串使丢了的永远不被记（重起的 prepare 不走）。

### 3.3　没有没等成借据、但设备丢了的卡（K3-3）

早记只对 `leases_kept_after_a_failed_wait_on_each_card` 列得出的卡做。没有没等成借据的卡，丢了的设备要等借到之后的重起才记。这一格我造出了反例（账层，T4，`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/gpu-lm-tests-baseline-t1-t5.txt` 「LM_T4 room_now=29550 truly_left=28950」）：设备起好（驱动实占 600）后丢了，丢了的设备的驱动那一份 600 还占着卡，账上没有没等成的借据，借之前那一步列不出这张卡；此刻能借的 29550，卡上真能再给的 28950，多给 600。借走 29550 之后重起：「LM_T4 restart_after_booking_room=Err(NoCardHasRoomNow { need_mebibytes: 2560, card_rooms_mebibytes: [0] })」，即 `prepare_the_driver_share_before_starting_the_device` 拒（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1360` 「if capacity_mebibytes < booked.leases_mebibytes {」），卡池报显存不够、本次未跑。
- 不是 OOM 风险：多给的那一笔在重起检查处被拦下，借据还掉，没有往卡上建东西。
- 这一形不是第十三块带进来的（改之前同样），但第十三块的目的是让账在借之前就知道丢了，这一格只对「有没等成借据的卡」做到。T5 的随机串里这种「丢了、账还不知道」的断言点有 13208 个超过真能再给的（上面日志同一行 `lost_but_unmarked_overgive_states=13208`），都落在这一形。
- 把借之前那一步改成对**所有**留着的设备都查丢没丢（不限列得出的卡）会补上这一格；这是改法建议，没有实现、没有量。

判：K3-3 **打中（窄）**：多给 600 一级（丢了的设备的驱动占用），被重起检查拦下，结局是「卡放得下的一笔报本次未跑」（借的量落在 `F−D` 与 `F+占用−D` 之间时）；不是 OOM。推翻条件：真卡上丢了的设备在被检出丢了的同一刻就把它的驱动占用交回驱动（则 600 不存在，T4 的前提不成立）——真卡没造，复核不了。

## 4　K4　第十三块 K5 判别力

做法：
- harness 档（账层，不要卡）：实现员交的 11 行变异（`_m3-batch10z-lm-code-r1-diff.md` mutations-append 里 `gpu_memory_ledger.rs` 那 11 行，`mut/gpu-rows.json`）套到冻结副本新拷贝上，跑 `-p singlefs-harness --lib -- gpu_memory_ledger`（整个模块 117 个用例，不是只跑点名那一个），加我自己写的 8 行（`mut/gpu-h.json` 的 Z1–Z8）。汇总 `research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/mutation-summary.txt`（gpu 19 行：红 14、照绿 5）。
- checker 档（要卡）：不跑。把两条扫源码用例逐字搬进 `scan-model/`（模型目录），在冻结副本原件上都 `GREEN`，再套实现员的 9 行与我自己的改写。

结果：
- 实现员交的 11 行 harness 变异 + 9 行 checker 档变异：**全红**。G1（gpu[08]）、G2（gpu[09]，10 秒超时那一件）、G4（gpu[10]）、Y1–Y4（gpu[03]–[06]）都拦得住；下游六层叫停 6 行与卡池 3 行，在扫源码模型上全 `RED`（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/scanmut.txt` 里 9 行 `table | … | RED`，`grep -c 'table.*RED'` 数得 9）。
- 我的 8 行里照绿的（harness 档）：
  - Z1 取不到设备就 `break`；Z6 在用的那台不进 `lost_devices_not_released`；Z8 poll 之后销该卡此刻全部没等成的而不只列出来的（原因与后果见 3.2）；Z7 只核卡位不核是不是同一本账（要两本账，造不出，不算）；Z4 加「且有没等成的借据」是等价变异（列出来就意味着非空）。
  - 补上的 T1、T2、T3 把 Z8、Z6、Z1 拦下（baseline 全绿、变异各红一个）：`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/gpu-lm-tests-on-mutants.txt` 的 Z1、Z6、Z8 三段；用例在 `src/lm-gpu-ledger-tests-T1-T5.diff`，不进仓，由主 agent 决定要不要采。

checker 档那几行在合后副本上真跑过（第二节所述 9 行 prove-red `点名 9 条：跑了 9 条，跳过 0 条，跑的都抓到了`），我没有重跑，复核不了。扫源码用例能不能被「不改行为的写法」骗过，我用搬运的模型判：

| 改写 | 行为 | 扫源码用例（模型） | 编得过？ |
|---|---|---|---|
| A2 函数体最前面加 `if true { return; }` | 借之前那一步整个不执行 | **GREEN**（`logs/scanmut.txt:13`） | 是（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/cargo-check-A2-A3-A7.txt:1` 「A2 函数体最前面 return（整句不执行） rc= 0」） |
| A3 把「丢没丢」与「poll」两个闭包对调位置 | 用 poll 结局当丢没丢、用丢没丢当 poll 结局：健康且空闲的设备被记成丢了 | **GREEN**（`logs/scanmut.txt:14`） | 是（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/cargo-check-A2-A3-A7.txt:2` 「A3 丢没丢与 poll 两个闭包对调位置（类型相同，编得过） rc= 0」） |
| A7 崩溃放量某一层「往下交的换成永不叫停、另加一句 `let _ = stop_is_requested`」 | 叫停被丢掉 | **GREEN**（`logs/scanmut.txt:18`） | 是（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/cargo-check-A2-A3-A7.txt:3` 「A7 叫停：往下交的换成永不叫停、另加一句 let _ = stop_is_requested rc= 0」） |
| B2 局部变量 `started` 改名 | 不变 | RED | 是（`logs/scan-false-red.txt` 末两行 `Finished`） |
| B3 `.as_deref()` 换成等价的 `.as_ref().map(\|device\| &**device)` | 不变 | RED | 是 |
| B1 闭包参数后多一个空格 | 不变 | GREEN（去空白，应当） | |

即：这两条扫源码用例按「子串各出现一次、闭包个数三个」判，**不判语句有没有执行、两个同型闭包的次序**；A2、A3、A7 三种改写让行为变了而它们仍绿，B2、B3 让行为不变而它们变红（后者是扫源码用例的通病，不算错）。A3 最值得看（后果是推的，按 Z3 的数）：真丢了的设备被拿去 poll（wgpu 的 panic 被接住、算没空），而后「丢没丢」那个位置上放的是 poll 结局，于是按「空闲」销列出来的借据却不记成丢了——和 Z3 同一个行为，T5 随机串上 1468 个断言点多给；健康的空闲设备反被记成丢了，只会少借。账层的用例钉的是账这一侧的行为，卡池这一侧「哪个闭包放哪个位置」只有这一条扫源码用例管，而它管不到次序。
- 采样口径：A2、A3、A7 是 1 次改写各 1 次判，不是随机抽样；它们说明存在这种改写，不说明现实里会这样写。
- checker 档的 `cargo check` 我只对 A2、A3、A7、B2、B3 编过，用的是 `--features gpu`（不带 `verdict-store`，后者在本机要 rocksdb 的 bindgen 环境，编不起来）。

判：K4 **打中（窄）**：入库的 20 行变异（11 + 9）全红，G1、G2、G4、Y1–Y4、下游六层与卡池三处都能拦；但「早记之后接重起（Z6）」「poll 当口新记的没等成借据不销（Z8）」「取不到设备不影响后面的卡（Z1）」三处改回去照绿，且扫源码用例能被 A2、A3、A7 三种行为变了的写法骗过。
推翻条件：给出入库里已有的、在 Z1、Z6、Z8 任一上红的用例；或证明 A3 的写法在卡池里类型不通（我用 `cargo check -p singlefs-checker-tier --features gpu --lib` 编过，`rc= 0`）。

## 5　K5　只防 OOM、不压并发两头（第十三块新代码）

### 5.1　账给的量大于此刻真能用的（K5-1）

- 新代码路径上（早记丢了之后到重起、放掉旧设备之前）：随机串 T5 的 35556 个断言点 0 违例（3.1），公式上 `容量(F) − 活着的借据 ≤ F − D − 还没建的`。
- 有一形：设备丢了、账上没有没等成的借据、借之前那一步列不出这张卡，丢了的设备的驱动占用还在卡上。T4：此刻能借的 29550，真能再给的 28950（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/gpu-lm-tests-baseline-t1-t5.txt` 「LM_T4 room_now=29550 truly_left=28950」）。这一形不是第十三块新引进的（改之前同样），多给的部分被重起时的检查拦下（3.3），不会往卡上建东西。

判：**打中（窄）**：一形账层读数多给 600 一级，被重起检查兜住，无 OOM 路径。推翻条件：真卡上丢了的设备被检出丢了的同一刻就把驱动占用交回驱动；或重起检查在这一形上不拒（改成放行）。

### 5.2　卡放得下却报本次未跑、判红、整趟弃卡、或并发被压住（K5-2）

- 报本次未跑：T4 那一形在借的量落在 `F−D` 与 `F+丢了的占用−D` 之间时，借得到、重起被拒、报本次未跑（`research/prompts/m3-batch10z-lm-code-r1-sonnet-model/logs/gpu-lm-tests-baseline-t1-t5.txt` 「LM_T4 restart_after_booking_room=Err(NoCardHasRoomNow」）。此刻卡上老设备还占着，严格说放不下；等老设备放掉就放得下。早记丢了不会让这一格变多：有没等成借据的卡，早记之后能借的是 `F−D−B`，改之前是 `F+占用−D−B`，借的量落在两者之间时改之前是「借到、重起被拒」，改之后是「借不到」，同一个未跑结局，少一次借了又还的瞬时占用（推的，没量）。
- 判红：借之前那一步里没有 panic 路径除两处断言（列的卡与取到的设备对不上，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1456` 起），卡池接线（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1339` 起）按卡位取格、按设备取账上的那一台，接错就是代码错，不是卡放得下却判红。
- 并发被压住：`started_now` 只在取克隆那一下拿 `started`（不等 `starting`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1798` 「pub fn started_now(&self) -> Option<Device> {」），poll 不在账的锁里，告诉账是锁里一次定点改；换下来的旧设备在放开 `started` 之后才丢（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1777` 「let replaced = self」，变异 G2 红）。没有新引进的串行点。
- 整趟弃卡：未追。`OutOfGpuMemoryStartingTheDevice` 在产品路径只转成一行未跑的说明（`crates/singlefs-checker-tier/src/crash_amplification.rs:2746` 「CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason } => {」），卡是否因此在这一趟里被标成缺显存而不再用，没有读到底（`rounds.is_short_of_memory` 在 `crates/singlefs-checker-tier/src/crash_amplification.rs:2976` 起），记「没追」。

判：**一致（除 5.1 那一形的未跑，其余没造出；整趟弃卡没追）**。推翻条件：在有没等成借据的卡上造出一串，早记之后的借被拒而改之前能借到且重起放得下；或产品路径里重起被拒把整张卡弃到整趟结束（则 T4 那一形的未跑会放大成整趟弃卡）。

## 6　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 三态读盘与三种结局 | 一致 | 对 jk 轮 tri 臂逐支同义，两处实现更护（记录解不开算不知道、同 txg 的 R_old 候选要每条可读）；1272 次挂载比对 0 差异、1309 行历史 0 红 |
| K1-2 宽护罩不罩得住 | 一致 | 在能造的形里罩得住；两条根同走宽护、R_old 不在环里没造出（挂载拒/回退做不成），靠读码与相邻形 |
| K1-3 放的时机与准入 | 打中（窄） | 放的时机照 R 那一槽被写掉；384 槽玩具盘 40 次覆盖写后隔离 720/768、写全被拒、不放（与参照臂逐字同，tri 固有，规模饱和在 968 槽一级） |
| K1-3 D28 第九项计数 | 一致；kb 字面规则没说 | 计数读分配器隔离数，宽护落点走同一路径；kb 第九项字面与 D16 第 37 行未写 tri 这一支，等判决说的写回 |
| K2 第十二块 K5 判别力 | 打中（窄） | 入库 22 行全红、E1/E2/E3/E7 拦得住；读盘一层「全部记录/全部单元」「解不开算不知道」改回去照绿，删除发布墓碑片读不出→宽护没有入库用例 |
| K3-1 早记丢了后多给 | 一致 | 公式与 T5 随机串（35556 点 0 违例）；旧设备未放掉、缓冲占卡都不多给 |
| K3-2 只在在用的那台时才改账 | 一致 | 按设备号在锁里比；重起前后、两线程同借都幂等；改回去 Z1、Z6、Z8 照绿归 K4 |
| K3-3 没有没等成借据而设备丢了的卡 | 打中（窄） | 不早记；T4 此刻多给 600，借走后重起被拒报未跑，无 OOM |
| K4 第十三块 K5 判别力 | 打中（窄） | 入库 20 行全红；Z1、Z6、Z8 改回去照绿；扫源码用例被 A2、A3、A7 三种改变行为的写法骗过，B2、B3 等价改写反变红 |
| K5-1 账给的量大于真能用的 | 打中（窄） | 同 K3-3 的一形，兜在重起检查，无 OOM |
| K5-2 放得下却未跑/判红/弃卡/压并发 | 一致 | 除 K5-1 那一形外没造出；未造出新的串行点；整趟弃卡没追 |

## 7　没做什么

- 没跑 checker 档测试、没起任何 GPU 设备、没跑 `.claude/gate.d/` 下的任何一道，没跑崩溃枚举；checker 档那 9 行 prove-red 我没重跑，用逐字搬运的模型代判扫源码那两条，模型不等于真用例。
- 没造出：同一次挂载里两条根同走宽护；R_old 已不在环里；多线程真交错（线程交错只靠锁与设备号推、加随机串）；真卡上丢设备与丢失标记何时变真；产品路径整趟弃卡是否放大 T4 那一形。
- 没判别的腿：不判别攻方腿的格；不替主 agent 采纳或出判决；补用例（T1–T5、删除发布墓碑片读不出的用例）只给在模型目录，没进 `crates/`。
- 没动 kb、没动 `crates/`；kb 里 D16 第 37 行、D28 第 27 行没写 tri 这一支是判决第五节说的待写回，我只记「规则没说」。
- 数字都是冻结副本拷贝上量的；入库装置上的复跑（E171 一类）不在这一轮。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet/`：量法副本与编译目录在交回前删掉（`tree-core`、`tree-m1`、`tree-m2`、`tree-m3`、`tree-gpu`、`tree-chk`、`target-core`、`target-m1`、`target-m2`、`target-m3`、`target-gpu`、`target-chk`、`target-scan`、`scan-model`，可由模型目录重建，见 `research/prompts/m3-batch10z-lm-code-r1-sonnet-model/README.md`），脚本与日志留着。

## 8　cite-check

`python3 scripts/cite-check.py research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree --root <仓根>` 末行：

  ✓ 核了 37 处引文，对上 37 处，没判 0 处（逐处列在上面）
