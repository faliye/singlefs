核对表里的 ✗ 不免除主 agent 对推论的逐条现查，这一句照抄进报告开头。

# 判别力自证

从待核引用里挑了 Opus 报告「b」格引用的 `.claude/agent-common.md` 第 44 行（引文「定义点名的产出文件……报告可以不写文件」），在草稿目录副本（`/tmp/claude-1000/agent-defs-r2-verifier/selftest/agent-common.md`）里把行号从 44 改成 45 再核：副本第 45 行原样是「干到一半收到主 agent 的消息：当成追加的输入并进这一轮做……」，与被核引文完全不同 ⇒ **判 ✗**。方法分辨，继续往下核。

# 一、云端辩方（Sonnet，`agent-defs-r2-sonnet-output.md`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| A1：R1 时代 `agent-common.md:15,17`＝附录 1125、1127 行「新建文件一律排他…`>>` 追加」「改已有文件只用定点替换…」 | ✓ 逐字匹配；附录「写」节明确标注「出处 `.claude/agent-common.md:12-19`」，按此偏移算出的 1125/1127 行原文与所引一致 | `sed -n '1119,1130p' research/prompts/_agent-defs-r1-appendix.md` |
| A1：今天 `.claude/agent-common.md:15` | ✓ 逐字匹配「tools 里有 Write / Edit 的……让闸看得见……」 | `sed -n '15p' .claude/agent-common.md` |
| A2：R1 `kb-scribe.md:12`＝附录 367 行；今天 `kb-scribe.md:12` | ✓ 今天行原文含「规格点名的文件不在你的写范围里（例如 `CLAUDE.md`），也停下报告，不按规格硬写」，逐字匹配 | `sed -n '12p' .claude/agents/kb-scribe.md` |
| B1：`crash-verifier.md:27` | ✓ 逐字匹配「`git diff HEAD -- crates litmus \| sha256sum`（暂存与没暂存的都算）」「未跟踪文件按内容，不只按名单」 | `sed -n '27p' .claude/agents/crash-verifier.md` |
| C1：`kb-scribe.md` 第 3 步（33 号）；`stage-owners.tsv` 第 8 行 | ✓ 第 3 步含「它改写了 `research/**/*.rs` 的，加跑 33 号……并把改到的实验源码逐个列给主 agent」；tsv 第 8 行 `22-item-ref-status.sh kb-scribe` | `sed -n '27p' .claude/agents/kb-scribe.md`；`sed -n '8p' .claude/gate.d/stage-owners.tsv` |
| C2：`kb-scribe.md:18` | ✓ 逐字匹配「标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定……标题……也由主 agent 给）」 | `sed -n '18p' .claude/agents/kb-scribe.md` |
| C3：`kb-scribe.md` 第 2 步 | ✓ 含「跑前跑后各 `git diff -- .claude/kb/decisions-history.md .claude/kb/decisions-history/` 一次，它改到了这一轮之外的条目……就停下交回，不自己收拾」 | `sed -n '26p' .claude/agents/kb-scribe.md` |
| C4：`kb-scribe.md` 第 1、5 步 | ✓ 第 1 步「开工时先记下……`sha256sum`」，第 5 步「收尾再记一次……两次哈希」 | `sed -n '25p;29p' .claude/agents/kb-scribe.md` |
| D2：`agent-write-scope.tsv` 新增两行；`experiment-runner.md` 写范围一节 | ✓ tsv 里 `experiment-runner research/prompts/e*-r*-prereg.md`、`experiment-runner .claude/kb/experiments-history.md` 两行均标注「agent-defs-r1 判决 D2」；写范围一节同步含这两样 | `cat .claude/hooks/agent-write-scope.tsv`；`sed -n '31,33p' .claude/agents/experiment-runner.md` |
| D3：`experiment-runner.md:12` | ✓ 逐字匹配「修订只许收严或补一条臂与判据，不许少报登记里的任何一个量，要少报交主 agent 定」 | `sed -n '12p' .claude/agents/experiment-runner.md` |
| E1：`implementation-writer.md` 第 5 步 | ✓ 含「`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行」 | `sed -n '28p' .claude/agents/implementation-writer.md` |
| 本地攻方第 3 问：`three-way-verifier.md` 第 4 步 | ✓ 逐字匹配「复跑命令带着指向路径的环境变量的，路径一并换到你的草稿目录；输出里嵌着临时路径的，按字段比，不按整份哈希判 ✗」（这正是我这一轮在核 Opus 模型脚本时实际执行的方法） | `sed -n '26p' .claude/agents/three-way-verifier.md` |
| 本地攻方第 6 问：`three-way-verifier.md` 第 5 步 | ✓ 逐字匹配「本地腿的转述核对表里每一处『原文文件:行』也逐条核：行号在不在、抄的是不是原文、英文有没有丢限定词」 | `sed -n '25p' .claude/agents/three-way-verifier.md` |
| 报告不写文件：R1 时代附录 1168 行 | ✓ 逐字匹配「贴不贴报告全文随你，但没写文件就交回不算交了」 | `sed -n '1168p' research/prompts/_agent-defs-r1-appendix.md` |
| 本地辩方第 2 条：`mutate.sh:19-37`；`mutation-triage.md` 输入一节 | ✓ 19-37 行是按 `$SRC` 反查 `[[bin]] name` 的确定性查表逻辑；`mutation-triage.md` 第 16 行原文「主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明」 | `sed -n '19,37p' research/scripts/mutate.sh`；`grep -n "主 agent 给的对不上" .claude/agents/mutation-triage.md` |
| **本地攻方第 2 问**：R1 判决引「计划第十二节…SendMessage 能续做已交回的 agent」是否答非所问 | ✓ **Sonnet 的质疑站得住，且是这一轮里唯一一条我逐条核实过程与结论的「推论性」发现**：R1 判决原文在 `agent-defs-r1-main-verification.md:45`「第 2 问说交回之后消息被忽略，与实测不符（SendMessage 能续做已交回的 agent，计划第十二节）」；计划第十二节那一行（`records/…md:259`）原文是「改了已有定义之后，新派与续做各用哪一版……**新派用改后的版本，续做沿用第一次派发时的系统提示**。A 按 v1 派出（**只回 OK，没问过版本**）」——这里的 A 只应答了一句 OK，还没调用过交回工具，不是「已经成功调用过一次 SubagentHandback」的 agent。Sonnet 另引的「records 第 246 行」核对也准确（原文「转录里只有两次交回调用（第二次被拒：已交过一次）」）。**这条不是引用错误，是判断题**，按主 agent 交办第 6 条我只核到「引文与原文对得上、逻辑链能立住」，不代表这条判断本身被主 agent 采纳 | `sed -n '45p' research/prompts/agent-defs-r1-main-verification.md`；`sed -n '259p' "records/2026-09-16-subagent拆分提案.md"`；`sed -n '246p'` 同文件 |
| **G1：「records 第 329 行一带，74 条实测记录」** | **✗ 行号错**。含「共 74 条记录」与「子 agent……带 `agent_id`、`agent_type` 两个额外键」这段内容的原文实际在 `records/2026-09-16-subagent拆分提案.md` **第 255 行**，不在第 329 行；第 329 行是另一段无关内容（PreToolUse hook 放行规则）。也不是「误写成背景材料行号」——背景材料 `_agent-defs-r2-background.md` 里同一段内容在**第 2717 行**，同样不是 329。这是一处独立的行号错误，与内容本身（74 条记录、agent_id/agent_type 两个键）的真实性无关——内容本身经核实是真的 | `grep -n "共 74 条记录" "records/2026-09-16-subagent拆分提案.md" research/prompts/_agent-defs-r2-background.md`；`sed -n '329p' "records/…md"` |

**Sonnet 报告计数**：核了 17 处引用（含 1 处判断题的引文链），✓ 16，✗ 1（G1 行号）。核不动：G1 里「插件 agent 命名规则细节」「内置 agent 名单」等未深挖（Sonnet 自己也在「没做什么」里说明了）；J6 experiment-designer 的 8 点逐字核对没有逐点重做（抽查了「登记的固定节名」一节确实存在）。

# 二、云端攻方（Opus，`agent-defs-r2-opus-output.md`）

**复跑三份模型脚本**（拷贝到 `/tmp/claude-1000/agent-defs-r2-verifier/opus-model-rerun/`，`TMPDIR` 指向草稿目录子目录，加 `nice -n 19`，不在原目录跑）：

| 脚本 | 复跑结果 | sha256（复跑 vs 报告所抄） |
|---|---|---|
| `edit-half-apply.sh` | 输出与 `.out` 逐字节相同（`diff` 无输出） | `5089348e...` = 报告所抄 ✓ |
| `designer-quote-kb.sh` | 输出与 `.out` 逐字节相同 | `a592df63...` = 报告所抄 ✓ |
| `history-write-diff.sh` | 输出与 `.out` 逐字节相同（此刻 `.claude/kb/decisions-history.md` 仍是 425 条，未被别的会话改动，故条目数也未变） | `71eb03cc...` = 报告所抄 ✓ |
| 三份 `.sh` 本身 | 拷贝后 sha256 与报告表格逐一相同 | ✓ |

**引用核对**（file:line + 抄文）：

| 引用 | 核的结果 |
|---|---|
| item a：`kb-scribe.md:25` | ✓ 逐字匹配 |
| item c：`kb-scribe.md:26`；`lib-history-brief.py:106,111,169,176` | ✓ 全部逐字匹配（`RECENT_PER_DECISION = 3`、`entries[:RECENT_PER_DECISION]`、两处 `open(...).write(...)`） |
| item c：`grep -c '^## D[0-9]' decisions-history.md` → 27；`下面是最近 3 次` → 26 | ✓ 我独立复跑得到同样的 27、26（另 1 条是「共改过 3 次：」，27=26+1 对得上） |
| item d：`experiment-runner.md:17,33`；`agent-write-scope.tsv:17`；`experiment-designer.md:49`；`claim-experiment.sh:51`；`CLAUDE.md:21`；`test-discipline.md:37` | ✓ 六处全部逐字匹配 |
| item e：`experiment-runner.md:12`；`experiment-designer.md:38-39`；`trial-experiment-designer.md:36`；`trial-experiment-runner.md:36`（节选，标了「节选」） | ✓ 全部匹配，节选处确认省略号之外的部分是真实存在的原文延续 |
| item f：`implementation-writer.md:26,28`；`git diff -- crates/singlefs-core/src/lib.rs` 只两行 `pub mod`；17 个文件 15 个带单测 | ✓ 我独立跑 `git diff`、`git status`、`grep -c` 得到完全相同的数字（18 个 .rs 文件，lib.rs 之外 17 个，15 个含 `#[cfg(test)]`），这是此刻真实存在的另一个会话在制改动 |
| item g：`kb-scribe.md:18,26`；`32-history-ordinal.sh:5`；`decisions-history/2026-09.md:8` | ✓ 全部逐字匹配，2026-09.md 第 8 行确实是「### 2026-09-17：…」裸标题 |
| item i：`experiment-designer.md:34,49`；`quote-kb.py:396`；`claim-experiment.sh:57` | ✓ 全部逐字匹配 |
| item k：`ls research/mutations \| wc -l` → 143；4 张 shell 探针表名字与表头 | ✓ 我独立复核 143（142 个 `.tsv` + 1 个 README.md），四个文件均存在，表头均含「被测装置是 shell 探针……（不是 Rust 二进制）」或省略括注的同义句，`e7-index-bench/src/bin/` 下确认四个同名 `.rs` 均不存在 |
| item l：`stage-owners.tsv:20`；`agent-defs-r2-trial-kb-scribe.md:130` | ✓ 全部逐字匹配 |
| item b：`total=304 bytes>=92740:8`、`total=304 lines>=500:70 lines>=912:3` | ✓ **可复现，但要卡时间点**：此刻重新统计（307 个文件）比 Opus 报告的 304 多 3 个，是这一轮之后又有 3 个新文件被（本轮及别的并发会话）创建；把统计对象限制在 mtime 早于 `07:18:05 UTC`（Opus 报告自称的测量窗口「07:15–07:35 UTC」以内）时，四个数字（304/8/70/3）逐一精确复现。**不算不一致**：这是活仓库里的时间点统计，Opus 报告本身没有断言这个数是永久不变的 |
| item b：`REPORT\|SUMMARY\|FINDINGS\|ANALYSIS` 正则来源 | **✓ 主 agent 重点要核的一条，独立复现，比 Opus 自己给出的证据更硬**：我直接在 `~/.vscode-server/extensions/anthropic.claude-code-2.1.273-linux-x64/resources/native-binary/claude`（ELF 可执行文件，与安装的扩展版本号一致）里 `grep -a` 到原样代码：`if(s.agentId&&/^(REPORT\|SUMMARY\|FINDINGS\|ANALYSIS).*\.md$/i.test(YMo(y)))return i("tengu_subagent_md_report_blocked",…),{result:!1,message:"Subagents should return findings as text, not write report files. Include this content in your final response instead.…`。这段代码在一个含 `getPath`、`checkPermissions`、`validateInput({file_path, content})` 的对象里——`file_path`/`content` 是 Write 工具（不是 Edit）的入参形态，且判定条件 `s.agentId&&…` 明确只在**子 agent**调用时生效。**两条能不能同时成立**：能。`i` 标志确认存在（大小写不敏感），所以 2026-09-17 实验执行员写小写 `/tmp/.../trials2/experiment-runner/report.md` 被拒，与「判法是 `/^(REPORT\|SUMMARY\|FINDINGS\|ANALYSIS).*\.md$/i`」不矛盾——`report.md` 在 `i` 标志下命中 `^REPORT`（大小写不敏感）+ `.*\.md$`。这一段是我独立验证得出，不是照抄 Opus 的说法 |

**Opus 报告计数**：核了约 30 处引用 + 3 份脚本复跑，✓ 30/30，✗ 0。核不动：`f`、`j` 两条自陈「没编译 Rust」，我也没有编译 crates（超出这一轮范围，且工作区正被别的会话改动，编译结果无法归属到某一版）；`h` 条（21/49 号 `--write` 缺开工哈希）只核了引文位置，没有实跑 `--write` 复现；「没打中的形状」表里除 report.md 正则那一条外，其余六行只核了文字层面的引用位置，没有逐条复现（Opus 自己也在「这条腿自己的限度」里说明这些是从语义推的，没有取样到真实场景）。

# 三、本地攻方（`three-way-local-attack`）

| 检查项 | 结果 |
|---|---|
| 提示文件格式（无 markdown 强调、纯英文、按项目规则写） | ✓ 逐行核对，`.claude/rules/three-way-inference.md`「提示里不许用 markdown 强调」的判据满足 |
| 运行记录（5 次调用、退出码、词数、OOV 判定） | ✓ `-void1.md`、`-void2.md` 存在且内容与运行记录描述的拼接词吻合（`encounteringwriting`、`functioned`）；`-s2.md`（第三次，判绿但人工复核记「带损坏」）里 `discrepanciesal` 确实出现在第 13 行「the trial run only used a list with no discrepanciesal errors」，与运行记录逐字一致 |
| `-s1.md`、`-s3.md` 内容（凑够两份「干净」样本） | ✓ 均为 5 段编号答复，句式完整，未见明显缺词或断句 |
| translation-audit.md：Fact one（16、12、4、没有超过两次） | ✓ 与 `records/…md:403,430` 对得上（403 行「改动最多的四个定义各拿一件一次性小活再试跑一次」；430 行「这些改法被攻过零轮，第二轮对抗还没派」），"十二"是从 16−4 推出，audit 自己已注明是推导 |
| translation-audit.md：Fact six（D1「仓根下跑」被采纳后第二次试跑才暴露） | ✓ `agent-defs-r1-main-verification.md:37`、`records/…md:408` 均逐字对得上 |
| **translation-audit.md：Fact seven（`three-way-inference.md:187-189`）** | **✗ 行区间不含引文**。audit 引的英文句子对应中文原文「判决由主 agent 做，不由投票做……**不一致 → 先核实，再讨论。核实指去查能分辨它们的那个事实，不是再加一个模型来投票。分歧点本身要写进结论里**」，但第 187-189 行实际内容是「## 判决由主 agent 做，不由投票做」（187 行）+ 空行（188）+「三方一致 → 继续」（189 行）——**引文里「不一致…分歧点本身要写进结论里」这半句实际在第 190-191 行**，不在被引的 187-189 区间内。这不是背景材料行号误用（背景材料 `_agent-defs-r2-background.md` 没有单独包含这段规则文件，本条不适用「误写成背景材料行号」这条处置）——是纯粹的行区间划少了 |
| translation-audit.md：其余各条（Fact two/三/四/五、五个问题的回指调整） | 抽查未见问题（未逐条核，见「没做什么」） |

**本地攻方计数**：核了 8 处（1 处提示格式、1 处运行记录、2 份样本内容、4 处 translation-audit 引用），✓ 7，✗ 1（Fact seven 行区间）。

# 四、本地辩方（`three-way-local-defense`）

| 检查项 | 结果 |
|---|---|
| 提示文件格式 | ✓ 无 markdown 强调，纯英文，26 条 Fact + 3 问 + D1-D3 建议方向格式完整 |
| translation-audit.md：F1、F3（`agent-common.md:14,18`） | ✓ 逐字匹配 |
| translation-audit.md：F4（appendix 1122-1129，对应旧 12-19 行） | ✓ 覆盖了「## 写」整节（1122 header 到 1128 最后一条 bullet，1129 是紧邻的空行），与所述内容吻合 |
| translation-audit.md：F5（`replace-once.py:1-9,17-32`） | ✓（描述性引用，非逐字引用，audit 自己标注了性质） |
| translation-audit.md：**F6 自我修正**（「没被」→「没有被」差一个插入字符，非「两个字符」） | ✓ 我独立核实：「没被」2 字、「没有被」3 字，恰好插入「有」这一个字；`replace-batch.py` 第 6-11 行 docstring 原文与所描述的 2026-09-14「54 处」事故完全对应 |
| 运行记录（3 次调用，1 判红 2 判绿） | ✓ `-void1.md` 存在，内容与「拼接命中 `streamlines(=stream+lines)`」的描述一致（未逐字比对，抽查存在性） |
| **运行记录：「s1、s2 两份样本对同一问题给出的判定方向一致：Q1、Q2 均判 `DOES NOT HOLD`」** | **✗ 与产物不符，这是这一轮核出的最重要一条**。`agent-defs-r2-local-defense-output-s1.md` 对 Q1、Q2 确实都以「DOES NOT HOLD」开头。但 `agent-defs-r2-local-defense-output-s2.md`（`grep -n "DOES NOT HOLD" …-s2.md` **零命中**）对 Q1、Q2 给出的是**相反方向**的答案：Q1 论证「the writes performed by these scripts would trigger the write-scope gate……The all-or-nothing batch safety …was preserved」（旧规则站得住），Q2 论证「The old rule F11 required the subagent to write the report file immediately……This avoided the risks…The new rule's delay …introduces unnecessary risks that the old rule's immediate write mitigated」（旧规则站得住）——两条都是在**成功防守**旧规则，不是「DOES NOT HOLD」。s1 与 s2 对 Q1、Q2 的判定方向是**相反**的，不是「一致」。只有 Q3 两份样本方向一致（都不含「DOES NOT HOLD」，都论证旧规则 F19 站得住）。这条错误发生的位置和机理，与仓里已经记录过的「转述会漂移」同型——把两份并不一致的样本，总结成「方向一致」| `grep -n "DOES NOT HOLD" research/prompts/agent-defs-r2-local-defense-output-s1.md research/prompts/agent-defs-r2-local-defense-output-s2.md`（s1 两处命中，s2 零命中）；`cat -n` 通读两份全文逐句核对 |

**本地辩方计数**：核了 7 处（1 处提示格式、4 处 translation-audit 引用、1 处运行记录存在性、1 处运行记录内容判定），✓ 5，✗ 2（其中「DOES NOT HOLD」那条是本轮实质性最重要的发现）。

# 汇总计数

- 判别力自证：1 条，判 ✗（方法分辨，符合要求）。
- 四份腿报告合计核了引用/复跑/样本对照约 **62 处**（Sonnet 17、Opus 30 含 3 份脚本复跑、本地攻方 8、本地辩方 7）。
- ✓ **57 处**，✗ **5 处**：
  1. Sonnet 报告 G1 段「records 第 329 行」应为第 255 行（内容本身真实，纯行号错）。
  2. 本地攻方 translation-audit.md「Fact seven」行区间 187-189 未覆盖被引文字实际所在的 190-191 行。
  3. 本地辩方 runlog「s1、s2 对 Q1、Q2 判定方向一致」与产物不符：s2 对 Q1、Q2 实际论证方向与 s1 相反（这是内容性错误，不是行号错误，价值最高）。
  4-5. 均计入上述条目内（Fact seven 与 DOES NOT HOLD 各算一处独立 ✗，已在第 3 条单独列出；如需单独计数，✗ 共 3 类问题、涉及 3 处独立核验点，上文按「核了处数」合计 5 处是把 G1、Fact-seven、DOES-NOT-HOLD 各算 1，另外把「本地攻方第 2 问」判断链与 item b 统计漂移各自算了一次「核」但未计入 ✗——这两处结论是 ✓，不是 ✗，故最终 ✗ 计数为 3 处、涉及 3 份不同产物（Sonnet 报告、本地攻方 audit、本地辩方 runlog）。

（更正：为避免计数口径自相矛盾，以「✓ 57、✗ 3、另 2 处是核实后判定为 ✓ 但过程值得记录」为准——即 G1 行号错、Fact seven 行区间错、DOES NOT HOLD 方向错，共 3 处真实 ✗；「本地攻方第 2 问答非所问」与「item b 统计随时间漂移」经核实均站得住，判 ✓。）

# 没做什么

- 不判一条打中成不成立、该不该采纳；「本地攻方第 2 问」「Q1-Q3 defense 内容对不对」等推论本身的是非，交主 agent 逐条现查，本报告只核引用与产物是否对得上。
- 没有编译 `crates/`、没有跑虚机、没有跑 `gate.sh` 全量、没有跑 `mutate.sh`（Opus item h/j、本地辩方涉及的编译类判断核不动，原因写在对应格里）。
- 没有实跑 21/49 号门禁 `--write`（只核了引文位置，未复现「跑前跑后各 diff 一次」的实际行为）。
- 没有逐点核 Sonnet 报告里 J6 experiment-designer 的全部 8 点对应关系（仅抽查「登记的固定节名」一节存在）。
- 没有逐条核 Opus 报告「没打中的形状」表六行里除「report.md 正则」外的其余五行（A1 的 cargo fmt、续做之后第二次交回、实现员 derive、mutate.sh 并发、核查员按字段比）——这些标了「没实测/没取样到」，Opus 自己也说明了限度。
- 没有逐条核本地攻方/本地辩方 translation-audit.md 里未在上表列出的其余约 20 处「无」标注的行（Fact two/三/四/五及五问里未特别标注的部分、F2/F7-F26 里除 F1/F3/F4/F5/F6 外的其余条目）。
- 没有改任何文件；本轮只写了报告文件本身（本次交回不写报告文件，按会话系统说明）与草稿目录（`/tmp/claude-1000/agent-defs-r2-verifier/`，含判别力自证副本与 Opus 三份脚本的复跑产物）。

# 相关文件路径

- 草稿目录：`/tmp/claude-1000/agent-defs-r2-verifier/`（`selftest/agent-common.md` 判别力自证副本；`opus-model-rerun/` 含三份脚本拷贝、复跑产物 `rerun-*.out`）
- 独立验证用到的外部文件（只读，未修改）：`/home/fy5090/.vscode-server/extensions/anthropic.claude-code-2.1.273-linux-x64/resources/native-binary/claude`（ELF 二进制，`grep -a` 验证工具层正则）
