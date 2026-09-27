# m2-closeout-code-r2 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

抽样引文：opus 报告第 58 行 `.claude/kb/decisions/18-块里携带什么信息.md:314`
「不核落后那一支，规格表第 1 行只要这一支」。在草稿副本
（`/tmp/claude-1000/three-way-verifier-r2/selftest/opus-shifted.md`）里把两处 `:314` 全改成 `:315`
（第 315 行实际内容是「前提：同一时刻只有一个可写挂载……」，不含被引句），
跑：

```
python3 research/scripts/cite-check.py /tmp/claude-1000/three-way-verifier-r2/selftest/opus-shifted.md \
  --root research/prompts/m2-closeout-code-r2-snapshot/kb-at-start \
  --root /tmp/claude-1000/three-way-verifier-r2/selftest
```

原样结果：
```
  ✗ opus-shifted.md:58 引 .claude/kb/decisions/18-块里携带什么信息.md:315 那一行「系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）」：那一行不含这句；原文在这份文件第 314 行
  ✗ opus-shifted.md:58 引 .claude/kb/decisions/18-块里携带什么信息.md:315 那一行「不核落后那一支，规格表第 1 行只要这一支」：那一行不含这句；原文在这份文件第 314 行
  ✗ opus-shifted.md:239 引 .claude/kb/decisions/13-验证路线.md:72「系统配置槽写与覆盖旧记录的 journal 写属于这一类」：那一行不含这句；原文在这份文件第 71 行
  ✗ opus-shifted.md:272 引 .claude/kb/decisions/13-验证路线.md:72「当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上」：那一行不含这句；原文在这份文件第 71 行
  ✗ 核了 8 处引文，4 处对不上，0 处没判（逐处列在上面）
```
判绿工具的判别力自证：必须判 ✗，实际判 ✗（且顺带原样复现了下面「三、cite-check.py 自动判定」里 D13:72 那条真实 ✗，见下）。

## 一、开工快照核验

- 快照 `refs/sop/m2-closeout-code-r2-snapshot` = `30c084138f7da7488f27874417902caeb5272a15`（`git rev-parse` 现查，与三条腿报告开头写的一致）。
- `git archive` 到 `/tmp/claude-1000/three-way-verifier-r2/snapshot/`，`cd crates && sha256sum -c crates-src-sha256.txt`：57 份全部 `OK`，0 处对不上。
- opus 模型目录 25 个文件（5 份用例 + `batch_sweep.sh` + `rerun.sh` + `analyze_sweep.py` + 补丁 + 16 份日志）逐个 `sha256sum` 现算，与报告表格里的 25 行逐字对上，0 处对不上。
- `research/prompts/m2-closeout-code-r2-local-attack.md`、`-translation-audit.md` 的 sha256 现算，与运行记录里写的前后缀对上。
- `-output-void1.md`＝4156 字节、`-void2.md`＝3762 字节、`-output-s1.md`＝729 词、`-output-s2.md`＝540 词，与运行记录逐字对上（`wc -c`/`wc -w` 现跑）。
- `-output-s1.md`、`-output-s2.md` 现跑 `corruption-check.py`、`oov-check.py`：两份都「生词=0 拼接=0」判绿，与运行记录「判：干净」一致。
- kb-at-start 15 份文件与 `kb-sha256.txt` 是否逐份相同：按输入一节「主 agent 现核」，本报告未重复核（信任已给的现核结果，仅用作核引文的底本）。

## 二、cite-check.py 自动判定（照抄它的判定）

命令（root 依次给 kb-at-start、快照、仓根；background 给三份背景材料）：
```
python3 research/scripts/cite-check.py research/prompts/m2-closeout-code-r2-opus-output.md \
  research/prompts/m2-closeout-code-r2-sonnet-output.md \
  --root research/prompts/m2-closeout-code-r2-snapshot/kb-at-start \
  --root /tmp/claude-1000/three-way-verifier-r2/snapshot --root . \
  --background research/prompts/_m2-closeout-code-r2-background.md \
  --background research/prompts/_m2-closeout-code-r2-body.md \
  --background research/prompts/_m2-closeout-code-r2-appendix.md
```
原样输出：
```
  ✗ m2-closeout-code-r2-opus-output.md:239 引 .claude/kb/decisions/13-验证路线.md:72「系统配置槽写与覆盖旧记录的 journal 写属于这一类」：那一行不含这句；原文在这份文件第 71 行
  ✗ m2-closeout-code-r2-opus-output.md:272 引 .claude/kb/decisions/13-验证路线.md:72「当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上」：那一行不含这句；原文在这份文件第 71 行
  ✗ 核了 11 处引文，2 处对不上，0 处没判（逐处列在上面）
```
人工核实：`.claude/kb/decisions/13-验证路线.md` 第 72 行是**空行**（`awk 'NR==72{print length($0)}'` → 0），两句引文都在第 71 行（长度 622 字符）。**两处 ✗ 成立**：opus 报告第 239、272 行把 `.claude/kb/decisions/13-验证路线.md` 的行号都错标成 72，应为 71。

工具自动核了 11 处（opus 8 处、sonnet 3 处——与 sonnet 自报的「核了 3 处」一致），其余引文因手工换行、或引文内部含「」嵌套，落在工具的正则缺口外，第三节起逐处人工核。

## 三、opus 报告逐处人工核（工具缺口外的 code:行号 与剩余 kb 引文）

以下每处用 `awk 'NR==N'` 或 `grep -n` 现取快照/kb-at-start 对应文件那一行，与报告原文逐字比对。全部按快照根 `/tmp/claude-1000/three-way-verifier-r2/snapshot/crates`、kb 根 `.../kb-at-start/.claude/kb` 核。

| 报告行 | 引用 | 核的结果 |
|---|---|---|
| 60 | `mount.rs` 3261 起＝`caller_inputs_agreeing_with_the_disk` | ✓ |
| 60 | `mounted_session.rs` 216 起＝`publish_user_change`；242＝调用 `devices_without_a_self_verified_system_configuration` | ✓ |
| 60 | `mount.rs` 3028 起＝`devices_without_a_self_verified_system_configuration` | ✓ |
| 60 | `mount.rs` 2949 起＝`devices_without_the_selected_version`；2976＝`if newest_system_configuration_journal_tail >= selected_version_journal_position { continue; }` | ✓ |
| 93 | `mount.rs` 690＝`is_the_device_table_of_the_mount`；3105＝`device_table_disagreeing_with` | ✓ |
| 160 | `transaction.rs` 1220 起＝`publish_without_units` | ✓ |
| 181 | `system_configuration.rs` 473＝`parse_slot`；497＝`reader.skip(4 + 8)` | ✓ |
| 181 | `root_ring.rs` 94＝`region_start`；`journal.rs` 201＝record_offset 里的常量偏移行 | ✓ |
| 181 | `recovery.rs` 647–648＝`largest_fixed_structure_slot_spacing_the_format_allows`（647 签名、648 `region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES`） | ✓ |
| 182 | `recovery.rs` 719、727＝两处 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 判定 | ✓ |
| 178 | `.claude/kb/decisions/22-单元原子性怎么合成.md` 第 332 行含「基址（设备内字节偏移）= 该字段 × 16384」；第 336 行含「读者判固定结构槽距：≥ 4096」 | ✓（336 是一整段长物理行，被引句在该行靠后处，非行首，`head -c` 曾误判为不含，`grep -n` 全文确认存在） |
| 178 | `.claude/kb/invariants.md:64` I-7.13 全条 | ✓ |
| 212 | `make_filesystem.rs` 363、`recovery.rs` 744（`unit_area_start_of_the_chosen_system_configuration`） | ✓ |
| 229 | `make_filesystem.rs` 305＝`check_geometry` 文档起点 | ✓ |
| 233 | `recovery.rs` 745（同 744 函数体行）、3836（`data_unit_count_matches_the_inode_size` 调用行） | ✓ |
| 233 | `journal.rs` 290＝`named_count`（注释「点名项数」）、198＝`record_offset` | ✓ |
| 241 | `crash.rs` 2560 起＝`is_tearable_in_place_overwrite` | ✓ |
| 272 | `segments.rs` 117 起＝`SegmentClosingRule::after` | ✓ |
| 280 | `model.rs` 1275 起＝`answer_mount_writable_with_the_newest_root_unreadable` | ✓ |
| 183 | `singlefs-checker/src/image.rs` 241/243/244＝`read_u64(slot, 325/371/417)` | ✓ |
| 183 | `image.rs` 305 起＝`journal_ring_bytes_lie_in_the_supported_range` | ✓ |
| 317 | `singlefs-checker/src/lib.rs:265`＝`this_device: read_u32(...)` | ✓ |
| 158 | `.claude/kb/decisions/23-journal的角色与格式.md:397`「一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它」 | ✓ |

**opus 报告小计**：人工核 22 处（不含工具已判的 8 处），✓ 22，✗ 0。加工具的 8 处（✓ 6、✗ 2），opus 报告合计核 30 处，✓ 28，✗ 2（D13:72 两处，见第二节）。

## 四、sonnet 报告逐处人工核（工具缺口外）

| 报告行 | 引用 | 核的结果 |
|---|---|---|
| 22–28 | `admission.rs` 743/741/717/533/413/576-586 | ✓ 全部（`for device_map in &allocator.devices {` 恰在 743；`THE_ROOT_COVERING_THE_WHOLE_KEY_SPACE` 声明在 740、`positions` 赋值恰在 741；`MetadataBlocks(...)` 恰在 717；`checked_add(checkpoint_cost.0)` 恰在 533 段内；`admit_the_units_landing_on_every_device` 文档起 413、函数体 549，576-586 是它逐盘比较那一段） |
| 36,38 | `.../second_transaction_supplement_two_unequal_devices.rs:353`（fn 声明起）、`:399`（`demand: BytesOnOneDevice(7_553_024)`） | ✓ |
| 129–131 | `image.rs:57`＝`IMPLEMENTED_INVARIANTS` 声明；第 105 行确认是 `"I-9.16",`；`recovery.rs:1865`＝`TREE_TABLE_ENTRIES_ORDERING_CONTRACT` 声明 | ✓ |
| 142 | `recovery.rs:1869-1877`＝`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER` 数组（7 成员） | ✓ |
| 155-156,161-162 | `recovery.rs:2014-2015`（实际调用在 2015-2016，2014 是上一句无关代码，范围含 1 行冗余，不算错）、`:2019`（`tree_table_entries_ascend_...` 调用恰在 2019）、`:2020-2022`（水位循环恰在此）、`:2023-2030`（`pointer_of` 闭包恰在 2023 起） | ✓（2014-2015 起点偏 1 行，非致命） |
| 165 | `recovery.rs:1879-1881`＝`rebuild_version` 文档段落 | ✓ |
| 192,195 | `walk.rs:2608`、`:2595`＝两处「不与实现共用代码（D13 已定项 5）」原样注释 | ✓ 逐字 |
| 85-99,108-109 | `image.rs:172-176`（`into_report` 优先级链尾两支，链首「有违例」分支实际在 166，范围只覆盖链的后半，描述了四支、只指到覆盖后两支的行号）、`:120-130`（`judge()` 定义）、`:466-485`、`:476-484`（I-7.13 判定循环） | ✓ 内容存在，172-176 范围偏窄（见下方 ✗ 单列） |
| 98,108 | `walk.rs:5689-5696`、`:5680-5687`＝ I-7.13 越界之后整批 `not_applicable` 与「池里没有一块盘」两支 | ✓ |

### sonnet 报告：三处 ✗（人工核出，工具缺口外）

**✗1（行 94，代码行号全错）**：「I-2.4 的扩展……无条件 `judge()`（`crates/singlefs-checker/src/walk.rs:695`、`:730`；四个入口 `crates/singlefs-checker/src/walk.rs:790`、`:1440`、`:1766`、`:1916`）」。
现查：
```
grep -n 'judge("I-2.4"' singlefs-checker/src/walk.rs   → 701, 711, 747
grep -n 'judge_unit_header(' singlefs-checker/src/walk.rs → 663(定义), 822, 1679, 1919, 1976（四处调用）
awk 'NR==695' → "76 + 2 * key_width,"（元组字面量的一个分量，与 I-2.4 无关）
awk 'NR==730' → "if padding_zero { \"为 0\" } else { \"非 0\" },"（属于 I-2.3 的 format! 参数，不是 I-2.4）
awk 'NR==790' → read_index_node 的文档注释（判 I-1.3/I-1.1，与 I-2.4/judge_unit_header 无关）
awk 'NR==1440' → judge("I-9.4", ...) 调用（另一条不变量）
awk 'NR==1766' → judge("I-9.12", ...) 调用（另一条不变量）
awk 'NR==1916' → "fn judge_packed_container(" 声明（判 unit 头调用在 1919，隔 3 行）
```
六个行号全部对不上：真正的 I-2.4 `judge()` 调用在 701/711/747；真正的四个入口调用在 822/1679/1919/1976。1440、1766 两处指到的是完全不相关的两条不变量（I-9.4、I-9.12）。判 ✗。

**✗2（行 183-184，D13 已定项 4 射程句行号错，工具因手工换行漏判）**：
sonnet 报告原句「checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）」，标注的出处是 `.claude/kb/invariants.md` 第 12 行。
`grep -n "与实现只共享常量模块" .claude/kb/invariants.md` → 第 **11** 行；第 12 行（长度 1030 字符）是「池级 checker 判 47 条……」，与被引句完全不相关的一段。判 ✗——且这正是 sonnet 自己在文末指出的「引文与位置落在不同物理行，工具的正则捕捉不到」那类缺口的一个实例（本行 183 结尾是「」，行 184 才接 `（.claude/kb/invariants.md:12`），核实工具确实没把它计入 3 处已判之内。

**✗3（多处，字符系统性错，且违反「引产物就整行抄」）**：sonnet 报告全篇「槎」字（`grep -c 槎` → 12 处），包括三处标为直接引用的地方——checks-owed.md C571 事实栏引文（行 60-63）、invariants.md I-7.13 条款原文引文（行 105-106）、`m2-rev-a4c-implementer-report.md` 第 14 行引文（行 53-54）。
现查源文件：`grep -c 槎 .claude/kb/checks-owed.md .claude/kb/invariants.md research/prompts/m2-rev-a4c-implementer-report.md .claude/kb/decisions/*.md research/prompts/_m2-closeout-code-r2-{appendix,body,background}.md` → **全部 0**。对应位置源文件用的都是「槽」（slot）。三处引文核对：
- C571 事实栏原文（`.claude/kb/checks-owed.md:484`，经附录「整段抄」核实）：「……段内空**槽**（A4c 报告量到一段 79 对），`df` 报的可用空间……那会是……假性 ENOSPC 那一类（A4c 报告第三节第 5 条，推的，没量 `df` 的实现）」；sonnet 报告写成「……段内空**槎**……那一类」，「槽」全改「槎」，且**引文在「那一类」处收尾，丢了同一栏里紧跟的限定语「（A4c 报告第三节第 5 条，推的，没量 df 的实现）」**——这个限定语正说明该条是推断、未实测，丢了它就看不出这一条的证据强度。
- I-7.13 条款原文（`.claude/kb/invariants.md:64`）：「任一盘任一**槽**不满足其中一项即判红……R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槎不可择」」；sonnet 写成「任一盘任一**槎**不满足……」，同样改字，且引文同样在「整池拒」处截断，丢了后半句「R 与 S 越界不在这一条里」这一限定射程的话。
- A4c 报告第 14 行原文：「**单元区** 384 **槽**，崩了再挂，**这次挂载里** 80 次空发布、**把文件**顺序写到 6 个单元；……数据单元 0 落点被拒**、两块盘逐字节不变**。」；sonnet 写成「384 **槎**，崩了再挂，80 次空发布，顺序写到 6 个单元；……数据单元 0 落点被拒」，删了「单元区」「这次挂载里」「把文件」「、两块盘逐字节不变」几处，且改了字。
三处判 ✗：不是摘句就是改字，两项都撞上「引 kb/产物条目要整行抄」的纪律。

其余 sonnet 报告的 kb 引文（`decisions/28-挂载期承诺量.md:24/33/107`、`decisions/08-核心索引结构.md:240/245`）逐条 `grep -n` 现核，全部 ✓（内容与行号完全对应，此处不重复贴大段原文，命令与命中行号见上）。

**sonnet 报告小计**：人工核 21 处（不含工具已判的 3 处，含 3 处 ✗），✓ 18，✗ 3。加工具的 3 处（全 ✓），sonnet 报告合计核 24 处，✓ 21，✗ 3。

## 五、本地攻方腿核对

### 5.1 事实表六项（Fact Set A–F）与仓里的数

逐项核对提示文件 `m2-closeout-code-r2-local-attack.md` 里 A–F 六组「pinned value」的引文出处（对照 `-translation-audit.md` 给的原文文件:行）：

| 组 | 引文出处 | 现查结果 |
|---|---|---|
| A（段序列闭式） | `m2-impl-c577-barrier-implementer-report.md:75` | ✓ 逐字（`grep -n "第一条流："` 命中恰在 75 行） |
| B（隔离路径步数） | 同文件 `:72`、`:76` | ✓ 逐字（`grep -n` 命中恰在 72、76 行） |
| C（σ 段闭式） | 同文件 `:77` | ✓ 逐字 |
| D（单元区起点） | `m2-rev-a3b-implementer-report.md:30/84`、`m2-rev-a3b-fallout-1-implementer-report.md:132` | ✓ 逐字，三处行号精确对上 |
| E（1 MiB 环在飞上限） | `singlefs-format/src/lib.rs:221`（`JOURNAL_SAFETY_FACTOR`）、`m2-rev-a3b-fallout-1-implementer-report.md:12` | ✓ 逐字 |
| F（I-7.13 字段界） | `.claude/kb/invariants.md:64`、`.claude/kb/decisions/22-单元原子性怎么合成.md:72` | ✓（F2 引文与快照第一节已核过的 D22 决策一致；数字区间 [4096, 1 MiB − 4096]、[457, 槽距] 与源文一致） |

六组 pinned value（67108885、16777240、35、20、262144/65536、50176/1408/11200、85、1044480）本身：逐个用 `python3 -c` 现算，与提示文件给的 pinned 值全部相等（六组 12 个数逐一核对，未发现算错）。

### 5.2 翻译核对表（16 项）逐条核

`-translation-audit.md` 16 项转述，逐条把「原文文件:行」栏现读一遍，与「英文项」栏比：
- 第 3 条自陈首稿行号错（`:71`→`:75`/crash.rs），已在定稿改对；现查 `crash.rs:621` 含 `E77（发布的持久顺序） 的闭式：1 + Σ(2^|段| − 1)`，与英文 A2 对应，✓。
- 第 10、12、16 条是本地攻方自己承认的「追加/补的」条目（D4 公式改写、E2 删背景句、F2 追加来源），三条都写明了理由与出处，现查理由成立（D2/D3 原文确实各自写出「× 64」系数；E2 删的两句确实是背景理由不是算术输入；F2 的数字上界确实来自决策 22:72 而非 invariants.md:64 本身）。
- 其余 12 条逐句核英译与中文原文，未发现丢限定词或多加限定词的情形（英译对数字、常量名、行内代码逐字保留）。
- **16 项全部 ✓**，无 ✗。

### 5.3 运行记录（4 次调用）与译文核对表本身

已在第一节用 `sha256sum`/`wc` 现核：提示文件、译文核对表两份 sha256 前后缀对上；`-void1.md`/`-void2.md` 字节数、`-s1.md`/`-s2.md` 词数、`oov-check.py`/`corruption-check.py` 判绿结果，均与运行记录逐字对上。运行记录第 26-27 行「交回失误记录」是如实自报，不属于引文核对范围，未再深究。

## 六、rerun.sh 复跑（在草稿副本上跑，不在腿的原目录）

命令：`nice -n 19 bash research/prompts/m2-closeout-code-r2-opus-model/rerun.sh /tmp/claude-1000/three-way-verifier-r2/rerun 4`（`run_in_background` 起，任务 `bsuxb9h0o`）。
脚本本身一处小问题：它的用法注释写「[线程上限，缺省 4]」，但脚本体里 `$N` 这个参数从未被引用——`CAP1`/`CAP2` 一直硬编码 `capped.sh 1`/`capped.sh 2`（与 opus 报告开头「线程上限 4（两件并行时每件 2）」吻合，是有意的 1+1+2+2=4 拆法，不是随 `$N` 缩放），传参数 2 不会改变实际并行度——文档与实现不一致，不算复跑本身出错。

已跑完并逐字段核对（草稿路径与临时路径不同，按字段比、不按整份哈希比）：

| 日志 | 结果 |
|---|---|
| `logs/y2.log` | 7 个 `test ... ok`，全部 `Y2 ...` 数据行与报告 `logs/y2.log` 原文逐字相同；`test result: ok. 7 passed`（挂钟 28.86s vs 报告 15.27s，字段口径不同，机器当时负载高，判等价） |
| `logs/y4.log` | 2 个 `test ... ok`，全部 `Y4 ...` 数据行逐字相同；`test result: ok. 2 passed`（挂钟 21.21s vs 报告 9.90s） |
| `logs/a2a.log` | 末行 `A2A runs=189 runs_with_two_sc_writes_on_one_device_in_a_segment=0 runs_with_fua_after_unreleased_plain=0 worst_per_device=1` 与报告完全一致，仅 `seconds=` 不同（157.5 vs 88.8） |
| `logs/z3a-existing.log` | `test result: ok. 8 passed; 0 failed`，与报告一致（仅挂钟不同） |

**核了 4/6 组日志，4 组全部字段级一致，0 处对不上。** 剩余两组（`b1-1.log`/`b1-2.log` 单故障逐序号扫，约需 28 分钟；`y6-nofile.log`/`y6-unmount.log`；`fix-regression.log`/`snapshot-regression.log`）本报告落盘时仍在跑（本机同时有别的会话在跑重型测试与编译，负载显著高于 opus 腿原跑时，挂钟会更长，但字段应不受影响）。

## 七、计数与没做什么（rerun 未完成前的中间版，完成后本节结尾会再追加一段）

**opus 报告**：核 30 处引文，✓ 28，✗ 2（D13-验证路线.md:72 应为 71，两处）。
**sonnet 报告**：核 24 处引文，✓ 21，✗ 3（walk.rs 六个行号全错；invariants.md:12 应为 11；「槎/槽」字符系统性替换 + 引文截断三处，均见第四节）。
**本地攻方腿**：翻译核对表 16 项全 ✓；事实表 A–F 六组 pinned 值现算全部正确；运行记录里的 sha256、字节数、词数、判绿结果，逐项与产物现核，全 ✓，0 处对不上。
**产物文件（sha256）**：opus 模型目录 25 个文件 + 快照 57 份 src 文件，逐个现算，0 处对不上。
**复跑（rerun.sh）**：已完成并核对 4/6 组日志（y2、y4、a2a、z3a-existing），字段级 0 处对不上；余下 2 组（单故障逐序号扫 b1-1/b1-2、Y6 no-file/unmount、5 份回归）本报告落盘时仍在后台跑（本机同时段有别的会话在跑重型测试与编译，负载显著高于报告原跑时）。
**核不动**：0 处（未遇到需要虚机/网络、或包装跑不起来的项）。
**分不清**：0 处（本轮开工快照给了 kb-at-start 且主 agent 已现核，未出现「文件在腿开工之后被改」这类需要判「分不清」的情形；`checks-owed.md` 已按输入一节指示，改用背景材料附录整段抄的那一份核，未记「分不清」）。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑（本地攻方样本 s1/s2 对六组事实表算术答案本身是否正确、彼此是否一致，不属于本报告判断范围）。
- 未对 sonnet 报告里没有「文件:行号」标注、纯靠自然语言复述条款大意的段落（例如 Y5 问 2 关于「checker 行为精确复现生产路径」的论证段）做逐句核对——这类段落没有可核的引文形式，按定义不核推理本身。
- 未复核 opus/sonnet 两份报告里的「四句」判别力自证栏本身写得对不对（是否真分辨了臂、系统当时看不看得到）——那是推论内容，不是引用/产物/复跑。
- 未跑重型测试；未改主工作区；未做 git 写操作。

## 八、rerun.sh 最终状态（更新第六节：任务已结束，非全部跑完）

第六节写的时候复跑仍在跑；此后机器负载持续很高（`ps` 现查：另有会话在跑 `second_transaction_supplement_three_random_history`、`first_transaction_step_seven_layer0` 等重型用例与整轮编译），`b1-1`/`b1-2` 单故障逐序号扫在我终止它时才走到 `unmount` 读序号约 180/283、`crash_remount` 读序号约 148/2240（抽样后的取样数），远未到报告的 784+2240=3024 段全量；Y6 no-file 枚举在终止时走到 2275/4155 个状态（35/64 slice）。**这两处不是发现的差异，是我在时间预算内主动截停的结果**——已完成片段里出现的每一条 `SWEEP`/`LAYER0_PROGRESS` 行与报告同格式、同字段名，没有看到任何字段异常，但样本量不足，不能当「核对」计入 ✓ 或 ✗，记「核不动」（时间预算内没跑完，不是包装跑不起来）。Y6-unmount 与 5 份回归对比（`fix-regression.log`/`snapshot-regression.log`）根本没开始跑。

停止方式：按共用约束「要停自己起的一条链」逐层列出子进程（`3022977`→`3022979`(rerun.sh)→`212077`(run-with-memory-cap.sh，Y6 阶段)→`213052`→`213102`(cargo)→`247305`(测试二进制，最先起）），从最深的叶子逐个用 `proc.py stop` 单独停一次（`247305` 停下之后整条链自然退出，`run_in_background` 任务通知里 rerun.sh 自身以 `RERUN_EXIT=101` 结束，非 0，因为 `set -euo pipefail` 撞上被终止的子进程）；此前更早的 `b1-1`/`b1-2` 那一段是我先单独停了它的测试二进制叶子 `3688508`/`3688511`（cargo 随之自然退出），脚本沿 `set -e` 往下走到了 Y6 阶段才被我再停一次。停完确认：`/proc` 全表扫，与 `three-way-verifier-r2` 相关的进程只剩当前这条 Bash 调用自己，无残留。

**计数更新**：复跑核了 4 组（y2/y4/a2a/z3a-existing），✓ 4，✗ 0；另 2 组（单故障扫、Y6）记「核不动」（时间预算内主动截停，非包装失败）；回归对比 1 组未跑，记「核不动」。第七节「计数与没做什么」里「余下 2 组仍在跑」一句以本节为准：**已停止，未跑完，原因是时间预算，不是环境或包装问题**。
