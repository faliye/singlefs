# 第 10z 批第十六至十九块：代码轮第一轮正推腿报告（2026-10-09）

轮名 m3-batch10z-pqrs-code-r1，立场正推。冻结副本 `~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/`（下文代码引文的路径都相对它），kb 引文相对仓根。
模型与日志：`research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/`（README 在 `README.md`；探针 `src/p10z_probe.rs.txt`，变异驱动 `scripts/mut.py`，扫源码模型 `src/scan-gpu-main.rs.txt`、`src/scan-crash-main.rs.txt`，日志 `logs/`）。

## 0. 开工核对与做法

冻结副本对快照：

```
$ cd <冻结副本> && sha256sum -c research/prompts/m3-batch10z-pqrs-code-r1-snapshot/crates-sha256.txt | awk '{n++; if ($NF!="OK") bad++} END {print "共", n, "行，非 OK", bad+0}'
共 336 行，非 OK 0
```

做法（都在冻结副本的拷贝上，冻结副本没改）：

- 第 K1–K3 格：拷贝 `tree-a`，把上一轮正推腿的随机历史探针（`research/prompts/m3-batch10z-no-code-r1-sonnet-model/src/s2_probe.rs.txt`）改到新接口，建成 harness 档用例 `p10z_probe`（release，`bash scripts/run-with-memory-cap.sh 3G bash scripts/capped.sh 1 <二进制>` 起，一件一个进程）。新增：动作 `W`/`N`/`S`/`G`/`H`（写；被「推满仍不够」拒时当场记报告、独立重算再显式推环、再写一次）、`P`（显式推环）、`X`（卸载再挂）、`K`（不卸载直接崩了再挂）、`J`（往根环表注入多槽 C）；环境变量 `P_FAIL_AT`（推环里第 n 次写报块设备错）、`P_CRASH`（推环里每条根写之前与之后的镜像各挂一次、再写 4 次、判 I-7.4 与 I-4.8）。
- 报告的独立口径：报里的次数 n 用「从现行 txg 起逐次发布数到 C 记着落点的每一槽都被写到」暴力数（不用 `root_ring_push_target` 的算法）；挡着的槽数用「C 全部落点盖住的槽里隔离位置着的个数」另数；R 的 txg 与录下来的 R 比。三个量任一对不上，探针打 `!needs`、`!iso`、`rNe`。
- 第 K4 格：拷贝 `tree-m`，在 `mounted_session.rs`、`allocator.rs`、`mount.rs` 上套 25 个变异，各编、各跑入库用例文件（`--test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table`，release，35 过 1 个 ignored）。
- 第 K4、K5 格的 GPU 与崩溃放量扫源码用例：checker 档测试不跑；把三条（`the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking`、`the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`、`a_card_hands_out_its_own_device_and_its_own_device_condition`）与 `every_layer_handed_the_stop_request_hands_it_on_or_asks_it` 的测试体与帮助函数从冻结副本源码按行区间机械抽出，只把 `include_str!` 换成读文件，做成两个独立小程序（不进 checker 包），喂改过的源码文本报 GREEN 或 RED。
- 没跑：门禁阶段、checker 档测试、gpu 特性、崩溃枚举（`P_CRASH` 只在推环里每条根写的前后各取一个镜像，不是崩溃枚举）、两进程竞态、G2 乙、tri 判法本身。

## 1. 用户定案与条款，逐支核入库实现是否同义

核的是 D16 已定项 1 第 29 行、第 37 行「根槽这一次读坏」那一行、D28 已定项 1 第 27 行第九项，与用户 2026-10-08 弹窗原话。kb 当前已写回这一批的口径（第 37 行含推环入口），所以下面是「kb 现行字 ↔ 入库代码」。

| 项 | kb 现行字（文件:行） | 入库实现（冻结副本:行） | 判 |
|---|---|---|---|
| 不自动推环 | 「不自动推环」（`.claude/kb/decisions/16-发布语义.md:37`） | 非测试代码里点名推环入口的只有 5 行，都是注释或字符串：`crates/singlefs-core/src/mounted_session.rs` 的 221、234、321 行，`crates/singlefs-core/src/mount.rs:5905` 与入口定义 690 行；非测试的 `.push_the_root_ring_…(` 调用 0 处（命令与输出见下） | 同义 |
| 只在显式调用时推 | 「推环入口 MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states 只在用户显式调用时发 F 不变的空发布，直到 R 那一槽被写掉」（`.claude/kb/decisions/16-发布语义.md:37`） | 入口 `pub fn`（`crates/singlefs-core/src/mounted_session.rs:690`）；发的是 `empty_publish_plan_after_file_version`，F 不变（见 K2-3，`FSame`）；停在 R 那一槽被写掉（末尾 `assert!` 核 C 记着的槽都已摘净） | 同义 |
| 带出路的错 | 「准入推抬 F 停在上限、C 那一圈这一刻还隔离着槽时（按整圈判，分不出宽护与 C 里别的几份）」（`.claude/kb/decisions/16-发布语义.md:37`） | 三个条件：停因是 `FloorAlreadyAtTheCeiling`（`crates/singlefs-core/src/mounted_session.rs:291`）、C 有记着落点的槽（`root_ring_push_target`，251 行）、盖住的槽里隔离数不全为 0（303 行）。字段 `ring_slot_of_r`、`checkpoint_txg_of_r`、`empty_publishes_until_the_ring_slot_of_r_is_written`、`isolated_slots_on_each_device`、`ways_out()` 齐 | 同义（字面）；因果上过宽，见 K1-1 |
| 两条出路 | 「两条出路（正常卸载之后再可写挂载；调推环入口）」（`.claude/kb/decisions/16-发布语义.md:37`） | `ways_out` 返回两句，次序卸载重挂、推环（`crates/singlefs-core/src/mounted_session.rs:211` 起；209 行注释写「按这个次序」） | 同义；出路是否走得通见 K1-2 |
| 调用即确认 | 「调用即确认回退候选集掉到少于」（`.claude/kb/decisions/16-发布语义.md:37`） | 入口没有确认参数，调用本身就是确认；`ways_out` 出路二写明「回退候选集掉到少于 4 个不同状态」；文字里写「整环写掉」对次数少于一环的推法偏重，见 K2-3 末条 | 同义；文字一处偏重 |
| I-7.4 | 「近 K 代块未被复用」（`.claude/kb/invariants.md:58`） | K1 到 K3 的每段随机历史每步之后池级 checker 判 I-7.4 与 I-4.8（探针的 `first_red`）：1020 段加推环中 322 个镜像，0 例红；漏补对照有牙（K3-1） | 没打中 |
| 第 29 行 | 「回退候选集保留最近 4 个可退到的不同状态」（`.claude/kb/decisions/16-发布语义.md:29`） | 只在显式推环时破（无自动路径）；F、文件内容推前后不变（`FSame`） | 同义 |
| D28 第九项 | 「只除去分配位置着的槽（含已释放而还在 defer 窗口里的」（`.claude/kb/decisions/28-挂载期承诺量.md:27`） | `isolate_the_slots_not_allocated` 只隔离分配位没置着的槽（`crates/singlefs-core/src/allocator.rs:598`）；被抛弃根那一圈与根槽读坏那一圈走它（`crates/singlefs-core/src/mount.rs:1607` 起） | 同义 |
| D28 第九项补隔离 | 「这次挂载里回收它们时分配器照还没解开的那一份补隔离，有两条路：挂载内记根时」（`.claude/kb/decisions/28-挂载期承诺量.md:27`） | 两条路的实现：`record_root_written_by_this_process`（`crates/singlefs-core/src/allocator.rs:2333`）与 `reclaim_released_up_to_isolating_what_the_isolation_still_protects`（2135 行）；抬 F 那一串的三处回收（`crates/singlefs-core/src/mount.rs` 4534、4552、4561 行）都走后者（K3-1） | 同义 |
| C762 那一支 | 「抬 F 那一串读盘得来、根环表没记着的被抛弃根，补的隔离写掉它那一槽也不放，回收时补不上、照旧整段隔离」（`.claude/kb/decisions/28-挂载期承诺量.md:27`） | `(RecordedInTheRootRingTable…, None) => false` 时仍调 `isolate_abandoned`（`crates/singlefs-core/src/mount.rs:1651`） | 同义（已知留着） |

```
$ grep -rnF '.push_the_root_ring_to_the_ring_slot_of_r' crates --include=*.rs | grep -v 'tests/' | wc -l
0
$ grep -rnF 'push_the_root_ring_to_the_ring_slot_of_r' crates --include=*.rs | grep -c 'tests/'
5
```

用户原话「显式推环命令加带出路的错」在 `.claude/kb/decisions-history.md:7403` 的用户原话一节。推翻「不自动推环」的现象：非测试代码里出现 `.push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(` 的调用。

## 2. K1 第十六块 带出路的错

### K1-1 报「被宽护卡住」的判据与三个量

做的事：随机历史（单元区 384 槽的两块盘，前缀覆盖写 4 到 160 次，R 取覆盖写 / 建 inode / 顺序写 / 回退，故障取 R 的账、谱系片、全部单元、根槽读回全零，动作串取自 `o n s W N S G H P F X K b`），每次准入被拒就记一条 `NS[停因;宽护;hold;isoC;needs]`，并独立重算。日志在模型目录 `logs/sweeps/`、`logs/final/`，汇总命令与原样输出：

```
$ python3 research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/scripts/count_reports.py
1676 ('384', 'NotWide')
365 ('384', 'Stuck')
39 ('384', 'Stuck后自动推环', 'retryNS')
226 ('384', 'Stuck后自动推环', 'retryOk')
94 ('非384', 'NotWide')
165 ('非384', 'Stuck')
21 ('非384', 'Stuck后自动推环', 'retryNS')
141 ('非384', 'Stuck后自动推环', 'retryOk')
384 推后仍被拒的来源 {'ff-stuck-then-push-still-refused': 1, 'f20': 15, 'f40': 8, 'f60': 15}
非384 推后仍被拒的来源 {'i256-small-geometry': 20, 'g256': 1}
$ python3 research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/scripts/count_notwide.py
0 个
```

- 算得对：384 槽盘上报「被宽护卡住」365 次（上面 `('384', 'Stuck')`），探针独立重算的「要推几次」（逐次发布数到每一槽被写到）、「挡着的槽数」（C 全部落点盖住的槽里隔离位置着的个数）、R 的 txg（对录下来的 R）三个量，打标记（`!needs`、`!iso`、`rNe`）的 0 次（上面输出没有 `flag` 行）。推环真发的次数等于独立数出的次数：`J` 注入多槽 C 的 6 种偏移里（`logs/final/jj-inject-multislot.log`），独立数出的 needs 依次是 3、9、17、3、23、22，推环发的依次是 pushed3、pushed9、pushed17、pushed3、pushed23、pushed22。
- 卡在宽护上却报成别的：日志里「报 NotWhatStopsTheAdmission、而 C 有记着落点又隔离着槽」0 次（`count_notwide.py` 输出 `0 个`）。这只说我造出来的历史里没有；「C 有记着落点而停因是预算用完 `PublishesPerAdmissionWouldBeExceeded` 或抬 F 被空间拒 `FloorRaiseRefusedForSpace`」这一形我一次都没造出来（C 在时第一次被拒的停因全是 `Ceil`），记「没造出来」，不记「没打中」。
- 不是宽护卡住却报成它（打中，因果上）：判据只看三样——停因是 F 已在上限、C 有记着落点、盖住的槽里隔离数不全为 0（隔离数那一句在 `crates/singlefs-core/src/mounted_session.rs:303`：「.all(|(_, isolated)| *isolated == 0)」），不看放开之后写得下。造法见 `logs/final/ff-stuck-then-push-still-refused.log`：前缀覆盖写 12 次、R 是覆盖写、R 的账读回全零；动作 `H` 是顺序写，文件每次比上次大 20 个数据单元。同一段历史里「没有故障」那一条（C 为空）在同一步（第 3 次 `H`）同样被拒，停因 `Ceil(15,15)`，是真的空间不够：

```
$ grep -h DIAG research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/ff-stuck-then-push-still-refused.log | sed -E 's/DIAG label=ff prefix=12 //' | cut -c1-260
test s2_sweep ... r=o faults=racct actions=HHHH step=2H NS[Ceil(14,14);S(r=Some(16),n=15,iso=24,rEq);hold=1;isoC=24;needs=Some(15)]|pushed15|stillIso=0/24|retryNS[Pub(6+3);N;hold=0;isoC=0;needs=None]
r=o faults=racct actions=HHHH step=3H NS[Pub(6+3);N;hold=0;isoC=0;needs=None]|pushRef:Nothing|stillIso=0/0|retryNS[Pub(6+3);N;hold=0;isoC=0;needs=None]
r=o faults=none actions=HHHH step=2H NS[Ceil(15,15);N;hold=0;isoC=0;needs=None]|pushRef:Nothing|stillIso=0/0|retryNS[Ceil(15,15);N;hold=0;isoC=0;needs=None]
r=o faults=none actions=HHHH step=3H NS[Ceil(15,15);N;hold=0;isoC=0;needs=None]|pushRef:Nothing|stillIso=0/0|retryNS[Ceil(15,15);N;hold=0;isoC=0;needs=None]
```

  C 隔离 24 个槽，报 `S(r=Some(16),n=15,iso=24)`；按出路二推了 15 次，24 个槽放出来（`stillIso=0/24`），重试仍被拒。这一形在 `logs/sweeps/f20.log`、`f40.log`、`f60.log`（H 步长 20、40、60）里共 38 行，加 `ff` 复跑 1 行，合 39 行，就是上面 `('384', 'Stuck后自动推环', 'retryNS')` 那一行。条款字面（「准入推抬 F 停在上限、C 那一圈这一刻还隔离着槽时（按整圈判，分不出宽护与 C 里别的几份）」，`.claude/kb/decisions/16-发布语义.md:37`）与实现同义，所以这一条不算打穿条款；算「判据只看 C 在不在，不看放开之后写不写得下」：用户照出路二调推环，付 15 次空发布与回退候选，写入照旧被拒。

推翻 K1-1 前三句的现象：任何一次 `Stuck` 报告里 `!needs`、`!iso`、`rNe` 任一出现；或造出 C 有记着落点又隔离着槽、停因是 `Ceil` 却报 `NotWhatStopsTheAdmission`。推翻最后一句的现象：同一历史里放开宽护之后写得下，即 `ff` 例里 `retryNS` 变成 `retryOk`。

### K1-2 两条出路走不走得通

出路二（推环）在 384 槽盘上，`Stuck后自动推环` 共 265 次，其中重试写得下 226 次、推后仍被拒 39 次（上面 K1-1 那一形）；推后 C 记着的槽全摘净（`stillIso` 无一例大于 0，`count_reports.py` 的 `推后仍隔离` 行没有出现）。

出路一（卸载再可写挂载）分两种成因，同一段历史（前缀 20 次覆盖写、回退）各跑一次；`X` 动作 = `mount::unmount` 做成后再可写挂载。读回全零只在第一次挂载那一刻（`j0`）与一直在（`j1`）：

```
$ grep -h DIAG research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/j0-unmount-remount-fault-lifted.log | sed -E 's/DIAG label=[a-z0-9]+ prefix=[0-9]+ //; s/actions=[A-Za-z]+ //' | cut -c1-210
test s2_sweep ... r=b6;n;o faults=racct,rlin step=0o NS[Ceil(20,20);S(r=Some(24),n=20,iso=440,rEq);hold=1;isoC=440;needs=Some(20)]
r=b6;n;o faults=racct,rlin step=1X X:remount-ok:hold=0:iso=38
r=b6;n;o faults=rall step=0o NS[Ceil(20,20);S(r=Some(24),n=20,iso=440,rEq);hold=1;isoC=440;needs=Some(20)]
r=b6;n;o faults=rall step=1X X:remount-ok:hold=0:iso=38
r=b6;n;o faults=rroot step=1X X:remount-ok:hold=0:iso=0
r=b6;n;o faults=racct step=1X X:remount-ok:hold=0:iso=38
$ grep -h DIAG research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/j1-unmount-remount-fault-persists.log | sed -E ... | cut -c1-210
test s2_sweep ... r=b6;n;o faults=racct,rlin step=0o NS[Ceil(20,20);S(r=Some(24),n=20,iso=440,rEq);hold=1;isoC=440;needs=Some(20)]
r=b6;n;o faults=racct,rlin step=1X X:remount-refused:NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandoned
r=b6;n;o faults=rall step=0o NS[Ceil(20,20);S(r=Some(24),n=20,iso=440,rEq);hold=1;isoC=440;needs=Some(20)]
r=b6;n;o faults=rall step=1X X:remount-refused:NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandoned
r=b6;n;o faults=rroot step=1X X:remount-ok:hold=0:iso=0
r=b6;n;o faults=racct step=1X X:remount-refused:NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandoned
```

K1-2 判：

- 出路一的话与实际同义。文字是「R 的账再挂时仍读不出就照旧拒可写（C694 的 B）」（`crates/singlefs-core/src/mounted_session.rs:218`）。`j0`（读回全零只在第一次挂载那一刻）里，卸载再挂 4 次都做成，被宽护卡住的两例再挂后 `hold=0`（宽护不再出现）、`iso=38`（只护 R 自己账里引用的落点）；`j1`（读回全零一直在）里 4 次再挂 3 次被拒，成员是 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandoned…`，就是文字里的「照旧拒可写」，没有第三种结局。
- 出路的次序有风险，条款没错：`j1` 里每一次都是卸载先做成、再挂才被拒，会话已经没了，出路二（推环）从此调不到。被宽护卡住的成因本来就是挂载那一刻 R 的账读不出；这个读不出若是盘上永久坏，出路一等于先丢掉会话再被拒挂载。文字写了这半句条件，但把它排第一、没说「读不出是永久坏时只剩出路二」。这是措辞与次序，不是同义问题，记「出路写得准、排序对永久读不出那一成因偏险」。
- 出路里的 R：`ways_out` 的 R 取根环表里 R 那一槽记着的有效根（`checkpoint_txg_of_the_valid_root_recorded_on`，`crates/singlefs-core/src/mounted_session.rs:308`）。被宽护卡住的 365 次里 `rNe` 0 次（R 的 txg 都等于录下来的 R）。C 里记着的槽若是「记录写了、根没落」的槽，那一槽记的是更旧的有效根，不是被抛弃的 R，文字「R 按新实例的实例表判成被抛弃」就不准；我把崩在根之前的镜像挂上（`S2_CRASH_BEFORE_ROOT`，R 取覆盖写、建 inode、顺序写、回退，`logs/final/nr-crash-before-root.log`）四种都是 `selected=16 c_r=0`、`hold=0`、推环 `pushRef:Nothing`，C 是空的，没造出这一形，记「没造出来」。

推翻 K1-2 判的现象：出路一再挂的成员是上面两种之外的第三种；或 `j1` 里再挂被拒而文字没点名这一形。

## 3. K2 第十六块 推环入口

### K2-1 预演与真发、盘与分配器与会话是否一致，崩溃落在推环中间

(a) 真发半路块设备错（预演过了、真发失败）：`P_FAIL_AT=n` 让设备在推环开始后第 n 次 `write_at` 报 `InputOutput`，随后 `K`（不卸载直接崩了再挂）。原样输出（`logs/final/k*-push-fails-at-write-*.log`）：

```
$ grep -h DIAG research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/k60-push-fails-at-write-60.log | sed -E 's/DIAG label=k60 prefix=20 //' | cut -c1-200
test s2_sweep ... r=b6 faults=racct,rlin actions=oPKoooKoo step=0o NS[Ceil(20,20);S(r=Some(24),n=20,iso=440,rEq);hold=1;isoC=440;needs=Some(20)]
r=b6 faults=racct,rlin actions=oPKoooKoo step=1P P[cur,rOcc=Some((28, Some(24))),needs=Some(20),hold=1,isoC=440,isoAll=440->440]pushRef:AfterRehearsal4/20:BlockDevice(InputOutput(Custom[n=3,stillIsoC=
r=b6 faults=racct,rlin actions=oPKoooKoo step=2K K:remount-ok:sessionTxg=31:selected=31:hold=0:iso=34
r=b6 faults=racct,rlin actions=oPKoooKoo step=6K K:remount-ok:sessionTxg=37:selected=37:hold=0:iso=34
$ for n in 0 20 60 90; do grep -h 'K:remount' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/k$n-*.log | sed -E "s/.*step=/n=$n /"; done
n=0 2K K:remount-ok:sessionTxg=28:selected=28:hold=0:iso=34
n=0 6K K:remount-ok:sessionTxg=34:selected=34:hold=0:iso=34
n=20 2K K:remount-ok:sessionTxg=29:selected=29:hold=0:iso=34
n=20 6K K:remount-ok:sessionTxg=34:selected=34:hold=0:iso=34
n=60 2K K:remount-ok:sessionTxg=31:selected=31:hold=0:iso=34
n=60 6K K:remount-ok:sessionTxg=37:selected=37:hold=0:iso=34
n=90 2K K:remount-ok:sessionTxg=33:selected=33:hold=0:iso=34
n=90 6K K:remount-ok:sessionTxg=38:selected=38:hold=0:iso=34
```

四个注入点（第 1、2、4、6 次发布处报错，各点的 `AfterRehearsal` 行见各自日志）再挂后选到的根 txg 等于会话 `current` 的 txg（`sessionTxg=selected`），`first_red=-`（I-7.4、I-4.8 不红）。推环出错的一支里 `*current = PoolVersion::of_a_version_of_trees(last_published);` 在 `crates/singlefs-core/src/mounted_session.rs:765`，这一句去掉的变异入库用例全绿（K4 `failNoCurrent`），所以「`current` 停在最后一次落盘的那一版」这一句靠的是我这里的 `K` 对照，不是入库用例。失败之后会话本身冻结，原样输出（`P_FAIL_AT=35`，动作串 `oPooXoo`，`TRACE=1` 打每一步）：

```
$ grep -h DIAG research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/k35-trace-write-failure-freezes-session.log | sed -E 's/DIAG label=k35t prefix=20 r=b6 faults=racct,rlin actions=[A-Za-z]+ //' | cut -c1-190
test s2_sweep ... step=0o NS[Ceil(20,20);S(r=Some(24),n=20,iso=440,rEq);hold=1;isoC=440;needs=Some(20)]
step=1P P[cur,rOcc=Some((28, Some(24))),needs=Some(20),hold=1,isoC=440,isoAll=440->440]pushRef:AfterRehearsal3/20:BlockDevice(InputOutput(Custom[n=2,stillIsoC=440,hold_after=1,ab_left=0,FSam
step=2o Publish{cause:PublishFrozenAfterAWriteFailureIsNotResentY
step=3o Publish{cause:PublishFrozenAfterAWriteFailureIsNotResentY
step=4X unmountErr:RaiseFloorSequencePublishFailed(PublishSequenceFailed{cause:PublishFrozenAfterAWriteFailureIsNotResentYet{checkpoint_txg:CheckpointTxg(31),instance:InstanceGeneration(2)},
step=5o Publish{cause:PublishFrozenAfterAWriteFailureIsNotResentY
step=6o Publish{cause:PublishFrozenAfterAWriteFailureIsNotResentY
```

后面每次写与卸载都报 `PublishFrozenAfterAWriteFailureIsNotResentYet`：这是发布路径原有的失败处置（D23 已定项 14 的「失败的那次冻结，下一次发布之前逐字节原样重发」，`crates/singlefs-core/src/transaction.rs:1272` 起），不是这一批带进来的；推环入口失败后既不重发也不清冻结，用户只能退出重挂。

(b) 预演拒绝：入库用例 `pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write`（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:4038`）断言盘、分配器 `{:?}`、现行根都不变；我套的 `noRehearsedCarry`（预演不接力）让它与另外 3 条红。预演用 `allocator.clone()` 的拷贝（「let mut copy = allocator.clone();」，`crates/singlefs-core/src/mounted_session.rs:726`）。「空间被别的抢走」那一支：会话独占分配器，我没造出预演过而真发因空间被拒的分叉，只有块设备错这一种分叉，记「没造出来」。

(c) 崩溃落在推环中间：`P_CRASH` 在推环发布的每条根写（`RootRecordFua`）之前与之后各取一个镜像，挂可写、再覆盖写 4 次、每步判 I-7.4 与 I-4.8（不是崩溃枚举，只是每条根的前后两个点）。样本数与阳性对照（每个日志里每段历史一组 `slots=…` 与 `crash_samples=… crash_bad=…`）：

```
$ grep -h 'S2 label' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/m5-crash-samples-b6.log | grep -o 'slots=[0-9]*\|crash_samples=[0-9]* crash_bad=[0-9]*' | paste -sd' '
slots=384 crash_samples=40 crash_bad=0 slots=384 crash_samples=40 crash_bad=0
$ grep -h 'S2 label' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/m6-crash-samples-n.log | grep -o 'slots=[0-9]*\|crash_samples=[0-9]* crash_bad=[0-9]*' | paste -sd' '
slots=384 crash_samples=40 crash_bad=0 slots=384 crash_samples=42 crash_bad=0
$ grep -h 'S2 label' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/m7-crash-samples-o-s.log | grep -o 'slots=[0-9]*\|crash_samples=[0-9]* crash_bad=[0-9]*' | paste -sd' '
slots=384 crash_samples=40 crash_bad=0 slots=384 crash_samples=40 crash_bad=0 slots=384 crash_samples=40 crash_bad=0 slots=384 crash_samples=40 crash_bad=0
$ grep -h 'S2 label' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/base-256-crash.log | grep -o 'slots=[0-9]*\|crash_samples=[0-9]* crash_bad=[0-9]*' | paste -sd' '
slots=256 crash_samples=38 crash_bad=0 slots=256 crash_samples=38 crash_bad=0
$ grep -h 'S2 label' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/ctl-early-release-256.log | grep -o 'slots=[0-9]*\|crash_samples=[0-9]* crash_bad=[0-9]*' | paste -sd' '
slots=256 crash_samples=38 crash_bad=27 slots=256 crash_samples=38 crash_bad=0
$ grep -h 'S2 label' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/sweeps/ctl3-control-early-release-384.log | grep -o 'slots=[0-9]*\|crash_samples=[0-9]* crash_bad=[0-9]*' | paste -sd' '
slots=384 crash_samples=40 crash_bad=0
```

384 槽盘 322 个样本（m5 80、m6 82、m7 160）0 个红。阳性对照：把推环入口改成「一开始就把 C 记着的槽的隔离位全清掉」（`src/control-early-release.diff.txt`，只在 `tree-a` 的编译副本里，不进库），256 槽盘前缀 20 次那一段历史 38 个样本红 27 个（成员 `imageRed:I-7.4`，指到被抛弃的 R），前缀 30 次那一段 38 个红 0 个；同两段不改的 `base-256-crash` 各 38 个红 0 个。同一个对照放在 384 槽盘上红不出来（最后一行 `ctl3-control-early-release-384`：40 个样本红 0 个，空闲槽多，空发布挑不到被早放的槽），所以 384 槽盘上的「0 个红」只证到「256 槽盘上这条探针有牙」，384 槽盘自己没证过。

推翻 K2-1 的现象：失败后再挂选到的 txg 不等于会话的 txg，或任一 `crash_bad` 非 0，或 `first_red` 非 `-`。

### K2-2 推环会不会提前盖掉别的被抛弃根或回退候选，让别的隔离提前解开

造法：`X`（卸载再挂，读回全零已撤）之后 R 在根环里是一条被抛弃根（环里记成被抛弃根的槽 1 个，护着它账里的 28 个槽）；再用 `J` 往根环表注入一个 C 槽（10 个槽，隔离位置上），偏移 k 是「从现行 txg 起第 k 次发布写到的那一槽」，k 从 1 扫到 23（环长 24）；然后 `P` 显式推环，再覆盖写 3 次，每步判池级 checker：

```
$ python3 research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/scripts/summ_jx.py
k needs 注入的C槽数 推前全部隔离->推后 推了几次 推后C里还隔离几槽 环里剩被抛弃根 F 池级first_red
1 1 10 38->28 1 0 1 FSame -
2 2 10 38->28 2 0 1 FSame -
3 3 10 38->28 3 0 1 FSame -
4 4 10 38->28 4 0 1 FSame -
5 5 10 38->28 5 0 1 FSame -
6 6 10 38->28 6 0 1 FSame -
7 7 10 38->28 7 0 1 FSame -
8 8 10 38->28 8 0 1 FSame -
9 9 10 38->28 9 0 1 FSame -
10 10 10 38->28 10 0 1 FSame -
11 11 10 38->28 11 0 1 FSame -
12 12 10 38->28 12 0 1 FSame -
13 13 10 38->28 13 0 1 FSame -
14 14 10 38->28 14 0 1 FSame -
15 15 10 38->0 15 0 0 FSame -
16 16 10 38->0 16 0 0 FSame -
17 17 10 38->0 17 0 0 FSame -
18 18 10 38->0 18 0 0 FSame -
19 19 10 38->0 19 0 0 FSame -
20 20 10 38->0 20 0 0 FSame -
21 21 10 38->0 21 0 0 FSame -
22 22 10 38->0 22 0 0 FSame -
23 23 10 38->0 23 0 0 FSame -
```

- 推到 R 那一槽之前，被抛弃根的 28 个槽照旧隔离着（k 在 1 到 14：全部隔离 38→28，环里还剩 1 条被抛弃根）；推过被抛弃根自己那一槽（k 从 15 起）才放（38→0，被抛弃根离开环）。注入的 C 槽每次都摘净（`推后C里还隔离几槽` 一列全 0），F 不变（`FSame`），池级 checker `first_red=-` 23 行全是。没有提前盖掉别的被抛弃根的隔离。
- 多槽 C（注入的 `J_OFFSETS` 取 3,9 / 5,30,41 / 1,2,3 / 20,46，`logs/final/jj-inject-multislot.log`）：独立数出的次数取各槽里最晚轮到的那个（3,9 → 9；5,30,41 → 17，因为 30 与 41 在环长 24 里等于 6 与 17；1,2,3 → 3；20,46 → 22），推环发的次数与它相等，R 取最后轮到的槽。入库用例没有 C 多槽的形（我没逐条数每条用例的 C 有几个槽，依据是下面这个变异全绿），把 `.max()?;` 改成 `.min()?;`（`crates/singlefs-core/src/mounted_session.rs:274`：「.max()?;」）入库用例全绿（K4 `maxmin`），所以多槽这一支入库用例没盯，我这里的注入探针盯了。注入的 C 是往内存表里塞的，盘上没有对应记录，所以只验了推环的算与摘，没验挂载读出多槽 C 的那一半：真实历史里 C 有 2 个以上槽的形我一个都没造出来（日志里所有「被宽护卡住」报告的 `hold` 恒为 1）。
- 回退候选被盖掉：用户已定（「调用即确认」），不算。
- 注入级的一形（可达性没证）：C 的落点与环里被抛弃根的落点重叠时（`J_OVERLAP`，C 的落点取被抛弃根的落点），推环摘净 C，但重叠的槽仍被被抛弃根护着：

```
$ grep -h 'DIAG' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/jo-overlap-with-abandoned.log | grep -o 'P\[cur,rOcc=[^]]*\]pushed[0-9]*\[n=[0-9]*,stillIsoC=[0-9]*' 
P[cur,rOcc=Some((25, Some(4))),needs=Some(3),hold=1,isoC=28,isoAll=28->28]pushed3[n=3,stillIsoC=28
P[cur,rOcc=Some((25, Some(15))),needs=Some(14),hold=1,isoC=28,isoAll=28->28]pushed14[n=14,stillIsoC=28
P[cur,rOcc=Some((25, Some(21))),needs=Some(20),hold=1,isoC=28,isoAll=28->0]pushed20[n=20,stillIsoC=0
```

  k=3、14 时 C 与被抛弃根重叠的 28 个槽推完一个都没放（`stillIsoC=28`），k=20 越过被抛弃根那一槽才放。此时 `ways_out` 出路二写的「隔离的 {} 个槽放出来」（`crates/singlefs-core/src/mounted_session.rs:221`）取的是 `isolated_slots_on_each_device` 之和，把重叠的槽也算进「放出来」，数字偏大；准入也不会因此恢复。这一形要 C 的落点与环里被抛弃根的落点重叠，我没造出自然走法（被抛弃根与宽护同时出现要两次崩溃加两次挂载，我的探针只有一次崩溃镜像），记「注入级」。
- 同样注入级：C 记着落点而没有一个槽隔离着（`J_NOISO`）时 `NothingHeldUntilThisProcessWritesARingSlot` 不拦，推环照发 9 次空发布一个槽没放：

```
$ grep -h DIAG research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/jn-inject-without-isolation.log | grep -o 'P\[cur,rOcc=[^]]*\]pushed[0-9]*\[n=[0-9]*,stillIsoC=[0-9]*'
P[cur,rOcc=Some((20, Some(5))),needs=Some(9),hold=1,isoC=0,isoAll=0->0]pushed9[n=9,stillIsoC=0
```

  拒的条件在 `crates/singlefs-core/src/mounted_session.rs:339` 注释里写「这个进程写掉哪一槽都放不出东西，不推」，实现只判有没有记着落点的槽（`.filter(|(_, placements)| !placements.is_empty())`，`crates/singlefs-core/src/allocator.rs:1275`），不判有没有隔离着的槽。用户显式调用，报告只在隔离数大于 0 时劝推，所以这一形只在用户自己调时碰到。

推翻 K2-2 的现象：`summ_jx.py` 表里 k 在 1 到 14 的某一行推后全部隔离数小于 28，或 `first_red` 非 `-`。

### K2-3 推完之后：R 那一槽写掉、宽护解开、准入恢复，有没有推完仍卡

- 384 槽盘：`Stuck后自动推环` 265 次，写得下 226 次；39 次推完仍被拒，全是 K1-1 那一形（真空间不够，H 大写，推前推后都拒）。推错槽 0 次：每次推环末尾的 `assert!`（C 记着的槽都已摘净，`crates/singlefs-core/src/mounted_session.rs` 推环入口末尾）没触发过 panic，独立数出的次数与实发次数全等。F 被抬到不该到的地方 0 次：`fs-push-keeps-floor-and-data.log` 里推环前后 `rollback_floor` 与数据落点相同：

```
$ grep -h 'FSame' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/fs-push-keeps-floor-and-data.log | grep -o 'FSame,txg+[0-9]*\|FDIFF'
FSame,txg+20
```

- 其他单元区槽数（正文第二节写明「384 槽盘之外的几何没在推环入口上量」，所以下面不算打中，只记线索）：

```
$ python3 research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/scripts/count_stuck_by_file.py | grep -E '^(i256|i288|g256|g512|g768|h2|h3|h4)'
i256-small-geometry: 推后写得下 0，推后仍被拒 20
i288-small-geometry: 推后写得下 24，推后仍被拒 0
g256: 推后写得下 0，推后仍被拒 1
h2: 推后写得下 36，推后仍被拒 0
h3: 推后写得下 42，推后仍被拒 0
h4: 推后写得下 39，推后仍被拒 0
```

  `i256`、`g256` 是 256 槽盘，`i288` 288 槽，`h3` 320 槽，`h4` 448 槽，`h2` 512 槽（`g512`、`g768` 没出 Stuck）。256 槽盘上 Stuck 共 21 次、推完重试仍被拒 21 次（重试停因 `Pub(…)`，预算用完，C 已空）；288 槽及以上全部写得下。256 槽盘上推环自己的 18 到 19 次空发布占去多少空间我没拆开量，原因不明。
- 出路二文字「根环里更早的回退候选会被这一串整环写掉」（`crates/singlefs-core/src/mounted_session.rs:221`）对次数少于一环的推法偏重：注入偏移 3 时只推 3 次（`jj` 第一行 `pushed3`），环长是 24。用户定的是「掉到少于 4 个不同状态」，文字多说了「整环」。

推翻 K2-3 的现象：384 槽盘上某次 `Stuck` 之后推环、重试被拒，而同一历史无 C 的对照里写得下（即 `ff` 例反过来）。

补一条 K2-3 的核对（独立数出的次数与实发次数逐条比，自动推环行与显式 `P` 行，不含推环里注入了块设备错的 `k*` 日志）：

```
$ python3 research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/scripts/count_needs_vs_pushed.py
45 ('显式P行', True)
427 ('自动推环行', True)
```

## 4. K3 第十六块 被抛弃根那一圈与根槽读坏那一圈只隔离分配位没置着的槽

### K3-1 回收时补得上那一支：每条回收路是不是都补上了

读码：被抛弃根那一圈靠 `reclaim_isolates_what_this_root_references`（「let reclaim_isolates_what_this_root_references = match (」，`crates/singlefs-core/src/mount.rs:1607`）分两支，补得上的一支改走 `isolate_the_slots_not_allocated`（`crates/singlefs-core/src/allocator.rs:598`），分配位置着的槽挂载那一刻不隔离。回收出来再补的路有三条：挂载内记根 `record_root_written_by_this_process`（`crates/singlefs-core/src/allocator.rs:2333`）；抬 F 那一串（准入一截、卸载两截）的回收 `reclaim_released_up_to_isolating_what_the_isolation_still_protects`（`crates/singlefs-core/src/allocator.rs:2135`），它在 `crates/singlefs-core/src/mount.rs` 的 4534、4552、4561 三处调用；挂载时的回收（`crates/singlefs-core/src/mount.rs:3021`）在隔离之前，回收出来的槽此刻已是分配位没置着，随后被隔离。补隔离看的保护集是被抛弃根记着的落点加 C 还没解开的落点（`crates/singlefs-core/src/allocator.rs:1306` 的 `placements_the_isolation_still_protects`）。

变异：每条路各去掉补隔离一次，入库用例文件的结果（原样，来自 `logs/mut-2.log`、`mut-3.log`、`mut-4.log`；`mut-4.log` 列全了红的用例名）：

```
$ grep -hE '^(recordRootNoReisolate|raiseReclaimNoReisolate|protectOnlyAbandoned|protectOnlyC|site1HeldAdmission|diskReadWholeIso|badSlotLoopOld):' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/mut-2.log research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/mut-3.log | cut -c1-120
recordRootNoReisolate: rc=101 红 1 条 defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_ | ['test 
raiseReclaimNoReisolate: rc=101 红 16 条 revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_r sessio
protectOnlyAbandoned: rc=101 红 13 条 revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_r a_rollbac
protectOnlyC: rc=101 红 5 条 session_writes_after_the_account_of_an_abandoned_root_turns_unreadable session_writes_aft
site1HeldAdmission: rc=101 红 12 条 revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_r session_wri
diskReadWholeIso: rc=101 红 2 条 defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_ defer_slots_an
badSlotLoopOld: rc=101 红 1 条 defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are | ['test result:
$ grep -hE '^(site2ImmediatelyUnmount|site3HeldUnmount)' -A4 research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/mut-4.log | sed -E 's/ \| .*//' | cut -c1-200
site2ImmediatelyUnmount: rc=101 红 4 条 unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero unmount_after_the_a
site3HeldUnmount: rc=101 红 1 条 unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero
accessorZero: rc=0 红 0 条 
```

随机历史：1260 段（前面 K1 的那批，含 1020 段每步判池级 checker、240 段不判），有没有一槽在 R 还在环里时被发出去：

```
$ sed -n 1,3p research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/summaries.txt | cut -c1-200
## summ2.py over logs/sweeps/{a1,a2,a3,b1,b2,b3,c1,c2,c3,d1,d2,d3,d4}.log (384 slots, random action strings)
histories 1260 first_red 0 c_unblocked_grew 0 c_handed_out 0 pu_pos 0 crash_samples 0 crash_bad 0
skipped {'根槽写3次': 2}
$ sed -n '/base-raise-reclaim-config-q3/,$p' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/summaries.txt | cut -c1-200
## summ2.py over logs/final/base-raise-reclaim-config-q3.log (unmutated, control config)
histories 96 first_red 0 c_unblocked_grew 0 c_handed_out 0 pu_pos 0 crash_samples 0 crash_bad 0
skipped {'根槽写3次': 2}
19 ('push:pushedN', 'stillIso0', 'retryOk')
## summ2.py over logs/final/ctl-raise-reclaim-no-reisolate.log (mutant)
histories 96 first_red 39 c_unblocked_grew 54 c_handed_out 61 pu_pos 77 crash_samples 0 crash_bad 0
skipped {'根槽写3次': 2}
```

`c_handed_out 0`（C 盖住的槽被这个进程新发出去）、`pu_pos 0`（保护集里没挡的槽）、`first_red 0`。阳性对照：把抬 F 那一串的回收改成不补隔离（`raiseReclaimNoReisolate`）、同一批 96 段历史（上面最后两个块，前一个不改，后一个改）`first_red` 从 0 变 39、`c_handed_out` 从 0 变 61，所以这条探针对「回收时漏补」有牙。

三个小结：

- 补得上的一支，每条回收路的去补变异入库用例都红，且各路都至少有一条专属用例（挂载内记根只有 1 条：`defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed`；卸载「当场放开」红 4 条、「扣到生效」只有 `unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero` 1 条），没找到哪一条回收路没人盯。
- 没造出 R 还在环里那几槽被发出去的形：K3-1 记「没打中」（一类探针 + 变异对照，不是读码）。
- 已知旧形：同起点去重（C 那一圈按起点槽去重，同起点较宽的那份少罩一槽）在 320 槽盘上能看到，挂载那一刻就有，不是这一批带进来的，上一轮判决已收（`research/prompts/m3-batch10z-no-code-r1-main-verification.md:28`：「K1-2 按槽跳过与起点槽豁免」）：

```
$ grep -h PUDIAG research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/pu-320-c-unblocked-at-mount.log | cut -c1-330
PUDIAG label=pu1 actions=oPoo step=0o pu=2 outcome=NS[Ceil(22,22);S(r=Some(26),n=19,iso=346,rEq);hold=1;isoC=346;needs=Some(19)] detail=dev0:slot1515:inC=true:recs=["(slot=1514,span=2,gen=8,rel=true)"]:placement=(1514,2) ; dev1:slot1515:inC=true:recs=["(slot=1514,span=2,gen=8,rel=true)"]:placement=(1514,2)
```

  384 槽盘上这一形是 0（上面 `c_unblocked_grew 0`）。

推翻 K3-1 的现象：任一随机历史里 `c_handed_out`、`pu_pos` 或 `first_red` 非 0，或抬 F 那一串的某处回收调用（`mount.rs` 4534、4552、4561 之外）出现不补隔离的回收。

### K3-2 补不上那一支（照旧整段隔离）

判据：`reclaim_isolates_what_this_root_references` 的 match 只有两格为真——挂载那一遍读出的（`ReadFromTheirAccounts`，落点随后整串记进根环表）与抬 F 那一串在表里找得到这条根的；表里没记着这条根（`None`）为假，照旧 `isolate_abandoned` 整段隔离（`crates/singlefs-core/src/mount.rs:1651`），与 D28 第九项括注里 C762 那一形同义。

- 判据对不对：真格的前提是「落点随后整串记进表里」。读码没找到一条挂载那一遍读出而没进表的被抛弃根；也没找到该整段隔离却被跳过的：跳过的只有被候选根引用的槽与已隔离过的槽（`crates/singlefs-core/src/mount.rs:1642`：「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」），与改之前同。
- 这一支入库用例不盯：把为假的一格改成真（`notRecordedAsNotAllocated`）入库 35 条全绿；把真格改成假（`diskReadWholeIso`）红 2 条。我用改成真的二进制跑了 192 段历史，没有红：

```
$ cat research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/final/summaries-k3-2.txt
$ python3 scripts/summ2.py logs/sweeps/mutant-none-arm-true-q1.log logs/sweeps/mutant-none-arm-true-q2.log | head -3
histories 192 first_red 0 c_unblocked_grew 0 c_handed_out 0 pu_pos 0 crash_samples 0 crash_bad 0
skipped {'根槽写3次': 4}
37 ('push:pushedN', 'stillIso0', 'retryOk')
```

  所以这一支的「改成不整段隔离会不会漏护」我没造出反例，也没有入库用例盯它；记「入库无用例、危害没证」，不记打中。C762 本身（整段隔离与「已分配」双扣）是用户已知留着的，不报。

推翻 K3-2 的现象：造出一段历史，让抬 F 那一串从盘上读来、表里没记着的被抛弃根，其引用的 defer 槽回收后被发出去（`first_red` 非 0）而同一历史在改前不红。

## 5. K4 判别力：新加的用例与变异能不能拦住改回去

### K4-1 实现员的变异行逐行重跑

把改动全文里各块的变异行（第十六块追加 13 行、替换 3 行，第十七块 2 行，第十九块 1 行）逐行套到 `tree-m`（冻结副本的拷贝），按各行自己点名的命令跑（release，`cargo test --release --offline <命令>`）。每行只留「行号、返回码、红几条、结果行」（`scripts/rows.py <起行> <止行>`，行号是 `_m3-batch10z-pqrs-code-r1-diff.md` 里的行；完整一行含变异名与红的用例名，在 `logs/rows-*.log`）：

```
$ cat research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/rows-1.log research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/rows-2.log research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/rows-3.log research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/rows-4.log | sed -E 's/^(行[0-9]+) .*: (rc=[0-9]+ 红 [0-9]+ 条).*(test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed).*/\1 \2 | \3/'
行1504 rc=101 红 4 条 | test result: FAILED. 2 passed; 4 failed
行1505 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1506 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1507 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1508 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1509 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1510 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1511 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1512 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1513 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1514 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1515 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1516 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行2397 rc=101 红 1 条 | test result: FAILED. 215 passed; 1 failed
行2398 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行2710 rc=101 红 1 条 | test result: FAILED. 34 passed; 1 failed
行1522 rc=101 红 1 条 | test result: FAILED. 0 passed; 1 failed
行1523 rc=101 红 2 条 | test result: FAILED. 1 passed; 2 failed
行1524 rc=0 红 0 条 | test result: ok. 35 passed; 0 failed
```

- 第十六块追加 13 行（行 1504 到 1516）：13 行全红。
- 替换 3 行（行 1522 到 1524）：行 1522、1523 红；**行 1524（第 10z 批第九块 K3 loop-span1，根槽这一遍读坏的被抛弃根那一圈把 `placement.span` 换成 1）不红**：用例文件 35 条全绿，加 `--include-ignored` 36 条也全绿（`logs/row24-include-ignored.log`）。这一行的原意是「跨两槽的落点后一槽空闲」；第十六块把这一圈改成 `isolate_the_slots_not_allocated` 之后，推测是那个用例里后一槽不再是空闲的（分配位置着），改变异后行为不变，这一行失去了牙（原因是推测，我没拆开验）。这是 K4 打中：被替换的旧变异里有 1 行已经不分辨「只隔离起点那一槽」。危害我没证（没造出后一槽空闲且根槽读坏的历史），所以它不是 K3 的反例，只是入库没有东西拦「把根槽读坏那一圈的跨度改成 1」。
- 第十七块 2 行：访问器恒交 0（行 2397）红在库内单元测试 `isolating_a_whole_span_over_allocated_slots_counts_each_overlapping_slot_both_as_allocated_and_as_isolated`（`crates/singlefs-core/src/allocator.rs` 测试模块）；同一个变异下 harness 用例文件仍 35 条全绿（`mut-4.log` 的 `accessorZero`），因为那里的断言全是 `== 0`。库内这一条是访问器唯一的阳性对照（改前 `mut-5-accessor-lib-baseline.log` 过，改后 `mut-5-accessor-lib.log` 红）。卸载「当场放开」（行 2398）红 1 条。
- 第十九块 1 行（行 2710）：卸载「扣到生效」红 1 条（`unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero`）。

### K4-2 我自己套的 25 个变异（`scripts/mut.py`，`logs/mut-1.log` 到 `mut-3.log`）

```
$ cat research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/mut-1.log research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/mut-2.log research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/mut-3.log | sed -E 's/ \| .*//' | cut -c1-120
maxmin: rc=0 红 0 条 
noIsoCheck: rc=0 红 0 条 
budgetAsStuck: rc=0 红 0 条 
spaceAsStuck: rc=0 红 0 条 
noRehearsedCarry: rc=101 红 4 条 a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is an_inode_creat
failNoCurrent: rc=0 红 0 条 
slotsWrittenEmpty: rc=0 红 0 条 
textCountZero: rc=0 红 0 条 
textRegionSlot: rc=0 红 0 条 
ringLenMinus: rc=0 红 0 条 
skipDevCheck: rc=0 红 0 条 
isoAllSlots: rc=101 红 3 条 a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is an_inode_creation_a
emptyHeld: rc=0 红 0 条 
diskReadWholeIso: rc=101 红 2 条 defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_ defer_slots_an
notRecordedAsNotAllocated: rc=0 红 0 条 
badSlotLoopOld: rc=101 红 1 条 defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are
txgOfRNone: rc=101 红 3 条 a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is an_inode_creation_af
countsDup: rc=101 红 3 条 a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is an_inode_creation_aft
recordRootNoReisolate: rc=101 红 1 条 defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_
raiseReclaimNoReisolate: rc=101 红 16 条 revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_r sessio
protectOnlyAbandoned: rc=101 红 13 条 revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_r a_rollbac
protectOnlyC: rc=101 红 5 条 session_writes_after_the_account_of_an_abandoned_root_turns_unreadable session_writes_aft
site1HeldAdmission: rc=101 红 12 条 revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_r session_wri
site2ImmediatelyUnmount: rc=101 红 4 条 unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held unmount
site3HeldUnmount: rc=101 红 1 条 unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held
```

红 13 个、活 11 个、等价 1 个（`ringLenMinus`：`1..=` 改 `1..` 之后找不到时取的是 `ring_length_in_txgs`，与找到的同值）。活下来的 11 个就是入库用例不盯的东西：

- `noIsoCheck`（C 有记着落点而一个槽都没隔离时仍报「被宽护卡住」）、`budgetAsStuck`、`spaceAsStuck`（停因是预算用完或抬 F 被空间拒时也报）：K1-1 的「不是宽护卡住却报成它」入库只有 C 为空这一个用例；行 1513 的变异同时去掉「C 有没有记着落点」与「隔离着没有」两个条件所以被它红，单去隔离那一个条件、或把停因放宽，全活。
- `maxmin`（R 取最早的槽而不是最晚的）：入库用例没有 C 多槽的形。
- `emptyHeld`（记着空串的槽也算有记着落点）、`slotsWrittenEmpty`（`RootRingPushed.ring_slots_written` 恒空）。
- `textCountZero`、`textRegionSlot`（出路二文字里的次数、根环槽号）：用例只断言文字里含「卸载」「再可写挂载」「少于 4 个不同状态」。
- `skipDevCheck`（推环入口不核交进来的盘表）、`failNoCurrent`（真发出错那一支不更新 `current`）：这两支正文第二节已写明没有用例。
- `notRecordedAsNotAllocated`（表里没记着的被抛弃根也改走不整段隔离，C762 那一支）：K3-2 已述，入库无用例。

### K4-3 第十八块扫源码用例仍拦不住的写法

把三条 GPU 用例与崩溃放量那条的测试体从冻结副本源码机械抽出（`src/scan-gpu-main.rs.txt`、`src/scan-crash-main.rs.txt`，只把 `include_str!` 换成读文件），喂改过的源码。先核模型对实现员的 9 行变异（改动全文第 2178 到 2186 行）都判红，再喂我手造的写法（`scripts/variants.py`、`scripts/variants_rows.py`，全部输出 `logs/scan-variants.txt`）：

```
$ grep -c 'RED' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/scan-variants.txt; sed -n '/## 九行变异/,/## 崩溃放量/p' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/scan-variants.txt | grep -c ' RED'
22
9
$ grep -v '^第10z批' research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/scan-variants.txt | cut -c1-110 | sed -n '/## 崩溃放量/,$p'
## 崩溃放量扫源码：手造的绕过
control_second_let: RED
control_if_false_wrap: RED
bypass_tuple_let_rebind: GREEN
bypass_let_extra_spaces: GREEN
bypass_nested_fn_item: GREEN
bypass_extra_layer_one_line: GREEN
bypass_extra_layer_other_name: GREEN
```

（第一行两个数是整份输出里 `RED` 的行数与「九行变异」一节里 `RED` 的行数，后者 9，与实现员报的「9 行 prove-red 全抓到」一致；`bypass_nested_fn_item` 只是多一个无用的内嵌函数，不改行为，不算绕过。GPU 那一半的手造写法输出在同一文件前 18 行，见 K5。）

仍拦不住的写法，崩溃放量那条（`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`），名单外三种：

1. 新增一层收叫停但参数不单独占一行（单行签名 `fn an_extra_layer(a: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { … }`）：用例按「某一行去掉空白恰等于叫停参数那一行」找层，找不到这一层，`bypass_extra_layer_one_line` 判绿。
2. 新增一层把参数换名（`stop: &(dyn Fn() -> bool + Sync),`）：`bypass_extra_layer_other_name` 判绿。
3. 长的那一层里钉住那一行之后用 `let (stop_request, _unused) = …` 解构重绑，或写成 `let   stop_request  = …`（`let` 与名字之间多空格）：用例数 `let stop_request ` 起头的行，两种写法数不到，`bypass_tuple_let_rebind`、`bypass_let_extra_spaces` 判绿。我没把这两种写法编进产品代码（扫源码层判绿，类型对不对没验）。

名单里已写的几种（早退、`match` 臂、闭包参数、`#[cfg]` 等）我没重造。GPU 卡池那条（`the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`）我在名单外没找到静默绕过：卡池模块里起叫 `std`、`wgpu` 的 `struct`、`enum`、`trait`、`type` 都会使 `std::panic::catch_unwind` 一类路径编不过（用 rustc 单独编小程序核过，`src/shadow-a.rs.txt` 到 `shadow-e.rs.txt`：`a` 是 `struct`，E0223；`b` 是 `enum`，E0433；`c` 是 `trait`，E0782；`e` 是 `type`，E0223；`d` 是 `const std: u8`，值名字空间，编得过也不遮路径），所以不是静默改行为。

推翻 K4 判的现象：行 1524 的变异在用例文件上红；或活下来的 11 个变异里有哪个被入库用例拦住。

## 6. K5 第十八块只动测试：有没有把产品行为带偏

- 改动只在测试模块里：改动全文第十八块的 5 个 hunk 头都带 `mod tests {`，且行号都在各文件 `#[cfg(test)]` 开头之后：

```
$ sed -n 1527,2173p research/prompts/_m3-batch10z-pqrs-code-r1-diff.md | grep '^@@' | cut -c1-90
@@ -4842,6 +4842,70 @@ mod tests {
@@ -4908,28 +4972,69 @@ mod tests {
@@ -4953,37 +5058,76 @@ mod tests {
@@ -2408,11 +2408,20 @@ mod tests {
@@ -2455,6 +2464,349 @@ mod tests {
$ grep -n '^#\[cfg(test)\]' ~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs ~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs | sed -E 's#.*/src/##'
gpu_unit_checks.rs:1899:#[cfg(test)]
crash_amplification.rs:3574:#[cfg(test)]
```

  `crash_amplification.rs` 的 hunk 从 4842 行起（测试模块 3574 行起），`gpu_unit_checks.rs` 的 hunk 从 2408 行起（测试模块 1899 行起）。产品代码没动。新增行里没有锁、线程、串行段（点名这些词的新增行 1 条，是注释里引的用例名 `…_the_card_threads_and_the_run`，不是代码），所以「只防 OOM、不压并发」这一条没被碰。
- 现行产品代码上扫源码模型判绿（不会因为这一批把已有的合法写法判红）：

```
$ GPU_SRC=~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs bash scripts/run-with-memory-cap.sh 1G ~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet/target-scan/release/scan-gpu
GREEN
$ CRASH_SRC=~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs bash scripts/run-with-memory-cap.sh 1G ~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet/target-scan/release/scan-crash
GREEN
```

- 卡池模块里以后合法的产品写法会被判红的（打中，窄）：下面这些我都用 rustc 单独编过是合法 Rust（`src/k5-legal-card-pool-snippets.rs.txt`，编得过），在卡池模块里写了就让 `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call` 红：

```
$ sed -n 1,19p research/prompts/m3-batch10z-pqrs-code-r1-sonnet-model/logs/scan-variants.txt | grep -E 'K5_|control'
control_mod_wgpu: RED the_card_pool_module_shadows_n
K5_use_as: RED the_card_pool_module_shadows_n
K5_nested_use_braces: RED the_card_pool_module_shadows_n
K5_thread_local_macro: RED the_card_pool_module_shadows_n
K5_const_ne: RED the_card_pool_module_shadows_n
K5_const_ne_ident: RED the_card_pool_module_shadows_n
K5_lifetime_char: GREEN
K5_raw_byte_strings: GREEN
K5_block_comment_nested: GREEN
K5_cfg_test_mod_inside: RED the_card_pool_module_shadows_n
K5_extern_c_block: RED the_card_pool_module_shadows_n
K5_pub_use_self_group: RED the_card_pool_module_shadows_n
K5_use_wildcard_enum: RED the_card_pool_module_shadows_n
```

  红的是：`use … as …` 改名、`use` 里两层花括号、`use std::sync::{self, …}`、`use …::*`、条目这一层的 `thread_local!` 一类宏调用、`#[cfg(test)] mod …`、`extern "C" { … }`；以及 `const TWO_DIFFER: bool = A != B;`——这一句不是故意禁的：用例用「条目层的标识符后面紧跟 `!`」认宏调用，`A != B` 里 `A` 在条目层、后面是 `!=` 的 `!`，被当成宏调用（`K5_const_ne`、`K5_const_ne_ident`）。判绿的：生命周期与字符字面量（`'a`、`'{'`、`'\u{7b}'`、`'é'`）、原始字符串与字节字符串里的 `mod`、嵌套块注释里的 `mod std {`。这些是对以后在卡池模块里写产品代码的限制，今天的产品代码不受影响（上面判绿）。「逼出别扭的产品代码」：要换名只能写全路径，要用宏只能放进函数体里。
- 崩溃放量那条里 `brace_depth_at_the_start_of_each_line` 遇到原始字符串直接 `assert!` 红（改动全文第十八块 `crash_amplification.rs` 新增的那个函数）：长的那一层函数体里以后若写原始字符串，用例红而产品没错；今天的产品代码判绿（上面）。

推翻 K5 判的现象：今天的产品代码在模型上判红；或第十八块 hunk 出现在测试模块之外。

## 7. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 报告的判据与三个量 | 三个量没打中；因果上打中（窄） | 365 次报告独立重算 0 次不符；判据只看「F 在上限、C 有记着落点、隔离数不全为 0」，`ff` 例里同一段历史无 C 的对照同步被拒，报了「被宽护卡住」、按出路二推 15 次后仍被拒（384 槽盘 39 行）；条款字面同义，不算打穿条款 |
| K1-1 报成别的 | 没造出来 | C 在而停因是预算用完或抬 F 被空间拒这一形没造出，变异 `budgetAsStuck`、`spaceAsStuck` 入库用例全绿 |
| K1-2 出路 | 没打穿条款 | 出路一读得出时走得通、读不出时按文字拒挂载；出路二走得通；次序对永久读不出那一成因偏险（卸载先成功、会话已丢才被拒） |
| K2-1 预演与真发、崩溃 | 没打中（有阳性对照） | 块设备错 4 个注入点再挂的 txg 等于会话的；322 个推环中镜像 0 个红，256 槽盘对照 27/38 红；失败后会话冻结是发布路径原有处置 |
| K2-2 提前盖掉别的 | 没打中；两形注入级 | 23 个偏移里被抛弃根的隔离推到它那一槽才放；C 与被抛弃根落点重叠时「放出来」的数偏大、C 有落点而无隔离槽时仍推，都是注入级、可达性没证 |
| K2-3 推完仍卡 | 384 槽盘没打中（K1-1 那一形除外）；256 槽盘线索 | 推错槽 0、F 不变、独立数出次数与实发次数 472 条全等；256 槽盘 21/21 推完仍被拒（正文已知没做的几何） |
| K3-1 回收时补得上 | 没打中 | 每条回收路去补变异入库用例都红；1260 段历史 `c_handed_out`、`pu_pos`、`first_red` 全 0，漏补对照 39/96 红 |
| K3-2 补不上那一支 | 入库无用例、危害没证 | `None` 改成真入库 35 条全绿；192 段历史不红；C762 本身是已知留着 |
| K4 判别力 | 打中 | 实现员 19 行里行 1524 已不红；我的 25 个变异活 11 个；崩溃放量扫源码名单外 3 种；推环入口两支无用例属已知 |
| K5 只动测试 | 没把产品带偏；一处误判 | 5 个 hunk 全在 `mod tests`、无锁无线程；卡池模块以后的合法写法有 8 类会被判红，其中 `A != B` 是被当成宏调用的误判 |

## 8. 没做什么

- 不判别的腿的格、不替主 agent 采纳或出判决；上面的「判」都是这一条腿读码加造反例得出的，被攻过零轮。
- 没跑门禁阶段、checker 档测试、带 gpu 特性的 cargo test、崩溃枚举、QEMU；没起 GPU 设备。第十八块的 4 条扫源码用例我没在 checker 档里跑，只跑了从源码机械抽出的模型（同一份测试体）；模型对实现员 9 行变异全红、对今天的产品代码全绿，但「真用例在 checker 档里的行为」复核不了，要主 agent 照 `merge-10z-r` 那一趟的做法认。
- 384 槽盘之外的几何没当结论：256、288、320、448、512 槽盘的数只当线索（正文第二节已写明没量）。
- 随机历史里没造出的形：C 有 2 个以上记着落点的槽（真实历史）、C 在而停因是预算用完或抬 F 被空间拒、被抛弃根与宽护同时出现；这几形只用注入（`J`）替代，注入只验推环的算与摘，不验挂载读出它们的那一半。
- 阳性对照在 384 槽盘上缺：K2-1 的推环中镜像对照（256 槽盘红 27/38）放到 384 槽盘上红不出来，所以 384 槽盘的「0 个红」靠 256 槽盘证过探针有牙。
- 没替实现员重跑 mutations 表之外的行，没查 `mutation-tables` 门禁格。
- 扫源码用例的三种名单外绕过（K4-3）没编进产品代码验类型；`let (stop_request, _)` 那一种要有一个返回同类型值的帮手函数才编得过，我没写。
- 不读攻方腿本轮的提示与产出（禁读清单），没读。

## 9. 与规则的出入、清理

- 出入：`shadowprobe/k5.bin` 第一次是直接执行的（只打一行 `ok`），随后经 `run-with-memory-cap.sh` 重跑一遍；`shadow-a.rs` 到 `shadow-e.rs` 编了没跑；其余编出来的二进制（`p10z_probe`、`scan-gpu`、`scan-crash`、用例文件的测试二进制）都经 `bash scripts/run-with-memory-cap.sh` 起，一次一件，内存上限 1G 到 12G，线程不超过 15。
- 草稿目录里的拷贝与编译目录已删（删前 `du -sh`）：`target-a` 528M、`target-ctl` 528M、`target-m` 842M、`target-scan` 1.1M、`tree-a` 22M、`tree-m` 22M；`tree-a` 与 `target-a` 为补 `nr-crash-before-root` 日志重建过一次（又 22M、528M），也已删。路径都在 `~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet/` 下。留着的是日志与脚本草稿（该目录合 28M，没有编译目录与仓副本）。
- 要主 agent 现查的最短命令（不用我的二进制）：行 1524 的变异——把 `crates/singlefs-core/src/mount.rs` 根槽读坏那一圈调用 `isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root` 的第三个实参 `placement.span` 改成 `1`，跑 `cargo test --release -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table`，应仍 35 过 1 个 ignored。

## 10. 引文核对

```
$ python3 scripts/cite-check.py research/prompts/m3-batch10z-pqrs-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree --root /home/fy5090/code/singlefs | tail -1
  ✓ 核了 17 处引文，对上 17 处，没判 0 处（逐处列在上面）
```
