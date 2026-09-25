# m2-safety-r2 核查报告（three-way-verifier）

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

抽样：Sonnet 报告第 19-26 行引 `admission.rs`（冻结快照）222-225 行整段文档注释。在草稿目录副本
`/tmp/claude-1000/m2-safety-r2-verifier/selftest/admission.rs` 里把行号加 1（核 223-226 而不是
222-225）：223-226 缺第一行「式子里的『已分配』……」、多了下一行 `pub allocated: BytesOnOneDevice,`，
逐字比对不上，判 ✗。核查方法能分辨行号错位，往下正常核。

## 快照与产物完整性（先核，不进下面计数）

- 代码快照：`/tmp/claude-1000/m2-safety-r2/tree/crates/` 对 `crates-sha256.txt`（去掉 `#` 行）
  `sha256sum -c`：125 个全 `OK`（`/tmp/claude-1000/m2-safety-r2-verifier/crates-check.log`）。
- kb 快照：`/tmp/claude-1000/m2-safety-r2/kb-snapshot/` 对 `kb-sha256.txt`：5 个全 `OK`。
- 三份报告 sha256：
  - `m2-safety-r2-sonnet-output.md`：`063dfdd1b1c36373428551e963174b8a65f70e886749914abdc062c5d5dea24e`，
    与交回时给的完全一致。
  - `m2-safety-r2-opus-output.md`：`c488004c08eafc60c295894a890a128fd38e25299339ff5ecbd798d8937f7bf1`，
    与交回时给的完全一致。
- Opus 模型目录 `SHA256SUMS`（19 个文件，含 `core-arms.patch`、`opus_s4_attack.rs`、`out/` 下全部日志与
  摘要、`rerun.sh`、三份 `summarize_*.py`）：`sha256sum -c SHA256SUMS` 全部 `OK`。

## 关键方法说明：Sonnet 报告的行号基准是「打完补丁之后」的工作树

Sonnet 报告第 9 行明written「以下代码引用的行号，是这份工作树（不是主仓 `crates/`）里改完之后的行号」。
我把冻结快照 `crates/` 拷进草稿目录，用 `research/prompts/m2-safety-r2-sonnet-model/diff/*.patch`
（六个文件）`patch -p1` 打上（全部 `checking file` 干净应用，无冲突），得到
`/tmp/claude-1000/m2-safety-r2-verifier/sonnet-reconstructed/`。下表除标注「冻结快照」的几行外，
均对这份重建树核；这不是对主树核（未给该轮主树权限），是对腿自己声明的基准复原后核，等价于
「引产物就整行抄」里「回读产物、逐字比对」那一步，只是产物是「快照 + 已核过的补丁」而不是一份现成文件。
Opus 报告的代码引用全部显式标「冻结副本」，对 `/tmp/claude-1000/m2-safety-r2/tree/crates/` 原样核；
`core-arms.patch` 另外验证——`opus-model/rerun.sh` 自带 sha256 门槛核对冻结副本的三个源文件，
`patch -p1 --dry-run` 干净应用，与其复跑一并验证（见下）。

## 一、云端正推（Sonnet）`m2-safety-r2-sonnet-output.md`

### 1.1 文件:行号 + 抄的原文 / 接线图行号

| 引用 | 核的结果 | 命令/依据 |
|---|---|---|
| `admission.rs:222-225`（`DeviceAdmissionTerms::allocated` 文档注释，整段） | ✓ 逐字一致（冻结快照与重建树该段未改动，两处核一致） | `grep -n` 定位 + `sed -n` 逐字比 |
| `allocator.rs:298`（`allocated_slots()`） | ✓ | 同上，重建树 |
| `allocator.rs:312`（`mark_released()`） | ✓ | 同上 |
| `allocator.rs:1601`（测试断言「占着的槽数不变」） | ✓（对重建树；冻结快照里该断言实为 1586 行，补丁在其前插入 15 行，1586+15=1601，与报告称「行号是改完之后」的声明一致） | `patch -p1` 重建 + `sed -n '1601p'` |
| `admission.rs:623-644`（`S4Candidate` 枚举定义） | ✓ | 重建树 `awk 'NR==623,644'` |
| `admission.rs:390-448`（表：`available_on_each_device_for_candidate`） | ✗ 范围偏——函数与其 `impl` 块实际在 390-444 行结束，445-448 行已经是下一个结构体 `DemandOnDevice` 的文档注释与派生宏，与该函数无关 | 重建树逐行核 |
| `admission.rs:494-541`（表：`admit_on_every_device_for_candidate`） | ✗ 范围偏——函数实际到 551 行才闭合（541 行只到 `filter_map` 内部一半），494-541 漏掉函数后半段（`short_devices.is_empty()` 判断与返回） | 重建树逐行核；正文另引的子范围 `513-527`（净释放豁免判据）✓ 精确 |
| `admission.rs:562-582`（`mount_admission_a4_switch_reserve_only`） | ✓（与报告「没做什么」自陈的修正 562-580→562-582 一致） | 重建树 |
| `admission.rs:605-621`（`S4Candidate` 文档注释） | ✓（与报告自陈修正 605-620→605-621 一致） | 重建树 |
| `admission.rs:142-143`（`PENDING_DELETE_OF_THE_FIRST_VERSION`） | ✓ | 重建树 |
| `admission.rs:411-419` / `420-425` / `427-433`（Baseline\|A3\|A4 分支 / A1 分支 / A2 分支） | ✓ 三段全部精确匹配 | 重建树 |
| `transaction.rs:4560-4595`（发布路径调用点） | ✓（区域准确，含调用与判定逻辑） | 重建树 |
| `mount.rs:2089-2120`（挂载路径调用点） | ✓（区域准确） | 重建树 |
| `.claude/rules/fs-design.md:23`（「释放空间……不需要申请空间」所在行） | ✓（该文件不在本轮 kb/crates 快照范围内；设计轮对主树核，逐字一致，无「分不清」需要记） | 主树 `awk 'NR==23'` |
| `.claude/kb/decisions/03-空间分配.md:180-197`（已定项 9 整节） | ✓ | kb 快照 `awk 'NR==180,197'` |
| `.claude/agent-common.md` 存在、`.claude/agents/agent-common.md` 不存在 | ✓ | `ls` 现查 |
| `research/prompts/alloc-basis-forks.md` 第 9/10 行（岔路 1/2） | ✓（与报告「没做什么」自陈修正 8/7→9/10 一致） | 现查该文件 |

### 1.2 复跑（草稿目录 `/tmp/claude-1000/m2-safety-r2-verifier/sonnet-rerun/`，冻结快照 + 六个 diff 补丁重建，
`bash research/scripts/run-with-memory-cap.sh 10G bash research/scripts/capped.sh 12 cargo test …`）

| 复跑目标 | 结果 | 命令/产物 |
|---|---|---|
| run1/run2 的 240、256 两个宽度是否逐字一致 | ✓ 确认：`sort` 后 `diff` run1 与 run2 在 240/256 范围内只差 1 行（S4R2-B width=240 candidate=baseline 的首行），且该行在 run1、run2、我的独立复跑里都是同一种「cargo 进度行与 println 挤在同一行」的格式伪差，不是数据差异；D3I9（`s4-r2-candidates-run1.log`）方向同理，run2 因中途被停没有跑到 D3I9（见下），不构成不一致 | `rerun-240-256.log`（独立编译重跑，未改产品代码，只改测试文件的 widths 数组把 384 去掉以避开 A3×384 的长循环）；`run1-sorted.txt`/`run2-sorted.txt`/`mine-sorted.txt` 三方 `comm` 比对 |
| 独立复跑 240/256 的 S4R2-B 与 S4R2-D3I9 数据 | ✓ 与 run1.log、正文表格逐格一致（会话上限、mount 首拒 k、D3I9 当场成不成功） | 同上日志 |
| 384 槽 A2「会话上限 22、挂载第 20/21 次首拒」 | ✓ 双重确认：(a) `run2.log` 原文第 141-164 行本身就有 `overwrites_admitted_in_one_session=22`、k=20 时 `again` 首次 refused、k=21 时 `next_writable_mount` 首次 refused；(b) 我独立重建的测试（`s4_r2_candidates_384_a2_a4.rs`，只留 384 槽 + A2/A4 两条候选，避开 A3 的长循环）跑出完全相同的三行 | `rerun-384-a2-a4.log` |
| 384 槽 A4「一次都没拒」 | ✓ 双重确认：(a) `s4-r2-width384-a4-run1.log` 原文 k=0..10 全部 `Applied`/`Applied`；(b) 我的独立重建同一测试同样跑出 k=0..10 全 `Applied` | 同上日志 |
| run2 的包装脚本是否在跑着的时候被改过（主 agent 的推论，要求复跑坐实） | ✓ 观测支持：`s4-r2-candidates-run2.log` 原文最后几行在测试二进制自身的 `SIGTERM` 退出信息之后，紧跟着一段来自 `run-with-memory-cap.sh` 本体的 `line 1006: syntax error near unexpected token '<'`，其后引用的字面串是「没给上限与命令」；今天的 `research/scripts/run-with-memory-cap.sh` 里这句话确实存在，但在第 991 行，不是 1006 行——说明触发这条报错时脚本文件的字节内容/行位置与今天不同，与「跑着的时候被原地改过」这一推论方向一致。同时确认：run2.log 里全部 `S4R2-B`/`S4R2-D3I9` 数据行在触发这条报错之前已经写完且与其他来源逐字一致（见上两行），支持「测试二进制自己打出的结果行不经过包装层的收尾」这一句——数据行是测试二进制直接写到被外层重定向捕获的 stdout，不依赖包装脚本收尾时那段代码是否完好 | `grep -n 'syntax error\|没给上限与命令' s4-r2-candidates-run2.log research/scripts/run-with-memory-cap.sh` |
| Z19-B 基线（`baseline-z19b.log`） | 核不动：未复跑，非本轮点名优先项，时间预算内未做 | 见「没做什么」 |

**1.1 计数**：核了 19 处，✓ 17 处，✗ 2 处（`admission.rs:390-448`、`admission.rs:494-541` 两处表格行号范围偏，均为「接线图」表格的整体函数边界，不影响文中另引的精确子范围与函数起点）。
**1.2 计数**：核了 6 处（含 1 处主 agent 推论的复跑坐实），✓ 5 处，核不动 1 处（Z19-B 基线未复跑）。

## 二、云端攻方（Opus）`m2-safety-r2-opus-output.md`

### 2.1 文件:行号 + 抄的原文（冻结快照，未打补丁的原文）

| 引用 | 核的结果 |
|---|---|
| `transaction.rs:4675`（「走得完」那段注释整行） | ✓ 逐字一致 |
| `admission.rs:224`（「⚠️ 它含 defer……」整行） | ✓ |
| `mount.rs:159`（D2 已定项13、D23 已定项14 引用行） | ✓ |
| `mount.rs:1084`（「今天只有测试入口」整行） | ✓ |
| `admission.rs` 151/192/210（`INSTANCE_SWITCHES_ALLOWED_PER_MOUNT` 三处命中） | ✓（`grep -rn` 全仓只命中这三行） |
| `.claude/rules/fs-design.md:23` | ✓（同 1.1 表） |
| `.claude/kb/decisions/28-挂载期承诺量.md` 第 26/34/81/83/84/85 行 | ✓ 六处全部逐字一致 |
| `.claude/kb/decisions/03-空间分配.md` 第 52/184/185 行 | ✓ 三处全部逐字一致 |
| `.claude/kb/decisions/16-发布语义.md:40`（准入那一行） | ✓ |
| `core-arms.patch` 对冻结快照 `patch -p1 --dry-run` | ✓ 干净应用，无冲突 |

### 2.2 引产物（逐字/逐格核对 `m2-safety-r2-opus-model/out/` 下原始日志与摘要）

| 引用 | 核的结果 |
|---|---|
| `E7 240 A1 step4` 整行（A1-1） | ✓ `grep -F` 命中 `e7.log` |
| `E7 240 A3ge step4/15/16` 三行（A3-1） | ✓ 三行全部 `grep -F` 命中 |
| `E1 slots={240,240,240,384,640} arm=A4` 五行（A4-1） | ✓ 五行全部命中 `e1.log` |
| e5.log「88 格 2472 个崩溃点、0 违例」（4.2） | ✓ 精确：`grep -c '^E5 '`=108，其中 `delete=ok`=88、`delete=SpaceAdmissionRefused`=20（这 20 行原文自带「没有崩溃点可枚举」）；88 行 `crash_points=` 之和 = 2472；`viol_points=` 之和 = 0 |
| e2-summary.txt「吸收态格数」A3ge 20/63、A3d 26/63、A2+A3ge 24/63、A2+A3d 26/63、A2+A3ge+A4 16/63，其余 0/63（A3-4） | ✓ 精确：写小脚本统计每条臂 63 格里 `---` 的个数，逐条与正文数字相符 |
| e3-summary.txt / e3-zero-round-summary.txt：T 40/60、A1 16、A2 0/69、A4 0/100、ND+A1n 0/63、A3gt 62、A2+A3gt 10、A1x 0（多处） | ✓ 全部精确匹配（A1-1、A1-3、A3-2、A4-1 四节引用的数字逐一核对） |
| e4-summary.txt 4.1 节表格 8 行（T/A1/A2/A4/A3gt/A2+A3gt 的建inode/抬F/重挂等列） | ✓ 全部精确匹配（含报告自己标的「节选」7 列，与全表对应列逐格核对） |
| e6.log 误差直方图（A1-1「768/1024/2048 三个盘宽直方图逐项相同」；A1-3「512 槽以上四个盘宽直方图逐项相同，19 次多扣 6、37 次多扣 18-20、4 次过渡」） | ✓ 精确：`formula=T` 在 768/1024/2048 三档直方图字面完全相同；`formula=ND` 在 512/768/1024/2048 四档直方图字面完全相同；`-6: 19`、`-20: 30 + -18: 7 = 37`、其余四项（-14/-12/-7/-4 各 1）之和 4，全部对上 |

### 2.3 复跑（草稿目录 `/tmp/claude-1000/m2-safety-r2-verifier/opus-rerun/`）

用腿自带的 `research/prompts/m2-safety-r2-opus-model/rerun.sh /tmp/claude-1000/m2-safety-r2/tree/crates
/tmp/claude-1000/m2-safety-r2-verifier/opus-rerun /tmp/claude-1000/m2-safety-r2-verifier/opus-rerun/out`
整份复跑（脚本自带 sha256 门槛核对冻结副本三份源文件，10G 内存上限 + 8 线程上限，未改路径以外的任何参数）。

| 复跑目标 | 结果 |
|---|---|
| E1–E7（含 E3/E4 的零轮臂分支）九次 `cargo test` 调用 | ✓ 全部 `test result: ok`，与原产物 9 个 `.log` 文件逐份对比：每份文件里全部 `E1`–`E7` 数据行与 `test result` 行，除去掉「finished in X.XXs」这个挂钟耗时字段外，逐字节完全相同（`diff` 只在这一个字段上有差异，9 个文件全部如此） |
| A1「S4 从 40 降到 16、240 槽第 5 次覆盖写可用 21 放行、挂载读到 −7」 | ✓ 复跑产物与原产物在这几行上完全相同（见 2.2 表已核） |
| A3 吸收态格数 | ✓ 复跑产物 e2-summary.txt 与原文件字节相同 |
| ND+A1n「S4 为 0、S4b 63 次」 | ✓ 复跑产物 e3-zero-round-summary.txt 该行与原文件字节相同 |
| 删除类发布中间崩 2472 个崩溃点 0 违例 | ✓ 复跑产物 e5.log 与原文件字节相同（含 88/20 的格数拆分） |

**2.1 计数**：核了 10 处，✓ 10 处。
**2.2 计数**：核了 7 组（共约 30 余个具体数字/行），✓ 7 组全部精确匹配，0 处不符。
**2.3 计数**：复跑 9 次 `cargo test` 调用，✓ 9 次全部通过且与原产物逐字节一致（仅挂钟耗时不同）。

## 三、本地攻方两份干净样本（s2、s3）+ 译文核对表 + 运行记录

只核事实表出处行号与译文（s1、void1 判损坏不算干净样本，按定义不核其答案）。

### 3.1 译文核对表 `m2-safety-r2-local-attack-translation-audit.md` 逐条出处行号

| 引用 kb/代码 位置 | 核的结果 |
|---|---|
| `28-挂载期承诺量.md` 第 19/24/25/26/27/34/81/83/84/85/103/108 行 | ✓ 十二处全部逐字一致（kb 快照） |
| `03-空间分配.md` 第 52/184/185 行、180-197 区间、206-213 区间 | ✓ 全部逐字一致（kb 快照） |
| `18-块里携带什么信息.md:216` | ✓ |
| `16-发布语义.md` 第 33/36/40 行 | ✓ |
| `admission.rs:581`（`admission_reading_before_a_publish` 定义）、`admission.rs:222-225` | ✓（对冻结快照核，核对表自己写明「行号现查冻结副本」） |
| `transaction.rs:4572`、`mount.rs:2100`（两处调用点） | ✓ 两处均命中该函数调用 |

核对表里「首稿缺的」「定稿怎么补」栏（Fact 8 的「删不删还开着」、Fact 11 的「只对覆盖写生效」、
Fact 14 的「只有一层的树没有『根以下的节点』」等）逐条与本轮所引 kb/代码原文核对，未发现多加原文没有
的限定词，也未发现丢限定词；Fact 15 明确记录「有意收窄，不搬『第 4 新的非空根』这个具体上限值」并给出
理由，属于核对表自证的「不抄，因为……」形式，不是无声消失。

### 3.2 运行记录 `m2-safety-r2-local-attack-runlog.md`

| 声明 | 复核结果 |
|---|---|
| s1：退出码 0，corruption 绿，oov 绿（生词=1: refuteail），人工判「带损坏」覆盖自动判定 | ✓ 重跑 `corruption-check.py`/`oov-check.py` 得到完全相同的绿/生词=1/`refuteail` |
| s2：退出码 0，两绿，生词=11（`shortfall`、`refutation`），词数 585、行数 91 | ✓ 完全一致，`wc -w`/`wc -l` 复核相符 |
| void1：`corruption-check.py` 判红，实词自复读=1（`reclaimable`），词数 194（脚本内部计数）/380（`wc -w` 实际） | ✓ 完全一致，包括 194 与 380 的差异说明本身 |
| s3（沿用 void1 的号重跑）：退出码 0，两绿，生词=0，词数 310、行数 23 | ✓ 完全一致 |
| 提示文件 `m2-safety-r2-local-attack.md` 全英文、无 markdown 强调、四次调用均前台跑 | ✓ `file` 判定纯 ASCII，`grep -c '\*\*\|##'` 为 0，无 CJK 字符 |

**3.1 计数**：核了 21 处，✓ 21 处。
**3.2 计数**：核了 5 条声明，✓ 5 条。

## 四、总计数

| 腿 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| Sonnet（1.1 + 1.2） | 25 | 22 | 2 | 1 |
| Opus（2.1 + 2.2 + 2.3，2.2/2.3 按组计） | 26 | 26 | 0 | 0 |
| 本地攻方（3.1 + 3.2） | 26 | 26 | 0 | 0 |
| **合计** | **77** | **74** | **2** | **1** |

## 五、没做什么

- 未判任何一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- Sonnet 的 `s4-r2-candidates-run1.log` 384 槽 A3（k 循环 0..=80）与 `run2.log` 同一段没有复跑到底——
  这是腿自己在报告里已声明的欠账（k=66 处主动停止），不在本轮优先复跑范围内，也没有另行复跑验证
  「k=67..80 没有反转」这句话；核查方法上这不需要我验证（腿自己没有据此下断言）。
- 未复跑 Z19-B 基线（`baseline-z19b.log`），非本轮点名优先项。
- 未验证 Opus 的三份 `summarize_e2.py`/`summarize_e3.py`/`summarize_e4.py` 脚本内部实现的正确性——
  只核了它们的输出（`.txt`）与原始 `.log` 数据能否对上（用独立脚本重新统计 e2-summary 的「吸收态格数」
  验证过一遍，与摘要文件相符），未逐行读脚本源码找潜在实现错误。
- 未检查 `m2-safety-r2-opus-model/opus_s4_attack.rs`（腿自己的攻击脚本源码）本身的实现是否有 bug，
  只核它编译通过、跑出的产物与报告引用一致、且独立复跑可重现。
- 未检查 `_m2-safety-r2-body.md`、`_m2-safety-r2-appendix.md`、`_m2-safety-r2-checklist.md` 背景材料本身
  是否有腿把行号误写成背景材料行号的情况——检查过程中核到的全部行号引用都能在其点名的 kb/代码文件里
  逐字命中，没有发现需要改记「误写成背景材料第 N 行」的情况。
- 未跑 `gate.sh`、未编译整个 workspace、未跑重型测试；复跑只涉及两条腿点名的测试目标（`singlefs-core --lib`
  未跑，只跑了 `singlefs-harness` 下的对拍测试与 `opus_s4_attack`），符合子 agent 不跑重型测试的约束。
- 本地攻方样本 s1/void1 按定义判损坏不核其答案，未核其内容正确性（只核了运行记录里关于它们的元数据声明）。
- 未对 `.claude/kb/decisions/28-挂载期承诺量.md` 等文件里除报告点名行号之外的其余内容做通读式核验。
