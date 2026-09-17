<!-- 主 agent 转存：核查员（three-way-verifier）说它没有写文件的工具，报告只在交回消息里；下面从「核查员报告」起是那条消息的原文，一字未改。 -->

核查员报告（alloc-basis-r3）。核对表里的 ✗ 不免除主 agent 对推论的逐条现查，这一句照抄进报告开头。

## 0. 判别力自证（先做）

取 Opus 报告引用「`effective_rollback_floor`（recovery.rs:351）」，在草稿目录副本（`/tmp/claude-1000/alloc-basis-r3-verifier/selftest/recovery.rs`）里把引用行号从 351 改成 352（+1）后核对：
- 原行（351）：`pub fn effective_rollback_floor<Reader: PoolReader + ?Sized>(` —— 与报告引用一致，✓
- 偏移后（352）：`    reader: &Reader,` —— 与「effective_rollback_floor」定义不符，判 **✗**

判别力自证通过：方法能分辨对错行号。

## 1. Opus 攻方腿（`research/prompts/alloc-basis-r3-opus-output.md`）

**核心复跑**：把 `research/prompts/alloc-basis-r3-opus-model/` 整份拷进 `/tmp/claude-1000/alloc-basis-r3-verifier/opus-check/`，改 `run-probe.sh` 的 `$model` 指向草稿目录（避免写回 `research/prompts/`），`nice -n 19` 跑 `outputs-rerun`：两次各自编译 vis/patched 副本（release），跑全部 53 份 `.txt` + 4 张表，脚本自带的逐个 `cmp` 判定 **「复跑与 outputs/ 逐个相同」**，退出正常。额外用 `diff` 比对两份 `SHA256SUMS`（草稿产出 vs 仓里 `outputs/`）：**完全一致**。这是对整份报告数值证据最强的单次核实——57 个产物文件全部字节级可复现。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `outputs/SHA256SUMS` 等 14 个文件在报告表里列的 sha256 | ✓ 全部逐字节匹配（`sha256sum`；`SHA256SUMS` 本身、`h1-T1.txt`、`h1f-T1.txt`、`h4-earlier-T1.txt`、`h4-later-T1.txt`、`h4c-T1.txt`、`h5-hole-advance-one-T1.txt`、`h6-z5-bug-at5-T1-G8.txt`、`z2adv-Fkou-cap.txt`、`z2-badimage-hold-removed-checkerNewestF.txt`、`x8a.txt`、`probe.rs`、`patch-copy.py`、`extract-tables.py`、`run-probe.sh`） | `sha256sum <文件>`，逐个比对报告表格 |
| 6 个 `crates-at-report/*.rs` 的 `git hash-object` | ✓ 全部匹配报告与正文第三节记的哈希（`allocator.rs 66c9eb1d…`、`mount.rs b228b7f4…`、`transaction.rs 6dd2ef9a…`、`recovery.rs ef4da610…`、`image.rs bea51e16…`；`walk.rs` **报告如实标出与正文第三节不符**，报告读的是 `1caa43f0…`，正文写的是 `b09366bf…`） | `git hash-object` |
| walk.rs 哈希差异是「正文第三节陈旧」还是「报告读错」 | ✓ **报告对**：今天工作区 `crates/singlefs-checker/src/walk.rs` 的 `git hash-object` 就是 `1caa43f0…`，与报告读的一致；正文第三节的 `b09366bf…` 是陈旧记录 | `git hash-object crates/singlefs-checker/src/walk.rs`；`git log --oneline -3 -- <该文件>` |
| §1.2 `outputs/h1-T1.txt:41-42`（m1 被豁免/回收/发出的读数） | ✓ 整行匹配报告引用 | `sed -n '41,42p'` |
| §1.2 `outputs/h1f-T1.txt:2-3`（抹掉回退实例根槽后 `UnitUnreadable { slot: SlotNumber(50176) }`） | ✓ 整行匹配 | `sed -n '2,3p'` |
| §3.4 第九项七行表（H1/H1m/H4×2/H4c×2 各历史的 `orig`/`set_reading`/`g5_def`/`narrow_by_valid`/`count_reading`/`G5_isolated`） | ✓ 7 行全部与产物 `grep -h "ninth"` 的原始输出逐字段一致，包括「两次回退少 20 槽」（44 vs 64 隔离数）与「回退之前抬过 F 上 −33 对 48」 | `grep -h "ninth" outputs/*.txt \| grep "after-rollback\|after-raise"` |
| §4.1 G8 坏镜像 txg 5–27 共 23 个状态只 G8′ 红、其余全绿 | ✓ 用 Python 逐行核对 29 个状态行，txg5–27（23 个）全部只 `G8p[` 红且无分号（无其他不变量红）；txg28 起才多出 I-3.1/I-2.1 等 | 见下方复算脚本输出 `only G8p in txg5..27: 23` |
| §4.2 G8′ 在 398 个合法状态行上 0 次误红 | ✓ 逐文件数状态行数：24+36+34+34+39+10+10+48+68+66+29=**398**，`G8p[` 命中 0 | `grep -c 'acct_alloc=.* \| '`、`grep -oP '(?<=\|).*' \| grep -c 'G8p\['` 逐文件核 |
| §4.1 `h1m-T0.txt` 里 G8（非 G8′）红 5 次 | ✓ | `grep -oP '(?<=\|).*' outputs/g8/h1m-T0.txt \| grep -c '\bG8\['` = 5 |
| §1.4① `h5-hole-advance-one-T1.txt:1,45`（甲-b′ 并集比走读少 1 槽：`acct1−precise=1`） | ✓ 整行匹配 | `sed -n '1p;45p'` |
| §2.4 `outputs/x8a.txt` 三行整行引用 | ✓ 逐字匹配（含 F-扣 reclaimed=64/G7 deferred=64/`after_release=Some(50240)`） | `cat outputs/x8a.txt` |
| §2.3 `mount.rs:477-481`、`:506` 循环代码引用 | ✓ 与今天工作区代码逐字匹配 | 已随 hash-object 核实（同一版本） |
| §2.3 引「步 4/步 5 第三轮判决第五节 4」`release_reclaim_holds` 欠账句 | ✓ `m2-step45-code-r3-main-verification.md:58` 逐字匹配 | `grep -n` |
| 「07:14–07:19 UTC 复跑逐字节相同」这句自证 | ✓ 本核查员在草稿目录独立复跑一遍，结论同（不是抄它自己的自证，是另一次独立编译） | 见上「核心复跑」 |

**Opus 计数**：核了 17 处（含 1 次整体复跑），✓ 17，✗ 0，核不动 0。

## 2. Sonnet 辩方腿（`research/prompts/alloc-basis-r3-sonnet-output.md`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/kb/decisions/16-发布语义.md:372/374/376` | ✓ 三行逐字匹配 | `sed -n` |
| `.claude/kb/invariants.md:50`（I-7.4） | ✓ 逐字匹配（含「有效」按 C113 P3 判的那句） | `sed -n '50p'` |
| `.claude/kb/decisions/23-journal的角色与格式.md:1209` | ✓ 逐字匹配（主句到句号止） | `sed -n '1209p'` |
| `.claude/kb/layout/01-first-txn.md:359`（回退下界 F 行） | ✓ 逐字匹配 | `sed -n` |
| `alloc-basis-r2-main-verification.md:22, 26` | ✓ 内容匹配报告问 2/问 1 引用 | `sed -n` |
| `_alloc-basis-r2-body.md:77,86,90,92` | ✓ 全部逐字匹配 | `sed -n` |
| 「此前误写 1143/1139，是背景材料拼合文件里的行号」 | ✓ **坐实**：`_alloc-basis-r3-background.md:1139` 正是「Z1 乙′ 的 I-5.2」那一行，`:1143` 正是「Z3 串」那一行，与 r2-main-verification.md 22/26 内容一致 | `grep -n "Z1 乙′ 的 I-5.2\|Z3 串 \|"` |
| §3.8「若有效根含 F，抬 F 上限的式子会循环定义」 | **需要主 agent 关注的发现，非简单✓/✗**：D16 已定项 1 那张表里「环里最旧有效根」定义确实不含 F（支持 Sonnet 的读法），但**今天 `crates/singlefs-core/src/mount.rs`（hash `b228b7f4…`）的 `rollback_floor_ceiling`（第 331-380 行）在计算喂给「非空持久有效根」（抬 F 上限公式的输入）的候选集时，明确加了 `.filter(\|root\| root.checkpoint_txg >= current_floor)`（第 345 行）——即代码里「有效根」候选集在算上限之前已经按当前 F 过滤过一次**。这不构成严格循环（用的是已知的 current_floor，不是待算的新上限），但说明 Sonnet「含 F 会循环因此必为 F-free」这条论证的机制在代码层面并不成立——代码本身就把 F 过滤与「有效」判定接在一起做，只是没有循环。Sonnet 未碰代码（其「没做什么」明写「没有重新跑 crates/ 里的任何代码」），这是本核查员核代码之后新增的观测 | `sed -n '331,352p' crates/singlefs-core/src/mount.rs`；`git hash-object` 核对为 `b228b7f4a5ebe159911fbf451eb4508b4962d60e`，与全轮统一使用的版本一致 |
| §1.5「9333 组扫描，0 反例」 | **部分核不动**：草稿目录只留了输出（`combos scanned: 9333 / counterexamples: 0`），**没有保留生成它的脚本**，按报告给的区间描述（`still∈[0,300,step15]×defer∈[0,300-still,step15]×isolated∈[0,50,step10]×lag∈[0,defer,step10]`）我用 8 种边界（含/不含端点）组合独立复算，得到 10225–14916 之间的多个候选值，**没有一个精确等于 9333**（脚本本身没留底，无法坐实这个具体数字）；但**恒等式本身**我做了解析证明——`allocatable_d16() − (gate1_value() − 7) = min(29, defer+isolated−lag) ≥ 0`（因 `lag ≤ defer` 且 `isolated ≥ 0`）——以及我自己写的多组扫描（10225~14916 组，覆盖 8 种边界组合）**全部 0 反例**，核心数学结论成立，只是「9333」这个具体计数没能重现 | 见草稿 `/tmp/claude-1000/alloc-basis-r3-verifier/sonnet-check/{rescan.py,rescan2.py,search.py}` |
| 问 1 的其余引用（`gate1_value()`、`allocatable_d16()`、`available_d28()`、`df_d16()` 行号，及常量） | ✓ 与 `research/prompts/alloc-basis-r2-opus-model/z3_gates.py` 逐行匹配 | `sed -n '1,170p' z3_gates.py` |

**Sonnet 计数**：核了 12 处，✓ 10，✗ 0，核不动 1（9333 具体计数），另有 1 处需主 agent 关注的实现层发现（不计入 ✓/✗，是补充观测）。

## 3. 本地攻方腿（`alloc-basis-r3-local-attack*`）

| 检查项 | 结果 | 命令 |
|---|---|---|
| 核对表所引 `.claude/kb/decisions/28-挂载期承诺量.md:64`（准入读数式子） | ✓ 逐字匹配 | `sed -n '64p'` |
| 「用户看到的 df = 第一道闸左边」翻译成英文时有没有加原文没有的限定词 | **✗（发现，非简单缺陷）**：提示文件第 51-54 行译为 "...equals the left-hand side of gate one at the moment of the query (gate1, as defined above, **using the currently published statistics**)"，**多加了一个括注限定词**，源句（r2-opus-output.md:334「用户看到的 `df` 就是第一道闸左边」）里没有这个限定；核对表第 15 行把这一条记成「无实质缺失，逐字照抄没有转述空间」，**这个自评不准确**——它确实加了字，只是没有像第 12 行（in-flight 那条）那样显式披露「这是解释性扩写」。两条添加处置不一致：第 12 行披露了，第 15 行没有。**这个添加本身看起来是合理的（用来区分 D28 已定项 2 的在飞层），不构成误导，但核对表的自我描述不准确** | `sed -n '45,54p' alloc-basis-r3-local-attack.md`；对照 `grep -n "用户看到的.*第一道闸左边" alloc-basis-r2-opus-output.md` |
| H1 事实表（txg=1..38）数字是否与 `z3-results.txt:255-323` 逐行一致 | ✓ 抽查 txg 1-6、13-14 等多行，字段（F/F_eff/still/defer/gate1/df/allocatable）与源文件逐字段一致 | `sed -n '255,284p' research/prompts/alloc-basis-r2-opus-model/outputs/z3-results.txt` 对照提示文件对应行 |
| 两份样本 SB 场景「gate1=5」是否能从事实表推出 | ✓ **可以**：SB 场景（一次推空落一块盘后崩溃，F_eff 回到 0）与 H1 表 txg=13 状态结构相同（F=9, F_eff=0, defer=52=48+4），H1 表 txg=13 行原样写着 `gate1=5`，两份样本（s1、s2）问题 6 都答 5，与 txg=13 完全对应 | 见「H1 事实表」同一处核对 |

**本地攻方计数**：核了 4 处，✓ 3，✗ 1（df 限定词添加未披露，性质是文档一致性问题不是事实错误），核不动 0。

## 4. 本地辩方腿（`alloc-basis-r3-local-defense*`）

| 检查项 | 结果 | 命令 |
|---|---|---|
| Fact A/B/C 引 `alloc-basis-r2-main-verification.md:31,36` | ✓ 逐字匹配 | `sed -n '31p;36p'` |
| Quote 1（`fs-design.md:34-36`）、Quote 2（`fs-design.md:23-24`） | ✓ 行号精确匹配（无 off-by-one），内容逐字匹配 | `grep -n` 定位 + `sed -n` |
| Q2 引文 A/B（`13-验证路线.md:369-372`、`:397-398`） | ✓ 逐字匹配 | `sed -n` |
| Q3 背景（`invariants.md:120`(I-3.1)、`:50`(I-7.4)、`:153`(I-4.8)） | ✓ 三处内容匹配 | `sed -n` |
| Q4 引文 A/B（`checks-owed.md:80`） | ✓ 逐字匹配（含「形状照 I-3.5 对 I-3.1 的补法」这个此前差点被摘掉的括注，定稿已补全） | `awk 'NR==80'` |
| 样本 s4「format parsing各写一份」是不是提示原文片段 | ✓ **部分是**：提示原文有纯中文「格式解析各写一份」（`13-验证路线.md:371`），但样本里的英中混合字符串「format parsing各写一份」**本身不是提示里的任何一个连续片段**，是模型把英译词「format parsing」与中文原句尾部「各写一份」直接拼接、外加引号伪装成一句引文；**该问题已被 runlog 自己发现并如实披露**（`alloc-basis-r3-local-defense-runlog.md:14`：「有一处需要如实报告的瑕疵」「未见其他粘连/复读/缺词迹象，人工通读判可用」），核查确认这条自我披露准确 | `grep -o "格式解析各写一份"` vs `grep -o "format parsing各写一份"` 分别对提示文件、样本文件 |
| `oov-check.py`/`corruption-check.py` 为什么对 `anewew`、`observationalomputational`、`root-reachableachable`（s1/s2）判绿 | ✓ 复跑并读规则逐条核实原因，三处各不同：<br>① **anewew**（s1，6 字符）：`oov-check.py` 的取词正则 `[A-Za-z][A-Za-z']{8,}` 要求词长 ≥9，"anewew" 只有 6 字符，**根本不进入扫描**；`corruption-check.py` 的 5 条规则全部要求空白分隔的重复/全大写自粘/markdown奇偶，"anewew" 不落在任何一种签名里<br>② **observationalomputational**（s2，25 字符）：被 `oov-check.py` 记成生词（在输出里），但 `splice_of()` 四条规则都失败——根因是**"observational" 本身不在项目词表 `en-words.txt`（源自 Linux 内核文档）里**（`grep -x observational` 无命中，即使它是真英语词），规则 A/C 都要求前缀是词表里的词，找不到任何有效切分点<br>③ **root-reachableachable**（s2）→ 去掉连字符后是 "reachableachable"（17 字符）：同样被记成生词，`splice_of()` 四条规则都失败——"reachable" 是词表词但唯一切分点的后半 "achable" 既不是词（规则 A 失败）也补不出真词（规则 C 逐字母试后 26 个候选无一命中）也太长不满足规则 D 的 `len(b)<=4`（"achable" 长 7）——**这是一种四条规则都没设计覆盖的模式：词 + 该词自己尾部片段的部分重复（无空格），不同于 LONG_DUP 要求的完整单词+空格+完整单词重复** | `python3 research/scripts/corruption-check.py <文件>`；`python3 research/scripts/oov-check.py <文件>`；逐条读 `splice_of()` 源码并用 Python 复算切分候选 |

**本地辩方计数**：核了 8 处，✓ 8（含 3 处工具规则分析，各自定位准确、有 Python 复算支撑），✗ 0，核不动 0。

## 总计

四条腿合计核了 41 处：✓ 38，✗ 1（本地攻方 df 限定词披露不一致），核不动 1（Sonnet 的 9333 具体计数，核心恒等式已用解析证明+独立扫描坐实）。另有 1 处 Sonnet §3.8 的补充观测（代码层 F 过滤与「有效根」候选集实际耦合，供主 agent 判断是否影响 D23 已定项 14 读法之争）。

## 没做什么

- 没判任何一条推论打中成不成立、该不该采纳（Z1′/Z2′/Z3′/Z4′/Z5′ 各格的结论一律不裁决）。
- 没核 Opus 报告全部 569 行里的每一条数字（正文表格逐格核对量太大），只核了主 agent 指定的重点段落（§1.2、§1.4①、§3.4、§4.1、§4.2）与产物哈希/复跑这条覆盖面最广的证据；§2（Z2′ 四组）、§二～三节里其余表格数字（如 3.1 两张逐次发布长表的其余行、2.1-2.3 崩溃态明细）没有逐格复核，只核了主 agent 点名的几行。
- 没有重新核 `m2-step45-code-r3-main-verification.md`、`m2-step6-checker-r1-main-verification.md` 这两份被引用判决本身的正确性，只核了 Opus/Sonnet 对它们的转述是否逐字。
- 没有编译 `crates/` 正式跑门禁，也没有跑 `check.sh`——按 `.claude/gate.d/stage-owners.tsv` 未登记给核查员，且主 agent 未要求。
- 没有核实本地攻方/本地辩方 12 道题里模型推理本身的正确性（比如 SA 场景 in-flight 减法算得对不对）——那是设计判断，不归核查员。
- Sonnet「9333」具体数字没能精确复现（脚本未留档），已如实标记为核不动而非 ✗（因为核心数学结论已用独立解析证明+多组扫描坐实为真，只是那一个具体计数值没法核）。
- 没有清理 `/tmp/claude-1000/alloc-basis-r3-verifier/opus-check/rerun-draft/`（约 113MB 编译产物），留作复核证据，未写回仓库任何路径。

草稿目录：`/tmp/claude-1000/alloc-basis-r3-verifier/`（含 `opus-check/`（完整副本+复跑产物+`outputs-rerun/`）、`sonnet-check/`（三份复算脚本）、`selftest/`（判别力自证用的偏移副本））。报告已按要求写入本次 SubagentHandback；未另写 `research/prompts/alloc-basis-r3-verifier-output.md` 文件（工具限制：本会话系统说明未提供文件写入工具，只能通过此次交回传递；若需要落盘存档，请主 agent 将本文本转存该路径）。
