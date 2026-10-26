# m3-close-c506-r3 攻方腿（Opus）报告（2026-10-23）

轮名 m3-close-c506-r3，腿 = 云端攻方。被判对象 = 第二轮判决 `research/prompts/m3-close-c506-r2-main-verification.md` 第三节「第三轮攻的形态」。攻击面 = 正文第五节云端攻方那一行：X1 已定项撑不撑得住、X2 未定项清单漏了什么、X3 前两轮的数能不能核。不重复前两轮攻过的角度。引 kb 条款写 kb 文件自己的行号（现查主工作区，`grep -nF` 定位）。

## 复跑

模型目录 `research/prompts/m3-close-c506-r3-opus-model/`，三份纯 Python 原型（不编译、不调 crates）：

- `src/geometry.py`：X3 的形状计数、min 公式各段、S ≤ 85、有效槽距上界；X2 的「按 D2 已定项 19 取槽距时 mkfs 写得出哪些 S」。
- `src/ring_alias.py`：X3 第一轮「24 / 24 判 I-7.17 红」换一条路复核：只按条款的落点公式在两块盘上摆根，逐状态按 I-7.17 字面判。
- `src/tree_counts.py`：在冻结副本上数 X2 / X3 要的落点（77 处、引「已定项 1」的位置、format-const 标记、造边上 S 池的用例与装置、GPU 字面量、prior-art 与 E50 页的行、E50 扫描集合）。

冻结副本：开工时从主工作区（HEAD 841b3beab 加未提交改动）拷出的 `crates/` 下全部 `.rs` / `.wgsl` 与模型读的六份文件，共 612 个文件，在草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c506-r3-opus/frozen`（整树摘要见 `out/frozen_digest.out`）。

复跑命令（仓根执行）：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-c506-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-c506-r3-opus/frozen --work <新的草稿目录> --model research/prompts/m3-close-c506-r3-opus-model
```

本腿跑的那一次（`--work /home/fy5090/.local/share/singlefs/work/m3-close-c506-r3-opus/rerun1`），退出码 0，原样输出：

```
name=rerun_compare file=geometry.out same=1 lines=21 sha256=6f35ba4f6fd233411022fd2352d3f8e118a49c84020222bd2f78d67451063b95
name=rerun_compare file=ring_alias.out same=1 lines=5 sha256=67ebe1f59a27fda583806caeed86c86b3d412b8671de4d5809cc26da30689f5c
name=rerun_compare file=tree_counts.out same=1 lines=63 sha256=04f6f12ee69bbfc714d6e0e579b08fdc8d790cdf254d83b3239b4fbffe5345fe
name=rerun_compare file=frozen_digest.out same=1 lines=1 sha256=a200b9e82c1fa600b989fdafa568c2bb5766844c417128ea6fc3c8531ce2200d
```

模型目录每个文件的 sha256：

```
2efd59a56b9f3c057101f424be6fd37f715f22f4b95358b46fb351ac45f6f3bc  src/geometry.py
a9b88e8a50a13d5da786f4a8949589963e01983f6b9c0e73e870d6da67dc048a  src/ring_alias.py
082b586dd016c355c06a4155175bec3c632cdc2e2f64287da8961f58e39de70e  src/tree_counts.py
6169c9e7bf87ba7c7d446696b14580aed5a2fc08ed5d5668185adb28c518ff58  tools/rerun.sh
a200b9e82c1fa600b989fdafa568c2bb5766844c417128ea6fc3c8531ce2200d  out/frozen_digest.out
6f35ba4f6fd233411022fd2352d3f8e118a49c84020222bd2f78d67451063b95  out/geometry.out
67ebe1f59a27fda583806caeed86c86b3d412b8671de4d5809cc26da30689f5c  out/ring_alias.out
04f6f12ee69bbfc714d6e0e579b08fdc8d790cdf254d83b3239b4fbffe5345fe  out/tree_counts.out
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 一段。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X1-甲 | **打中（字面，现查）** | 被判形态只换掉 S 那一格的前半段，同格后半句「具体取多少不是格式决策……取值由第一版实现按 D25（目标负载优先级） 的负载形态定」原样留着；新加的「mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，结果小于 5 拒绝 mkfs」把一条按几何算、不按负载算的取值式写进格式条款，与留下的那半句在同一格里说反话 |
| X1-乙 | **打中（推的，第一版够不着）** | 「P 与第一版常量 3 不等整池拒」与同一张表的「P 素数且 P > devs」、D2 已定项 5「P 素数且大于任何合理设备数」放在一起：devs ≥ 3 时没有一个 P 两条都过；而 D2 已定项 6 / 18 在系统配置里声明 w_max = 4，D2 已定项 20 把条带表开线定成取新树 ID（D15 已定项 9 判 compat）。第一版 mkfs 断言恰 2 块盘，今天造不出这一盘 |
| X1-丙 | **打中（字面）** | 读者整池拒的那张表现在住 D22 已定项 16 第 1 句与 I-7.13「共八项」；被判形态把 P、chunk 与 S × 槽距三判写进已定项 2 的 S 格，两处清单对不上；已定项 16 第 3 句仍把 P、chunk 叫「第一版的 mkfs 常量」 |
| X1-丁 | 没打中（量过） | 「小于 5 拒绝 mkfs」与 D2 已定项 19 射程「固定结构互相重叠才拒」：按 D2 已定项 19 取槽距、过 I-7.5 对齐判时 io_min 只剩 2 的幂，「小于 5」单独拒掉的 io_min 0 个 |
| X1-戊 | 没打中 | 量纲：S × 槽距 ≤ P × chunk、min 式、钉值 3S − 1、偏移 363 / 367 与 layout 字段表逐个对上 |
| X2-1 | **漏** | P 要钉成读者整池拒的常量，`ROOT_RING_PRIME_STEP` 与 layout P 那一行都没有 format-const 标记（chunk 有）；第二轮只列了 S 两条边的标记 |
| X2-2 | **漏** | 引错「已定项 1」的 S 相关位置有 11 行、7 个文件，第二轮只点了 `root_ring.rs`、`lib.rs`；另 5 个文件是 checker `lib.rs`、core `system_configuration.rs`（2 行）、`recovery.rs`、harness 用例文件头、e180 装置；其中几行引的正是要被换掉的旧句 |
| X2-3 | **漏（与被判句二选一打架）** | 今天靠 mkfs 参数造 S = 4 / 16 池的：21 个用例文件（其中 2 个在 checker 档，含崩溃点重放登记项 `instance-table-sweep`）、13 条 C568 用例、5 个实验装置；被判形态让 mkfs 自己写 min(8, …)，槽距 4096 时只写 8。保留调用方给 S 的入口，「所写的值不取两条边」对入库 mkfs 就是假话；去掉它，这些池造不出来 |
| X2-4 | **漏** | GPU 恢复侧 `crash_judge_common.wgsl` 第 113–114 行把 P × chunk、基址写成 `3145728u`、`1048576u` 字面量，正是新判的右边；「GPU 字面量钉不住」那一行没列 |
| X2-5 | **漏** | `.claude/kb/prior-art.md` 第 657–660 行说本工程「区域多大不是格式常量」、「抬槽宽 = 减槽数」要先有一条把区域大小固定的条款「而仓里没有这条」；被判形态正是那一条，这句变成假话（同一段还引着早已作废的「下界 1」） |
| X2-6 | **漏** | E50 页第 91、118 行（「得到 256 槽」「1..16 怎么取都不碰任何一条边」）与被判形态换上的依据句对不上，页上的「影响的决策」没有回看行；256 本身是扫描集合缺 128 的产物（X3-丙） |
| X2-7 | **漏** | I-7.13 那一行写「共八项」、成员表里没有 363 / 367，加三判要改行文；同一行的 `physical_block_size` 下界还写 457（根记录已是 466，与本轮无关，顺带记） |
| X2-8 | **漏** | D2 已定项 19 射程「S = 8 时槽距 384 KiB 收、512 KiB 起拒」：被判形态下 io_min 512 KiB 写 S = 6、收；这句的例子与 C568、里程碑 03 那一行要跟着改 |
| X2-9 | **漏** | 第二轮两张表没有一处提 `.en.md` / `.ja.md`：D22、D16、D2、E50、layout、prior-art、checks-owed 都有三语 |
| X2-10 | 答问（量过） | 按 D2 已定项 19 取槽距、过 I-7.5 对齐判：mkfs 只写得出 8（io_min 512 B..256 KiB，10 个）与 6（512 KiB）；1 MiB 被槽距上界拒；「小于 5 拒」这一支一个 io_min 都轮不到它单独拒 |
| X3-甲 | 核得上 | 第一轮「24 / 24 判 I-7.17 红、对照干净」：换模型逐状态判，别名臂 201 / 201 个状态全红、从 mkfs 刚写完（txg 0）就红，四条对照臂 0 红；相叠的四对槽与第一轮 `name=history_geometry` 那一行逐字相同 |
| X3-乙 | 核得上 | S ≤ 85（3 × 85 − 1 = 254，3 × 86 − 1 = 257） |
| X3-丙 | 改写句核得上；**E50 页自己的 256 核不上** | 粗粒度 S ≤ 25、散开 S ≥ 10 按 13 / 34 块与 1000 块线算出来对；但 E50 页写的「首个够到 1000 块这条线的 N 是 256」是扫描集合 `[2, 4, 8, 16, 32, 64, 256, 1024]` 跳过 128 的产物，按它自己的式子是 77（它自己单测的文档注释也写 77） |
| X3-丁 | 核得上，另记一处两腿口径不同 | 26429 = 13 × 2033、29744 = 13 × 2288、规则收 9411 / 1184、与入库 mkfs 逐形状相等、错拒 0 都核上；第二轮两腿报的「规则拒而物理不相叠」是两种口径：攻方按根槽字节 13784（pbs 512），正推按区域外延 8052（= 17463 − 9411），判决没写这一分歧 |
| X3-戊 | 核得上 | 槽距 629248..786432 写 4（pbs 512 时 308 个、pbs 4096 时 39 个）；有效槽距上界 786432 = 768 KiB |
| X3-己 | 核得上，口径要补 | 77 = 今天 `RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 命中的行数，其中 5 行是文档注释，代码 72 行；不含三个装置里各自写死 `= 8` 的常量（e168、e180、e183）、harness 模型的别名常量 `MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`，也不含 26 行直接用 format 常量名的 |

## X1 已定项撑不撑得住

### X1-甲 S 那一格：新加的 mkfs 取值式与同格留下的「不是格式决策」打架（打中）

第二轮判决第三节换掉的只是 D22 第 60 行 S 那一格里「上界由 I-7.4…挂载时算；4..16 之间取值不碰任何一条边。」这一段；同一格在这一段之后还有一句，被判形态没动它：

- `.claude/kb/decisions/22-单元原子性怎么合成.md:60`：「具体取多少不是格式决策——格式承诺的是」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:60`：「取值由第一版实现按 D25（目标负载优先级） 的负载形态定」

模型在冻结副本上照第二轮判决做了这次替换，看替换段之后剩下的尾巴（`out/tree_counts.out` 原样一行）：

```
name=d22_s_cell at=.claude/kb/decisions/22-单元原子性怎么合成.md:60 replaced_span_found=1 kept_after_span_has_not_a_format_decision=True kept_after_span_has_load_shape=True
```

替换之后这一格里同时住着「mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，结果小于 5 拒绝 mkfs」与上面两句。打架的三处：

1. 一条 S 的取值规则写进了格式条款，同一格紧接着说取多少不是格式决策。
2. 取值式的自变量是槽距（几何），留下的那半句说取值按 D25 的负载形态定。按 X2-10 量出来的，照 D2 已定项 19 取槽距时 S 由 io_min 唯一定（8 或 6），负载一个变量都进不去。
3. 「格式承诺的是『S 是系统配置字段 + 挂载时判区间』」是一张列举；被判形态在同一格又让格式承诺 P、chunk 等于常量与 S × 槽距 ≤ P × chunk，列举少了这三样。

D15 把这件事的后果写死了：`.claude/kb/decisions/15-格式冻结政策.md:173`：「已冻结组件的选取规则本身也被钉死」。系统配置在第 1 层；min(8, …) 一旦写进 D22，冻结之后要按 D25 改 S 的取法就得走新格式版本号，与留下的那半句正相反。

具体的冲突（读条款的人就是判别子）：实现者照留下的那半句，按某个负载在槽距 4096 的池上让 mkfs 写 S = 12。12 × 4096 = 48 KiB ≤ 3 MiB、12 ∈ [4, 16]，被判形态的读者全收，格式上合法；照新加的那半句，mkfs 只许写 8。两句对同一次 mkfs 给出不同的 S。代码那一侧把留下的半句抄成了 S 不是格式常量的理由（`crates/singlefs-format/src/lib.rs:296`：「S 自己不是格式常量」），见 X2-2。

四句：

- 分不分辨：分辨「被判形态原样」与「被判形态把 mkfs 那半句挪出格式条款，或同时改掉留下的那半句」两种写法；读者那半边两种写法相同，不受影响。
- 判别子当时看得到吗：条文对条文，两句都在同一格，读的人都看得到。
- 满足的分句：X1「被判形态哪一句与它们字面或量纲打架」，对象是 D22 已定项 2 本格。
- 跑前改法：第二轮给 min 公式的两个改法（门槛改成小于 5；或 mkfs 判槽距等于 D2 已定项 19 的式子）都只动 min 式的值域，在这一格上照样中。

自提改法（只在我的模型上量过、被攻过零轮；这一格连模型都不涉及，是条文推的）：格式条款只留读者那半边（读者收 S ∈ [4, 16]、P 与 chunk 等于第一版常量、S × 槽距 ≤ P × chunk），把「格式承诺的是」那句补成这几样；「mkfs 写 min(8, …)」挪进实现欠或 D25 那一侧，不写进格式条款。

### X1-乙 P 钉成 3 与「P 素数且 P > devs」（打中，推的；第一版够不着）

条款原文：

- `.claude/kb/decisions/22-单元原子性怎么合成.md:58`：「区域 r 落在 r × P × chunk，P 素数且 P > devs」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:350`：「取 3 是第一版两盘下满足…的最小素数」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:70`：「约束本身还要不要，仓里一处都没写过」
- `.claude/kb/decisions/02-RAID条带策略.md:99`：「P 素数且大于任何合理设备数」
- `.claude/kb/decisions/02-RAID条带策略.md:110`：「上界 4 是系统配置声明的常量」
- `.claude/kb/decisions/02-RAID条带策略.md:302`：「devs 上限 2，直到条带表开线」
- `.claude/kb/decisions/02-RAID条带策略.md:329`：「w ≥ 3 开线时取新树 ID 与新种类码」
- `.claude/kb/decisions/15-格式冻结政策.md:194`：「留了开放树表 → compat」
- `.claude/kb/decisions/02-RAID条带策略.md:97`：「加盘 / 换盘之后不重放置根环区域」

被判形态加的是「读者解槽时读 P（偏移 363）与 chunk（偏移 367），与第一版常量（3、1 MiB）不等一律整池拒」。

两段历史（都是条带表开线之后的版本才走得到，推的）：

1. 开线后的 mkfs 在 3 块盘上建池。照第 58 行与第 350 行的取法，P 取大于 3 的最小素数 5。系统配置里写 P = 5。第一版读者（被判形态）整池拒。条带表开线按 D2 已定项 20 是取新树 ID，D15 已定项 9 把新增一棵 keyspace 判 compat：按判定表这类池第一版读者本该读得了（或至少不按 incompat 处置）。
2. 开线后的版本在第一版建的池上加第三块盘（D2 已定项 4 的几何再求值）。根环不重放置，P 照旧是 3，devs 变 3：第 58 行的「P > devs」不成立；第一版读者照被判形态全收。

合起来：devs ≥ 3 时，第一版读者收下的池都违反第 58 行，满足第 58 行的池第一版读者都拒；D2 已定项 6 / 18 在系统配置里声明 w_max = 4，格式自己就预告了 devs ≥ 4 的池。被判形态没有选下面三条出路中的任何一条：删掉「P > devs」（C217 的账）、把条带表开线改成 incompat、或者读者另判 devs ≥ 3 整池拒。

第一版为什么造不出：`crates/singlefs-core/src/make_filesystem.rs:578`：「第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）」（`assert_eq!(devices.len(), 2, …)` 的消息），加盘准入按 D2 已定项 18 拒。所以这一格没有今天能跑的盘面，只有条款对条款。

四句：

- 分不分辨：分辨「钉 P = 3 而留着 P > devs」与「钉 P = 3 同时删掉 P > devs（或写明 devs ≥ 3 走 incompat）」两种写法。chunk 那一半不受这一条影响：没有按盘数选 chunk 的条款（`grep -n chunk` 在 D22 里只命中区域公式、已定项 16 的第一版取值与未定项 6 乙那两行——那里「要求 zone_size ≤ chunk」是 zoned 线的事，zoned 自带 incompat 位；在 D2 里只命中 chunk 映射表与 pitfalls 第 9 条）。
- 判别子当时看得到吗：读者手里有偏移 143 的 devs 与 363 的 P，都在同一槽里；被判形态只判 P，不判 devs。
- 满足的分句：X1 第二问「P、chunk 与第一版常量不等一律整池拒会不会撞上 D2 / D22 里 P 素数且 P > devs、按盘数选 P 一类条款」。
- 跑前改法：第二轮的改法没有一个碰 P > devs。

### X1-丙 读者整池拒的清单分在两处写（打中，字面）

- `.claude/kb/decisions/22-单元原子性怎么合成.md:348`：「与第一版常量（64、1024）不等一律整池拒」——已定项 16 第 1 句是现行的「读者读哪几个池级字段、不等整池拒」清单：根环起点、journal 环起点、R、S。
- `crates/singlefs-checker/invariants/02-根环.md:20`：「共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）」——I-7.13 把成员表绑在第 1 句上。
- `.claude/kb/decisions/22-单元原子性怎么合成.md:350`：「P = 3、chunk = 1 MiB 是第一版的 mkfs 常量」——第 3 句把 P、chunk 定成 mkfs 那一侧的常量。

被判形态把 P、chunk 的整池拒与 S × 槽距那一判写进已定项 2 的 S 格，没碰第 1 句与第 3 句。改完之后：已定项 16 第 1 句与 I-7.13 列的成员里没有 P、chunk；第 3 句仍说它们是 mkfs 常量，已定项 2 说读者见到别的值整池拒。根环起点那一处的先例是把「第一版写 64」与读者拒放进同一句（第 1 句），P、chunk 照先例该进第 1 句或改第 3 句的措辞。

四句：分辨「只改已定项 2」与「同时改已定项 16 第 1、3 句和 I-7.13」；条文对条文，看得到；满足 X1「字面打架」；第二轮改法不碰。

### X1-丁 「小于 5 拒绝 mkfs」与 D2 已定项 19 射程的拒绝判据（没打中，量过）

想打的形状：D2 已定项 19 射程说 mkfs 只拒「固定结构互相重叠」的设备（`.claude/kb/decisions/02-RAID条带策略.md:317`：「io_min 大到固定结构互相重叠的设备 mkfs 拒、不支持」）；被判形态在槽距 (629145, 786432] 上拒，而那里 S = 4 放得下、不相叠。

打不中：mkfs 还有 I-7.5 对齐判，判定宽度是 max(pbs, io_min)（`crates/singlefs-core/src/system_configuration.rs:251`：「根槽与系统配置槽的起始偏移要按的判定宽度：声明的 max(physical_block_size, io_min)」），根环三个区域起点 1、4、7 MiB 要是 io_min 的倍数 ⇒ io_min 只能整除 1 MiB，即 2 的幂。模型扫 io_min = 512 的倍数、512 B..2 MiB（`out/geometry.out` 原样）：

```
name=d2_item19_io_min outcome=refused_by_alignment io_min_count=4084 io_min=[1536, 2560, 3072, 3584, 4608, 5120, '...', 2096128, 2096640, 2097152]
name=d2_item19_io_min outcome=refused_by_spacing_range io_min_count=1 io_min=[1048576]
name=d2_item19_io_min outcome=writes_S=6 io_min_count=1 io_min=[524288]
name=d2_item19_io_min outcome=writes_S=8 io_min_count=10 io_min=[512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144]
```

「小于 5」单独拒掉的 io_min 一个都没有（没有 `refused_only_by_below_five` 行）。被判形态在 D2 已定项 19 的取法下不多拒任何一台真设备；那一支只对不按 io_min 取槽距的调用方（用例与装置直接给槽距）起作用。

### X1-戊 量纲（没打中）

逐个核过：S × 固定结构槽距（字节）≤ P（无量纲）× chunk（字节）；min(8, ⌊P × chunk ÷ 槽距⌋) 是个数；钉值 3·槽 + 区域 最大 3S − 1，与一字节、255 哨兵同量纲；偏移 363（P，4 字节）、367（chunk，4 字节，单位字节）与 layout 字段表（`.claude/kb/layout/01-first-txn.md:141`：「素数步长 P | 4 | 3」，`.claude/kb/layout/01-first-txn.md:142`：「根环 chunk | 4 | 1 048 576」）一致；chunk 字段按字节存，不是 16 KiB 槽号（与根环起点字段不同），比较常量 1 MiB 没有换算错。

## X2 未定项清单的完整性：第二轮「实现欠」「另登欠账」漏掉的落点

对照的两张表：`research/prompts/m3-close-c506-r2-main-verification.md:42`：「两条边的 format-const 标记」（实现欠那一段），`research/prompts/m3-close-c506-r2-main-verification.md:49`：「补上 C_MAX_CANDIDATES = 48u 与 harness u64 位图（S ≤ 21）」（另登欠账那一行）。下面每一条都是这两张表里没有、被判形态一落地就必须跟着动的地方；逐条给冻结副本上的命中（`out/tree_counts.out` 原样行）与原文。

### X2-1 P 的 format-const 标记

被判形态让读者见到 P ≠ 3 整池拒，P 与 chunk 成了同一判的右边。今天 chunk 有标记，P 没有：

```
name=format_const_marker_in_lib constant=ROOT_RING_CHUNK_BYTES markers=1
name=format_const_marker_in_lib constant=ROOT_RING_PRIME_STEP markers=0
name=layout_row at=.claude/kb/layout/01-first-txn.md:141 has_format_const_marker=False text=| 几何 | 素数步长 P | 4 | 3 | D22（单元原子性怎么合成） 已定项 2 |
name=layout_row at=.claude/kb/layout/01-first-txn.md:142 has_format_const_marker=True text=| 几何 | 根环 chunk | 4 | 1 048 576 <!-- format-const: ROOT_RING_CHUNK_BYTES = 1048576 --> | D22（单元原子性怎么合成） 已定项 2 |
```

`crates/singlefs-format/src/lib.rs:311`：「pub const ROOT_RING_PRIME_STEP: u64 = 3;」上面没有文档注释，没有 `format-const:`。门禁 code-source-discipline 的 format-constants 格只绑带标记的常量，P 改成 5 不会红。第二轮实现欠只列了 S 两条边的标记。

### X2-2 引错「已定项 1」的还有 5 个文件

S 那一格在 D22 已定项 2；第二轮实现欠只点了 `root_ring.rs`、`lib.rs` 两个文件。冻结副本上按两种写法（带决策名、不带）搜，S 相关的一共 11 行、7 个文件：

- 第二轮点了的：`crates/singlefs-format/src/lib.rs:293`、`:307`；`crates/singlefs-core/src/root_ring.rs:18`、`:52`、`:289`。
- 没点的：`crates/singlefs-checker/src/lib.rs:125`：「D22（单元原子性怎么合成） 已定项 1 的字段表」；`crates/singlefs-core/src/system_configuration.rs:235`：「挂载时判 4..16 区间，D22（单元原子性怎么合成） 已定项 1 的字段表」；`crates/singlefs-core/src/system_configuration.rs:641`：「按 D22 已定项 1 的字段表判每区槽数 S 的区间」；`crates/singlefs-core/src/recovery.rs:500`：「落在格式承诺的区间之外（D22（单元原子性怎么合成） 已定项 1 的字段表」；`crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:1`：「每区槽数 S 住系统配置（D22（单元原子性怎么合成） 已定项 1 的字段表逐字」；`crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs:66`：「mkfs 的每区槽数 S：D22 已定项 1 字段表」。

其中几行不只是编号错，引的就是要被换掉的那句：`crates/singlefs-core/src/recovery.rs:501`：「上界字段表写的是」后面接旧条款原文；`crates/singlefs-core/src/root_ring.rs:52`：「4..16 之间取值不碰任何一条边」（编译期断言的报错消息）；`crates/singlefs-format/src/lib.rs:298`：「格式承诺的、因而登记成格式常量的，只有区间的两条边。」——被判形态之后格式还承诺 P、chunk 与 S × 槽距，这句变假。第二轮的「placeholder 注释同一次撤」只罩 `lib.rs` 那两段 placeholder，罩不到这些文档注释与断言消息。另有与 S 无关的 3 行（`e156_allocation_basis_counts.rs` 第 93、1174 行与 `e180_…rs:68` 讲区域归属与 R）同样错号，顺带记。

### X2-3 mkfs 的 S 参数与靠 mkfs 造边上 S 的池（与被判句二选一打架）

被判形态写「mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，结果小于 5 拒绝 mkfs，所以所写的值不取两条边」。今天 mkfs 的 S 是调用方给的（`MakeFilesystemParameters` 的 `geometry.root_ring_slots_per_region`），入库用例与装置大量用它在槽距 4096 上建 S = 4 或 16 的池：

```
name=edge_pool_constant at=crates/singlefs-harness/tests/common_root_ring_holes/mod.rs:30 text=pub const SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS: u64 = 4;
name=edge_pool_constant at=crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs:32 text=const SLOTS_PER_REGION_OF_THE_CLAIM_POOLS: u64 = 4;
name=edge_pool_constant at=crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs:42 text=const SLOTS_PER_REGION_OF_THE_BIRTH_POOLS: u64 = 4;
name=edge_pool_users files=21 checker_tier_files=['crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs', 'crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs']
name=crash_replay_item at=.claude/gate.d/crash-point-replay-items.tsv:40 text=instance-table-sweep
name=c568_mkfs_cases_passing_S count=13
name=mkfs_builds_edge_S at=crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:239 text=for declared in [4u64, 8, 16] {
name=mkfs_builds_edge_S at=crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:305 text=for (declared, cleared_bytes) in [(4u64, 16_384u64), (8, 32_768), (16, 65_536)] {
```

实验装置里按 mkfs 参数给 S、期待消息自述取 4 或 16 的：`e158_root_choice_repair.rs`（「S 在 [4,16] 区间内」）、`e165_replay_bounded_enumeration.rs`（「每区槽数 4 或 8」）、`e168_delete_ring_window_borrow.rs`（「每区槽数只取 4、8、16」）、`e180_admission_reading_versus_writable_bytes.rs`（第 532、537 行 `slots_per_region: 4,` / `16,`）、`e183_capacity_read_cache_reserve_and_sweep_costs.rs`（第 1999 行起 4 / 8 / 16 轮着取）。

`common_root_ring_holes/mod.rs` 的池是槽距 4096、S = 4（`crates/singlefs-harness/tests/common_root_ring_holes/mod.rs:30`：「pub const SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS: u64 = 4;」），经 `crash_points_of_the_instance_table_sweep.rs` 进崩溃点重放登记项（`.claude/gate.d/crash-point-replay-items.tsv:40`：「instance-table-sweep」）。槽距 4096 时 min(8, 768) = 8。

两种读法都打中：

- 读成「mkfs 不再收调用方给的 S，自己算」：上面 21 个用例文件、13 条 C568 用例、`system_configuration_slots_per_region.rs` 的 S = 4 / 16 挂得上那一条、5 个实验装置的 S = 4 / 16 臂，都造不出它们要的池；崩溃点重放登记项的状态数与实验产物跟着变。第二轮实现欠只写了「S 按槽距算的运行时构造入口与 harness 模型跟着走」，没写这批怎么办（改走改字节造盘，还是留一个只给测试的入口）。
- 读成「mkfs 默认写 min(8, …)，调用方仍可给 S」：入库 mkfs 今天就写 S = 4 与 16，「所写的值不取两条边」对入库 mkfs 是假话。

四句：分辨两种读法（被判句没写 mkfs 的 S 是否还是参数）；判别子是 `MakeFilesystemParameters` 有没有那个字段，看得到；满足 X2「漏了哪些该随这一改一起动的地方（mkfs、harness 模型）」；第二轮改法不碰。

### X2-4 GPU 恢复侧的 P × chunk 与基址字面量

```
name=gpu_literal at=crates/singlefs-checker-tier/src/crash_judge_common.wgsl:113 text=const ROOT_RING_REGION_STRIDE: u32 = 3145728u;     // P × chunk = 3 × 1 MiB
name=gpu_literal at=crates/singlefs-checker-tier/src/crash_judge_common.wgsl:114 text=const ROOT_RING_BASE_BYTES: u32 = 1048576u;        // 64 × 16384
```

`3145728u` 就是新判「S × 槽距 ≤ P × chunk」的右边，GPU 恢复那一份按它放区域、不读 363 / 367。第一轮登的「GPU 字面量钉不住」那一行与第二轮的补充只列了 `4u` / `16u`、`MAX_ROOTS`、`C_MAX_CANDIDATES`；这两个字面量同样不受 format-const 格约束。

### X2-5 prior-art 那一段变成假话

- `.claude/kb/prior-art.md:657`：「区域多大不是格式常量」
- `.claude/kb/prior-art.md:658`：「下界 1、上界由」
- `.claude/kb/prior-art.md:660`：「才成立，而仓里没有这条」

这一段说 ZFS「抬槽宽 = 减槽数」在本工程不成立，因为没有一条把区域大小固定的条款。被判形态把 P × chunk 钉成常量、又要求 S × 槽距 ≤ P × chunk：读者收的 S 上界 = min(16, ⌊3 MiB ÷ 槽距⌋)，槽距一抬，能收的 S 就降——正是那条等式。658 行引的「下界 1」更早已作废（今天是 4）。第二轮两张表没有它，也没有它的 `.en` / `.ja`。

### X2-6 E50 页与它的「影响的决策」

- `.claude/kb/experiments/50-根环槽数的上下界.md:91`：「要够到跑前定死的 1000 块那条线得到 256 槽」
- `.claude/kb/experiments/50-根环槽数的上下界.md:118`：「S 在 1..16 这个区间里怎么取都不碰任何一条边」

被判形态把 D22 依据段的 E50 那一行改成「粗粒度档 S ≤ 25 不过线，散开档 S ≥ 10 就过线」。改完之后 D22 说的与 E50 页第 91、117–118 行说的不是一回事（页上 256 是 N，按 R = 3 折 S ≈ 85；且 256 本身不对，见 X3-丙），E50 页的「影响的决策」表（`.claude/kb/experiments/50-根环槽数的上下界.md:139` 那一行，D22 已定项 2「支撑」）没有这一次的回看。

### X2-7 I-7.13 那一行

```
name=i713_row at=crates/singlefs-checker/invariants/02-根环.md:20 says_eight_members=True names_prime_step_363=False names_chunk_367=False pbs_lower_457=True
```

成员表写「共八项」，另列 R 与 S；加 P、chunk 与 S × 槽距三判要改成员数与行文（第二轮写了「挂 I-7.13」，没写改不变量表本身）。同一行 `crates/singlefs-checker/invariants/02-根环.md:20`：「physical_block_size ∈ [457（ROOT_RECORD_BYTES）, 槽距]」——根记录已是 466（`.claude/kb/decisions/22-单元原子性怎么合成.md:158`：「合计 466 字节」），与本轮无关，读的时候看见，记一笔。

### X2-8 D2 已定项 19 射程的例子与 C568

`.claude/kb/decisions/02-RAID条带策略.md:317`：「S = 8 时槽距 384 KiB 收、512 KiB 起拒」。被判形态下 io_min 512 KiB 写 S = 6、mkfs 收（X2-10 那一行 `writes_S=6 io_min=[524288]`）。这句是带「S = 8 时」的条件句，字面不假，但它是「mkfs 拒到哪」的那一句（里程碑 03 第 132 行就指它），被判形态之后拒到哪变成「只有 io_min 1 MiB」；第一轮另登的那笔「S = 8、io_min = 512 KiB 被拒」欠账的见证也跟着消失。

### X2-9 三语

第二轮两张表一处都没提译文。被判形态与上面各条要动的 kb 文件全是三语：D22、D16、D2、E50、layout、prior-art、checks-owed 各有 `.en.md`、`.ja.md`（`ls` 现查，各三份都在）。

### X2-10 「小于 5 拒」在 D2 已定项 19 的槽距取法下还剩什么取值（答问，量过）

见 X1-丁贴的四行：io_min 取 512 的倍数、512 B..2 MiB 共 4096 个值，过 I-7.5 对齐判的只有 12 个（512 B..1 MiB 的 2 的幂；2 MiB 也是 2 的幂，但不整除 1 MiB，被对齐判拒）；mkfs 写 8 的 10 个（512 B..256 KiB），写 6 的 1 个（512 KiB），1 MiB 被槽距上界（1 MiB − 4096）拒；「结果小于 5」那一支单独拒的 0 个。所以按 D2 已定项 19 取槽距时 S 只取 {8, 6}，两条边都碰不到，「小于 5 拒」只对直接给槽距的调用方起作用。槽距自由取（不绑 io_min，今天的 mkfs 就是这样）时写出的值见 X3-戊的 `name=min_formula` 行：5、6、7、8 都有，4 那一段（629248..786432）被「小于 5」拒。

## X3 前两轮报出的数能不能核

每个数都用一条不走原腿装置的路重算：`geometry.py` 与 `ring_alias.py` 是纯算术、不调 crates（原腿两份都调入库 mkfs 或读 crates）；`tree_counts.py` 是在冻结副本上按行数。

### X3-甲 第一轮「S = 16、槽距 512 KiB，24 / 24 判 I-7.17 红；对照干净」（核得上）

出处：`research/prompts/m3-close-c506-r1-main-verification.md:31`：「暖机挂载 24 次 24 / 24 判 I-7.17 红，四条对照臂 96 / 96 干净」。

换模型：只按条款落点公式摆根（区域起点 1 MiB + r × 3 MiB、归属 0 / 1 / 0、槽 j 在起点 + j × 槽距；mkfs 清三个区域再种第 0 代根；第 n 次发布写区域 n mod 3 的槽 (n div 3) mod S），每个状态按 I-7.17 字面判（`out/ring_alias.out` 原样）：

```
name=i717_arm arm=aliased_S16_spacing512KiB S=16 spacing=524288 states=201 red_states=201 first_red_txg=0 aliased_slot_pairs=['d0@7340032:r0s12=r2s0', 'd0@7864320:r0s13=r2s1', 'd0@8388608:r0s14=r2s2', 'd0@8912896:r0s15=r2s3']
name=i717_arm arm=control_S8_spacing512KiB S=8 spacing=524288 states=201 red_states=0 first_red_txg=None aliased_slot_pairs=[]
name=i717_arm arm=control_S16_spacing256KiB S=16 spacing=262144 states=201 red_states=0 first_red_txg=None aliased_slot_pairs=[]
name=i717_arm arm=control_S8_spacing4KiB S=8 spacing=4096 states=201 red_states=0 first_red_txg=None aliased_slot_pairs=[]
name=i717_arm arm=control_S16_spacing4KiB S=16 spacing=4096 states=201 red_states=0 first_red_txg=None aliased_slot_pairs=[]
```

相叠的四对槽与第一轮攻方 `research/prompts/m3-close-c506-r1-opus-output.md:150`：「d0@7340032:r0s12=r2s0」那一行逐对相同。补一句口径：别名臂从 mkfs 刚写完（txg 0）就红，此后每个状态都红——盘 0 偏移 7 MiB 那一槽既被读成区域 0 槽 12、又被读成区域 2 槽 0，任何一条根（txg 0 要求槽 0，txg ≥ 1 要求区域 = txg mod 3）都过不了两个读法。所以「24 / 24」与「1 / 1」带的信息一样，挂多少次都是全红，不是抽样出来的比例。第一轮攻方说的对照里有 S = 8 / 512 KiB 与 S = 16 / 256 KiB 两格（`research/prompts/m3-close-c506-r1-main-verification.md:32`：「改字节造的这两份池各挂 24 次 0 红」），这里同两格 0 红，另两格是我加的。没核第一轮四条对照臂各自是什么。

### X3-乙 「S ≤ 85」（核得上）

```
name=pinned_slot_byte largest_S=85 value_at_85=254 value_at_86=257
```

钉值 = 3·槽 + 区域，最大 3(S − 1) + 2 = 3S − 1；255 是哨兵，3S − 1 ≤ 254 ⇔ S ≤ 85。

### X3-丙 「粗粒度 S ≤ 25、散开 S ≥ 10」（核得上）；E50 页的 256 核不上它自己的式子

```
name=e50_first_crossing load=coarse blocks_per_fsync=13 scanned=[2, 4, 8, 16, 32, 64, 256, 1024] first_in_scan=256 first_over_all_n=77 largest_S_below_line_R3=25 smallest_S_on_line_R3=26
name=e50_first_crossing load=scattered blocks_per_fsync=34 scanned=[2, 4, 8, 16, 32, 64, 256, 1024] first_in_scan=32 first_over_all_n=30 largest_S_below_line_R3=9 smallest_S_on_line_R3=10
```

改写句的两个数对：13 × 3 × 25 = 975 < 1000、13 × 3 × 26 = 1014；34 × 3 × 9 = 918、34 × 3 × 10 = 1020。

核不上的是 E50 页：扫描集合跳过了 128（`scripts/experiment-apparatus/e50_ring_slots.rs:46`：「const TOTAL_SLOT_COUNTS_SCANNED: [u64; 8] = [2, 4, 8, 16, 32, 64, 256, 1024];」），粗粒度 13 × 128 = 1664 早过线，页上却写「首个够到 1000 块这条线的 N」是 256、「要够到…得到 256 槽」。装置自己的单测文档注释写着真值：`scripts/experiment-apparatus/e50_ring_slots.rs:167`：「粗粒度下要 77 槽才够到 1000 块这条线」。散开档同理，页上 32 是扫描集合里第一个，真值 30。第一轮判决用的就是页上的数（`research/prompts/m3-close-c506-r1-main-verification.md:24`：「E50 散开档首个够线的 N = 32，R = 3 折 S ≈ 10.7」），S ≈ 10.7 应为 N = 30、S = 10 整。改写句写的是对的那个（S ≥ 10），所以被判形态不受影响；受影响的是 E50 页第 89、91、117 行与它们的三语译文（X2-6）。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 那条「扫描出来的极值落在扫描区间的端点」的同类问题：极值不在端点，但落在扫描网格的空档里。

### X3-丁 第二轮「26429 / 29744 格逐形状相等、错拒 0」（核得上）；两腿的「物理不相叠」是两种口径

```
name=shapes pbs=512 spacings=2033 shapes=26429 rule_accepts=9411 mkfs_today_accepts=9411 rule_minus_mkfs=0 mkfs_minus_rule=0 extent_clean_full=17463 extent_clean_byte_exact=18733 root_slot_clean=23195 rule_refuses_and_extent_clean_full=8052 rule_refuses_and_root_slot_clean=13784 rule_accepts_and_root_slot_collides=0
name=shapes pbs=4096 spacings=255 shapes=3315 rule_accepts=1184 mkfs_today_accepts=1184 rule_minus_mkfs=0 mkfs_minus_rule=0 extent_clean_full=2191 extent_clean_byte_exact=2348 root_slot_clean=2901 rule_refuses_and_extent_clean_full=1007 rule_refuses_and_root_slot_clean=1717 rule_accepts_and_root_slot_collides=0
name=shapes_total grand_total=29744
```

26429 = 13 × 2033、29744 = 13 × (2033 + 255)；规则收 9411 / 1184，与按 `make_filesystem.rs` 的 `fixed_structure_extents` 重写的判法逐形状相等（`rule_minus_mkfs=0 mkfs_minus_rule=0`），规则收下而根槽相撞的 0 个——与攻方 `research/prompts/m3-close-c506-r2-opus-output.md:98`：「judgment_refuses_and_no_bytes_overlap=13784」、`research/prompts/m3-close-c506-r2-opus-output.md:99`：「judgment_refuses_and_no_bytes_overlap=1717」逐数相同，与正推 `research/prompts/m3-close-c506-r2-sonnet-output.md:66`：「physical_clean_full=17463 physical_clean_byte_exact=18733」逐数相同。

两腿报的「规则拒、物理上却不相叠」不是同一个量：攻方按根槽实际写的字节判（槽交错但不撞也算不叠），pbs 512 时 13784 个；正推按区域外延判（区域 0 与区域 2 的范围交上就算叠），17463 − 9411 = 8052 个。我的模型在两种口径下各自重出了对方的数。第二轮判决只采了「错拒 0」（对 mkfs），没写这两个数；主 agent 要是把「读者判得比盘上要的严」登成欠账并带个数，要写明取哪种口径。

### X3-戊 第二轮「槽距 629248..786432 写 4」「有效槽距上界 768 KiB」（核得上）

```
name=min_formula pbs=512 written_S=4 spacings=308 from=629248 to=786432
name=min_formula pbs=4096 written_S=4 spacings=39 from=630784 to=786432
name=effective_spacing_upper_bound bytes=786432 kib=768
```

629248 = 1229 × 512 是第一个大于 3 MiB ÷ 5 = 629145.6 的 512 倍数，786432 = 3 MiB ÷ 4。其余各段（`out/geometry.out` 的 `name=min_formula` 行）：写 8 的 4096..393216、写 7 的 393728..449024、写 6 的 449536..524288、写 5 的 524800..628736、写 3 的 786944..1044480。

### X3-己 第二轮「77 处直接调用 AT_MAKE_FILESYSTEM」（核得上，口径要补）

出处 `research/prompts/m3-close-c506-r2-main-verification.md:25`：「77 处直接填」。

```
name=at_make_filesystem_associated lines=77 doc_comment_lines=5 code_lines=72 files=44
name=at_make_filesystem_format_constant lines=26 files=9
name=local_copy_of_eight at=crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:74 text=const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
name=local_copy_of_eight at=crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs:67 text=const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
name=local_copy_of_eight at=crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:90 text=const SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
name=local_copy_of_eight at=crates/singlefs-format/src/lib.rs:309 text=pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
name=local_copy_of_eight at=crates/singlefs-harness/src/model.rs:1815 text=const MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =
```

77 是今天 `RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 命中的行数（44 个文件），其中 5 行是文档注释，真调用 72 行。它不含：三个装置各自写死的 `= 8`（e168、e180、e183；这三个只在开头与 format 常量比一次，按槽距算 S 之后比的那个常量本身要变）、harness 模型的别名常量（`crates/singlefs-harness/src/model.rs:1815`）、26 行直接用 format 常量名的。「按槽距算 S 的运行时构造入口」要罩到的面比 77 大。

## 推翻条件

- X1-甲：第二轮判决第三节的替换范围其实连「⚠️ 具体取多少不是格式决策……」那半句一起换掉（主 agent 写回时整格重写），或主 agent 认定 mkfs 取值式进 D22 是「说明」不是条款——前者这一格不成立，后者要在条文里写明「不是格式承诺」。
- X1-乙：仓里已有一条条款写明「devs ≥ 3 / 条带表开线走 incompat」或已删掉「P > devs」，我没搜到（`grep -rn 'P > devs' .claude/kb` 去掉译文与变更史，命中 D22 第 58、70、350 行、checks-owed 的 C217 与 E34、E115、E124、E125 四个实验页，没有一处删掉这条约束或给 devs ≥ 3 写出路）。
- X1-丙：主 agent 把第 1 句与 I-7.13 的成员表本来就当成「不完整列举」，那就只剩措辞问题。
- X2-3：mkfs 的 S 参数另有只给测试的入口计划（我没在第二轮两张表里看到）。
- X3-丙：E50 装置另有一份含 128 的扫描产物；今天 `research/results/` 下没有 `e50-ring-slots-2026-08-30.out`（`ls` 现查不在），页上的 256 只能对着源码判。

## 没打中的形状

- X1 量纲：S × 槽距 与 P × chunk、min 式、钉值 3S − 1、偏移 363 / 367 的宽度与单位，逐个对过 layout 字段表，没有对不上的（X1-戊）。
- X1「小于 5 拒」与 D2 已定项 19 射程的拒绝判据：被 I-7.5 对齐判吃掉，按 D2 已定项 19 取槽距时一台真设备都不多拒（X1-丁，io_min 512 B..2 MiB 每 512 字节一档，4096 个值）。
- X1「小于 5 拒」与 D16 已定项 1 下界 4：D16 只给下界，不要求 mkfs 一定收 S = 4 的几何，字面不冲突。
- X1 与 D22 已定项 25：25 禁的是「块复用的界」做格式常量，被判形态钉的是读者收的 S 上界；块复用的界照旧 max(F_生效, 环里最旧有效根)。没打中（这一侧与已排除的「焊死」措辞挨得近，没往下攻）。
- X1 与 D22 已定项 16 第 2 句「槽距 = max(4096, io_min)」对 D2 已定项 19「向上取整到 io_min 的整数倍」：两种写法只在 io_min < 4096 且不整除 4096 时不同，那时 S 都写 8，与本形态无关。
- X1 与 D15 已定项 7 分层：S 两条边、P、chunk 都随系统配置槽落第 1 层，一致。
- X1 chunk 钉 1 MiB 与 D22 未定项 6 乙「要求 zone_size ≤ chunk」：zoned 线另占 incompat 位，读者见不认得的 incompat 位先判不可择，不走到 chunk 那一判，不冲突。
- X3：第一、二轮报的 9 个数（24 / 24 的结构、S ≤ 85、S ≤ 25、S ≥ 10、26429、29744、9411 / 1184 与错拒 0、629248..786432 的 308 / 39、768 KiB、77）逐个用独立算法重出，全部对上；对不上的只有 E50 页自己的 256 / 32。

取样范围：S 4..16；槽距 pbs 512 时 4096..1044480 每 512 一档（2033 个），pbs 4096 时每 4096 一档（255 个）；io_min 512 B..2 MiB 每 512 一档（4096 个）；I-7.17 模型 5 条臂 × txg 0..200 共 201 个状态。

## 这条腿自己的限度

- 三份原型都是条款或代码的复述，不调 crates：`geometry.py` 里「入库 mkfs 收不收」是我照 `make_filesystem.rs` 的 `fixed_structure_extents` / `check_geometry` 重写的判法，不是调它；与第二轮两腿调入库 mkfs 得出的集合逐形状相等，算两条路互证，不算入库装置上的数。
- `ring_alias.py` 只摆根环槽、只判 I-7.17 一条，不写 journal、系统配置与单元，也不走入库读者；它说明的是「别名臂每个状态都红」这个结构，替不了第一轮那次真挂载。
- X1-乙 没有第一版能走到的盘面，是条款对条款的推论；按 `.claude/rules/three-way-inference.md` 算一次观测、推的。
- 冻结副本是开工那一刻的主工作区（HEAD 841b3beab 加未提交改动）；别的会话之后改了 crates 的，X2 / X3 的行号与计数要在当时的树上重数（`tree_counts.py` 指到新树即可）。
- 只读了前两轮腿报告里要核的那几行（grep 定位），没整读；第一轮四条对照臂各是什么没核。
- 「没打中」各格只抽了这一次样。
- 自提改法（X1-甲 那一条）只在条文上推过，被攻过零轮。

## 没做什么

- 没编译、没跑任何 crates 测试与装置；没跑崩溃点枚举（这一轮的三格都不要新造崩溃流，枚举状态数 0）；重型测试一条没碰。
- 没改前两轮的模型；没读禁读清单里的三样。
- X2 各条只列落点，没替主 agent 判哪些进「实现欠」、哪些另登欠账；三语译文只核了文件在，没逐句看译文里有没有同样的句子。
- 不判正推、辩方那几格；副本与模型上的数不算入库装置上的数。

## 草稿目录

`/home/fy5090/.local/share/singlefs/work/m3-close-c506-r3-opus/frozen`（34M，612 个文件）：复跑 `tools/rerun.sh` 的 `--frozen` 指它，`tree_counts.py` 只读它；主 agent 复跑核完可以删。复跑用的 `rerun1` 已删。没有编译目录。

## 要主 agent 跑的命令

无。
