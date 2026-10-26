# landing-audit-r5 辩方腿报告（K1：第一轮判决够不够得着）

日期：2026-10-23。腿：云端辩方（three-way-defense）。被复核：`research/prompts/landing-audit-r4-main-verification.md`。读了第一轮两份腿报告（`research/prompts/landing-audit-r4-sonnet-output.md`、`research/prompts/landing-audit-r4-opus-output.md`）与第十批的三份交回（`research/prompts/subagent-landing-audit-2026-10-22/` 下只有 `b10-hooks.md`、`b10-defs-sync.md`、`b10-replay-decode.md` 三份）。没读禁读清单里的攻方腿报告 `research/prompts/landing-audit-r5-opus-output.md`。

## 复跑

模型目录 `research/prompts/landing-audit-r5-sonnet-model/`。探针不编译、不起 cargo：读本机会话记录 `~/.claude/projects/-home-fy5090-code-singlefs`，把真派发提示、真交回会话逐份喂第九批（旧）与第十批（新）的钩子，原样打退出码。会话目录一直在长，所以用一个截止秒数（纪元秒 1792722600，在这一腿被派出去之前）把之后的记录丢掉。

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r5-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

`--frozen` 我给的是仓根本身（开工时 `sha256sum -c research/prompts/landing-audit-r5-snapshot/judged-sha256.txt` 27 份全 OK）。旧钩子是第十批开工前的四份原文（`inputs/pre-b10-hooks/`，sha256 与 `b10-hooks.md` 改过的文件表里「改前」一栏的开头相同）。末尾 9 行 `name=<探针> 与 out/ 逐行相同` 是比对。

复跑的实情：整条 `rerun.sh` 完整跑过 8 趟，中间几趟是改探针后重跑；`out/` 里 `guard_compare.out`、`handback_compare.out` 两份是最后一趟完整跑的结果（9 行都逐行相同）。那一趟之后我又改了四处，让模型里不出现钟点写法：`extract_prompts.py` 的截止改成纪元秒、同一次派发取最早的那条记录（续跑的会话文件会把老记录连老 id 重抄，原来取最后读到的那条，读文件的次序一变数就变），`scratch_line_stats.py` 只打日期，`shapes.py` 的假时间戳现算，`rerun.sh` 的默认值跟着改。改后没有再整条跑（整机负载 80 到 95，两个重探针一趟要一小时以上）：`extract`、`scratch_line_stats`、`naming_stats`、`misc_facts`、`shapes`、两个 `opus_naming_probe` 七个轻探针按 `rerun.sh` 里的同一条命令单跑：`naming_stats`、`shapes`、两个 `opus_naming_probe` 四份与 `out/` 逐字节相同，`extract`、`scratch_line_stats`（取法改了）与 `misc_facts`（别的会话又加了读门禁表的脚本，读表的脚本数 26 变 29）三份的 `out/` 是这次单跑的结果；`guard_compare.py`、`handback_compare.py` 两份源文件自最后一趟完整跑之后没动（sha256 同下），它们只读子 agent 首条消息，那部分改后仍是 3342 份。主会话派发数改了取法，所以 `scratch_line_stats.out` 里主会话那两行与我前面几趟报过的数不同，以 `out/` 为准。

```
0ca8de769e5c47c03a57a1d3bf7f0d42d2bfd196c19d4c1f9c40a5abd5be27ef  inputs/pre-b10-hooks/completion-cleanup-guard.sh
bab11748f599621eaf13ef5fd47364b537513a32de04fecae9ad6a169fa07548  inputs/pre-b10-hooks/handback-guard.sh
be47321288e64d3509f51312898febd05ca8a72a658d876bf45751ad9053a3bf  inputs/pre-b10-hooks/lib_three_way_legs.py
ce5cc2616c2667979836c6ddf2dfd460f358350cae533b17992af3a1e953c6a1  inputs/pre-b10-hooks/runner-dispatch-guard.sh
0efca8429dfd5baa9b54390d56775c999f87dc695d4e75d18040360344db8376  out/extract.out
fa4a56f875a6309e808e32d7c65ad2c9d37953bfb29b52c1afee373fa5cfa07b  out/guard_compare.out
784c72f99f327c48eccd7b5d6b50c5b79e51ff816a6f4c8a05f55059768132ef  out/handback_compare.out
07a3b168e6547acf16a641544e9e67f467b483f629221f5de03491998cc841ae  out/misc_facts.out
72e0f4d3fe1f0cafb205f4c55958f8d32f866489aa4566f4d519a907631b2a48  out/naming_stats.out
0ecf0ec1e913653b611a0b39d1871837fc211e36d073cd72b2f4ebd54851c911  out/opus_naming_probe_new.out
12829ccfa2e6ef94bf71576368a4246a17270b2988366931076b3a1d4de6b55d  out/opus_naming_probe_only0.out
4dcbeca27b2415ab54c2bcb4b4bf991180786b7349b2aa9dcea4421a37f79e52  out/scratch_line_stats.out
79232a9aaa439f369c624ace38ab9f66da15aa9870e1adc294c6315cdd87cd5b  out/shapes.out
83917505c736018387299b9d8597f30fbb77509208f48d94537f16b2bbb34b4d  probes/extract_prompts.py
16f7a07f2ddb69bdb34a96be2cc1776fcaab2a93a6e1d086e7298cb440ed700e  probes/guard_compare.py
442a6ff775681da74f15e8a8bc3a8a6088d730e29c6a0c4962303829a02375cc  probes/handback_compare.py
5030e0318ffaf5c6d4986787b8b3c5cf3d6cbd0f820ee56b72cd3b082838c0e9  probes/misc_facts.py
eabfcbad9b93f980be5d5698e861dfe4294cf23e9c2953edd6022b4c32720d13  probes/naming_stats.py
57ef111213fab3c8e8fbd6d208cddb7a1e7bdbd792fe40d056b50040b3e5a5c0  probes/scratch_line_stats.py
6969b88a8fccd701062d7cf17a42d3fd59d38edcf3bd674a8d97b4d2079d17a3  probes/shapes.py
413596c6433bd3dfefaf75516bbce971d2d1fa92e4676742efe120ab7b595595  tools/rerun.sh
```

## 语料与口径

- 子 agent 首条消息（= 派发提示）3342 份，来自 50 个会话；主会话里 Agent / Task 的派发 3951 次（按工具调用 id 去重）。会话记录里同一次工具调用会重复出现，数命令、数派发一律按 id 去重。
- 「新」是主工作区今天的钩子，「旧」是第十批开工前的钩子。派发闸用 `--check-prompt`（不建锁、不读主会话记录；标准输入要接 `/dev/null`，不接它会先读 hook JSON 挂住），交回闸喂 hook JSON，`transcript_path` 指主会话、子 agent 那份在它的 `subagents/` 下。
- 骨架（`python3 scripts/agents/dispatch-prompt.py`）打的提示 2026-10-09 起出现；派发闸 ⑯ 写者锁在 2026-10-14 的提交里进的（`git log -S WRITER_LOCK_DIRECTORY_LINE -- .claude/hooks/runner-dispatch-guard.sh`）。「骨架普及之后」我取 2026-10-15 起。
- 报告里没有会话号、没有钟点，日期都是首条消息的日期。

## 判定一览

| 判决条目 | 辩方判定 | 证据（细节在下面各节） |
|---|---|---|
| ⑯ 前一半：派得出就算坐实（`research/prompts/landing-audit-r4-main-verification.md:36`） | 一半：结论（不为它再开一项）站得住，写下的理由（派得出 = 观测）够不着 | 加键之后的 8 次三方腿派发都在两个更早开的会话里，分辨不了读的是新定义还是会话开头缓存的；拆分提案 534 行写了同一会话里改 frontmatter 不立刻生效。站得住靠先例：14 份定义带未知键 `required-inputs:` |
| ⑯ 后一半：线程上限与预计挂钟两张表不是三方腿类型表（同一行 36） | 站得住；「全部定义类型」四个字不对 | 两张表 9 种与 10 种，定义共 14 份；`clerk` 在表里没有 `leg`，`three-way-materials` 不在表里；两边的表靠自证互比 |
| 没打中：结局账 `.txt` 放 `records/`（:52） | 站得住 | 已经落了一份 63 行的真账，钟点与家目录两样 grep 0 命中；全仓文本格射程含 `records/` |
| 没打中：共用约束与交回闸文件头（:52） | 够不着 | 判的是第九批的文件头；第十批把交回闸 ⑤ 与文件头整段重写了，归攻方腿 K2 |
| 没打中：交回闸 ⑤ 与 rsync 一支（:52） | 站得住（对第九批）；量级补一个 | 1464 份实现员 / 工具员 / 执行员会话：125 份改了主工作区的 `.rs`、563 份只改别处的 `.rs`、776 份没改 `.rs` |
| 没打中：`experiment-number-directories.tsv` 的位置（:52） | 站得住，另有先例 | `scripts/` 下 29 份脚本读 22 张 `.claude/gate.d/*.tsv` |
| 没打中：⑬ 峰值为 0（:60） | 「没打中」站得住；记进 ㉙ 排的那一件活没有对象 | 本机峰值表最小 1 MiB（按 MiB 向上取整记），峰值为 0 的 0 条 |
| 没打中：⑭（:61） | 没复核 | 没起真进程；还是没量 |
| 采为观测：merge-and-verify 闭包（:62） | 站得住 | 工作区 8 个成员，闭包外 0 个 |
| K2-4 并 K3-2 收严，「收严不丢罩住的对象」（:49） | 一半：字面不成立，当前做法下代价是 0 | 旧钩子拒、新钩子放的 61 份（2026-10-09 到 10-14，占那几天 523 份的 11.7%），其中 `a9717cf07fb6e7c21` 是九十节 ⑳ 点名要拒的 4 份之一；2026-10-15 起 874 份首条消息 0 份受影响，⑯ 丢锁 0 份，行首带记号的 0 份 |
| 交回闸 ⑤ 也认退 1（`b10-hooks.md:14`） | 站得住；不会同样打中「只认退 0」 | 只认退 0 连第一轮攻方自己的对照（两条都真跑、lint 判红退 1）也拒；真会话 1464 份里新与只认退 0 判得不同的 0 份 |
| K2-6 只补互指，不改 C879 结论（:51） | 站得住 | 删那五行：比对对象表 #15 到 #19 立刻既没登记又没点名，格按 `doc-experiments.sh:2691` 判红；互指是文字，没有门禁绑它，会烂，但替代方案一样 |
| 额外（不在三问里，交主 agent） | 新的发现 | 第十批交回闸 ⑤ 在 1464 份真交回上改了 4 份的判决，4 份都真跑完了两条命名检查；真放过的 0 份；命名检查被 `timeout` 杀退 124 的 0 次（668 次调用） |

## 一、第一轮判决不采的两处，与记「没打中」的几处

### 1.1 ⑯ 前一半：派得出就算坐实

判决原文：「派得出就是这一条要的观测」（`research/prompts/landing-audit-r4-main-verification.md:36`），依据是「改了定义要新派才生效」（`./CLAUDE.md:16`）。

复核结论：**一半**。结论（不为 ⑯ 再开一项）站得住；写下的理由（这一轮派得出 = 观测）够不着。

依据：

- 这 8 次三方腿派发都在两个更早开的会话里：三份定义的修改日期是 2026-10-22（`stat -c %y .claude/agents/three-way-*.md`），其后派出的三方腿共 8 次，开工会话（2026-10-22 开）4 次、另一个会话（2026-10-18 开）4 次（草稿目录里的一次性脚本从 `out/extract.out` 同一份语料现算，没进模型）。没有一次是在定义改完之后新开的会话里派的。
- 第一轮正推腿当时就说了这一点：「我没有办法证明 Claude Code 读的是改后的定义还是会话开头缓存的」（`research/prompts/landing-audit-r4-sonnet-output.md:229`）。判决引的 `./CLAUDE.md:16` 是「新派沿用改后的定义」，而拆分提案 534 行有一条实测写了另一半：「已注册过的定义改 frontmatter，在同一会话里立刻派发不生效」（`records/2026-09-16-subagent拆分提案.md:534`）。判决只引了前者。
- 开工会话记录里有 4 条 `agent_listing_delta`（Claude Code 在会话里列出可派类型的附件），没有一条晚于三份定义的改动，所以没有「改动之后重载过」的通知可指（查法：`grep -c agent_listing_delta <开工会话记录>`，再逐条看时间戳，时间戳不抄进报告）。
- 站得住靠的是先例：`grep -c '^required-inputs:' .claude/agents/*.md` 14 份定义各 1 行，`git log -S'required-inputs:' -- .claude/agents/three-way-defense.md` 里最早一条是 2026-09-27（`260fa60a9`；三份三方腿定义同一次提交加的）。语料里 2026-09-27 起的 2708 份首条消息中有 585 份是三种三方腿，另一个会话 2026-10-18 才开，那时盘上已经是带这个未知键的定义，它派三方腿照常。「未知的 frontmatter 键不会让定义派不出」在新开的会话里被观测过；`leg:` 与它同一类。

推翻条件：定义改完之后新开的会话里派三方腿，因为 `leg:` 失败；或 Claude Code 升级后加载器对未知键变严。零成本的核法：下一个全新会话里派第一条三方腿时看它派不派得出，记一行，不必再开一项。

### 1.2 ⑯ 后一半：线程上限与预计挂钟两张表不算三方腿类型表

判决原文：「那两张是派发闸 ⑭ ⑮ 判的全部定义类型的名单，不是三方腿类型表」（`research/prompts/landing-audit-r4-main-verification.md:36`）。

复核结论：**站得住**；「全部定义类型」四个字不准。

依据：

- 线程上限那张 `THREAD_LIMIT_REQUIRED_AGENTS` 在 `.claude/hooks/runner-dispatch-guard.sh:1121`，9 种；预计挂钟那张 `LONG_RUN_AGENTS` 在 `.claude/hooks/runner-dispatch-guard.sh:1620`，是设计员加那 9 种共 10 种。定义共 14 份（`ls .claude/agents`），所以两张都不是「全部」。
- 三方腿在表里不是因为它们是腿：代码注释写的是「three-way-forward、three-way-defense 的定义只跑模型目录里的 tools/rerun.sh、不编译」（`.claude/hooks/runner-dispatch-guard.sh:1122`），在表里是因为偶尔带重活，不带的写「线程上限已判」。挂钟那张的范围是「用户 2026-10-12 定的判定射程」（`.claude/hooks/runner-dispatch-guard.sh:1620`）。
- 两个属性对不上：`clerk` 在两张表里，frontmatter 没有 `leg`；`three-way-materials` 有 `required-inputs`、没有 `leg`，不在两张表里（`.claude/agents/clerk.md`、`.claude/agents/three-way-materials.md` 的 frontmatter）。按 `leg` 派生这两张表只能搬走 9 种里的 3 个名字，另 6 个仍手写，变成两个来源并列。
- 骨架一侧有同样两张表，文件头写明靠自证互比：「逐个相同，自检解析派发闸那两行逐个比」（`scripts/agents/dispatch-prompt.py:32`；比较在 `scripts/agents/dispatch-prompt.py:465` 一带）。
- 判决没写的残留：以后有人给第四种定义加 `leg: three-way` 而忘了加进这两张表，没有任何一格会提醒；后果是那一种的派发可以不写线程上限一行（朝放行的方向错）。

推翻条件：给第四种定义加 `leg: three-way`，它的派发该带线程上限一行，而派发闸放行了没写这一行的提示。

### 1.3 记「没打中」的几处

判决原文：「记没打中；单条腿一次没打中不支撑「没有问题」」后接「这几处留给下一轮换角度攻」（`research/prompts/landing-audit-r4-main-verification.md:52`）。这一句的做法本身合规则（`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`）。下面逐处看「没打中」的事实还成不成立。

**结局账 `.txt` 放 `records/`：站得住。**

- 这份账已经落了：`records/landing-audit-b9-outcome-2026-10-23.txt`，63 行、未跟踪（`git status --short` 的 `??`）。`grep -nE '[0-9]{1,2}:[0-9]{2}'` 与 `grep -nE '/home/|/tmp/'`（家目录与临时目录路径）对它都是 0 命中（输出为空）。
- 扫 `records/` 的专用格只收 `.md`，但全仓文本格的射程含它：「射程是全仓，冻结证据（research/prompts/、records/、变更史、research/results/）也在内」（`.claude/gate.d/doc-text.sh:33`），所以钟点、角标、私有值三类照样会扫这份 `.txt`，上面两条 grep 就是那几类。
- 推翻条件：提交时门禁 doc-text 的任何一格因这份 `.txt` 红。我没起门禁阶段。

**共用约束与交回闸文件头：够不着。** 判的是第九批那一版；第十批把交回闸 ⑤ 与文件头整段重写了（`b10-hooks.md` 第二节），对得上不对得上归攻方腿 K2。我没复核。

**交回闸 ⑤ 与 rsync 一支：站得住（对第九批），补一个量级。**

- `out/handback_compare.out` 的 `edit_classes`：1464 份实现员 / 工具员 / 执行员会话里，125 份改了主工作区 `crates/` 或 `scripts/` 下的 `.rs`（交回闸判），563 份只改别处的 `.rs`（副本与草稿目录；没有 `.git` 的 rsync 副本交回闸不判），776 份没改 `.rs`。563 里有多少是 rsync 副本、多少是带 `.git` 的 worktree，我没拆。
- rsync 副本不判是定下来的写法：「只修命中在自己改动的文件里的」（`.claude/agent-common.md:99`）同一行前面写明在 rsync 拷的副本里改的不在副本上跑、两条命令交主 agent。
- 推翻条件：一份只改 rsync 副本的会话，交回里既没有命名检查的两条命令、也没交给主 agent，而交回闸放行。

**`experiment-number-directories.tsv` 的位置：站得住，另有先例。**

- 攻方引的「只被门禁自己读的配置（格名、阶段归属、排除表）留在门禁旁边」（`.claude/rules/directory-layout.md:22`）管的是「只被门禁读」的；这张表还被占号脚本读，字面不矛盾。
- 先例：`out/misc_facts.out` 的 `gate_tsv_readers`，`scripts/` 下 29 份脚本读 22 张 `.claude/gate.d/*.tsv`（变异、复跑、归档、占号、门禁辅助都有）。非门禁的脚本读门禁目录里的表是常态，不是这一张的新做法。
- 推翻条件：`directory-layout.md` 另有一句专管「被脚本合读的配置」放哪。

**⑬ 峰值为 0：「没打中」站得住；记进 ㉙ 排的那件活没有对象。**

- 判决原文：「记没打中，阈值不单调记进九十节 ㉙ 一并给下一轮」（`research/prompts/landing-audit-r4-main-verification.md:60`），㉙ 的去向列排了「⑬ 峰值为 0 的条目也走下限」（`records/2026-09-16-subagent拆分提案.md:1500`）。
- 事实：峰值表（仓外本机数据 `~/.local/share/singlefs/memory-peaks.tsv`）文件头的口径写按 MiB 向上取整记，`out/misc_facts.out` 的 `memory_peaks`：最小 1048576 字节，峰值为 0 的条数 0。表在长，每次读都是现读；至少今天，「峰值正好为 0」这个状态在表里不会出现，阈值不单调是一个够不着的角落。
- 不是说不能改，是说 ㉙ 排这件活得到的收益是 0；要不要留，主 agent 定。推翻条件：表里出现峰值为 0 的条目（口径改了，或某条命令的外壳 bash 没读到 cgroup 峰值而记 0）。

**⑭：没复核。** 判决写「只读了代码，推的」（`research/prompts/landing-audit-r4-main-verification.md:61`），下一轮换腿量。第二轮正文的格里没有它（`research/prompts/_landing-audit-r5-body.md` 的 K2、K3 都没点 ⑭ 或看门狗自证），这一轮也没人量；第三轮只攻前两轮站住的形态。它会带着「只读过代码」进第三轮之后，没有一次真起进程的观测。

**采为观测：merge-and-verify 闭包。站得住。** `out/misc_facts.out` 的 `workspace_closure`：工作区成员 8 个，依赖闭包外 0 个；成员表在 `./Cargo.toml:5-14`，`crates/singlefs-checker-tier/Cargo.toml:9-19` 里 7 个路径依赖加它自己正好是那 8 个。判决的「`scripts/experiment-apparatus` 与空补丁两种」也对：`./Cargo.toml:15` 写 `exclude = ["scripts/experiment-apparatus"]`。推翻条件：成员表加了一个不被 checker 档包依赖的包。

## 二、K2-4 并 K3-2 收严成只认行首「草稿目录：」

判决原文：「照骨架写的提示都有这一行，没有这一行的提示照旧只判裸文件名」「所以收严不丢罩住的对象」（`research/prompts/landing-audit-r4-main-verification.md:49`）。第十批交回写了两样代价。⑯ 一面：行首带列表记号、引用记号、粗体的不再建写者锁，「这是推出来的，没量过真提示里有几份这样写」（`research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md:354`），推翻条件写的是「真会话里照骨架写的提示被 ⑯ 漏建写者锁」（`research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md:18`）。⑰ 一面：`a9717cf07fb6e7c21` 一类不再拦，「这是判决 K2-4 定的收严代价」（`research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md:355`）。判决当时没量（判决自己写了「攻方没扫真会话记录与真提示语料，K3-1、K3-2 那几种形状实际出现过几次不知道。」，`research/prompts/landing-audit-r4-main-verification.md:78`）。下面是量出来的。

复核结论：**一半**。「不丢罩住的对象」字面不成立，当时就该算进代价；但按今天的派发写法，代价是 0。

### 2.1 量法与读数

- 语料：子 agent 首条消息（派发提示）3342 份、主会话 Agent / Task 派发 3951 次（`out/extract.out`）。
- 旧认法：第九批两处正则逐字抄自 `inputs/pre-b10-hooks/runner-dispatch-guard.sh` 的 ⑯（`WRITER_LOCK_DIRECTORY_LINE`：行首可带 `> * _ -`，冒号必须有）与 ⑰（`REPORT_SCRATCH_MENTION`：不限行首，冒号可无）。新认法直接调主工作区的 `lib_prompt_scratch_line.py`。读数是 `out/scratch_line_stats.out`：

```
name=subagent_first_messages ⑯ 进派发闸之后（2026-10-14 起） 共=944 新认法认不出草稿目录=17 ⑯丢锁=0 ⑰旧认法有而新认法没有=17 行首带记号的草稿目录行=0
name=subagent_first_messages 骨架普及之后（2026-10-15 起） 共=874 新认法认不出草稿目录=0 ⑯丢锁=0 ⑰旧认法有而新认法没有=3 行首带记号的草稿目录行=0
name=marked_label_lines 全部=3342 行首带列表/引用/粗体记号的「草稿目录：」行=76 最晚一份的日期=2026-10-12
name=skeleton_prompts 骨架头「# 派发：」起头的=844 其中新认法认不出=0
```

主会话那边（`out/scratch_line_stats.out` 的 `main_session_dispatches`、`main_session_marked` 两行）：2026-10-15 起 967 次派发（去掉 general-purpose / Explore 这类内置类型），962 次认出规范行，认不出的 5 次提示里连「草稿目录」四个字都没有；行首带记号的 0 次。

### 2.2 ⑯ 丢锁：0

- 行首带列表 / 引用 / 粗体记号的「草稿目录：」行，全部 3342 份里有 76 份，最晚一份是 2026-10-12 的；⑯ 在 2026-10-14 进派发闸之后的 944 份里 0 份（`marked_label_lines`、第一行 `subagent_first_messages`）。骨架头起头的 844 份新认法全认得出。
- 所以 `b10-hooks.md:18` 的推翻条件（照骨架写的提示被漏建写者锁）没有被满足，`b10-hooks.md:354` 的担心在真提示里 0 份。这 76 份都发生在骨架普及之前（骨架是 `# 派发：` 头，2026-10-09 起出现，2026-10-15 起 874 份全带行首的规范行）。
- 判决那句话对 ⑯ 站得住。推翻条件：以后有真派发提示行首带记号写「草稿目录：」，且派发闸因此没建写者锁。

### 2.3 ⑰ 不再拦：61 份（全在 2026-10-14 之前），现在 0

`out/guard_compare.out`：1397 份首条消息（2026-10-09 起）逐份喂旧、新两棵派发闸的 `--check-prompt`：

```
name=guard_compare 提示=1397（2026-10-09 起） 退出码（旧,新）分布={(0, 0): 368, (2, 0): 10, (2, 2): 1019}
name=report_name_reason 旧拒新放=61 旧放新拒=0 按日={'2026-10-09': 12, '2026-10-11': 18, '2026-10-12': 1, '2026-10-13': 16, '2026-10-14': 14}
name=report_name_reason_since_2026-10-15 旧拒新放=0
name=only_reason 旧只因这一条拒、新放行=10
```

- 61 份在「报告文件名带 report」这一条上旧拒新放；旧放新拒 0 份。61 份都在 2026-10-09 到 10-14 那 523 份里（11.7%），2026-10-15 起 874 份 0 份。其中 10 份旧钩子只因这一条拒，新钩子整条放行。
- 这 61 份的草稿目录都不写在行首那一行：句中带冒号的 38 份、句中不带冒号的 23 份。把旧钩子拒的路径与该提示第一处「草稿目录」点名的目录比，路径都落在它自己的草稿目录（含 `patch/` 子目录）下，就是它自己的 `report.md`，不是转述别人的——所以旧钩子的这 61 次拒是真阳性（这两个数与比法是草稿目录里的一次性脚本现算的，没进模型）。
- 九十节 ⑳ 写的是「逗号之后不认又放过 4 份写在逗号后的真报告行」（`records/2026-09-16-subagent拆分提案.md:1491`），这 4 份是 `a0a2dfee322e26786`、`a789e7091d91430aa`、`a8afb2185cfa840cc`、`a9717cf07fb6e7c21`（`research/prompts/landing-audit-r4-sonnet-output.md:299`）。新钩子还拒前三份，放了第四份（`out/guard_compare.out` 的 `named_in_90_section_20` 四行）。判决那一行自己也写了「⑰ 放宽是为第三轮攻方那份不照骨架写的工具员提示」（`research/prompts/landing-audit-r4-main-verification.md:49`）——就是 `a9717cf07fb6e7c21`。也就是说收严丢的对象在判决当时就看得见，且正是 ⑳ 点名要拦的那一份；判决把它归进「不照骨架写的」所以不算「罩住的对象」，但 ⑳ 的自证期望因此从拒改成放行（`research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md:277`），这一笔该写进判决。
- 收严换来的：K3-2 的两种误拦（转述里的「，草稿目录 <别处>」、列表项里的「- 草稿目录 <别处>」）在真提示里没有造成过一次判决不同：2026-10-15 起 874 份里旧 ⑰ 多认出别处目录的有 3 份（`⑰旧认法有而新认法没有=3`），这 3 份在「报告文件名」那一条上旧、新钩子判得一样（`report_name_reason_since_2026-10-15 旧拒新放=0`）。所以收严在真提示上既没丢也没得，剩下的是四处共用一份认法的整洁。

### 2.4 判决那句话还成立吗

字面不成立（61 份、其中 1 份是 ⑳ 点名的），应当改写成「不丢照骨架写的提示罩住的对象；丢 2026-10-14 之前 61 份不写行首规范行的提示，其中 `a9717cf07fb6e7c21` 是 ⑳ 点名的一份，⑳ 的期望随之改了」。按今天的派发写法（2026-10-15 起 874 份首条消息全带行首规范行）代价为 0。

推翻条件：2026-10-15 之后出现一份没有行首「草稿目录：」的真派发提示，且报告行点名它自己草稿目录下的 `report.md`；或出现一份行首带记号写草稿目录的提示被 ⑯ 漏建锁。主会话里「派发提示在 …/prompt.md」这一类（5 次里的 4 次，把真提示写进文件、只给指针）⑯ 本来就看不到，新旧一样，不算收严的代价。

## 三、主 agent 判下的两处，是不是同样打中替代方案

### 3.1 交回闸 ⑤ 也认退 1 的命名检查

判决原文是「只认工具结果退出码 0」（`research/prompts/landing-audit-r4-main-verification.md:58`）。第十批的工具员没照字面做，写了「所以我按两种都认来做」（`research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md:14`），常量在 `.claude/hooks/handback-guard.sh:186`（`NAMING_COMPLETED_EXIT_CODES = (0, 1)`）；退 1 只在整条里只有赋值、env、cd、export 和命名检查、最后一条是命名检查时收（同一份交回 `research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md:136-137`）。主 agent 判下了。

复核结论：**站得住；不会同样打中「只认退 0」**（反过来，只认退 0 才被攻方自己的对照打中）。

依据：

- 规则文字本来就许这种交回：「只修命中在自己改动的文件里的」（`.claude/agent-common.md:99`），「不在的不修，照写」（`.claude/agent-common.md:100`）；命名检查判红退 1：「1 判红（配置不合规或名字违规）」（`.claude/singlefs-ai-sop/scripts/naming-lint.sh:8`）。共用树里别的会话的命中修不了，交回时检查就是退 1。只认退 0 会把这种照规则办的交回拒掉。
- 复现。`out/shapes.out`（三棵交回闸：旧 / 新 / 只认退 0）：

```
两条各一次，lint 判红退 1：old=退0  new=退0  only0=退2
两条用 ; 连在一条里，后一条判红退 1：old=退0  new=退0  only0=退2
```

  第一轮攻方模型的十种写法（`research/prompts/landing-audit-r4-opus-model/probes/naming_probe.py`）喂新与只认退 0 两棵，`out/opus_naming_probe_new.out` 与 `out/opus_naming_probe_only0.out` 逐行比：五种没跑完的（`false &&`、`true ||`、`if false`、只定义不调用的函数、`timeout 0.1` 被杀退 124）两棵都退 2，也就是它们是被位置规则与退 124 挡住的，跟认不认退 1 无关；唯一不同的一行是攻方自己的对照「两条都真跑」（lint 判红退 1）：新退 0，只认退 0 退 2。派发提示要求这一行「照旧」，只认退 0 做不到。
- 真会话上没有差别。`out/handback_compare.out`：1464 份实现员 / 工具员 / 执行员会话，命名检查那一条拒的：旧 148、新 152、只认退 0 152，新与只认退 0 判得不同的 0 份。原因在 `out/naming_stats.out`：两条命名检查命令的 668 次 Bash 调用，整条退出码 0 的 660 次、1 的 4 次、2 的 4 次；调用过 `naming-lint.sh` 的 148 份会话，最后一次整条退出码全是 0。那 4 次退 1 是多命令组合里别的命令的退出码（`tail` 参数错 2 次、rustc 报错 1 次），和 1 次拿副本做探针时副本里的检查自己判红（配置不合规），没有一次是交回前在主工作区跑的那一条判红。换句话说，「命中在别人文件里照写交回」这种情形，规则允许，但真会话里还一次没发生过。
- 残留的一条缝（判决与交回都没写）：退 1 也包括「配置不合规、名字这一轮没有查」，认退 1 就把这种没查名字的也认成跑过；只认退 0 没有这条缝，但要拒规则许可的交回。两头各有代价，谁也不压倒谁；这一条取舍由规则文字（`.claude/agent-common.md:99`）定，不是数据定的。

推翻条件：真会话里出现一份交回，命名检查整条退 1 的原因不是检查自己判红（例如 `cd` 失败后检查没跑），而新钩子放行了；或出现一份只认退 0 会误拒的合法交回（真会话里目前 0 份）。

### 3.2 K2-6 只补互指，不改 C879 结论

判决原文：「不是判法错（五行确实还没单元，不登记是如实），是两处说法没连上」（`research/prompts/landing-audit-r4-main-verification.md:51`）。

复核结论：**站得住**；替代方案「删那五行」被同一条事实打中。

依据：

- `out/misc_facts.out` 的 `comparison_table`：比对对象表 19 行，登记表 14 行，表里对不上登记的行号是 15 到 19，文件头点名不登记的行号也是 15 到 19，正好一一对上。删掉文件头那五行（`.claude/gate.d/replay-comparison-units.tsv:17` 起），格就按「里没有一行对得上，文件头也没点名不登记」（`.claude/gate.d/doc-experiments.sh:2691`）对这五行判红，样本 `replay-comparison-units-table-red` 的期望里是同一句（`.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-table-red/expect:2`）。红的原因就是 C879 记的那件事：单元表里没有这五条正式跑的单元。所以删五行 = 让门禁红在一个已记欠账、装置侧还没解的状态上；要不红就得登记不存在的单元名，格也会红（单元名在单元表出处里找不到）。这正是「不登记是如实」的含义。
- 现在的互指两头都在：登记表写「就删掉这五行、改登记成正式行」（`.claude/gate.d/replay-comparison-units.tsv:17`），C879 写「那五行指回 C879」（`.claude/kb/checks-owed.md:560`）；英文、日文两份的 C879 同一行也有对应句（`grep` 看 `.claude/kb/checks-owed.en.md`、`.claude/kb/checks-owed.ja.md` 的 `| C879 |` 行）。
- 残留：互指只是文字。`grep -rl C879 .claude/gate.d scripts` 只命中登记表这一处，没有门禁核「文件头点名不登记的五行指的欠账还开着」。以后 C879 还清、挪进已还清而那五行没删，什么也不会红。攻方提的绑定做法（不登记行必须带一个开着的 C 号，C 号关掉就红）是零轮的推论，没被采；用任何文字互指的替代方案都有同一条缝，所以这条缝不能当「删五行」更好的理由。

推翻条件：E165 那一侧判定这五条正式跑本来就不该有单元（C879 的前提一列），那时 C879 挪进已还清、五行该删；或补了单元而那五行还在、门禁仍绿（缝变成事实）。

## 四、额外（不在 K1 三问里）：第十批交回闸 ⑤ 在真交回上的读数

这是量 3.1 时顺带量出来的，归攻方腿 K2 的「误拒真跑完的」，写在这里交主 agent 一并看；我没替它采纳什么。

- `out/handback_compare.out`：1464 份会话里，第十批新钩子比第九批旧钩子多拒的恰好 4 份：`a7aa1608c74c3e21e`、`aa68d151c2fa72bb7`、`aa87dd9fe761d727f`、`ab6dd5f35228122b7`（`newly_refused` 一行），旧拒新放 0 份。这 4 份都调用过 `naming-lint.sh`（`其中调用过 naming-lint.sh 的=4`），并且都在主工作区、改完 `.rs` 之后真跑了两条命名检查：命令是 `timeout 230 nice -n 19 bash …命名检查… > 文件 2>&1; echo "rc=$?"` 这一类，输出落文件、`echo` 把检查自己的退出码（红，命中在别的会话改的文件里）打出来，整条 Bash 退 0。
- 新钩子拒它们的原因是交回里写的「包着 timeout 的，要是最后一条」（`research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md:136`）。复现：`out/shapes.out` 第二行（timeout 包着、重定向到文件、`echo rc` 收尾、整条退 0）：旧退 0、新退 2、只认退 0 退 2；第三行同一条去掉 `timeout`：三棵都退 0。所以拒它们的是「`timeout` 不是最后一条」这条规则，不是认不认退 1。
- 这条规则要挡的是 K3-1 的第五种（`timeout` 把检查杀了，整条退 124）：`out/naming_stats.out` 里两条命名检查命令的 668 次调用，整条退出码只有 0、1、2 三种，退 124、137、143 的 0 次（其中命令里带 `timeout` 的 51 次）。另外四种（`false &&`、`true ||`、`if false`、只定义不调用的函数）：新旧两棵判得不同的只有上面那 4 份，且都是真跑完的；没有哪一份是旧放、新拒、其实没跑的。也就是说在 1464 份真交回上，第十批这一收严改了 4 份的判决，4 份都是误拒，真拦住的 0 份。
- 代价是被拒的子 agent 要在交回前重跑一遍（检查几秒到几十秒）；收益在真会话里目前是 0，在攻方造的输入上是 5 种。该不该为 `echo rc` 收尾的写法放一道，是主 agent 的事。

推翻条件：以后真交回里出现一份命名检查被 `timeout` 杀掉（整条退 124）或没执行（`true ||` 一类）而旧钩子放过、新钩子拦下。

## 没做什么

- 没替主 agent 撤销或维持任何判决，只交了复核结果；「该改写判决里哪一句」的几处是提示，不是改。
- 没起门禁阶段（扫仓的阶段只许提交时跑），没起任何 checker 档、没编译、没跑 `cargo`；只跑了样本之外的轻探针，全部经 `scripts/hosts/run-with-memory-cap.sh 4G` 加 `nice -n 19`。
- 没复核 ⑭（没起真进程）、没复核「共用约束与交回闸文件头」那一条（第十批已重写）、没核 `a9717cf07fb6e7c21` 以外那 60 份旧拒新放的提示在当时真实的后果（没有当时的派发闸，没法知道它们被拦下会怎样）。
- 报告里有三处数字是草稿目录里的一次性脚本现算的，没进模型：1.1 的「8 次三方腿派发在两个更早开的会话里」，2.3 的「61 份里 38 份句中带冒号、23 份不带」与「拒的路径落在它自己的草稿目录下」，3.1 里「那 4 次退 1 各是什么」（我逐份看了会话记录）。其余数字都在 `out/` 里。
- 没验：Claude Code 是否会在同一会话里热加载 `.claude/agents/` 里改过的 frontmatter（1.1），没法在子 agent 里造全新会话；没验 `rerun.sh` 在改完最后四处之后整条再跑一趟（见「复跑」一节写明的范围）。
- 回看决策：不涉及。
- 清理：草稿目录 `~/.local/share/singlefs/work/landing-audit-r5-sonnet/` 里自己建的钩子树（`tree_old`、`tree_new`、`tree_only0`，各约 3.2M，每棵拷了 `.claude/hooks`、其余用符号链接指回仓根）、各趟复跑的工作目录与语料抽取、探针假仓，交回前已删，删之前 `du -sh` 的大小写在交回正文；模型目录里没有仓副本、编译目录与库拷贝（`du -sh` 约 650K，是探针、四份第九批钩子原文和输出）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/landing-audit-r5-sonnet-output.md --background research/prompts/_landing-audit-r5-background.md` 末行原样：

```
  ✓ 核了 31 处引文、19 处裸行号，对上 31 处，没判 19 处（其中裸行号没有引文可比 19 处；逐处列在上面）
```
