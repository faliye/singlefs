# m3-close-c656-r2 辩方腿报告（云端辩方，Sonnet，2026-10-20）

轮名 m3-close-c656-r2。格 U1 为主、兼 U3（`research/prompts/_m3-close-c656-r2-body.md` 第三、四节）。立场：复核第一轮判决 `research/prompts/m3-close-c656-r1-main-verification.md` 够不够得着、是不是同样打中所有替代方案，替出局的甲辩护。方案按 `crates/` 今天的实现来谈。

数的标记：【量】= 本腿这一次跑出、贴了原样输出；【r1量】= 第一轮腿的产物里现读到的数，本腿没重跑，路径与行号给出；【推】= 读码或推出，没有跑过。没标的是引文或定义。

## 复跑命令与哈希

模型目录 `research/prompts/m3-close-c656-r2-sonnet-model/`。原型来自第一轮攻方腿（写序 today / invalidate / sysconf-first / new-then-sysconf 的补丁与探针原样拷在 `patches/` 下），本腿加三样：`patches/make_filesystem-order-benign.patch`（写序变体 `LatestRootLast`）、探针里的历史名后缀 `fpdiff`（旧 fsid 与新 fsid 的前 8 字节也不同）、`tools/summarize.py`（按结局类汇总）。

```
bash scripts/hosts/run-with-memory-cap.sh 3G bash research/prompts/m3-close-c656-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

冻结副本 = 仓里 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 的一份拷贝；本腿开工时 `crates/` 整树哈希 `bba88b6b967a10eb2ca90df6114a112a988c4d7395d81f46103919dcd661bd30`（与第一轮攻方腿的冻结副本只差四个无关文件：`mutations.tsv`、`e185_gc_batch_wall_clock_split.rs`、`mounted_session.rs`、`transaction.rs`，`make_filesystem.rs` 与 `recovery.rs` 逐字节相同，`diff -rq` 数的）。从头编要编 `librocksdb-sys`，内存紧时本腿的跑法是：编译在包装外面做（`cargo build` 不要求经包装），`RERUN_PREBUILT_BINARY=<编好的二进制>` 指给 `rerun.sh` 跳过编译；`RERUN_STAGES="a b"`（默认）可只跑一个阶段。

```
6d54aa5646dfc15597eb05541c262fe1ea25d6d6e254829511c84b39fba4dad2  patches/make_filesystem-order-benign.patch
fb3a1e7c2fc8120a6ce9e06286e6f277a6539e975ca945faca15a4a5b809d19b  patches/make_filesystem-order.patch
1a7aae7bfab678e8d459e25ef55b99ed838ed8f757f07b072ecf55ab6d162e0e  patches/workspace.patch
60893ccb15c1de5e72236d853e1155cf18f3415291a1e104274cbae85bd8fb76  probe/Cargo.toml
0b4d8409cabde07ad2e81d8de093ae223e7c0ed724caf4149216069c54ee1ebd  probe/main.rs
f8c781c227b6207ae53f4de7f8c8b5ccd535a9fa8a1b17c1c4b1175438164b17  tools/rerun.sh
a34669be01c6ff76773abae8aa31882484732e6dc515afef16fbd6b523858e41  tools/summarize.py
```

`out/` 下各文件的哈希见末尾「产物哈希」一节。

## 一、U1：第一轮「甲出局」够不够得着

### 1.1 引理：fsid 相同时，没有一种写序同时给出「提交前旧池完好」与「提交后不混合」（推的，在 D13 已定项 4 的崩溃模型里）

前提都有出处：

- P1 择根只认自证过、16 字节 fsid 与所择系统配置相同的根：「if &filesystem_identifier != expected_filesystem_identifier {」（`crates/singlefs-core/src/root_record.rs:139`），读环的那一处「match RootRecord::parse_slot(&bytes, filesystem_identifier) {」（`crates/singlefs-core/src/recovery.rs:1568`）；择新的键是 `(checkpoint_txg, 实例代号)`，没有别的字段参与。
- P2 旧池最新根 R_L（用过的盘上 txg ≥ 3）的键大于新池第 0 代根：「let genesis = CheckpointTxg(0);」（`crates/singlefs-core/src/make_filesystem.rs:490`）。
- P3 崩溃域：`13-验证路线.md@已定项 4：崩溃点重放只枚举整写子集` 写「当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上」，段内任意真子集可持久，之前各段全落。
- P4 新池的系统配置只有 mkfs 的系统配置槽写能给（记这些写为 G），旧池的最新根 R_L 只有「清掉它」或「写一条键更大的根压过它」两种办法让它不再被择中。

推导：R_L 不再被择中的那一（几）次写落在三个位置之一。(a) 在 G 之前的段：存在态「G 之前各段全落、G 一个没落」，此时择到的仍是旧系统配置而 R_L 已经没了，旧池读回少了最后确认的一版（有 journal 补得回来时是靠 journal，不是原样）。(b) 在 G 之后的段：存在态「G 全落、之后一个没落」，新系统配置加 R_L，混合。(c) 与 G 同段：取「G 全落、清除写没落」的子集，同 (b)。三处穷尽，所以 fsid 相同时一种写序给不出「前半完好、后半不混合」。journal 环、被 R_L 引用的单元同形。

乙躲开它的办法不是更会排次序，而是在 (a) 与 G 之间插一个没有系统配置的态（作废段）：(a) 的违反变成「整池拒」，而第一轮题面允许整池拒（`_m3-close-c656-r1-body.md` 第一节乙那一条）。所以引理支持「fsid 相同时只有先杀旧池的写序不混合」，不支持「甲在 fsid 不同时也做不到」。

引理的逃生口：让 R_L 不被择中的不是写，而是系统配置里读者用来读根的字段变了。读根用到的字段有 fsid（相同，堵死）、槽距与每区槽数、`region_devices`。槽距与每区槽数改了也盖不住每个区域的槽 0（`slot_offset` 里槽 0 恒在区域起点：`crates/singlefs-core/src/root_ring.rs:101` 起，区域起点是常量），而 R_L 恰住槽 0 的历史存在（旧池 u = 23 时 txg 24 落区域 0 槽 0，【r1量】`research/prompts/m3-close-c656-r1-opus-output.md` 第 83 行）。`region_devices` 换归属（新旧两池读的是不同盘上的区域副本）能让新旧根集合不相交，但归属 0 / 1 / 0 是暖机常量的前提：「⚠️ 归属改了这两个常量要重算（加盘或开条带表时）。」（`.claude/kb/decisions/22-单元原子性怎么合成.md:341`），属于改格式常量，而且连续两次 mkfs 之后归属换回去，更早一代的根又读得到了。【推】都没量。

什么现象会推翻引理：找到一种写序（或系统配置的一个读者字段），使得某一个崩溃态既读回旧池完好、又不存在「新系统配置 + 可被择中的旧根」的态——即 P4 的两种办法之外还有第三种让 R_L 不被择中的办法。

### 1.2 「三份结论一致」说的不是同一句话

判决行：「**甲出局**：三份结论一致（正推推出做不到，攻方两种写序在 fsid 相同时都量出新旧混合）」（`research/prompts/m3-close-c656-r1-main-verification.md:23`）。三份报告各自断言的是：

- 正推：「**字面的甲：做不到。**」（`research/prompts/m3-close-c656-r1-sonnet-output.md:118`），不分 fsid，理由是首改写必落在新旧同偏移的固定结构上，且「mkfs 自己没有读盘调用，无法绕开」（`research/prompts/m3-close-c656-r1-sonnet-output.md:54`）。
- 攻方第一次抽样：fsid 相同时 new-then-sysconf 混合；fsid 不同时「提交点之前 18 态……混合 0 态」（`research/prompts/m3-close-c656-r1-opus-output.md:81`），只在字面上写到旧池读得到的字节。
- 攻方第二次抽样：候选写序「提交点之前只写旧池没写过的单元槽，旧池读的系统配置、根环、journal 环、单元一个字节不动。」（`research/prompts/m3-close-c656-r1-opus-s2-output.md:76`）；fsid 不同时该候选站住，同一份报告的原话是「**混合 0、旧池读回不对 0**」（`research/prompts/m3-close-c656-r1-opus-s2-output.md:127`），条件是 mkfs 先读旧池。

所以第一轮真正坐实的是「fsid 相同（且写序单独）甲做不到」，不是「甲做不到」。fsid 不同的半边，两次抽样都没打中，正推的「做不到」则被第二次抽样的候选绕开了（候选的条件正是正推那句「mkfs 没有读盘调用」不成立）。第一轮判决自己的第三列其实记了这一点（第 23 行末「第二次的候选要 mkfs 先读旧池」），但「甲出局」那一句没带条件。

fsid 不同的半边，第一轮攻方的数现读如下（命令与原样输出，汇总用本腿的 `tools/summarize.py`，挂钟一行不贴）：
```
$ python3 research/prompts/m3-close-c656-r2-sonnet-model/tools/summarize.py research/prompts/m3-close-c656-r1-opus-model/out/new-then-sysconf.txt | grep -E "^==|^   all "
== h=o512-u5-fdiff order=new-then-sysconf 总态数 7874
   all checker_green  24
   all checker_red    7850
   all new_empty      7790
   all old_intact     53
   all refused        31
== h=o512-u5-fsame order=new-then-sysconf 总态数 7874
   all checker_green  39
   all checker_red    7835
   all mixed_last     6493
   all mixed_older    160
   all new_empty      1152
   all old_intact     68
   all refused        1
== h=o512-u23-fdiff order=new-then-sysconf 总态数 7874
   all checker_green  24
   all checker_red    7850
   all new_empty      7790
   all old_intact     16
   all old_other      37
   all refused        31
```

什么现象会推翻 1.2：fsid 不同时有人给出一个写序的混合态或旧池读回不对的态（两次抽样与本腿 3.1、3.2 的 fsid 不同历史合计都是 0），或正推补一条「读盘的 mkfs 也躲不开」的论证。

### 1.3 「fsid 相同」是不是合法输入：合法，而且用户已就它选过路

- API 契约：fsid 由调用方给，mkfs 不生成。代码注释：「拿同一个 fsid 在用过的盘上重做 mkfs，旧池留在槽 1..7 里的根照样自证得过、解得开，」（`crates/singlefs-core/src/make_filesystem.rs:533`）；恢复一侧把它写成要承受的情形：「fsid 相同（调用方拿同一个 fsid 重做 mkfs）时按旧池的几何挂。」（`crates/singlefs-core/src/recovery.rs:1283`）。
- 有专门的用例：`fn remaking_the_pool_with_the_same_filesystem_identifier_leaves_no_root_of_the_old_pool() {`（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:451`）。
- 用户 2026-09-22 就这个输入定过路：现行条目「用户定的路 ① 连根环一起清」（`.claude/kb/checks-owed.md:611`）。当时摆出的三条路（取自提交 d96b39807 里 C484 原行，命令与原样输出）：

```
$ git show d96b39807:.claude/kb/checks-owed.md | grep '^| C484' | grep -o '今天没有 mkfs 命令行工具，测试用的都是写死的 fsid，所以这条走得到 | 三条路各有代价，要定：[^。]*。'
今天没有 mkfs 命令行工具，测试用的都是写死的 fsid，所以这条走得到 | 三条路各有代价，要定：① mkfs 连根环一起清（验收里写死的字节数 116 224 + 2 × 768 MiB 要改，段序列再加一种形状）；② mkfs 自己生成 fsid（随机源从哪来要定，测试要一个只供测试的固定入口）；③ mkfs 在一块已有有效池的盘上拒绝跑、除非显式覆盖。
```

  所以第二轮正文 U4 的丙七（mkfs 读旧池拒，或强制新 fsid）就是当年的 ③ 与 ②，用户选了 ①。今天没有 mkfs 命令行工具，同 fsid 的调用方目前只有测试与装置；辩方找不到任何「同 fsid 不合法」的出处。
- 判：同 fsid 是合法输入，「取样点不合法」这条辩护不成立。丙七要重开，得有当年没有的理由；2026-10-20 的新前提（不接受挂上丢了已确认写的旧池）算不算这个理由，不是本腿能判的（它只约束第一道屏障之前，丙七管的是整条 mkfs）。
- 什么现象会推翻 1.3：C484 之后有一条用户定案或 kb 条款写明 fsid 必须由 mkfs 生成或必须与盘上旧池不同（本腿用「fsid」加「mkfs 生成 / 随机 / UUID / 不得相同 / 同 fsid」几组词在 `.claude/kb/decisions/` 与 `.claude/kb/layout/` 里 grep，除 D22 已定项 8 欠账那一行外没有命中相关条款；没全仓穷尽）；或 mkfs 的契约改成自己生成 fsid。

### 1.4 出局的判据是不是同样打中乙

把第一轮用过或可能用来排甲的几条判据，各在乙上过一遍（乙的数：【r1量】两次攻方抽样共九份历史，下面是本腿用 `summarize.py` 对第一轮攻方产物的现读）：

```
$ python3 research/prompts/m3-close-c656-r2-sonnet-model/tools/summarize.py research/prompts/m3-close-c656-r1-opus-model/out/invalidate.txt | grep -A8 "h=o512-u5-fsame order=invalidate"
name=done h=o512-u5-fsame order=invalidate states=31195 observed=31195 violations=31122 seconds=14.9
name=done h=o65536-u5-fsame order=invalidate states=31150 observed=31150 violations=31110 seconds=17.4
name=done h=o512-u6-fdiff order=invalidate states=31195 observed=31195 violations=31122 seconds=28.5
name=done h=o512-u23-fdiff order=invalidate states=31195 observed=31195 violations=31122 seconds=67.1
== h=o512-u5-fdiff order=invalidate 总态数 31195
   all checker_green  48
   all checker_red    31147
   all new_empty      65
   all old_intact     8
--
== h=o512-u5-fsame order=invalidate 总态数 31195
   all checker_green  48
   all checker_red    31147
   all new_empty      65
   all old_intact     8
   all refused        31122
    s0 old_intact     8
    s1 refused        31103
    s2 refused        1
```

| 判据 | 打中甲？ | 打中乙？ | 出处 |
|---|---|---|---|
| K1 新系统配置 + 旧池的根并存（混合） | 打中，仅 fsid 相同：new-then-sysconf 混合 6 653 / 7 874，第二次抽样候选 7 517 / 7 874（【r1量】） | 没打中：两次攻方抽样共九趟全量（其中 fsid 相同四趟）混合 0（上面 `invalidate` 一行没有 mixed 类；fsame 一份 31 195 态里旧池 8、整池拒 31 122、新池 65） | 本节上方；`research/prompts/m3-close-c656-r1-opus-s2-output.md:149` |
| K2 mkfs 要先读旧池 | 打中第二次抽样的候选（找单元落点、判 fsid）；new-then-sysconf 与 sysconf-first 不读 | 没打中，乙不读盘 | `research/prompts/m3-close-c656-r1-opus-s2-output.md:193` |
| K3 用过的盘上做完池级 checker 报 I-7.7 | 打中 | **同样打中**（三种写序做完都红） | 判决已写「不分辨臂，不拿它判甲乙。」（`research/prompts/m3-close-c656-r1-main-verification.md:27`） |
| K4 第一个破坏性写落地之后旧池不能再挂 | 打中（提交点之后） | 打中（作废之后） | 同形，不分辨 |
| K5 一盘已变一盘没变时只读挂得上、可写挂载拒；择系统配置取第一块可择盘那一份 | 打中（甲提交段 10 态） | 打中（作废段 4 / 8 态） | 判决已写「附带两样不分辨臂（第二次抽样）」（`research/prompts/m3-close-c656-r1-main-verification.md:24`）；甲的 10 态见 `research/prompts/m3-close-c656-r1-opus-s2-output.md:170` |
| K6 崩溃后只能「整池拒、重跑 mkfs」的态占比 | 看写序：new-then-sysconf（fsid 不同）31 / 7 874；第二次抽样候选 ≈ 7 821 / 7 874（根最后写） | 31 122 / 31 195 | 本节上方；`research/prompts/m3-close-c656-r1-opus-s2-output.md:127` |

读法：K1 是唯一把甲排出去、又没打中乙的判据，而它只在 fsid 相同时成立（引理）。K3、K4、K5 同样打中乙，第一轮判决已经这样记了。K6 是个没人拿来判过的量：它不是出局理由，但说明「先杀旧池」的乙在 fsid 不同的常见情形下，比把根提前写好的甲多出两个数量级的「重跑 mkfs」窗口（按态数算，不是按挂钟）。第一轮判决没有这一格。什么现象会推翻 1.4 的读法：乙在某份历史上出现一个新系统配置加旧根的态（两次抽样九趟全量都是 0），或 K1 之外有一条判据把甲排出去而乙躲得开。

## 二、U3：「完整的新池」写成什么

### 2.1 40 个态到底是什么（【r1量】，fsid 相同一份历史的系统配置段原样行）

乙的系统配置段（四次系统配置写各取没落 / 撕裂 / 落，80 个态，作废、单元、三条根都已持久）：

```
$ grep -n 'h=o512-u5-fsame order=invalidate' research/prompts/m3-close-c656-r1-opus-model/out/invalidate.txt | grep -E ' s5__' | cut -c1-230
32:name=count h=o512-u5-fsame order=invalidate count=15 s5__torn__err_no_valid_system_configuration__fail_no_valid_system_configuration__red__ro_err_recovery__rw_not_tried__notmixed
33:name=count h=o512-u5-fsame order=invalidate count=34 s5__torn__new__newempty__green__ro_err_open__rw_err_system_configuration_unreadable_after_the_read_stage__notmixed
34:name=count h=o512-u5-fsame order=invalidate count=16 s5__torn__new__newempty__red__ro_err_open__rw_ok_i0t0__notmixed
35:name=count h=o512-u5-fsame order=invalidate count=1 s5__whole__err_no_valid_system_configuration__fail_no_valid_system_configuration__red__ro_err_recovery__rw_not_tried__notmixed
36:name=count h=o512-u5-fsame order=invalidate count=6 s5__whole__new__newempty__green__ro_err_open__rw_err_system_configuration_unreadable_after_the_read_stage__notmixed
37:name=count h=o512-u5-fsame order=invalidate count=8 s5__whole__new__newempty__red__ro_err_open__rw_ok_i0t0__notmixed
```

分类（本腿用 awk 对 `invalidate.txt` 里五份历史的 `s5__` 行按类相加：fsid 相同 / 不同、u = 5 / 6 / 23 的四份都是拒 16、缺盘 40、两盘有 24；旧池槽距 65536 那份是拒 4、缺盘 16、两盘有 15，共 35 态，因为它的新槽 1 落在旧池没有内容的位置、不是原地覆写、只取两态，每盘 3 × 2 = 6 态，两盘 36 减去全落 1 态，是推的）：

| 类 | 态数 | 恢复读回 | 只读挂载 | 可写挂载 | 池级 checker |
|---|---|---|---|---|---|
| 整池拒 | 16 | `NoValidSystemConfiguration`（四槽没有一个自证过） | 拒 | 不试 | 红（没有系统配置） |
| 新池，一块盘一份新槽都没有 | 40 | 第 0 代空池，根 (0, 0) | `Open` 错（空池没有文件可开，做完的那一态同样） | 拒，`SystemConfigurationUnreadableAfterTheReadStage` | 绿 |
| 新池，两块盘各至少一份新槽 | 24 | 第 0 代空池，根 (0, 0) | 同上 | 做成 | 红（I-7.7，用过的盘上做完的那一态同红，是 C659，见下） |

那 40 个态里根、journal、单元都已持久（三条根各占一段，单元在它们之前隔着屏障）；缺的只是一块盘的系统配置。这正是已定项 8 射程里已经定过的形：「容错承诺是单故障：每盘一槽系统配置坏在承诺里；」（`.claude/kb/decisions/22-单元原子性怎么合成.md:180`）与「某块盘一份自证过的系统配置都没有时，取号那一步也拒（用户 2026-10-03 定，原话在变更史）。」（同文件同一行）。缺盘那 40 个态的可写挂载结局就是那一句的结果，只读照读。池级 checker 的红绿在这三类上不是完整性的度量：红的那 24 个是 C659 的 I-7.7（用过的盘上 mkfs 做完同红；旧 fsid 前 8 字节不同的 fpdiff 历史上这 24 个同为绿，`research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u5-fpdiff-invalidate.txt`，见 3.3），绿的 40 个是 I-7.7 在缺盘时报「不适用」（`crates/singlefs-checker/invariants/02-根环.md:14` 的 I-7.7 ①：「任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」」那一类写法），所以条文不要拿 checker 的红绿定义「完整」。

### 2.2 条文候选

候选 X（宽，含那 40 个态）。写进 D22 已定项 8 第 3 条、取代射程里「mkfs 中途崩溃（第一道屏障之前）该挂上旧池、整池拒还是别的，没有条款」那一句的措辞（草稿，措辞待主 agent 定）：

> mkfs 先把每块盘的 [0, 根环基址) 整段写 0，两块盘各过一道屏障（作废），之后才写别的。作废持久之前崩溃：下一次挂载照没跑过 mkfs 处置，旧池读回最后确认的一版，或整池拒。作废持久之后、最后一道屏障之前崩溃：下一次挂载只有两种结局——整池拒（`NoValidSystemConfiguration`），或读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）；后一种里系统配置的冗余可以缺，缺一块盘的新槽时那块盘按本条射程现行那一句处置（可写挂载在取号那一步拒，读照读）。任何结局不得读回旧池的任何一版，也不得新系统配置与旧池的根并存。

候选 Y（严）。「完整的新池」限定为每块盘都有至少一份自证过的新系统配置（24 态）；那 40 态单列为第三种结局「新池缺一盘系统配置」，条文与验收各写三类。

两者的差别：
- X 的验收口径是两类（拒 / 空池），与现行射程那一句不重复立类；Y 多一类，但这一类的行为（只读读得出、可写取号拒）已经由那一句定了，立新类不增加任何一个判红的态。
- 两者对旧池与混合的约束相同（量过：整个乙的作废持久之后，择到旧池 0 态、混合 0 态，两次攻方抽样共九趟全量）。
- 推荐 X 的理由只有一条：条文不要把已经有条款的形再立一个名字。要不要 Y 的三类口径是文风取舍，不影响判红。【推】

什么现象会推翻 X：那 40 态里有一态恢复读回的不是 (0, 0) 空池，或可写挂载做成而某块盘没有自证过的槽（违反 2026-10-03 那条定案），或作废持久之后的任一态读回旧池的一版。会推翻「那 40 态是现成形」的：把一份正常池的某块盘两槽都清成零，可写挂载的结局不是 `SystemConfigurationUnreadableAfterTheReadStage`——本腿没有单独量这一格，是从 r1 的 s0 态（一盘清了、另一盘没清，同一个错）【r1量】加已定项 8 射程那句推的。

## 三、U1 续：甲还剩什么（本腿新量的三件）

### 3.1 fsid 相同时的一种写序，在「不挂上丢了已确认写的旧池」的口径下量出 0 个违例（候选名 latest-root-last）

写序（`patches/make_filesystem-order-benign.patch` 的 `LatestRootLast`，和第一轮的补丁一样只换次序、字节与 `make_filesystem` 相同，单元落点除外）：

1. 两单元写进旧池全 0 的槽（盘末往回 256 槽，探针核过两块盘上那三槽全 0）→ 屏障；
2. 第 0 代根写进各区域的槽 0（最新的旧根正住在某区域槽 0 的那个区域先不写）；
3. 系统配置 × 4（提交）→ 屏障；
4. 清旧根，最新的那一条除外（每区域至多两段，只清归属那块盘）→ 屏障；
5. 清最新的那一条旧根（一次写，一个根槽宽）→ 屏障；
6. （最新旧根住槽 0 的区域补写第 0 代根 → 屏障；）
7. 清 [槽 1 末尾, 根环基址) 与 journal 环 → 屏障。

它要 mkfs 先读旧池两样：最新旧根住哪个区域的哪个槽、单元写进哪几个全 0 的槽（探针里这两样由历史直接算出来传给写序，所以这里量的只是写序，不是「mkfs 读旧池」那一段）。段序列 `4+1+1+1+4+4+1+4`，20 次写，u = 5 时全量 188 态（乙 31 195 态；原因是没有一次写 12 件的大段）。

命令与原样输出（`out/` 下是 `rerun.sh` 跑出来的 `name=run|done|count|checker` 行；`summarize.py` 的类见它文件头）：
```
$ python3 research/prompts/m3-close-c656-r2-sonnet-model/tools/summarize.py research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u5-fsame-latest-root-last.txt research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u6-fsame-latest-root-last.txt research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u23-fsame-latest-root-last.txt research/prompts/m3-close-c656-r2-sonnet-model/out/o65536-u5-fsame-latest-root-last.txt research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u5-fdiff-latest-root-last.txt | grep -E "^name=done|^==|^   all "
name=done h=o512-u5-fsame order=latest-root-last states=188 observed=188 violations=1 seconds=64.1
== h=o512-u5-fsame order=latest-root-last 总态数 188
   all checker_green  16
   all checker_red    172
   all mixed_last     83
   all new_empty      36
   all old_intact     68
   all refused        1
name=done h=o512-u6-fsame order=latest-root-last states=188 observed=188 violations=1 seconds=17.3
== h=o512-u6-fsame order=latest-root-last 总态数 188
   all checker_green  16
   all checker_red    172
   all mixed_last     83
   all new_empty      36
   all old_intact     68
   all refused        1
name=done h=o512-u23-fsame order=latest-root-last states=161 observed=161 violations=1 seconds=25.8
== h=o512-u23-fsame order=latest-root-last 总态数 161
   all checker_green  16
   all checker_red    145
   all mixed_last     56
   all new_empty      37
   all old_intact     67
   all refused        1
name=done h=o65536-u5-fsame order=latest-root-last states=150 observed=150 violations=0 seconds=8.4
== h=o65536-u5-fsame order=latest-root-last 总态数 150
   all checker_green  20
   all checker_red    130
   all new_empty      112
   all old_intact     38
name=done h=o512-u5-fdiff order=latest-root-last states=188 observed=188 violations=31 seconds=5.9
== h=o512-u5-fdiff order=latest-root-last 总态数 188
   all checker_green  24
   all checker_red    164
   all new_empty      104
   all old_intact     53
   all refused        31
```

可写挂载按结局类再分（命令与原样输出；类 = 键里 `__mixed` 的算 mixed，择到 `old` 开头的系统配置算 old，`new` 且读回空池算 new，其余算 refuse）：
```
$ for f in out/o512-u5-fsame-latest-root-last.txt out/o512-u23-fsame-latest-root-last.txt; do echo "== $f"; grep '^name=count' $f | sed -E 's/name=count h=[^ ]+ order=[^ ]+ count=([0-9]+) (.*)/\1 \2/' | awk '{c=$1; k=$2; n=split(k,p,"__"); cls=(k ~ /__mixed/)?"mixed":((p[3] ~ /^old/)?"old":((p[3]=="new")?"new":"refuse")); rw=p[7]; cnt[cls" "rw]+=c} END {for (x in cnt) print cnt[x], x}' | sort -k2,2 -k1,1nr; done   # 工作目录 research/prompts/m3-close-c656-r2-sonnet-model
== out/o512-u5-fsame-latest-root-last.txt
77 mixed rw_ok_i1t6
6 mixed rw_err_system_configuration_unreadable_after_the_read_stage
36 new rw_ok_i0t0
58 old rw_ok_i1t6
10 old rw_err_system_configuration_unreadable_after_the_read_stage
1 refuse rw_not_tried
== out/o512-u23-fsame-latest-root-last.txt
50 mixed rw_ok_i1t24
6 mixed rw_err_system_configuration_unreadable_after_the_read_stage
36 new rw_ok_i0t0
1 new rw_err_writable_mount_refused_by_devices_without_the_selected_version
57 old rw_ok_i1t24
10 old rw_err_system_configuration_unreadable_after_the_read_stage
1 refuse rw_not_tried
```

读法（量过的部分）：
- fsid 相同的三份同槽距历史（u = 5 / 6 / 23；u = 23 时最新旧根正住区域 0 槽 0，是第一轮 new-then-sysconf 要靠 journal 才不丢的那一格，这里 `old_other` 0）：择到旧池系统配置的态读回都与最后确认的一版逐字节相同（没有 `old_other`）；择到新系统配置而读回旧池内容的态（mixed）全部是 `mixed_last`——读回的恰是最后确认的一版，**没有 `mixed_older`**（第一轮 new-then-sysconf 在 fsid 相同的同一份 u = 5 历史上是 160 个，见上面 1.2 的输出）。
- 旧池槽距 65536（fsid 相同）：新读者读不到旧池的根，无混合，整池拒 0。fsid 不同：混合 0，拒 31（两盘系统配置对不上），其余旧池完好或空池。
- 所以在用户 2026-10-20 的前提（不接受挂上丢了已确认写的旧池）的口径下，fsid 相同并不推出「甲做不到」：这一支没有一个态丢已确认的写。它违反的是第一轮正文替乙写的那句「不许挂上旧池或新旧混合」里的「新旧混合」：83 / 188 个态（u = 5）是新系统配置加旧池原样的内容，可写挂载做成 77 个（【量】）；这 83 个态池级 checker 全红（【量】，键里 `red`；读法是旧根的实例代号 1 大于新系统配置的 0，I-7.7），可写挂载取号之后转不转绿没量。

代价与前提（推的，没量）：
- mkfs 得先读旧池：每块盘三个区域的根槽（解自证、比 (txg, 实例) 找出最新那一条）、候选单元槽是否全 0（活单元不会全 0，所以不用走旧池的账）、旧池的系统配置（判几何是否相同；几何不同时第一轮 V4 的论证说明新读者读不到旧根，也可以走这条或退回乙）。读不出、没有全 0 的槽、旧池没有系统配置时退回乙的写序。这是 mkfs 第一次读盘，也是 C484 当年的路 ③ 的一半。
- 提交点之前第 0 代根已经写进各区域槽 0，盖掉了槽 0 里更老的旧根（u = 5 时是 txg 0、1、2 那三条），旧池的回退深度变浅；读回不受影响（`old_intact` 全是原样），但旧池再碰一次单故障时可退的根少了。
- 写 20 次（乙 21）；池屏障 5 道（最新旧根住槽 0 时 6 道；补丁里该分支的 `barriers!();` 共 6 处，其中 1 处在 `latest_slot == 0` 的分支里），乙 3 道。
- 没覆盖：旧池处在发布半截（根已落、journal 记录比根新）的历史；旧池几何与新池不同而根环偏移有重合的历史（o65536 那份新读者看不见旧根，不是重合的）；探针传给写序的两样没经过「mkfs 自己读」；journal 比最新根新的历史没造（journal 环在最后才清，推的：这一步之前新读者读到的最新根是旧根，journal 照旧补；旧根清掉之后新根的 journal 链首对不上旧记录，s7 的 35 个态读回都是空池，【量】）。

什么现象会推翻：任何一个态 `mixed_older` 或 `old_other` 非 0（对 u = 5 / 6 / 23 这三份之外的历史、journal 比根新的历史尤其要看）；或 mkfs 先读旧池的任何一样在真设备上做不到（读错时没有安全退路）。

### 3.2 先写系统配置的写序（第一轮只有推的混合）：本腿补量，fsid 相同混合 119 837 / 124 499

第一轮攻方腿对 sysconf-first 在 fsid 相同时的混合写的是「推的」，主 agent 复跑时三件大枚举没出分类数，判决写「核不动」（`research/prompts/m3-close-c656-r1-main-verification.md:15`）。仓外的复跑日志（`m3-close-c656-r1-main-checks/fsame-sysconf-first.log`，在 `~/.local/share/singlefs/work/` 下）里紧挨 `RecordedOnly` 那一行之前是 `NOT_RUN_NO_GPU_ADAPTER`：配置写了 `ENABLE_GPU=1`、二进制没编 gpu 特性，判段那一趟不跑（读日志读到的，不是本腿复现的）。本腿在没有 `multi-host.env` 的草稿目录里起同一个探针，判段走 CPU，全量出数。命令与原样输出（四份历史各一趟全量）：
```
$ python3 research/prompts/m3-close-c656-r2-sonnet-model/tools/summarize.py research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u5-fsame-sysconf-first.txt research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u5-fdiff-sysconf-first.txt research/prompts/m3-close-c656-r2-sonnet-model/out/o512-u5-fpdiff-sysconf-first.txt research/prompts/m3-close-c656-r2-sonnet-model/out/o65536-u5-fdiff-sysconf-first.txt | grep -E "^name=done|^==|^   all "
name=done h=o512-u5-fsame order=sysconf-first states=124499 observed=124499 violations=4609 seconds=276.1
== h=o512-u5-fsame order=sysconf-first 总态数 124499
   all checker_green  21
   all checker_red    124478
   all mixed_last     115741
   all mixed_older    4096
   all new_empty      3
   all old_intact     50
   all refused        4609
name=done h=o512-u5-fdiff order=sysconf-first states=124499 observed=124499 violations=124461 seconds=185.9
== h=o512-u5-fdiff order=sysconf-first 总态数 124499
   all checker_green  15
   all checker_red    124484
   all new_empty      3
   all old_intact     35
   all refused        124461
name=done h=o512-u5-fpdiff order=sysconf-first states=124499 observed=124499 violations=124461 seconds=6.2
== h=o512-u5-fpdiff order=sysconf-first 总态数 124499
   all checker_green  18
   all checker_red    124481
   all new_empty      3
   all old_intact     35
   all refused        124461
name=done h=o65536-u5-fdiff order=sysconf-first states=279974 observed=279974 violations=279955 seconds=134.6
== h=o65536-u5-fdiff order=sysconf-first 总态数 279974
   all checker_green  16
   all checker_red    279958
   all new_empty      3
   all old_intact     16
   all refused        279955
```

读法：
- fsid 相同（同槽距）：混合 119 837 / 124 499（`mixed_last` 115 741 + `mixed_older` 4 096），旧池完好只有 50 个，整池拒 4 609。第一轮的「推的：有」坐实，数是量的。`mixed_older` 的 4 096 个是丢了已确认写的那一类，所以这个写序在用户前提的口径下也不行。
- fsid 不同（后 8 字节不同、前 8 字节也不同、旧池槽距 65536 三份）：混合 0，旧池完好 35 / 35 / 16，整池拒 124 461 / 124 461 / 279 955，也就是 99.97% 以上的崩溃态是「整池拒、重跑 mkfs」。无读盘的甲（提交点 = 系统配置最先写）在 fsid 不同时和乙的结局同形：第一个破坏性写落地，旧池就不能挂了，之后全是拒；它和乙的差别只在 fsid 相同那一格，而那一格它是坏的（上一条）。
- 所以甲和乙在结局上真正分开的，只有那些「新单元和新根提前写好、提交点只剩系统配置」的写序（new-then-sysconf、latest-root-last），它们的前提都是 mkfs 先读旧池。第一轮正推说「首改写必落在旧池读得到的字节上」对无读盘的 mkfs 成立；对读盘的 mkfs 不成立。

什么现象会推翻：fsid 不同时无读盘的甲出现一个混合态或旧池读回不对的态（四份历史都是 0）；或 fsid 相同时 sysconf-first 的混合态数不是 119 837（同一份冻结副本的复跑数）。

### 3.3 C659 的 I-7.7：「fsid 不同也红」是探针的 fsid 造出来的（量过，对照只差 fsid 字节）

第一轮两个原型的「不同 fsid」只差第 9 字节起（`c656-attack-new!` 与 `c656-attack-old!` 前 8 字节都是 `c656-att`；第二次抽样的 `c656-s2-pool-A!!` 与 `c656-s2-new-pool` 前 8 字节都是 `c656-s2-`：`research/prompts/m3-close-c656-r1-opus-model/probe/main.rs:51-52`、`research/prompts/m3-close-c656-r1-opus-s2-model/probe/src/main.rs:45-47`），而单元头里的 fsid 只有 8 字节：「单元头里的 fsid 是 8 字节：系统配置 fsid 的低 8 字节」（`crates/singlefs-core/src/unit.rs:42`），I-7.7 ① 扫单元写序时按这 8 字节认「fsid 与本池相同」：「if let Some(instance) = unit_write_order_instance(&bytes, filesystem_identifier_low) {」（`crates/singlefs-checker/src/walk.rs:4861`），那个 `filesystem_identifier_low` 在 `crates/singlefs-checker/src/walk.rs:4899` 取自 fsid 前 8 字节。第二次抽样把「fsid 不同、槽距相同也红」报成比 C659 字面更宽的一格，判决只引了「不分辨臂」，没引这个加宽。本腿加一份 `fpdiff` 历史（旧 fsid `old-prefixdiff!!`，前 8 字节与新 fsid 不同），其余全同：

```
$ for f in research/prompts/m3-close-c656-r2-sonnet-model/out/*-{invalidate,new-then-sysconf,sysconf-first}.txt; do echo "$(basename $f .txt) $(grep '^name=checker' $f | tr ' ' '\n' | grep -E '^I-7.7=')"; done    # 通过/违例/不适用
o512-u5-fdiff-invalidate I-7.7=29/25/31166
o512-u5-fpdiff-invalidate I-7.7=29/0/31166
o512-u5-fdiff-new-then-sysconf I-7.7=7833/7790/41
o512-u5-fpdiff-new-then-sysconf I-7.7=7833/0/41
o512-u5-fdiff-sysconf-first I-7.7=124458/124433/41
o512-u5-fpdiff-sysconf-first I-7.7=124458/0/41
o512-u5-fsame-sysconf-first I-7.7=124482/124457/17
o65536-u5-fdiff-sysconf-first I-7.7=279962/279946/12
```

读法（量过）：同一个写序、同一份历史，只把旧 fsid 的前 8 字节换掉，I-7.7 的违例从 25（乙）/ 7 790（new-then-sysconf）/ 124 433（sysconf-first）降到 0；做完的那一态（乙的 s6）也由红变绿（`out/o512-u5-fpdiff-invalidate.txt` 的 `checker_green` 73 = 旧池完好 8 + 新池 65，红的全是拒的态）。所以：

- 第一轮把「fsid 不同、槽距相同也红」当成比 C659 字面更宽的事实，是探针 fsid 造出来的。红的条件是旧池的单元与新池的 fsid 前 8 字节相同（同 fsid，或前缀撞）。
- 丙七（不许同一个 fsid 在用过的盘上重用，强制新 fsid）对 I-7.7 的作用因此可以说：新 fsid 取随机值时（前 8 字节撞的概率 2⁻⁶⁴）C659 的红在乙与甲的写序上都是 0 违例【量，仅限 fpdiff 这一份历史】；同 fsid 的红不动。同 fsid 合法（1.3），所以丙七收不掉 C659，只能收掉其中 fsid 不同的那部分；同 fsid 那部分归甲七（mkfs 清单元区）或乙七（I-7.7 射程收窄）。这一段回答正文 U4 里「与 U1 的同 fsid 是不是合法输入是同一个问题的两面」那一句，U4 其余几格归攻方腿。

什么现象会推翻：fpdiff 历史上 I-7.7 违例非 0（复跑数应为 0 / 0 / 0）；或前 8 字节不同的两个 fsid 在别的几何（旧池 io_min 65536、u 取别的值）上仍红。后两样本腿没量。

## 四、判定一览

| # | 第一轮判决的条目 | 辩方判定 | 证据 |
|---|---|---|---|
| 1 | 甲出局（fsid 相同、写序单独、提交后不混合） | **站住** | 引理 1.1【推】；new-then-sysconf 混合 6 653 / 7 874、第二次抽样候选 7 517 / 7 874【r1量】；sysconf-first 119 837 / 124 499（其中丢已确认写的 4 096）【量】 |
| 2 | 「三份结论一致」 | **够不着**（三份断言的不是同一句） | 1.2：正推断言所有 fsid，攻方两次只在 fsid 相同打中；fsid 不同两次都没打中 |
| 3 | 正推：字面的甲做不到（不分 fsid） | **够不着** | 第二次抽样候选 fsid 不同混合 0（`research/prompts/m3-close-c656-r1-opus-s2-output.md:127`）、提交点之前旧池读回 115 / 115 对（`research/prompts/m3-close-c656-r1-opus-s2-output.md:232`）【r1量】；new-then-sysconf 在 fsid 不同混合 0【r1量，1.2 输出】；本腿 latest-root-last 在 fsid 不同混合 0【量】。条件是 mkfs 读旧池，正是正推论证里「mkfs 没有读盘调用」那一步 |
| 4 | 病根：根环基址与 journal 环起点是常量、新旧逐字节重叠 | **够不着**（重叠不是判别量） | fsid 不同时重叠一样在，混合 0；读根认 16 字节 fsid（`crates/singlefs-core/src/root_record.rs:139`）；引理的前提是「读者没有新旧区分字段」，不是「字节重叠」 |
| 5 | 取样点「fsid 相同」是不是合法输入 | **合法，辩护不成立** | 1.3：mkfs 与恢复的注释、用例 `remaking_the_pool_with_the_same_filesystem_identifier_leaves_no_root_of_the_old_pool`、C484 用户 2026-09-22 在三条路里选 ① |
| 6 | 出局的判据是不是同样打中乙 | **K1（新旧混合）不中乙；K3 / K4 / K5 同样中乙；K6 窗口乙更差** | 1.4 的表。唯一把甲排出去又没打中乙的是 K1，且只在 fsid 相同时成立 |
| 7 | 「做完 I-7.7 红，不分辨臂」 | **站住**；但其中「fsid 不同也红」**够不着** | 3.3【量】：前 8 字节不同则违例 0（乙 31 195 态、new-then-sysconf 7 874 态、sysconf-first 124 499 态） |
| 8 | （第一轮没有）用户前提口径下，fsid 相同的甲 | **有一种写序量出 0 个丢确认写的态** | 3.1【量】latest-root-last：u = 5 / 6 / 23 的 188 / 188 / 161 态，`mixed_older` 0、`old_other` 0，仍有 `mixed_last` 83 / 83 / 56；要 mkfs 读旧池、屏障 5–6 道；违反的是乙那句「不许新旧混合」 |
| 9 | 乙的「附带两样」不分辨臂 | **站住** | 甲的提交段同样出（`research/prompts/m3-close-c656-r1-opus-s2-output.md:170`） |
| 10 | U3「完整的新池」40 态 | **条文候选 X（宽）/ Y（严）** | 2.2；40 态是已定项 8 射程里现成的「某盘一份自证系统配置都没有」形，根 / journal / 单元都已持久 |

给主 agent 判的一个分叉（不是本腿能定的）：第一轮把甲排掉用的是乙的后半条（「不许挂上旧池或新旧混合」），而用户 2026-10-20 的前提只是「不接受挂上丢了最后几版已确认写的旧池」。两者在 fsid 相同时分开：按前者甲出局（行 1）；按后者 latest-root-last 在量过的三份历史上没有违例（行 8），代价是 mkfs 读旧池和多出的屏障。选哪条口径是用户的事。

## 五、要主 agent 跑的命令

本腿的数是分阶段各跑一次 `rerun.sh`（`RERUN_STAGES=b` / `a` / `c`，编译在包装外面做、`RERUN_PREBUILT_BINARY` 指进去）得到的；没有把三个阶段合在一条命令里从头跑过（`rerun.sh` 只过了 `bash -n`，模板自带的 `--selftest` 没跑）。主 agent 复核按规则在副本里整条跑，从头编要编 `librocksdb-sys`：

```
mkdir -p <冻结副本>/.claude && rsync -a crates Cargo.toml Cargo.lock .cargo <冻结副本>/ && cp .claude/heavy-thread-default <冻结副本>/.claude/     # 工作目录：仓根
RERUN_SEED_TARGET=<别处编好的 target，可不给> RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-c656-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

线程 31（脚本里写死 `capped.sh 31`）。默认阶段 `b a`（各 188 / 7 874 / 31 195 态量级，几分钟）；`RERUN_STAGES=c` 是 sysconf-first 四份历史（124 499 + 124 499 + 124 499 + 279 974 态，机器忙时每份几分钟到二十分钟）。判结果：`<草稿目录>/out/run-*.log` 的 `name=done` 行（去掉 `seconds=`）、`name=count` 行、`name=checker` 行，与模型目录 `out/` 里同名文件（去掉 `run-` 前缀）逐行比；`name=example` 行序号随线程次序变，不比。本腿内存：`/usr/bin/time -v` 量过 fsid 相同 new-then-sysconf 7 874 态一趟 `Maximum resident set size` 917 956 KB（1G 包装下过）；124 499 态的几趟在 2G 包装下过，没单独量峰值。

## 六、没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；U2 与 U4 除 3.3 一段外是攻方腿的格，没碰。
- latest-root-last 里「mkfs 先读旧池」那一段没写：最新旧根的位置与全 0 单元槽由探针从历史里算出来传给写序；没量旧池读不出、旧池自己半截发布、journal 比根新的历史；没量真设备（写零的实现、屏障生效）。
- 没在入库装置上重做：所有【量】都是仓副本上打了补丁的探针（补丁在模型目录）；按三方规则它们是「副本上的数」，要引进 kb 须入库装置重做。
- 没量 fsid 前 8 字节不同在别的几何（旧 io_min 65536、u 取别的值）上 I-7.7 是不是仍 0；只有 `fpdiff` 这一份。
- 没验乙作废之后与在飞的 α' 离线出路（批 8）的交互：`research/prompts/_m3-close-sysconf-r3-body.md:15` 写 α'「只重写不自证的槽」，乙作废后四个槽全零、没有可抄的自证槽，推的是 α' 在这些态上无从下手，没量，α' 也还没落进 `crates/`。
- 没跑重型测试、没调 checker 档测试目标；编译只 `cargo build --release -p c656-attack-probe`（包装外面，线程 31）；探针二进制全部经 `run-with-memory-cap.sh`（1G / 1500M / 2G）起。
- 过程里有一次错：把一条排队等准入的 `rerun.sh` 当成还在排队而去停它，其实它刚放行、已跑了一个 188 态的小跑；停掉重跑，数以后面各次为准，前面那次的产物没用。
- 前台命令被工具超时自动转后台过几次（排队等准入）；没有用 `run_in_background`，没留下在跑的进程。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c656-r2-sonnet/` 里只留各次 `run*/out` 与构建、排队日志；编译目录（含 715 MB 的 target）、冻结副本、每次 `rerun.sh` 的仓副本 `run*/tree`、RocksDB 库目录、编好的二进制都已删。入库的产物在 `research/prompts/m3-close-c656-r2-sonnet-model/out/`（每趟只留 `name=run|done|count|checker` 行）。

## 七、产物哈希

```
ffcf3da76ab2c3eab67675e33f8cdc4b5b5c5cd23c48b4f5f2d4ff0b1241fdbc  out/o512-u23-fsame-latest-root-last.txt
3e72e3efb0f53a930230d6aa3ec5c45fd72e207658dae295a981a405ff51cf9c  out/o512-u5-fdiff-invalidate.txt
1f1eed3a4423641e6917c00485d5701e533872396904d2e4ba20babf96b1dcc2  out/o512-u5-fdiff-latest-root-last.txt
3c433b0d9d34fe867695e438b64228402c58bf92efcd52ceb33fd440d03a1a4e  out/o512-u5-fdiff-new-then-sysconf.txt
a5dfbc3c68cd09ded7e5f5cb917193bc166ed39a725a0d4438c462cecb665c08  out/o512-u5-fdiff-sysconf-first.txt
cbf6b223e564958386201b67ebd5e4884bb03cd1cde4b58228a46940a7bb3180  out/o512-u5-fpdiff-invalidate.txt
b72f2c6cd511432075bc0a34e2ec90fb96866058f803ed6e5a2476f3b01179a5  out/o512-u5-fpdiff-new-then-sysconf.txt
6b04ceaa86a524dd6fbebdeb7c82ea4b8f0b630ea1127fb0a510fa906177c1ac  out/o512-u5-fpdiff-sysconf-first.txt
a21d0ac6b69e8e1a22eb6336a3ef154ec5dbafd98425b3a17b3508112345c8af  out/o512-u5-fsame-latest-root-last.txt
99598ee71853b368e8ddc022e5d23e91758aa75e75d882ebeb622062cbaa9f2f  out/o512-u5-fsame-sysconf-first.txt
aa1dbfdf785c008a4e1cc9d0848537980c6df7aaa3dde37039608bb48c3fbadf  out/o512-u6-fsame-latest-root-last.txt
bee796bc05179c759efdf6336ca888e46404d3c2afd691ca599ea69c123230c6  out/o65536-u5-fdiff-sysconf-first.txt
70feb3ed9958689e7d5d0c72f67a4e0d493f77b35f7b4cf83a93291ee62a8fd2  out/o65536-u5-fsame-latest-root-last.txt
fd720cd721b6cfd3310f0a3bad51c1a408e15b69422687b0db56894263d4169f  （仓外）编好的探针二进制 probe-built，只供 RERUN_PREBUILT_BINARY 省编译，交回前已删
```

## cite-check 末行

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-close-c656-r2-sonnet-output.md --root . --background research/prompts/_m3-close-c656-r2-background.md | tail -1
  ✓ 核了 20 处引文、14 处裸行号，对上 20 处，没判 14 处（其中裸行号没有引文可比 14 处；逐处列在上面）
```
