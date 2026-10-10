# m3-batch27-code-r1 云端攻方（Opus）报告

轮：m3-batch27-code-r1；攻击面：正文第三节 K2（重点 J1、J3）与 m3-batch26-code-r1 K2 三格的第二次抽样。立场：攻方。
被判对象一律取冻结副本 `/tmp/claude-1000/m3-batch27-code-r1/tree/`（5 份 sha256 与 `research/prompts/m3-batch27-code-r1-snapshot/crates-sha256.txt` 逐份 OK）。
副本上量出的数都是副本上的数，不算入库装置上的数。


## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2 J3 与原写法比漏判（H-J3a） | **打中**（副本实跑，CPU；GPU 读码同漏） | inode 树根条目类型段不认得、子指针两条位置条目的整单元校验和都与孩子对不上（或指到别处）：新写法只红 I-9.2，I-2.1、I-4.8、I-7.2 报成立；同一张镜像类型段写 2 时这三条都红，第 27 批之前的写法也都红。去掉 Class2Carried 那一族之后 88 形（4 份底 × 2 种层级 × 11 种故障组合），类型段 7、1、65535 逐形相同 |
| K2 J3 槽数与孩子真占的不符 → I-3.1 一族 | **不算打中**（附带，口径不收） | 层级 1、真孩子是一槽的码 2 节点：新写法按两槽登记，I-3.1、I-3.11 多红（另一份底上 I-3.5 也红），旧写法不红；层级 2、真孩子是两槽叶容器：新旧都多红 I-3.1、I-3.11、I-5.1。都在坏镜像上，按第五节口径不是误红（不是可达状态）；漏红只推出了一种要三处故障叠在一起的形，没造 |
| K2 J3「孩子是另一类」那一支（H-J3b，第 27 批之前就有） | **打中**（附带，不归第 27 批的改动） | 条目里一条位置条目的校验和坏、另一份孩子自证成别的类（类型段 0 / 7 / 1 / 65535 指叶容器）：I-2.1、I-4.8、I-7.4 报成立；m3-batch26-code-r1 判决「附带」那一行说「池造的镜像上映射树那一遍照判」，坏在指针里的校验和时映射树那一遍判不到。新旧写法逐形相同 |
| K2 J1 链长 255 / 256 / 257、中途断开、重复片 | 没打中（一次抽样） | CPU 实跑 62 形；GPU 走读循环逐行转写，在 CPU 实跑上核了门开门关 62/62 一致；GPU 判了的形与 CPU 同判，判不了的只有走满 256 片还要往下的那几形（回落 CPU）。环回那一形 CRC 自洽造不出来，读不出先挡住 |
| K2 CPU 与 GPU（J3 那一支） | 没打中（读码，一次抽样） | `w_walk_inode_root` 类型段不认得那一支与 walk.rs 逐项同判（不读孩子、按层级登记、只判 I-9.2）；H-J3a、H-J3b 在 GPU 上照样漏（推的，GPU 不跑） |
| 第二次抽样：m3-batch26 K2 H1 新写法翻而原写法不翻 | 没打中（第二次） | 456 对镜像形（孩子故障与指针故障点名的盘号一起对调），新、旧都 0 对翻面 |
| 第二次抽样：m3-batch26 K2 H3 误红 | 没打中（第二次，取样窄） | 坏镜像矩阵里 I-9.2 字面成立的 192 形，新、旧 I-9.2 都 0 红；合法历史的崩溃状态这一轮没跑（见「这条腿自己的限度」） |
| 第二次抽样：m3-batch26 K2 CPU 与 GPU | 没打中（第二次，读码） | GPU 自证读头、两份依次看、两份都不过交回 NONE 与 CPU 同；读不出的那一处 GPU 与 CPU 读法不同（GPU 认不得的落点交回空片），这一轮的形上结局相同（推的） |

## 复跑命令与模型文件

模型目录 `research/prompts/m3-batch27-code-r1-opus-model/`（只读冻结副本、自己拷三份副本；变量与分步写在 `rerun.sh` 文件头）：

```bash
export SINGLEFS_REPO=/tmp/claude-1000/m3-batch27-code-r1/tree   # 被判的仓树
export SINGLEFS_TOOLS=/home/fy5090/code/singlefs                 # 带 research/scripts/run-with-memory-cap.sh、capped.sh 的仓根
# 同一会话另有一条经内存包装在跑时：export RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由>
bash research/prompts/m3-batch27-code-r1-opus-model/rerun.sh <空目录> all      # 或分步：build → j3（OPUS_B27_VERSIONS=new / old / fix）→ j3types → fixcheck → j1
python3 research/prompts/m3-batch27-code-r1-opus-model/analyze.py <空目录>/summary.txt
bash research/prompts/m3-batch27-code-r1-opus-model/rerun.sh compare <空目录>/summary.txt   # 不读 SINGLEFS_REPO，两边先排序
```

交回前在空目录 `/tmp/claude-1000/m3-batch27-code-r1-opus/run2/` 里从头分步跑了一遍（每步经内存包装：编译 6G、跑 4G；前台每条不超过 240 秒，j3 new 按 `OPUS_B27_J3_FROM/TO` 切两半），`compare` 原样输出：`SUMMARY_MATCHES 5613 行`。

| 文件 | sha256 |
|---|---|
| analyze.py | 1bb57178b5b26b71af7adbd3409b92d69f4288971ca5d3a7fddcad0dcc208e01 |
| attack.rs（追加进副本的 `checker_known_bad_images.rs`，测试名不带 layer0） | df024c9cf2497d16500ea3a2983393caf2038c629c365597f2806af935690ee4 |
| expected-summary.txt | 5cd3367536f0eb3e3c460a7c2e5e44e54f77e18f82a8f77272802ca5727443c8 |
| rerun.sh | 83dc166baa3a4c8dacd3251c3bf65b705e6311904cac2a783234ab4a0f598041 |
| walk-fix-alpha.patch（F-α，零轮） | c14e73643af0a29760169d2c097ffbfbeb9271ada8205e7f9025f34c0a25a262 |
| walk-rs.patch（第 27 批 walk.rs 那一段原样，反打出 old） | 8954d7edfe8dc1dae5eab54d449e810fb65380260ea2d5356fa199e1f6e183e5 |

取样与估时：先跑 20 形（单线程 9.7 秒，含建 B1），估全量 1512 形 × 三份约 12 分钟单线程；正式跑 12 线程（协调方把线程上限改成 31 之后的 j3types、fixcheck、j1 用 24 线程）每份 1512 形 60–180 秒。没缩取样。这一轮不造崩溃历史，没调 `enumerate_layer0`，崩溃状态数 0（第八节）。

## 一、J3：打中 H-J3a——类型段不认得、子指针校验和对不上时 I-2.1、I-4.8、I-7.2 报成立

### 1.1 最小的一张镜像（B1，副本实跑）

底：`build_pool` 的新池新建文件那一版（B1）。只改 inode 树根第 0 条条目这 120 字节里的两处，根重封、沿引用链补到根槽：
类型段 2 → 7；子指针两条位置条目里的整单元校验和各翻一字节（`e=FlipBoth`）。叶容器本身一个字节没动，映射树里它那一条照旧对得上。

`research/prompts/m3-batch27-code-r1-opus-model/expected-summary.txt` 里的原样行（第 1838、3854、1836 行；new = 冻结副本，old = 冻结副本上 walk.rs 退回第 27 批之前）：

```text
new	J3	b=B1-new-pool-file	c=None	e=FlipBoth	t=7	l=1	lit21=V	i21=H	i48=H	i72=H	i74=H	i31=H	i51=H	red=I-9.2	partial=
old	J3	b=B1-new-pool-file	c=None	e=FlipBoth	t=7	l=1	lit21=V	i21=V	i48=V	i72=V	i74=V	i31=V	i51=V	red=I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.2,I-7.4,I-9.2	partial=
new	J3	b=B1-new-pool-file	c=None	e=FlipBoth	t=2	l=1	lit21=V	i21=V	i48=V	i72=V	i74=V	i31=H	i51=H	red=I-2.1,I-4.8,I-7.2,I-7.4	partial=
```

`lit21` 是原型自己按字面算的（推的读法，只看这一条子指针）：每条位置条目读孩子头里的类标签，登记过的类按那一类的长度、没登记的 16 KiB 与 32 KiB 两种长度都试，整单元 CRC 与位置条目对不上就是 V。

### 1.2 每一步许可它的那一句

1. 两份副本都自证不过，`child_unit_class_other_than_the_entry_type_names` 交回 None，走到类型段不认得那一支（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2507`：「if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(」）。
2. 那一支只登记引用、判 I-9.2、`continue`，孩子一字节都不读（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2539`：「let child_span_in_slots_by_the_level = if node.level >= 2 { 1 } else { 2 };」，`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2547`：「不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不读）」）。I-2.1 只在读的地方判（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/image.rs:1504`：「judgements.judge("I-2.1", matches, || {」），这条指针一次都没进去。
3. 不记走读失败、I-2.1 计数不涨，I-4.8 / I-7.4 那一格跟着不红（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:10618`：「!newest_failures.is_empty() || walk.judgements.violation_count("I-2.1") > 0;」）；I-7.2 同样只看走读失败。
4. 映射树那一遍按映射条目里的校验和读同一片叶，对得上，补不回来（坏的是父指针里的校验和，不是叶的内容）。
5. 第 27 批之前：这一支按 16 KiB 读孩子（被删掉的那一支，`research/prompts/_m3-batch27-code-r1-diff.md:917`：「InodeInternalEntryType::InternalNode | InodeInternalEntryType::Unregistered(_) => {」），读不出就红 I-2.1、记走读失败。第 27 批的坏镜像把这一点写成了要钉住的期望（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:10979`：「不连带 I-2.1、走读失败那几条与 I-3.1」）——对类标签改成 9、校验和沿链补齐的那一张是对的（孩子按 32 KiB 对得上），对校验和本身就错的这一张不对。

条款字面（kb 自己的行号）：
- I-2.1 `.claude/kb/invariants.md:130`：「任一被引用的块，其校验和与内容匹配」。这条子指针引用的块，按它头里的类（码 3，32 KiB）读回，与两条位置条目的校验和都不符。
- I-4.8 `.claude/kb/invariants.md:181`：「所有块的校验和均与其父指针记录的一致」。
- I-7.2 `.claude/kb/invariants.md:56`：「无一处失配」。
- 口径 `research/prompts/_m3-batch22-code-r1-body.md:68`：「漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立」。三条都报 Holds（不是部分没判、不是不适用），按这一句是漏判。

### 1.3 打到的范围（副本实跑，`analyze.py` 的 G 段）

```text
G1 新漏旧判 I-2.1：136 形，其中不含 Class2Carried 的 88 形；这 88 形里新写法 I-4.8 成立 88、I-7.2 成立 88；类型段 ['7']
```

88 形 = 11 种（孩子故障，指针故障）组合 × 4 份底 × 2 种根层级：孩子不动 / 类标签改 1 / 类标签改 9（后两种整单元校验和沿链补齐）各配 FlipBoth、RedirectToHole（两条位置条目改指 4 GiB 盘上没写过的槽 250 000）、RedirectToNodeKeepSum（改指分配记录树的根、校验和不动）；类标签改 9 另配 FlipSum0、FlipSum1（只坏一条，两份同内容、两条校验和不同，按哪种长度都至少一条对不上）。
类型段 1 与 65535 在 B1、B2 上与 7 逐形相同（`D1 类型段 1、65535 与 7 在同一形上红与部分没判集合相同 1008 形、不同 0 形`）。
Class2Carried（类标签改 2、整单元 CRC 按 32 KiB 补）那 48 形「内容按 16 KiB 还是 32 KiB 算」有两读，不算进打中。

### 1.4 四句

- **分不分辨臂**：分辨。被比的是第 27 批新写法与原写法：同一批镜像上原写法红、新写法不红；类型段换成 2 时两种写法同红。改法 F-α 也在同一批形上分得开（1.5）。病根不在两种写法共用的前提里。
- **系统看不看得到判别它的东西**：看得到。走到这一支时 checker 手里有子指针的两条位置条目，原型的 `lit21` 只用这两条与盘上的字节就算出来了；`child_unit_class_other_than_the_entry_type_names` 自己已经把每一份读过、验过一遍，只是不判。
- **满足的是判据哪一个分句**：第五节「漏判」的第一分句（报成立）。不是「部分没判」，也不是「误红」。
- **跑前条款给的改法**：正文第五节只写「打中：主 agent 现查坐实，改法进下一批」，没有点名改法，没有「改法碰不到」这一问。

### 1.5 改法 F-α（只在我的模型上量过、被攻过零轮）

F-α：类型段不认得那一支照旧不往下读、照旧按层级登记、照旧判 I-9.2；在判 I-9.2 之前对每条位置条目判 I-2.1——头里的类登记过的按那一类的长度比，没登记的 16 KiB 与 32 KiB 两种长度都对不上才算对不上；两份都对不上记走读失败。补丁原样 `research/prompts/m3-batch27-code-r1-opus-model/walk-fix-alpha.patch`（只改 CPU 的 walk.rs，31 行）。GPU 要同改（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2646`：「if (class_the_inode_entry_type_names == NONE) {」那一支），墓碑树、谱系树类型段不认得那一支同形（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2257`：「let span_in_slots = if node.level >= 2 { 1 } else { 2 };」那一支同样不读孩子），要不要一起改交主 agent。

| 格 | 新写法（被判的） | F-α | 标 |
|---|---|---|---|
| H-J3a 那 88 形 I-2.1 | 88 形报成立 | 88 形都红 | 量过（`G3`、本表下的原样行） |
| H-J3a 那 88 形 I-4.8 | 88 形报成立 | 88 形都红 | 量过 |
| H-J3a 那 88 形 I-7.2 | 88 形报成立 | 72 形红；另 16 形（类标签改 9 只坏一条校验和）不红，与类型段 2、一份坏一份好时同判 | 量过 |
| 第 26、27 批那 6 条坏镜像（变异行点名的两个筛子 `stale_copy_of_the_child`、`child_that_does_not_read_back`） | 6 ok | 6 ok | 量过（`G7`） |
| F-α 相对新写法别的红 | — | 只在类型段 7 的形上变：I-2.1 +136、I-4.8 +136、I-7.4 +136、I-7.2 +248、I-3.5 −16 | 量过（`G6`） |
| H-J3b（「孩子是另一类」那一支） | 88 形报成立 | 不碰，仍 88 形 | 量过（`G4`）：F-α 对这一格不起作用 |
| GPU 走读内核 | 同漏 | 没改、没跑 | 推的 |

`analyze.py` 在 `expected-summary.txt` 上的原样输出：

```text
G3 F-α 副本上这 88 形 I-2.1、I-4.8、I-7.2 都红的 72 形
G4 F-α 副本上字面 I-2.1 违反而报成立 88 形（新写法 224 形）
G5 F-α 与新写法红集合不同的形按类型段：{'7': 264}
G6 F-α 相对新写法多红（+）少红（-）的不变量：I-2.1+:136, I-3.5-:16, I-4.8+:136, I-7.2+:248, I-7.4+:136
G7 F-α 副本上第 26、27 批那几条坏镜像：6 条 ok；['test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 126 filtered out;']
```

I-7.2 比 I-2.1 多出来的 112 形（248 − 136）逐形看过（副本上的一次性 python，没进模型）：多出的 I-7.2 + 而 I-2.1 不 + 的 128 形全是「一份孩子内容坏（ContentDev0/1 或 StaleNodeDev0/1）、另一份的位置条目坏或两份都改指」的类型段 7 形——新写法上 I-2.1 已由映射树那一遍红，两份都对不上却不记走读失败，I-7.2 报成立；F-α 记走读失败之后红。128 − 112 = 16 是 I-2.1 + 而 I-7.2 不 + 的那 16 形（上表第三行）。

### 1.6 GPU 那一侧（读码、用 CPU 算期望，没跑）

`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2638`：「if (class_of_the_inode_child != NONE) {」之后、`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2651`：「w_note_pointer_references(child, child_span_by_the_level);」那一支同样不读孩子、不加 `w_failures`、不置 I-2.1 位。所以 H-J3a 那 88 形 GPU 给的就是新写法 CPU 那一行（I-2.1、I-4.8、I-7.2 位都是 0、I-9.2 位是 1），与 CPU 同判、同漏；对拍不会红，这一格要靠坏镜像钉。

## 二、J3 附带：打中 H-J3b——「孩子是另一类」那一支不判坏的那条位置条目（第 27 批之前就有）

同一个矩阵里，新旧写法逐形相同地漏 80 形（不算 Class2Carried 那 8 形）：类型段 0 / 7 指着叶容器、或类型段 0 / 2 / 7 指着类标签改成 1 的叶容器，条目里只坏一条位置条目的校验和（FlipSum0 或 FlipSum1）。另一份自证成「另一类」，走「孩子是另一类」那一支：按它的类登记、判 I-9.2、`continue`，坏的那一条从头到尾没判 I-2.1。原样行（第 1844、1842 行）：

```text
new	J3	b=B1-new-pool-file	c=None	e=FlipSum0	t=7	l=1	lit21=V	i21=H	i48=H	i72=H	i74=H	i31=H	i51=H	red=I-9.2	partial=
new	J3	b=B1-new-pool-file	c=None	e=FlipSum0	t=2	l=1	lit21=V	i21=V	i48=V	i72=H	i74=V	i31=H	i51=H	red=I-2.1,I-4.8,I-7.4	partial=
```

类型段 2 时同一条坏校验和红 I-2.1、I-4.8、I-7.4；类型段换成 7（或 0）就都报成立。按第五节口径是 I-2.1、I-4.8 的漏判（I-7.4 的字面是「被重新分配」，这张镜像上没有，不算）。
m3-batch26-code-r1 判决第二节「附带：「孩子是另一类」那一支不判陈旧那份 I-2.1」那一行判「不算打中：装置造出来的，池造的镜像上映射树那一遍照判」——坏在父条目里的校验和时映射树那一遍按映射条目自己的校验和读，判不到；这一行的理由在这一形上不成立。
不是第 27 批改出来的（新旧逐形相同），按派发只记线索，不判归属。墓碑树、谱系树那一支（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker/src/walk.rs:2275`：「fn judge_a_container_tree_entry_whose_child_is_of_another_class(」）同形，推的、没造。GPU 那一支同样不读（1.6 那一处的前一支），推的。F-α 不碰这一格。

## 三、J3：槽数与孩子真占的不符时 I-3.1 一族——附带，不算打中

| 形 | 新写法 | 旧写法 | 说明 |
|---|---|---|---|
| 层级 1、类型段 7、子指针改指分配记录树的根（一槽的码 2 节点，校验和不动，两份都自证不过） | 多红 I-3.1、I-3.11（4 份底）；B2（回退到第一条根那一份）上另多红 I-3.5，层级 1、2 都有（`A3` 那几行） | 不红 I-3.1 | 按层级登记两槽，(盘, 起点, 跨度) 与分配记录树根的 (起点, 一槽) 是两项，引用数多算 2 槽：盘 0「记账的已分配 Some(278528)，遍历全部有效根得到 311296」（B1 那一形的违例说明，`OPUS_B27_J3_DETAIL=1` 单跑，副本） |
| 层级 2（根层级改 2、重封）、类型段 7、真孩子是两槽叶容器、两份都自证不过 | 多红 I-3.1、I-3.11、I-5.1 | 同样多红 | 按层级登记一槽，与映射条目的两槽是两项 |

两种都在坏镜像上（类型段 7 本身就要改坏），第五节口径里误红只认可达状态与合法实现，这里不算误红。I-3.1 的字面（`.claude/kb/invariants.md:143`：「已分配空间统计 == 实际遍历所有引用得到的和」）在这两形上我判不出该红不该红：被改指的那一槽到底「被这条指针引用了几槽」，镜像上没有答案。
漏红一侧只推出一种形：层级 1、真孩子一槽、紧挨着的那一槽恰好是一槽泄漏（已分配、没人引用），多登记的那一槽把泄漏盖住、I-3.1 报成立——要三处故障叠在一起，没造、没量（推的）。

正文第三节 K2 点名的两形：「孩子是码 3 叶容器却在层级 ≥ 2 下」就是上表第二行（新旧同）；「孩子是码 2 节点却在层级 1 下」只在两份都自证不过时走到按层级登记那一支（自证过的走「另一类」那一支、按真类登记），就是上表第一行，比原写法多红 I-3.1、I-3.11。

## 四、J1：实例表链长在上限附近、中途断开、重复片——没打中（一次抽样）

### 4.1 装置

底 B4（被崩溃恢复抛弃的 C 与现行线同号不同出生代的那一份，`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`），最新根的单片实例表改成 N 片的链：第 0 片放实例 2 之外的行，第 1 片起写在槽 200 000 + 2k、只放链指针记录（与对拍用例同一个造法，原型自己写的）。实例 2 那一行去掉（Dropped：门开就红 I-9.29、门关报部分没判）或放在末片（OnTheLastPage：门开滤得掉、I-9.29 成立）。
CPU 实跑 `check_pool_image`；GPU 不跑，把 `w_walk_instance_table_chain` 的循环（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1601`：「for (; page < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; page = page + 1u) {」、`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1686`：「if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {」）逐行转写成 `transcribed_walk`，同一个函数不设上限就是 walk.rs 那一条的转写；不设上限的转写在 62 形上与 CPU 实跑的门（I-9.29 是不是部分没判）逐形相符，再拿设上限的那一份推 GPU。

形（两种行的放法各 31 形，共 62）：链长 1、2、255、256、257、258、300；第 k 片读不出 / 容器号写错 / 链指针记录写 2，(N, k) 取 (256, 255)、(257, 255)、(257, 256)、(257, 1)、(258, 256)、(2, 1)；末片指回第 j 片，(N, j) 取 (255, 0)、(256, 0)、(256, 1)、(257, 0)、(257, 200)、(3, 1)。

### 4.2 结果（`analyze.py` 原样，节选）

```text
E1 J1 形数 62；转写与 CPU 实跑的门不符 0；GPU 判了而与 CPU 不同 0
```

`expected-summary.txt` 第 1530、1534、1546、1542、1564 行：

```text
new	J1	shape=len255	rows=Dropped	pages=255	cpu_i929=V	cpu_transcribed=ReachedTheLastPage	transcription_matches_cpu=true	gpu=door-open i929=V	gpu_cpu_divergence=false	red=I-3.1,I-3.9,I-3.11,I-9.10,I-9.29	partial=
new	J1	shape=len256	rows=Dropped	pages=256	cpu_i929=V	cpu_transcribed=ReachedTheLastPage	transcription_matches_cpu=true	gpu=door-open i929=V	gpu_cpu_divergence=false	red=I-3.1,I-3.9,I-3.11,I-9.10,I-9.29	partial=
new	J1	shape=len257	rows=Dropped	pages=257	cpu_i929=V	cpu_transcribed=ReachedTheLastPage	transcription_matches_cpu=true	gpu=undecidable(61)->cpu	gpu_cpu_divergence=false	red=I-3.1,I-3.9,I-3.11,I-9.10,I-9.29	partial=
new	J1	shape=len256-back-to-0	rows=Dropped	pages=256	cpu_i929=P	cpu_transcribed=StoppedAt(256)	transcription_matches_cpu=true	gpu=undecidable(61)->cpu	gpu_cpu_divergence=false	red=I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.2,I-7.4,I-9.10	partial=I-9.29
new	J1	shape=len257-Unreadable@256	rows=Dropped	pages=257	cpu_i929=P	cpu_transcribed=StoppedAt(256)	transcription_matches_cpu=true	gpu=undecidable(61)->cpu	gpu_cpu_divergence=false	red=I-2.1,I-3.1,I-3.11,I-4.8,I-7.2,I-7.4,I-9.10	partial=I-9.29
```

（I-3.1、I-3.9、I-3.11、I-9.10 是第 1 片起那些片不在分配记录里带出来的，每一形都有，与链长无关。）

| 形 | CPU（实跑） | GPU（推的） |
|---|---|---|
| 链长 ≤ 256、走得到末片 | 门开，I-9.29 照比 | 判，门开、同判 |
| 链长 ≥ 257、走得到末片 | 门开 | 第 256 片之后还说有下一片：记判不了（站点 61），回落 CPU |
| 第 k 片坏（读不出、容器号错、链指针记录不合法），k ≤ 255 | 门关，I-9.29 部分没判 | 判，门关、位 0，同判 |
| 第 256 片（下标）坏、链长 ≥ 257 | 门关 | 走满 256 片还说有下一片，判不了，回落 CPU（GPU 没读到坏的那一片） |
| 末片指回第 j 片 | 门关（指回那条的整单元 CRC 只能写错：环上 CRC 自洽造不出来，读不出先挡住，轮不到「走过的单元」那一判） | 链长 ≤ 255 时判、门关、同判；链长 ≥ 256 时判不了、回落 CPU |

「链里有重复片」能造的只有两种：指回前面某一片（CRC 必错，读不出）、另放一份同内容的片（容器号与片号对不上，I-1.1 红、走读失败）。两种 CPU 与 GPU 都在同一处停下。
GPU 另有一道更早的上限：最新根那张表的行超过 1024 条记判不了（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1652`：「if (w_instance_row_count >= C_MAX_INSTANCE_ROWS) {」），写者写满一片是 369 行（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-harness/tests/common_pointer_head_corruption/mod.rs:52`：「const INSTANCE_ROWS_PER_PAGE: u32 = 369;」），写者自己写出的满片链到第 3 片就先撞行数上限；256 片这道上限只在几乎全空的片上够得着（推的，没量写者能不能写出空片）。

## 五、CPU 与 GPU（J3 那一支）——没打中（读码，一次抽样）

逐项对过 `w_walk_inode_root` 与 walk.rs `walk_inode_internal_node`（GPU 只走根这一层）：
- 类型段 → 「它说的类」：2 → 码 3、0 → 码 2、别的 → NONE，与 CPU 的 `class_the_entry_type_names` 同。
- 「孩子是另一类」：两份依次看，头按 16 KiB 读、按头里的类长度验（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2982`：「let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);」），与 CPU 逐份 `for location in &child.locations` 同序。
- 类型段不认得、两份都不过：层级 ≥ 2 一槽、否则两槽，判 I-9.2 红，`continue`；CPU 同。
- 层级 ≥ 2 的根上只有类型段 2 与不认得的条目时，GPU 不再撞类型段 0 那一处判不了（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2776`：「undecidable = UNDECIDABLE_TOO_DEEP;」只剩类型段 0 走得到），自己判；CPU 在同一张根上判的项与它相同（读码，推的）。
- 读法上唯一不同的一处：GPU 的 `read_at` 只认判器表里登记过的落点（`/tmp/claude-1000/m3-batch27-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:471`：「let location = find_location(device, offset);」），认不得的交回空片；CPU 读没写过的槽读回全零。RedirectToHole 那一形上 CPU 读回全零、类标签 0 不登记、看下一份；GPU 交回空片、看下一份——结局都是两份都不过、走类型段不认得那一支（推的，GPU 不跑）。

所以 H-J3a、H-J3b 不是「CPU 与 GPU 判得不一样」的形：两边同漏，GPU 对拍抓不到它们。

## 六、第二次抽样（m3-batch26-code-r1 K2 那三格）

| 格 | 这一次的取样 | 结果 |
|---|---|---|
| H1 新写法翻而原写法不翻 | J3 矩阵里 456 对镜像形：孩子故障（ContentDev0/1、StaleNodeDev0/1）与指针故障（FlipSum0/1）点名的盘号一起 0 ↔ 1 对调，比红与部分没判两个集合 | `C1 new 盘序翻面（红或部分没判集合随故障在哪块盘而变）：比了 456 对，翻面 0 对`；old 同样 0 对 |
| H3 误红 | J3 矩阵里 I-9.2 字面成立的 192 形（类型段 2、孩子是那片叶容器、身份不动；孩子一份或两份坏、一条或两条位置条目坏、根层级 1 或 2） | `F1 new I-9.2 字面成立的形 192，I-9.2 红 0`；old 同样 0 |
| CPU 与 GPU | 第五节读码 | 没找到判得不一样的一支 |

H1 这一次的形与上一次不同：上一次是陈旧副本换成另一类单元的字节；这一次另加了父条目里位置条目的校验和坏、孩子内容只坏一份、两种根层级、四份底。H3 误红这一次只在坏镜像上取样，合法历史的崩溃状态没跑（第八节）。

## 七、没打中的形状

- J3 其余：类型段 0 / 2 的 1008 形新旧红集合逐形相同（`A1 类型段 t=0：新旧红集合不同的形 0`、`A1 类型段 t=2：新旧红集合不同的形 0`）；类型段 7 有 292 形不同，逐类归进第一、三节（多红的只有 I-3.1、I-3.11、I-3.5，少红的是 I-2.1、I-4.8、I-7.2、I-7.4 与旧写法那几处按 16 KiB 读带出的 I-3.1、I-3.11、I-5.1）。
- J3 取样范围：4 份底（新池新建文件 B1、回退到第一条根 B2、顺序写 3 个数据单元 B3、崩溃恢复抛弃第三条根 B4）× 9 种孩子故障 × 7 种条目指针故障 × 类型段 {2, 0, 7}（B1、B2 另加 {1, 65535}）× 根层级 {1, 2} = 1512 形，每形在新、旧、F-α 三份副本上各跑一次 `check_pool_image`。只改 inode 树根第 0 条条目；层级 ≥ 3、根下面再有码 2 节点的 inode 树没造（这一版写者不长高）。
- J1：62 形，第四节。链长取到 300；没取 1024 片以上。
- 用户决定的那几步：这一轮的「历史」是 4 份底各自的写入序列（建文件、覆盖写、回退、崩溃恢复抛弃），放开扫的是这 4 种前缀；故障固定在 inode 树根第 0 条条目与它的孩子上。H-J3a 在 4 份底上都中（`G2` 每组 8 = 4 底 × 2 层级）。

## 八、这条腿自己的限度

- 全部数是仓副本上的：`check_pool_image` 在冻结副本拷出的 new / old / fix 三份上跑，不是入库装置上的数。old 只把 walk.rs 退回第 27 批之前（wgsl 不动，GPU 本来就没跑）。
- `lit21` 是原型按 I-2.1 字面自己算的读法（只看 inode 树根第 0 条子指针，类标签不登记时两种长度都试），不是 checker 的输出；Class2Carried 那一族两读，没算进打中。
- GPU 一侧全是读码推的；对拍、GPU 变异一条都没跑。J1 的 GPU 结论来自循环的逐行转写，转写只在 CPU 那一半（不设上限）上与实跑核过。
- 4 份底用的是现成的测试帮手（`build_pool` 一族），它们经文件镜像建池（临时稀疏文件，用完删）；每份底在每个进程里只建一次、之后从内存拷。三份副本各自建一次，不共享。
- m3-batch26 H3 误红的第二次抽样只在坏镜像上取样：合法历史的崩溃状态没跑（`enumerate_layer0` 要另起一个依赖 checker 档库的原型，这一轮没做）。第 27 批在合法状态上走不到类型段不认得那一支（写者只写类型段 2），这一格的第二次抽样窄于上一次。
- 只改了 inode 树根第 0 条条目；墓碑树、谱系树的同形（类型段不认得那一支、「孩子是另一类」那一支）都是读码推的，没造镜像。
- F-α 只在我的模型上量过、被攻过零轮，只改 CPU；没跑整份 `checker_known_bad_images`（只跑了变异行点名的两个筛子，6 条），没跑 clippy、fmt。

## 九、没做什么

- 不判 K1、K3（正推腿的格）；不替主 agent 采纳 F-α；没碰 mutations.tsv 与入库代码。
- 重型测试一条没跑（checker 档包的测试、层 0、GPU 对拍、变异整表都没跑）；只跑了副本上 `singlefs-harness` 的 `checker_known_bad_images` 测试二进制，都经内存包装。
- 中途收到协调方消息（全量 J3 三份跑完、j3types 之前）：线程上限 15 → 31；之后起的步骤原型线程数改成 24，输出与线程数无关（`compare` 逐行相符）。
- 内存包装同会话并行退 252 一次（`OPUS_B27_J3_DETAIL` 那次单跑，那一次输出不算），照出路设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 重跑；编译上限 6G 超过峰值表两倍退 251，设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 放行。开工时 `ps` 看到别的会话在跑 `cargo test`（singlefs-format 等包）与 harness 测试，没有性能测量在跑。
