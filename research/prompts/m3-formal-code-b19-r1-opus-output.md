# m3-formal-code-b19-r1 云端攻方（Opus）报告（2026-10-04）

攻击面：K1（f2 的「空」与开放段取法、几处近似在合法历史上会不会误红；判不了那几格是不是过宽）、K2（除 H1 之外哪些坏实现判成立；GPU 与 CPU 会不会分歧）。
代码引文按冻结副本 `/tmp/claude-1000/m3-formal-code-b19-r1/tree/` 的路径与行号；量出的数全部是**副本上的数**（冻结副本 + 我的补丁，见下），不是入库装置上的数。checker 档、GPU 一条没跑。

## 复跑

```
REPO=/home/fy5090/code/singlefs WORK=/tmp/claude-1000/<新目录> bash research/prompts/m3-formal-code-b19-r1-opus-model/tools/rerun.sh
```

工作目录 `$WORK` 须不存在；脚本把冻结副本拷进去、打 `patches/b19-opus.patch`、装 `prototype/`，逐格跑，输出落 `$WORK/out/*.log`，汇总行落 `$WORK/out/all-name-lines.txt`。`ONLY=<格名片段>` 只跑名字含它的格。全程约 10 分钟（本机挂钟：f1-sweep 3 分 36 秒、z-default-ring 1 分 43 秒、两格 h2-default-ring 各 46 秒，其余各格不到 1 分钟）。`ONLY=layer0` 那一段在 `/tmp/claude-1000/m3-formal-code-b19-r1-opus/rerun-test` 上试跑过、退 0、输出与下文第 K2-1 节同（试跑目录已删）。

模型目录各文件 sha256（`find . -type f | sort | xargs sha256sum` 原样）：

```
4fe0f2a209535e4125956ccaa7a9a6c6c3c6d58bb2ab240881b79b32e647099a  ./out/bad-h1_from_u_plus_4.txt
872b703bbe003c9d6a30c0f0b3f051edef5d1cecf0ba1fb21e2120d395d65f3b  ./out/bad-ignore_cluster_segments.txt
d5ea2c90833a94dfd7f3d4085ead668dd6824d7f8c5d2f1ef8f43b3f261d5cd8  ./out/bad-skip_first_free_pair.txt
6d90c1629f18c31e0db4156db214dce9b14f844a4afdc355d94d77bde63c0ac1  ./out/f01-1.txt
6519b2abbb90811877dcf3a1223e9cb1a4e538318c759a4f3ddb2bf51f635dfb  ./out/f0-2.txt
000c9a7735908dd519dff78439c7fcab81a7bf19893c623835e8e700a650906a  ./out/f1-sweep1.txt
da4c98a2447e33d1cd45a93bfa79e1f20cc7879c592ab399deb01c344ebae62a  ./out/f1-zero.txt
45a92ec37cae7434f81a6411c5ef696ea5ecf4911a5ce925dbea728508f70c87  ./out/gpu-1.txt
1842e9c08f379328575e5ef186e66debf05b2d03a6329e34eddd172a4c3cdf21  ./out/gpu-mutant.txt
720dd9a1077616bcb9693b699ffa1983c312ccf7c20a1c99fb4510af8d6c8212  ./out/h2-default-s0.txt
3fd7e37f2759e8e5a9939229e4238bbd9b2e4c8fd32c81703344acd9c2ab6720  ./out/h2-default-s1.txt
ec5466c5bd1445045b3e2cc9f9a1ffacd49b247a7dc976afb6ee33aadf77f9b2  ./out/h2-slack0.txt
a1542c372613fe24223a7fded2e602c7a7ef34f5f1aab60a73c4a29682192321  ./out/h2-slack1.txt
16fa711709d86fa2c3101882e76c41d7db3733d03282bcf6dcfb478f81b9d101  ./out/h2-slack3.txt
750c880789bc6c6b3e67db4b7ae8e6412be8d0d52248e9eb9338278afe09186f  ./out/l0e-m9-s0.txt
41393ac40727b57912293716d641d370f68d3f2001f26f5a336ebf1c18f3aff7  ./out/l0e-m9-s1.txt
224d4f22aeb8fc1ff9d1544ffd9abd8f4616faa69396e6eaada88a5fb342b233  ./out/z-1.txt
2cad03e744e02d42deda03fca5573f8f627330f60282a954f10f2bad845c8fbc  ./out/z-2.txt
9bd196f35f24b6e36cb280992063c2a56d6b0499c3b4f87bc20f44f397e37132  ./patches/b19-opus.patch
bedd46af494586e3fdd8d3b4c3e5d79962e3163c7b96605b3533e553620b824b  ./prototype/Cargo.toml
f1a58afde5ae89930fe00a20a3f83bb0352e46a86353845b02e69e027f81a033  ./prototype/src/main.rs
c82498606fb6464a650f8d907f0def58138374c5f95b80645708d96e6fa8e592  ./tools/rerun.sh
```

`out/*.txt` 是我这一次跑的各份日志里 `name=` 打头的行原样抽出来的（`grep -o 'name=.*'`）。补丁改三处副本文件、加一个探针：`crates/singlefs-core/src/allocator.rs`（坏实现开关 `B19_GATE_SLACK`、`B19_BAD`，不设时与冻结副本逐行为同）、`crates/singlefs-checker/src/walk.rs` 末尾加一个测试模块（WGSL 候选法的 Rust 转写对拍）、`crates/singlefs-harness/tests/b19_opus_probe.rs`（探针，名字不带 layer0）。`prototype/` 是一个链 checker 档库的独立二进制，只调 `enumerate_layer0` / `enumerate_layer0_selecting`，流在原型里自己造。

## 各格判定一览

| 格 | 判定 | 依据 | 量过 / 推的 |
|---|---|---|---|
| K1 合法历史上误红 | **没打中**（造不出误红的合法镜像） | 原实现上 1 MiB 环小盘、默认环 4 GiB 盘合计建成 621 次（各份日志相加，个别几何在两份日志里重复；另 137 次准入拒），I-5.5 判成立 621、红 0、不适用 0，池级 checker 全不红；落点相对 U 只出现 0（140 次）与 4（481 次） | 量过（副本） |
| K1 开放段取法 | **有逻辑缺口、合法历史上没造出**：提交内生块回落（D3 已定项 8 第 2 条）时树表单元不在任何开放段里，f2 仍把它所在的 64 槽段当开放段排除 | 回落要求那一刻每块盘都没有可开段；打了段状态的 617 次里 `open_rel_U=none` 0 次，准入（保留量约 690 槽）先于回落拒 | 缺口是推的；够不着是量过 |
| K1 隔离、抬 F 扣住、回收、非默认环、第一个文件多单元 | **没打中** | 这一类版本的最小落点只可能落在 [U, U+64) 里，那一段里在这一版之前被占过的只有 U、U+1、U+2，三槽都有分配记录（已分配或已释放），f2 当占着；影子账能隔离的、扣住的槽在这一段里都带已释放记录。树 ID 水位取根环 max 让带着被抛弃文件根的池不再属于这一类 | 推的（621 次建成只见 0 / 4，与它相容） |
| K1 判不了那几格是否过宽 | **不过宽**（推的） | 种类 3 不是恰好一条、分配记录树读不全：合法镜像上读不出时别的不变量同读这棵树；我的历史里 I-5.5 报不适用只出现在「没有这一类版本」的状态 | 推的 |
| K2-1 坏实现 H2：回收门槛放松一个 txg | **打中**：I-5.5 判成立，最终镜像上池级 checker 一条不红 | mkfs 后可写挂载 9 次写第一个文件：正确实现落 U+4，H2 落 U；1 MiB 环 2000 槽与默认环 4 GiB 两种几何都是 `i55=holds red=[]`；同一版写流的 13 个崩溃态（只展开第 1 段之后）里根落盘之前的 4 个由 I-2.1 / I-4.8 / I-7.4 红，I-5.5 在 9 个状态上评估、红 0 | 量过（副本） |
| K2-2 H2 的放大形：门槛放松 3 个 txg、ForcedToZero | I-5.5 判成立，别的不变量红 | 放松 3：挂载 2–8 次 I-2.1 / I-4.8 / I-7.4 红，挂载 9–12 次只剩 I-3.1；ForcedToZero 挂载 2、3、9 次 I-5.5 成立 | 量过（副本） |
| K2-3 同一次发布当场释放又重用（落 U+2） | I-5.5 不红（整条报不适用），别的五条红 | ForcedToZero 新池流：数据落 U+2，I-5.5 报「有一版的树表读不出」不适用；I-2.1、I-3.1、I-4.8、I-5.1、I-7.4 红 | 量过（副本） |
| K2-4 跳过第一个合格空对 | 一般判红；**挂载 10 次那一格判成立** | 挂载 1、2、9 次落 U+6、I-5.5 红；挂载 10 次正确落点是已回收的 U，跳过它落 U+4，I-5.5 成立——就是 C689 的 H1 那一族 | 量过（副本） |
| K2-5 开放段算错（用户数据不排除聚簇段） | 判成立，但这一类版本上落点与正确实现逐格相同 | 挂载 1、2、9、10 次落点 4、4、4、0，与原实现同 | 量过（副本） |
| K2-6 GPU 与 CPU 在同一张镜像上分歧 | **没打中**（转写对拍 0 处不同；WGSL 本身没跑） | WGSL 候选法照字面转写成 Rust，与 CPU 逐对扫在 300000 组对抗取样上 0 处不同；阳性对照（去掉向上取偶）22919 处不同。记录装不下时 GPU 置判不了位、不是判成立 | 转写是量过；WGSL 是推的 |

## K1　f2 的现算在合法历史上会不会误红

### 先说结论的骨架：这一类版本上 f2 只看得见 [U, U+4) 那四槽

被判对象把三样近似都写在函数注释里：`crates/singlefs-checker/src/walk.rs:4316`：「这一版自己分配的记录（未释放、分配代等于这一版的 txg）当空」；`crates/singlefs-checker/src/walk.rs:4317`：「开放段盘上没有字段：拿这一版树表单元所在的段当它」；`crates/singlefs-checker/src/walk.rs:4319`：「没有分配记录罩着的槽一律当空」。占着的判法是 `crates/singlefs-checker/src/walk.rs:4331`：「&& (record.is_released || record.generation < version_checkpoint_txg)」。

推的（下面量过的数与它相容，没反例）：属于这一类（认法 ①②）的版本只能是池里第一次发树的那一版，它之前的版本没有用户数据。实现在同一次发布里先取数据单元的落点、再取提交内生块：数据单元的角色排在最前（`crates/singlefs-core/src/transaction.rs:4257`：「roles.push(TransactionUnit::Data(DataUnitIndexInFile(」），落点在装任何单元之前全部取定（`crates/singlefs-core/src/transaction.rs:7745`：「落点先于内容」）。它之前的提交内生块都 bump 在「最低的全空段」里（`crates/singlefs-core/src/allocator.rs:766`：「self.used_per_segment[*segment] == 0」），而 mkfs 的实例表与树表占着 U、U+1、U+2，[U, U+64) 那一段在第一个文件之前从来不是全空段，于是：

- 第一个数据单元一定落在 [U, U+64) 里，正确实现只会落 U（U、U+1 已回收）或 U+4；
- f2 在这一段里看到的：U、U+1 有记录（已分配，或已释放），U+2 有记录（这一版之前已分配，这一版里释放），U+3 与 U+4 起从没分配过；
- 所以 f2 交回的落点只取决于 U 与 U+2 两条记录是不是「这一版自己的」，开放段、隔离、扣住这几样近似在这一类版本上都碰不到落点。

量过：原实现上合计建成 621 次（下表各格相加），落点相对 U 只出现 `data_rel_U=[0]` 140 次、`data_rel_U=[4]` 481 次，I-5.5 判成立 621 次、红 0 次、不适用 0 次，池级 checker 每一次都 `red=[]`。

| 扫的形状（原实现、副本） | 放开扫的用户动作 | 几何 | 建成 / 准入拒 | 出处 |
|---|---|---|---|---|
| 新池流（mkfs → 取号 → 暖机 → 第一个文件） | 单元区槽数 64…1000 | 1 MiB 环 | 3 / 14 | `out/f0-2.txt`、`out/f01-1.txt` |
| 可写挂载 k 次再写第一个文件 | k = 1、4、8 × 单元区 704、768 | 1 MiB 环 | 1 / 5 | `out/f01-1.txt` |
| 可写挂载 k 次再写第一个文件 | k = 1…12 × 单元区 740…1600 步 20 | 1 MiB 环 | 472 / 56 | `out/f1-sweep1.txt` |
| 同上，最后一次挂载里先推 z 次零单元发布 | k = 1…3 × z = 0…30 | 1 MiB 环，2000 / 800 槽 | 124 / 62 | `out/f1-zero.txt` |
| 可写挂载 k 次再写第一个文件 | k = 1…12 | 1 MiB 环 2000 槽 | 12 / 0 | `out/h2-slack0.txt` |
| 同上 | k = 2、3、9、10、新池流 | 默认 768 MiB 环 4 GiB | 9 / 0 | `out/z-1.txt`、`out/z-2.txt`、`out/h2-default-s0.txt` 里原实现那几行 |

几何选 1 MiB 环是为了让盘小到段数少（环不许超过盘的四分之一：`crates/singlefs-core/src/make_filesystem.rs:392`：「if ring_bytes > smallest / 4 {」，默认环最小盘是 3 GiB）。非默认环那一格由 1 MiB 环全部这些历史覆盖（U = 1088），回收翻转点与默认环同（挂载 10 次起落 U，`out/h2-slack0.txt` 第 10 行）。

### K1-a 开放段取法的缺口：提交内生块回落时（推的，合法历史上没造出）

D3 已定项 8 第 2 条给了回落（`.claude/kb/decisions/03-空间分配.md:164`：「提交内生块的段耗尽时回落到该设备内槽号最小的空槽」），实现照做，回落那一刻把开放段清掉（`crates/singlefs-core/src/allocator.rs:1645`：「self.open_segment = None;」）。这时树表单元落在单元区最小的空槽上，不在任何开放段里；而 f2 照样拿树表单元所在的那 64 槽当开放段排除（`crates/singlefs-checker/src/walk.rs:4301`：「first_slot: location.slot - location.slot % CLUSTER_SEGMENT_SLOTS,」）。若这一版整次发布都在回落（发布开始时就没有可开段），数据单元先取到 [U, U+64) 里的最小空对，树表单元随后回落也落进 [U, U+64)，f2 就把数据所在那一段整段排除、交回 ≥ U+64 的落点——误红。

够不够得着：要这一版提交那一刻每块盘上都没有「已分配、隔离、扣住都是 0」的整段。我在 1 MiB 环上把单元区从 64 槽扫到 1600 槽、可写挂载 1…12 次、零单元发布 0…30 次，617 次打了段状态的建成里 `open_rel_U=none` 0 次：单元区 720 槽时第一个文件的准入已经拒（`research/prompts/m3-formal-code-b19-r1-opus-model/out/f01-1.txt:6`：「available: AvailableBytesOnOneDevice(-114729)」），740 槽起才放行，那时至少 11 个整段；挂载增多时准入先于段被占满拒（`out/f1-sweep1.txt` 里 56 次准入拒）。零单元发布不经分配器、不占段。所以按今天的准入，这条缺口在我能造的合法历史里够不着；写死的是「准入保留量约 690 槽 > 段数能被提交内生块占满所需」这一比例，准入式或段长变了要重看。

四句（按「打中之后先问四句」，虽然没造出合法镜像，缺口本身照答）：① 分辨臂：只打 f2「拿树表单元所在段当开放段」这一读法；换成「回落时不排除任何段」（树表单元的位置条目不在任何这次挂载开过的段的段首对齐范围，盘上分不出）判别子看不到，见 ②。② 判别子：回落与否盘上没有字段；checker 只能从「树表单元所在段里有没有这一版之前的已分配记录、有没有这一版之后 bump 的痕迹」间接推，推的、没实现。③ 满足的分句：I-5.5 那一行「不在开放段里」（`.claude/kb/invariants.md:199`：「两槽都空、不在开放段里」）——条款没定义回落时的开放段。④ 跑前条款没给改法，这里不适用。

### K1-b 隔离、抬 F 扣住、回收、第一个文件多单元、多盘（没打中）

- 隔离（影子账）：只被被抛弃根引用的槽 f2 当空（walk.rs:4319 那一句），方向是误红。要误红得让被隔离的槽落在数据单元的落点之下，而这一类版本的落点在 [U, U+64) 里：这一段里能被隔离的只有 mkfs 的 U、U+1、U+2，它们在现行账里都带着记录（已分配或已释放），f2 当占着，与实现一致。被抛弃的第一个文件根引用的 U+4 若被隔离，接下来的写行发布带的水位是根环里的 max（`crates/singlefs-core/src/transaction.rs:5616`：「回退到树表 0 条的一版之后水位带着环里的 max，号高于此前发过的每一个，不重发。」，与 `tests/tree_identifier_watermark_after_the_first_file_root_is_abandoned.rs` 同一条），新实例的第一个文件树 ID 不从 65536 起，不属于这一类。推的；被抛弃根要靠暂时读故障造，我没造。
- 抬 F 扣住：扣住的槽都是回收出来的，带已释放记录，f2 当占着，与实现一致；`raise_rollback_floor` 要一个带文件的现行版本，第一个文件之前抬不了 F。推的。
- 回收之后：f2 把已释放一律当占着，方向是漏判（K2），不是误红；回收翻转点挂载 10 次、零单元发布 22 次附近 621 次建成 0 红，量过。
- 第一个文件多单元：取每块盘最小的那一个，其余是这一版自己的、当空；最小那一个是最先取的。round 自己的两单元用例在；我没另扫，推的。
- 多盘：第一版只许两块盘（`make_filesystem` 断言 2 块），各盘落点必须相同，两盘记录对称；盘不等大时小盘先满就拒（假性 ENOSPC），不会出现两盘落点不同的这一类镜像。推的。

### K1-c 判不了那几格是否过宽（不过宽，推的）

新加的两格是种类 3 不是恰好一条、分配记录树读不全（`crates/singlefs-checker/src/walk.rs:4245`：「"从 mkfs 水位第一次建树的那一版分配记录树读不全：落点算不出",」）。第一个文件那一版恒发一棵分配记录树；读不全只在坏镜像或读故障上出现，那时 I-5.4、I-3.10 这些读同一棵树的不变量同样读不下去。我的历史里 I-5.5 报不适用只出现在「这一版不在」的崩溃态（`research/prompts/m3-formal-code-b19-r1-opus-model/out/l0e-m9-s0.txt:3`：「name=i55 gate_slack=0 evaluated=9 red=0 not_applicable=4」，那 4 个是根没落盘、候选里没有第一个文件那一版的状态）。另见 K2-3：一条更早的候选根树表读不出时，整条 I-5.5 报不适用，哪怕这一类版本本身判过成立——这是「没有违例时整条报不适用」的既有口径，不是这一批新加的格。

## K2　f2 漏判什么

### f2 在这一类版本上的放行集合（推的，下面各格量过的都落在里面）

由 K1 的骨架：数据单元第一个落点 P 只会在 U、U+2、U+4 三处与 f2 相等——P = U 当且仅当 U 那条记录被这一版改写成自己的；P = U+2 当且仅当 U 那条不是这一版的、U+2 那条被这一版改写成自己的；P = U+4 当且仅当这两条都不是这一版的。也就是说 I-5.5 在这一类上只判得出「跳过了一对从没分配过的槽」与「没对齐」，**判不出这一版重用 U、U+2 时回收门槛对不对——两个方向都判不出**：太严的一边是 C689 的 H1，太松的一边是下面的 H2。f2 把「这一版自己的记录」当空（walk.rs:4316），而重用会把旧记录改写成这一版的，门槛过没过在这一版的分配记录里不留痕。

### K2-1　坏实现 H2：回收门槛放松一个 txg（打中，量过、副本）

定义：回收谓词照 D3 已定项 7（`.claude/kb/decisions/03-空间分配.md:133`：「有条目的再要求已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」），实现是 `crates/singlefs-core/src/allocator.rs:1730`：「.filter(|record| record.is_released && record.generation <= floor)」。H2 把这一句改成「释放代 ≤ floor + 1」（补丁里 `B19_GATE_SLACK=1`），别的一行不动。

历史：mkfs，可写挂载 9 次（每次只写行、暖机），第 9 次挂载上写第一个文件（txg 26）。实例 2 写行把 mkfs 的实例表 U、U+1 释放在 txg 3；写 txg 26 之前环里最旧有效根是 txg 2（26 − 24），2 < 3，U 还不可回收，正确实现落 U+4；H2 按 3 ≤ 2 + 1 提前回收，落 U。txg 26 的根盖掉的正是 txg 2 那一槽。

| 几何 | 实现 | 输出原样 |
|---|---|---|
| 1 MiB 环 2000 槽 | 原实现 | `research/prompts/m3-formal-code-b19-r1-opus-model/out/h2-slack0.txt:9`：「name=F1 unit_area_slots=2000 mounts=9 zero_unit_publishes=0 reuse=reuse_window_gated_by_the_rollback_floor open_rel_U=512 cluster_segments_rel_U=[512] txg=26 data_rel_U=[4] tree_table_rel_U=526 i55=holds red=[] detail=」 |
| 1 MiB 环 2000 槽 | H2 | `research/prompts/m3-formal-code-b19-r1-opus-model/out/h2-slack1.txt:9`：「name=F1 unit_area_slots=2000 mounts=9 zero_unit_publishes=0 reuse=reuse_window_gated_by_the_rollback_floor open_rel_U=512 cluster_segments_rel_U=[512] txg=26 data_rel_U=[0] tree_table_rel_U=526 i55=holds red=[] detail=」 |
| 默认 768 MiB 环 4 GiB | H2 | `research/prompts/m3-formal-code-b19-r1-opus-model/out/h2-default-s1.txt:1`：「name=F1 unit_area_slots=211968 mounts=9 zero_unit_publishes=0 reuse=reuse_window_gated_by_the_rollback_floor open_rel_U=512 cluster_segments_rel_U=[512] txg=26 data_rel_U=[0] tree_table_rel_U=532 i55=holds red=[] detail=」 |

最终镜像上 I-5.5 判成立（真被评估过，不是不适用），池级 checker 68 条一条不红。

同一次发布的崩溃态（原型 `prototype/src/main.rs`，流在原型里自己造）：这一版写流 25 个写、切成 4 段 `[20, 2, 1, 2]`，层 0 全量 `closed_form_states=1048583`，超过这一条腿每次约 10⁶ 的线，全量没跑；只展开第 1 段之后的段（第 0 段只以整段持久进入，13 个状态，含「单元与记录都落了、根没落」那一格）跑了一次，作取样看，不是层 0 全量：

- 原实现：`research/prompts/m3-formal-code-b19-r1-opus-model/out/l0e-m9-s0.txt:3`：「name=i55 gate_slack=0 evaluated=9 red=0 not_applicable=4」，没有一条不变量红。
- H2：`research/prompts/m3-formal-code-b19-r1-opus-model/out/l0e-m9-s1.txt:5`：「name=checker-red gate_slack=1 invariant=I-7.4 red_states=4 first=候选根 txg 2 引用的单元已被复用或抹头（校验和对不上或头用不了）」，I-2.1、I-4.8 同样各 4 个状态红；`research/prompts/m3-formal-code-b19-r1-opus-model/out/l0e-m9-s1.txt:6`：「name=i55 gate_slack=1 evaluated=9 red=0 not_applicable=4」。

所以 H2 不是逃逸：根落盘之前那几个崩溃态里 txg 2 还是候选，它的实例表被数据单元盖了，I-7.4 抓得到——前提是入库的层 0 流里有「挂载 ≥ 9 次再写第一个文件」这样的历史（我没查，推的）。但 I-5.5 本身对它一个状态都不红；最终镜像上整套池级 checker 都放行。三条红的各 4 个状态，与根没落盘的状态数（13 − 9 = 4）相同，推的是同一批 4 个，没逐个核。

四句：
1. 分不分辨臂：分辨。今天的 f2 放行 H2；写死 U+4 的旧判据会红 H2（落 U），但同时误红挂载 ≥ 10 次的正确实现；C689 里记的 f3（近似复算回收门槛，推的、没实现）在这一格上能从环长 24 与这一版 txg 26 推出被盖掉那一槽是 txg 2，门槛 = 2 < 3，交回 U+4，H2 红、正确实现成立。
2. 被判的系统看不看得到：做决定那一刻实现看得到环里 txg 2；最终镜像上那一槽已是 txg 26，checker 只能按「环满且没有洞」推回来，环上有洞或挂载时读不出的槽推不回（与 C689 同一个限度）。
3. 满足的是判据哪一句：正文 K2「除了 H1，还有哪些坏实现在这一类镜像上判成立」字面；H2 写的这一版过认法 ①②，I-5.5 判成立，而 D3 已定项 8 第 1 条要的落点（「整跨度都过分配准入合取」，合取 2 是落点谓词）是 U+4。
4. 跑前条款的改法：正文第五节只写「打中而坐实 ⇒ 改法并进下一批实现员」，没给具体改法；C689 的 f3 在这一格上会中（推的），没量。

### K2-2　H2 的放大形与 ForcedToZero（量过、副本）

- 门槛放松 3 个 txg：挂载 2–8 次落 U，I-5.5 成立，I-2.1、I-4.8、I-7.4 红（`out/h2-slack3.txt:2` 那一行 `i55=holds red=["I-2.1: 实例表单元 在盘 0 槽 1088 …` 起）；挂载 9–12 次 I-5.5 成立，只剩 I-3.1 红（`out/h2-slack3.txt:9`）。I-3.1 那一红是记账口径对不上，与落点无关。
- `ReuseWindow::ForcedToZero`（实现自带的只供测试开关，`crates/singlefs-core/src/allocator.rs:834`：「/// 只供测试：释放那一刻就回收、当场可再分配，F 与延迟窗口都不看。」）：挂载 2、3、9、10 次都落 U、I-5.5 成立（`research/prompts/m3-formal-code-b19-r1-opus-model/out/z-1.txt:7`：「name=Z1 unit_area_slots=211968 mounts=9 zero_unit_publishes=0 reuse=reuse_window_forced_to_zero open_rel_U=512 cluster_segments_rel_U=[512] txg=26 data_rel_U=[0] tree_table_rel_U=532 i55=holds red=["I-3.1"] detail=」）。

### K2-3　同一次发布里当场释放又重用（落 U+2，量过、副本）

ForcedToZero 下的新池流（mkfs 同一个进程，txg 3）：这一版先释放 mkfs 的树表 U+2，当场回收，数据单元落 U+2。I-5.5 不红：整条报不适用，理由「有一版的树表读不出：认不出它是不是从 mkfs 水位第一次建树的那一版」（更早的候选根那张树表被数据盖了）；另外五条红（`out/z-2.txt` 第 1 行：I-2.1、I-3.1、I-4.8、I-5.1、I-7.4）。按放行集合推，若那几条更早的候选根已离开根环，I-5.5 会判成立（P = U+2、U+2 那条记录是这一版的）；没造。

### K2-4　跳过第一个合格空对（量过、副本）

`B19_BAD=skip_first_free_pair`：挂载 1、2、9 次落 U+6，I-5.5 红（`out/bad-skip_first_free_pair.txt:1` 起）；挂载 10 次正确落点是已回收的 U，跳过它落 U+4，I-5.5 成立（`research/prompts/m3-formal-code-b19-r1-opus-model/out/bad-skip_first_free_pair.txt:4`：「name=F1 unit_area_slots=2000 mounts=10 zero_unit_publishes=0 reuse=reuse_window_gated_by_the_rollback_floor open_rel_U=576 cluster_segments_rel_U=[576] txg=29 data_rel_U=[4] tree_table_rel_U=594 i55=holds red=[] detail=」）。这一格与写死 U + 4（`B19_BAD=h1_from_u_plus_4`，四格全判成立，`out/bad-h1_from_u_plus_4.txt`）同属 C689 的 H1 一族，不是新东西。

### K2-5　开放段算错、只在一块盘上挪（没打中）

- 用户数据不排除聚簇段（`B19_BAD=ignore_cluster_segments`）：挂载 1、2、9、10 次落点 4、4、4、0，与原实现逐格相同（`out/bad-ignore_cluster_segments.txt`）。这一类版本的落点在 [U, U+64)，开放段在 U+64 以上，算错也碰不到 t1；t2–t12 不判（条款射程）。量过。
- 只在一块盘上挪：实现取落点时各盘答案必须一致、一个 Placement 两盘同槽，造不出合法的单盘偏移；若坏实现只把盘 1 那份写到 U+6，盘 1 的记录（不论记在 U+4 还是 U+6）在 f2 下交回 U+4，判红。推的。

### K2-6　GPU 与 CPU（没打中；WGSL 没跑）

`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1812` 不逐对扫，只试 U、开放段末端、每条被占记录末端向上取偶这几个候选。我把它照字面转写进 walk.rs 副本的测试模块，与 CPU 的 `first_data_unit_slot_the_allocation_records_leave_on_a_device` 在 300000 组对抗取样上逐例比（记录起点从 U − 8 起、跨度取 0、1、2、3、64，分配代取 txg − 1、txg、txg + 1，三分之一已释放，五分之一在盘 1，开放段取无或 U − 64 … U + 192 五档）：`research/prompts/m3-formal-code-b19-r1-opus-model/out/gpu-1.txt:1`：「name=gpu-transliteration cases=300000 disagreements=0 first=None」。阳性对照：转写里去掉「末端向上取偶」，`out/gpu-mutant.txt` 报 `disagreements=22919`。读代码：记录多于 2048 条时 GPU 置判不了位（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:102`：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (20u << 8u);」），不是判成立；`c_alloc_record_count` 在 I-5.5 之后的几条判法里都是先清零再读（推的，按函数逐个看过清零处，没跑），I-5.5 清它不影响后面几条。限度：比的是我的转写，不是 naga 编出来在 GPU 上跑的那一份；vec2 拼 64 位的进位没覆盖到（取样槽号都在 32 位内）。

## 改法（我提的，只在我的模型上量过、被攻过零轮；下表每格都是推的）

| 改法 | 修哪一格 | 在打中的格上 | 标 |
|---|---|---|---|
| 甲：树表单元所在的 64 槽段里有这一版的数据单元时，不把它当开放段（D3 已定项 8 第 2 条：数据单元从不落进这次挂载开过的段，同段共处只可能是回落） | K1-a 回落缺口 | 回落在 [U, U+64) 时 f2 不再排除数据所在段，交回数据单元那一对；没回落时开放段在 U+64 以上，结果不变 | 推的（没实现、没跑；合法历史上也没造出回落） |
| 乙：C689 的 f3（近似复算写这一版那一刻的回收门槛）照两个方向用：被这一版改写成自己的那几条记录，若在前一条候选根的分配记录里是已释放、释放代 r，要求 r ≤ max(F, 这一版 txg − 根环槽数)；不满足判红 | K2-1 H2、K2-2；同时是 C689 H1 那一格的改法 | 挂载 9 次那一格：环长 24、txg 26 ⇒ 门槛 2 < 3，H2 的 U 判红，正确实现的 U+4 成立；挂载 10 次：门槛 5 ≥ 3，正确实现的 U 成立 | 推的（环上有洞、挂载时读不出的槽、前一条候选根已离环时推不回，要另记判不了） |
| 丙：不改 I-5.5，在 C689 里补记「太松一边（H2）I-5.5 同样判不出，靠层 0 里根落盘之前的崩溃态由 I-7.4 抓；前提是层 0 流里有挂载 ≥ 9 次再写第一个文件的历史」 | K2-1 的记账 | 不改判定 | 推的（层 0 流里有没有这段历史我没查） |

## 没打中的形状与取样范围

- 合法误红（K1）：新池流单元区 64…1000 槽；可写挂载 1…12 次 × 单元区 740…1600 槽（步 20）；挂载 1…3 次 × 零单元发布 0…30 次 × 2000 / 800 槽；默认环 4 GiB 上挂载 2、3、9、10 次与新池流。合计原实现建成 621 次、准入拒 137 次，I-5.5 红 0。没扫：第一个文件多单元（只靠 round 自己那条两单元用例）、盘不等大、128 MiB / 1024 MiB 环（只有 1 MiB 与 768 MiB 两种）。
- 回落（K1-a）：617 次打了段状态的建成里 0 次回落；准入在单元区 720 槽时仍拒，740 槽起放行。
- 隔离、抬 F 扣住：没造历史。隔离要靠暂时读故障让一条根先读不出、后读得出；抬 F 要带文件的现行版本，第一个文件之前调不了 `raise_rollback_floor`。
- 坏实现（K2）：H2 放松 1、3 个 txg（挂载 1…12 次）、ForcedToZero（新池流与挂载 2、3、9、10 次）、写死 U+4、跳过第一个空对、不排除聚簇段，各在 1 MiB 环 2000 槽上挂载 1、2、9、10 次。
- GPU：只比了转写，300000 组。

## 这一条腿自己的限度

- 全部数在副本上：冻结副本加 `patches/b19-opus.patch`；不设 `B19_GATE_SLACK`、`B19_BAD` 时补丁不改实现行为，但我没在冻结副本原样上重跑一遍对照。
- 层 0：H2 那段历史全量 1048583 个状态，超过约 10⁶ 的单次线，没跑；只跑了 13 个状态的取样（第 0 段不展开），两次合计 26 个状态。按取样 13 个状态 0.3 秒粗估，全量一次约 6.7 小时（8 线程），这个估法只有一个点。
- GPU、checker 档一条没跑；WGSL 的结论只到「我的转写与 CPU 一致」。
- 「这一类版本的落点只会在 [U, U+64)」是从代码推的，621 次建成与它相容，我没找到反例，但被抛弃根、读故障那几族历史一条没造。
- 一条腿只抽了一次；「没打中」那几格按规则不算定论。

## 要主 agent 跑的命令

1. 复跑我的全部格（副本，约 10 分钟，前台不超 4 分钟的限制我跑不了整段）：仓根下 `REPO=$PWD WORK=/tmp/claude-1000/<新目录> bash research/prompts/m3-formal-code-b19-r1-opus-model/tools/rerun.sh`，内存上限由脚本里 `CAP`（缺省 4G）、线程 `THREADS`（缺省 8）定；看 `$WORK/out/all-name-lines.txt`，与本报告引的 `out/*.txt` 逐行比（`h2-slack1` 第 9 行要是 `data_rel_U=[0] … i55=holds red=[]`，`layer0-sample-slack1` 要有三条 `checker-red`）。
2. 若要 H2 那段历史的层 0 全量：原型里 `STATE_CAP` 写死 10⁶，全量要改它，约 1048583 个状态、粗估约 6.7 小时，归主 agent 定要不要跑、在不在入库装置上做。

## 没做什么

- 没判正推那一腿的格（K3、K1 的逐句核）；没替主 agent 采纳任何改法；没碰仓内 `crates/`。
- 没跑 checker 档、GPU、层 0 全量、门禁。
- 没读禁读清单里的文件；起编译前看到别的会话的 cargo 在跑（含正推腿的副本编译），各自用自己的目标目录，没等锁。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b19-r1-opus/`：交回前删掉仓副本 `ws/`、编译目录 `target/` 与做补丁用的 `pa/`、`pb/`；留下各次日志（`*.log`），模型目录 `out/` 里的行都抽自它们。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b19-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b19-r1/tree --root . --background research/prompts/_m3-formal-code-b19-r1-background.md` 末行原样：

```
  ✓ 核了 30 处引文，对上 30 处，没判 0 处（逐处列在上面）
```
