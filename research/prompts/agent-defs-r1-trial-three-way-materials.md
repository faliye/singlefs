# 材料员试跑报告（agent-defs-r1）

## 做了什么

1. 门禁阶段归属表登记给 `three-way-materials` 的两个阶段（47、58 号）开工前各跑一次，均绿：
   - `47-research-script-selftests.sh`：`✓ research 脚本的自证都通过（查了 8 份……）` EXIT=0
   - `58-implementation-premise.sh`：`✓ 三方论证正文都写了实现今天的样子（检查了 10 份，标题日期 ≥ 2026-09-17）` EXIT=0
   两阶段都不点名 `quote-kb.py`/`kb-sections.py`/`checklist-specs.py`，也不点名 `_agent-defs-r1-body.md`，照写、继续。
2. 读正文 `research/prompts/_agent-defs-r1-body.md`，grep 正文里「已经如何」型断言的动词，找到一个主 agent 输入清单里没有的补充项：正文第 18 行「用户已定……见第九节」的列表（按立场拆腿、实现员在主工作区改、写代码按活的形态分等）逐条 grep，源头是 `records/2026-09-16-subagent拆分提案.md` 第九节「用户定案」——补进清单标「抄」。
3. 对全部 39 个 md 文件（16 个定义 + `agent-common.md` + `records/...拆分提案.md` + `CLAUDE.md` + 20 份规则）跑 `kb-sections.py`，机械清单不再过滤，逐行按主 agent 给的规则填「抄 / 不抄 / 理由」。
4. 5 个非 md 文件（`stage-owners.tsv`、`agent-write-scope.tsv`、`agent-write-scope.sh`、`62-stage-owners.sh`、`63-agent-write-scope.sh`）按主 agent 指示不进 `kb-sections.py`，整份用 `--extra 文件:1-末行` 带进 `checklist-specs.py`。
5. 用 `checklist-specs.py --cited 正文 --out 附录` 抽附录，撞到一次工具本身的解析缺陷（见「试跑观察」第 2 条），改清单后重跑到退出码 0。
6. 拼背景材料：正文 + 清单 + 附录，`research/prompts/_agent-defs-r1-background.md` 排他新建。

## 产出

- `research/prompts/_agent-defs-r1-checklist.md`（594 行，39 个文件的小节清单）
- `research/prompts/_agent-defs-r1-appendix.md`（1742 行，109 段整抄）
- `research/prompts/_agent-defs-r1-background.md`（2435 行，正文 + 清单 + 附录）

清单行数：抄 123 行，不抄 253 行（数据行合计 376，`grep -oP` 数出，见下方核对命令）。

核对命令与结果：
```
$ grep -oP '^\| .* \| \K(抄|不抄)(?= \|)' research/prompts/_agent-defs-r1-checklist.md | sort | uniq -c
    253 不抄
    123 抄
```

`checklist-specs.py` 三次调用的原样末行与退出码：

1. 第一次（未处理 `## 十二、...` 标题含冒号）：
   `quote-kb: 没有这个文件：records/2026-09-16-subagent拆分提案.md@## 十二、第 0 步实测（2026-09-16 UTC 22`
   `  → 用仓根相对路径`
   EXIT=1
2. 第二次（改成「不抄」+ `--extra` 同时提供，撞上反向核对）：
   `quote-kb: 清单标了「不抄」，却有 1 节其实被抄进了附录：`
   `    records/2026-09-16-subagent拆分提案.md：## 十二、第 0 步实测（2026-09-16 UTC 22:31 与 23:05–23:30，东京 09-17 07`
   `  → 把清单那一行改成「抄」并写理由，或者收窄带出这一节的那个 文件:A-B 行区间，别让它连带把它抄进来`
   EXIT=6
3. 第三次（改成「抄」+ 理由以「随」开头，跳过自动生成的 `@` 取法，改用 `--extra` 的行区间覆盖同一段）：
   `  ✓ 109 段整抄进 research/prompts/_agent-defs-r1-appendix.md，回读逐字节一致，清单里标「抄」的每一节都在，正文提到的每个 kb 文件都有清单`
   EXIT=0

## 全部「不抄」行

### 差异化理由的 29 行（`records/2026-09-16-subagent拆分提案.md` 21 行 + `CLAUDE.md` 8 行）

| 文件 | 小节 | 理由 |
|---|---|---|
| records/...拆分提案.md | # subagent 拆分提案（2026-09-16） | 主 agent 未点名此节，正文未引用其内容，不进附录 |
| records/...拆分提案.md | （subagent 标题之下、第一个下级标题之前的正文：第 2-6 行） | 同上 |
| records/...拆分提案.md | ## 一、现状：这个仓已经在派 agent，只是每次现写规矩 | 同上 |
| records/...拆分提案.md | ## 二、拆分判据 | 同上 |
| records/...拆分提案.md | ## 三、留在主 agent 的 | 同上 |
| records/...拆分提案.md | ## 四、agent 清单：十六个，分四族 | 同上 |
| records/...拆分提案.md | ### 族一　三方论证（压在 `.claude/rules/three-way-inference.md`，只能项目本地） | 同上 |
| records/...拆分提案.md | ### 族二　实现（压在 `.claude/rules/implementation-workflow.md`） | 同上 |
| records/...拆分提案.md | ### 族三　门禁与回扫 | 同上 |
| records/...拆分提案.md | ### 族四　kb、实验、外部资料 | 同上 |
| records/...拆分提案.md | ## 五、写范围怎么拦：先有闸，再铺 agent | 同上 |
| records/...拆分提案.md | ## 六、调度形态 | 同上 |
| records/...拆分提案.md | ## 七、落地次序 | 同上 |
| records/...拆分提案.md | ## 八、放哪一层 | 同上 |
| records/...拆分提案.md | ## 十、这份提案没做的 | 同上 |
| records/...拆分提案.md | ## 十一、第一轮查出、还没做的欠账 | 同上 |
| records/...拆分提案.md | ## 十三、定义的静态核实（2026-09-17） | 同上 |
| records/...拆分提案.md | ## 十四、CLAUDE.md 与规则做减法的次序 | 同上 |
| records/...拆分提案.md | ## 历史版本 | 同上 |
| records/...拆分提案.md | ### 2026-09-16 | 同上 |
| records/...拆分提案.md | ### 2026-09-17 | 同上 |
| CLAUDE.md | # singlefs | 主 agent 只点名「什么时候派哪个 agent」一节，这一节不在其内 |
| CLAUDE.md | （singlefs 标题之下、第一个下级标题之前的正文：第 2-7 行） | 同上 |
| CLAUDE.md | ## 规则（始终生效） | 同上 |
| CLAUDE.md | ## 规范从哪来 | 同上 |
| CLAUDE.md | ## 项目本地规则 | 同上 |
| CLAUDE.md | ## 项目本地事实 | 同上 |
| CLAUDE.md | ## 本项目的特殊性 | 同上 |
| CLAUDE.md | ## 一句话版本 | 同上 |

注：`## 九、用户定案`、`## 十二、第 0 步实测……`、`## 十五、……`、`## 十六、……`（含子节「写范围闸」「实现员试跑」）、`CLAUDE.md` 的「什么时候派哪个 agent」这 7 行标「抄」，不在上表。

### 统一理由的 224 行（20 份规则文件，逐节列在清单文件里，此处按文件汇总计数）

理由对全部 224 行逐字相同：「腿已继承全文，计划第十二节实测（CLAUDE.md `@` 引用的全部规则随会话系统提示整份进各条腿的上下文，不需要再抄进背景材料）」。

| 文件 | 不抄行数 |
|---|---|
| .claude/rules/format-evolution.md | 6 |
| .claude/rules/fs-design.md | 21 |
| .claude/rules/implementation-first.md | 5 |
| .claude/rules/implementation-workflow.md | 5 |
| .claude/rules/mutation-sampling.md | 11 |
| .claude/rules/three-way-inference.md | 11 |
| .claude/singlefs-ai-sop/rules/code-discipline.md | 24 |
| .claude/singlefs-ai-sop/rules/command-safety.md | 13 |
| .claude/singlefs-ai-sop/rules/design-doc-discipline.md | 5 |
| .claude/singlefs-ai-sop/rules/engineering-philosophy.md | 8 |
| .claude/singlefs-ai-sop/rules/evidence-discipline.md | 18 |
| .claude/singlefs-ai-sop/rules/kb-discipline.md | 10 |
| .claude/singlefs-ai-sop/rules/machine-first.md | 13 |
| .claude/singlefs-ai-sop/rules/pushback-discipline.md | 8 |
| .claude/singlefs-ai-sop/rules/session-wrapup.md | 6 |
| .claude/singlefs-ai-sop/rules/show-me-test.md | 15 |
| .claude/singlefs-ai-sop/rules/sop-first.md | 9 |
| .claude/singlefs-ai-sop/rules/test-discipline.md | 16 |
| .claude/singlefs-ai-sop/rules/verify-before-claiming.md | 6 |
| .claude/singlefs-ai-sop/rules/writing-discipline.md | 14 |

（合计 224；逐节文本一字不差地在 `research/prompts/_agent-defs-r1-checklist.md` 里，未在此处重复 224 行——20 个文件、每份一个理由字符串，重复贴出不比给文件路径多任何可核信息。）

## 试跑观察（定义里不清楚 / 做不下去 / 与实际对不上的地方）

1. **非 kb 文件与非 md 文件怎么进清单，定义没写。** 定义第 1 步只说「正文里提到的每个 kb 文件…都用 `kb-sections.py` 生成清单」，`kb-sections.py` 只认 markdown 标题（`#{1,6}`），对 `.tsv`/`.sh` 这类没有标题结构的文件完全不适用。这一轮能处理，是因为主 agent 在派发提示里已经把这五个非 md 文件连同取法（`--extra 文件:1-末行`）都点好了；如果换一轮主 agent 只说「这几个脚本也要核」而不点出具体做法，材料员自己无法从定义正文推出「非 md 文件绕过 kb-sections.py、直接用 --extra 整份带」这条路——定义应当把这条路径写成通用规则，而不是依赖每次派发提示单独交代。

2. **`checklist-specs.py`/`quote-kb.py` 有一个会撞上、而且提示语具体误导的解析缺陷：标题含冒号时 `@标题` 取法必炸，报错却说「没有这个文件」「用仓根相对路径」。** `quote-kb.py` 的 `pick()` 按 `(':','range')→('@','head')→('~','re')` 的顺序查分隔符，只要标题里出现一个冒号（这一轮撞到的是时间戳「22:31」「23:05–23:30」），就会被错误地当成 `文件:A-B` 语法在冒号处切开，永远走不到 `@` 分支。定义第 3 步写「退出码非 0 就按它给的下一步改清单再抽，不绕过」，而这里工具给的下一步（「用仓根相对路径」）对这个失败模式是错的——路径本来就对，问题是标题含冒号。按下一步再抽只会在「路径没变」上打转，得自己读 `pick()` 源码才看得出真正原因。
   - 更麻烦的是**看似正确的修法会撞进第二个坑**：把该行标「不抄」、另用 `--extra` 相同区间带入附录（这正是定义里「分项索引表标『不抄』并写『用 --extra 按行区间取』」那条给的先例形状），会立刻撞上反向核对（C320，退出码 6：「清单标了『不抄』，却有 1 节其实被抄进了附录」）。退出码 6 给的下一步是「把清单那一行改成『抄』并写理由，或者收窄行区间」——只字未提「抄」配合「随」前缀可以跳过自动生成的坏取法这条机制。真正的解法（标「抄」、理由以「随」开头、用 `--extra` 顶替自动生成的 `@` 取法）只能从 `checklist-specs.py` 的 `specs_from_checklist()` 源码里 `reason.startswith('随')` 那一行反推出来，定义和三方论证规则里都没有写「随」前缀除了「被父节标题取法带出的子节」之外还能用来规避这一类工具缺陷。这一轮靠现读脚本源码才绕出来，换一个不读源码、只照定义机械操作的材料员大概率会在退出码 1 与退出码 6 之间反复横跳。

3. **「已经如何」grep 这一步对非 kb/decisions 轮次完全没有工具兜底。** `quote-kb.py --cited` 的自动检测（`cited_kb_files()`）只认 `.claude/kb/...md` 显式路径和 `D<n>（`/`E<n>（`/`C<n>（`/`I-<n>.<m>（` 这几种编号形态；这一轮正文一个都没有（是 agent 定义评审轮，不是文件系统决策轮），所以 `--cited` 检查形同虚设，不会因为我漏抄了某个「已经如何」的源头文件而报错。这一次靠自己动手 grep 正文里的动词短语才找到 `records/2026-09-16-subagent拆分提案.md` 第九节是正文第 18 行「用户已定」列表的源头；但没有任何机械检查能证明我「找全了」——比如我还查到 `records/2026-09-17-CLAUDE.md去冗余.md` 第四步同样复述了「写代码按活的形态分」这条定案，判断它是转述而非源头、不必单独加清单，这个判断纯靠人读，工具一个字都不会提醒漏没漏。

4. **「规则文件整份标不抄」这一步产出 224 行几乎纯重复的清单行，除了满足「清单不许再过滤」的字面要求外，可核信息很少。** 20 份规则文件、每份的判定理由逐字相同，但定义第 1 步要求对它们也全量跑 `kb-sections.py`、不许过滤，于是清单里 253 行「不抄」中有 224 行只是在重复同一句话。这不是做不下去，是「机械但笨重」：往下如果规则数量再涨，这部分会持续线性变胖，而它对主 agent 复核「该抄的有没有漏」这件事的边际帮助接近于零（因为这些行从一开始就不可能标「抄」）。

5. **写文件时我自己第一次没有照 `agent-common.md`「新建文件一律排他」执行**：生成 `_agent-defs-r1-checklist.md` 时用了普通的 `cat header labeled-*.md > 目标文件`，没有先 `set -o noclobber`。当时已经用 `ls`/`git status` 确认目标路径不存在，事后也没有发生覆盖，但这是流程上的疏漏，不是定义写得不清楚——是我自己漏做了共用约束里明写的一步，后续两个文件（`appendix.md`、`background.md`）都补上了 `set -o noclobber`。记在这里是为了如实交代，不是定义的问题。

## 没做什么

- 不判正文「一~五」节问得对不对；机械抽取只保证「抄的没错」，不保证「该抄的都抄了」——这一句照抄进本节。
- 不读试跑报告 `research/prompts/agent-defs-r1-trial-*.md`：正文已指路给各条腿自己读，材料员的清单与附录都不含它们。
- 不派、不判四条腿；判决与坐实交主 agent。
- 未跑全量门禁、未跑 cargo：这一轮不改代码，`.claude/agent-common.md`「不做」一节也不要求材料员做这些。
