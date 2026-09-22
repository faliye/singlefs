# agentdef-r3 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

目标：opus 腿报告第 44 行引 `.claude/main-agent.md:43`（称「整行抄的那一截」），引文起句「交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**……」。

做法：在草稿目录 `/tmp/claude-1000/agentdef-r3-verifier/selftest/main-agent.md`（`.claude/main-agent.md` 的副本）里，把待核的行号从 43 改成 44（模拟腿引错了一行），按正文第 2 步核对。

```
$ python3 - <<'PY'
with open('.claude/main-agent.md', encoding='utf-8') as f:
    lines = f.readlines()
quoted = "交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看"
print("在第43行(原引用):", quoted in lines[42])
print("在第44行(行号+1，模拟核这一句):", quoted in lines[43])
PY
在第43行(原引用): True
在第44行(行号+1，模拟核这一句): False
```

第 43 行判 ✓（引文与原文完全对上）；把行号改成 44 后，第 44 行实际内容是「暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |」，与引文完全不符，判 **✗**。核查方法能分辨，往下做。

## 一处主 agent 自己没提到、但我核查中发现的第三次变动

`.claude/gate.d/68-knowledge-sync.sh` 在主 agent 派发时是 381 行（sha256 `86083df1…`，主 agent 已说明），但我 2026-09-22 00:45 UTC 现查时它已变成 **393 行**（sha256 `b26cb1b6bad8f0667931d905cbfeadd304bbe042e8585bf825a90eb48171ebac`），判据⑤的措辞也变了：381 行版写「这一轮**新写**的每份同步记录」「已归档进版本库（archive-past-rounds.py 会删掉工作区里的那一份）」，393 行版写「这一次提交要带上的每份同步记录（**还没进 HEAD** 的那几份……）」「已经提交的记录是冻结证据、改不得」。这是**第三个版本**，既不是 HEAD 的 290 行、也不是主 agent 记的 381 行。这份文件在我核查期间仍在被别的会话继续改，我没有碰它。凡是本报告里核 68 号行号对不上 HEAD 也对不上 393 行现状的，记「分不清：文件在腿交回之后、以及核查过程中又被别的会话继续改动」，不记 ✗，不算进下面的计数。

## 云端攻方（Opus）腿报告核对表

背景材料 `_agentdef-r3-background.md` 未被误当作行号来源——腿引的都是仓文件自己的行号，逐条核对如下。这一轮是设计轮，没有 `crates/`、`.claude/kb/` 快照；除 68 号文件另有说明外，其余按主树现查。

| 引用 | 核的结果 | 命令 / 证据 |
|---|---|---|
| `.claude/main-agent.md:43`（整行抄的那一截，第 44 行前的引文） | ✓ | 见判别力自证 |
| `sweep-acceptance-2026-09-18` v2/v3 各 8 份逐行判定报告，同在 `852e4b3` | ✓ | `git show --stat 852e4b3` 命中 v2-judge-1..8、v3-judge-1..8 共 16 个文件 |
| v2 载体数 1281 / v3 载体数 1342 / 共有载体 1087 | ✓ | 独立脚本 `indep_diff3.py`（`load()` 用 `setdefault` 首次出现优先，与 `forge-recheck-table.py` 同一语义）复算得 1281/1342/1087，逐字一致 |
| 两版判定不同的载体数 **256**（`diffjudge.py` 跑出来的数） | **核不动**：`diffjudge.py` 是 `/tmp/claude-1000/agentdef-r3-opus/` 下的一次性脚本，报告自己交代「没入库」，会话已结束，无法重跑复核这个精确数字 | 报告自己在 59 行披露「`run.sh` 里那一份挑法略有不同，报 259 组」——这句披露本身经核实为真：见下一行 |
| 「`run.sh` 里那一份」（`forge-recheck-table.py`）挑法产出 **259** 组 | ✓ | `nice -n 19 bash research/prompts/agentdef-r3-opus-model/run.sh /tmp/claude-1000/agentdef-r3-verifier/opus-run` 原样输出「# 候选改判对 259 组」；独立脚本复算同样得 259 |
| diffjudge.py 输出的前三行样本（checks-owed.md:131/150/159 三行的判定与理由片段） | ✓（内容真实存在，但报告没有标注这三行的理由文本被截断） | 用独立脚本按「首次出现优先」重建 v2/v3 对齐表，三行的 carrier、judgement 转换、理由前缀逐字命中；理由文本在 131/159 两行处于比报告更长（例如 131 行报告断在「现在」，完整文本还有「写什么」），159 行报告断在「2026-09-13 的」，完整文本还有后续。诊断：diffjudge.py 大概率对理由做了截断（与 `forge-recheck-table.py` 自身的 `reason[:60]` 截断同型），报告贴的是截断后的原样输出，不是摘句 |
| 跨轮：2026-09-18 v3 的 1342 个载体 × `owed-batch1to3` 的 123 个载体，共有 5、判定不同 2，两行样本原样 | ✓ 全部 | 独立脚本 `cross_diff.py`：`v3 载体 1342 owed 载体 123 / 跨轮共有载体 5 / 跨轮判定不同 2`，两行 `('.claude/kb/checks-owed.md:403', '不相干', '事件句不改')`、`('.claude/kb/checks-owed.md:49', ...)` 逐字重现 |
| `git show 364d323^:research/prompts/owed-batch1to3-sync-judge-f1-f11.md` 第 7 行整行抄 | ✓ | `git show 364d323^:...` 第 7 行与报告第 82-83 行引文逐字一致 |
| 同文件第 42 行括注「（主 agent 抽查指出：这一格说的是 core 内部同源，94 号罩不到）」 | ✓ | 第 42 行末尾逐字一致 |
| `owed-batch1to3-sync.md:10`（回扫走的是阶段同步那套工具……） | ✓ | `git show 364d323^:research/prompts/owed-batch1to3-sync.md` 第 10 行逐字一致（该提交与 `11a551b` 内容相同，已用 `diff` 核对两个提交号取出的文件字节相同） |
| `owed-batch1to3-sync.md:51`「抽样这一次漏掉了 06:42 那一处判错」 | ✓ | 第 51 行含此子串，逐字一致 |
| 超几何概率：N=138,K=16,n=20 → 0.9304 | ✓ | `python3 -c` 复算 `1-comb(122,20)/comb(138,20)` = 0.9304475656544308 |
| 超几何概率：N=123,K=16,n=20 → 0.9526 | ✓ | 复算 `1-comb(107,20)/comb(123,20)` = 0.9525569752547243 |
| 单点命中概率：0.1449 | ✓ | 20/138 = 0.14492753623188406，四舍五入 0.1449 |
| N、K、n 是否写清 | **部分写清**：n=20（「随机抽 20 行」明写）、K=16（「（16 行）」明写）在三处都出现；N 在第二式明写「按不同载体 123 行算」，但第一式（0.9304 那一式）的 N=138 没有在算式那三行文字里重述，要回读上文「判定行数 138」才能补齐 | 报告第 91-99、101-107 行原文核对 |
| `classes.py` 输出：判定行数 138 不同载体 123，54 类，06:42 所属类 `('C12',)` 该类共 16 行 | ✓ 全部 | 独立脚本 `classes_check.py`（按理由文本里出现的 `C\d+` 集合分类）复算：`判定行数 138 不同载体 123` / `类数 54` / `06:42 所属类 ('C12',) 该类共 16 行`，逐字重现 |
| `dist.py` 输出：全轮分布、全轮「要改」占比 0.109、21 个组、F3 行数19要改9占比0.47 等六行 | ✓ 全部 | 独立脚本 `dist_check.py` 复算六行数字逐字重现（含 F11/F1/F2/F4/F5 五行与 F3 组构成） |
| 「仓里入库过 19 份 `*-sync.md`」 | ✓ | `git log --all --diff-filter=A --name-only --format= -- "research/prompts/*-sync.md" \| sort -u \| wc -l` → 19 |
| 「逐行全看」写进 `main-agent.md` 之后只有三份同步记录（`owed-batch1to3-sync.md`@`11a551b`、`c224-format-impact-sync.md`@`364d323`、`c48-multipath-registry-sync.md`@`13bccea`） | ✓ | `git log --diff-filter=A --name-only 11a551b~1..HEAD -- "research/prompts/*-sync.md"` 恰好三份，逐一核对新增提交号一致 |
| 「其中只有一份报过判错」 | **存疑，非清白 ✓**：`owed-batch1to3-sync.md`（:51）与 `c224-format-impact-sync.md`（`git show 364d323:` 取出，:21）都含「判错」字样；c48 一次都没有。c224 那处的「判错」是**回指**同一件 06:42 历史事件（写在"这一处走过三方而没有定死"那句里），不是 c224 自己那一轮新判错的一次——若报告本意是「只有一份报的是『自己这一轮新查出的判错』」，狭义解读能成立；字面「报过判错」按原始字符串搜，2/3 份都命中，不是 1/3 | `grep -n 判错` 对两份归档文件各命中一处；c48 零命中（`grep` 退出码 1） |
| c48「连『全看』二字都没出现」 | ✓ | `grep -n 全看 c48-multipath-registry-sync.md` 退出码 1，零命中 |
| `.claude/gate.d/68-knowledge-sync.sh:22-26`（判据⑤整段抄） | **分不清**：核对时文件已是第三个版本（393 行），与报告贴出的 381 行版本、以及 HEAD 的 290 行版本均不同（见前面「一处…第三次变动」一节）| 393 行版本第 22-26 行内容见上文，与报告引文有实质措辞差异 |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md:183`「判别子观测不到」整行 | ✓ | 第 183 行逐字一致 |
| `run.sh` 复跑：甲（造法甲，6 行改判，259 组，退出码 0） | **✓ 数字与退出码**，**✗ "原样输出"的逐字性**：报告贴的"原样输出"块（149-157 行）漏掉了脚本实际会打到 stderr 的一整行「# 全程 open 过的文件：[16 个路径]」——我在草稿目录用 `nice -n 19 bash .../run.sh <草稿工作目录>` 与单独调用 `forge-recheck-table.py 2>单独文件` 两种方式都复现出这一行，стderr 与其余几行同一个流，不存在"只截获 stdout故而看不到"的解释 | 见下方"复跑记录" |
| `run.sh` 复跑：乙（0 行表，退出码 0） | ✓ 逐字（含无中间诊断行） | 同上 |
| `run.sh` 复跑：丙（判别力自证，4 处红，退出码 1） | **✓ 语义**，**✗ 路径逐字性**：脚本实际打印的是完整临时路径（如 `/tmp/.../opus-run/red.md:7`），因为脚本把 `"$WORK/red.md"`（绝对路径）整个作为 `path` 变量传给 `check-recheck-table.py`；报告贴出的却是裸文件名 `red.md:7`，缺了目录前缀——即使在腿自己原来的 `/tmp/claude-1000/agentdef-r3-opus/run/` 目录下跑，输出也会带那个前缀，不是"环境不同所以路径不同"能解释的 | 同上 |
| `run.sh` 复跑：丁（5 行改判，138 行判定，退出码 0） | ✓ 逐字（除 tmp 路径因跑在草稿目录而不同，其余字段——138、5、"全程 open 过的文件"含 2 个 .md、"断言通过"——完全一致） | 同上 |
| `forged-section.md` 前两行数据行 | ✓ 逐字节 | `diff` 草稿产物与报告贴出文本，含尾部 ` \|` 均一致 |
| `forged2-section.md` 前两行数据行 | ✓ 逐字节 | 同上 |

**复跑记录**：模型目录四个文件先拷到 `/tmp/claude-1000/agentdef-r3-verifier/opus-model-copy/`，sha256 与报告给出的四个哈希（`fcd41d95…`、`ad0a4c8f7…`、`cda9318a…`、`9dc02eec…`）逐一比对，`diff -r` 与原目录逐字节相同。因脚本用 `dirname "$0")/../../..` 自动定位仓库根（需要真实 git 历史 `852e4b3`、`364d323^` 才能跑），复跑实际以 `bash research/prompts/agentdef-r3-opus-model/run.sh /tmp/claude-1000/agentdef-r3-verifier/opus-run` 方式执行——脚本本体是拷贝过来核对哈希用的静态副本，真正复跑用的是它在仓内的可运行拷贝（哈希已核对逐字相同），产物与中间文件全部写入草稿目录 `/tmp/claude-1000/agentdef-r3-verifier/opus-run/`，不落回腿原来的 `/tmp/claude-1000/agentdef-r3-opus/`。

**Opus 腿计数**：核了 27 处；✓ 21 处（含数字类核对，"258/259 挑法差异"记为已解释、非 ✗）；✗ 2 处（甲步"原样输出"漏一行诊断信息；丙步路径被裁短且未加省略号说明）；核不动 1 处（diffjudge.py 的 256 这个精确值，脚本未入库）；存疑非清白 1 处（"只有一份报过判错"）；分不清 1 处（68 号文件第三次变动）；另有 1 处（N/K/n 是否写清）判定为"部分写清"，不计入 ✓/✗。

## 云端辩方（Sonnet）腿报告核对表

| 引用 | 核的结果 | 命令 / 证据 |
|---|---|---|
| `agentdef-r2-main-verification.md:58`（「复核改判」表判决原文整行） | ✓ | 第 58 行逐字一致 |
| `agentdef-r2-opus-output.md:288-289`（收严原提出处两行） | ✓ | 288-289 行逐字一致 |
| `.claude/rules/three-way-inference.md`「第三轮之后停，不开第四轮」整句 | ✓ | 第 133 行逐字一致 |
| `c381-r3-main-verification.md:57`（`## 四、交用户` 标题） | ✓ | 第 57 行确为该标题 |
| `c381-r3-main-verification.md:65`（末句「按『第三轮之后停』，这条线到此为止……」） | ✓ | 第 65 行逐字一致 |
| `c381-r3-main-verification.md` 第四节表第 2 行（候选乙三条改法全部零轮） | ✓ | 表格行内容与 sonnet 转述一致（三条改法名称、"全部零轮"措辞对应） |
| `.claude/kb/decisions-history/2026-09.md:26`（「这条改法是攻方腿自己提的、被攻过零轮……用户定案 2026-09-21（改法 B）知情接受」，中间用省略号） | ✓ | 第 26 行含此开头与结尾子串，省略号处为该行中段内容，未被曲解 |
| `.claude/singlefs-ai-sop/rules/sop-first.md:70-71`（「一条规则想进来，先问它能不能变成一个会失败的检查」「变不成的多半是还没想清楚」） | ✓ | 70、71 行逐字一致 |
| `.claude/singlefs-ai-sop/rules/show-me-test.md:170`「门禁绿不等于可以不看代码」 | ✓ | 逐字一致 |
| 同文件 `:174`「语义对不对，写在 kb/decisions.md 和 kb/invariants.md 里，不在门禁里」 | ✓（sonnet 指出判决只摘了首尾、漏了中段这一半——现查属实，中段「不在门禁里」这半句在判决 `agentdef-r2-main-verification.md:27` 里确实没被转述进去） | 逐字一致；判决原文核对同上 |
| 同文件 `:176`「它们本身写得对不对，靠人」 | ✓ | 逐字一致 |
| `agentdef-r1-main-verification.md:26-27`（C 格「治不了它自己的触发案例」、F 格「改前可数改后不可数」） | ✓ | 26、27 行逐字一致；但 sonnet 转述 F 格引文时把「五种判定各至少 2 行」压成「各至少 2 行」，漏了「五种判定」这个限定词（省略号标了中段但没标这处词内省略）——小瑕疵，不是伪造 |
| `agentdef-r1-main-verification.md:29`（B 格原判「坐实（判定分布来自攻方的量尺……）」，与「旧条款『五种判定各至少 2 行』对最稀那档是写死的 100% 下限」） | ✓ 全部逐字 | 第 29 行两处引文均逐字一致 |
| `agentdef-r1-main-verification.md:31`（A 格「不独立复跑，记攻方的数并标明出处」） | ✓ | 第 31 行末尾逐字一致 |
| `agentdef-r1-main-verification.md:33`（「这一处不在这一轮定」） | ✓ | 第 33 行开头逐字一致 |
| `agentdef-r2-main-verification.md:27`（F 格判决全段，含辩方引文与⚠️补充半句） | ✓ | 第 27 行逐字一致，含「先留在 kb 里当待议，别写成规矩」半句 |
| `agentdef-r2-main-verification.md:28`（B 格判决「弱……降为『攻方的量尺如此，未独立复核』」） | ✓ | 第 28 行逐字一致 |
| `agentdef-r2-main-verification.md:47`（C468 与 C 格关系那段长引文） | ✓ | 第 47 行逐字一致，含 sonnet 用省略号省去的括注部分核对无误 |
| `owed-batch1to3-sync.md:51`（3.1 节第 1 步，用省略号引「不改：这一批不带这条改动……抽样这一次漏掉了 06:42 那一处判错」） | ✓ | 与上文 opus 腿核对同一行，逐字一致 |
| **`owed-batch1to3-sync.md:36`**（3.1 节第 2 步，「不改：**不删**——这一格说的是 core crate 内部两段代码同源，门禁 94 号只判 checker 与 core 的边界，罩不到」，sonnet 称「逐字一致」） | **✗ 明确判错**：第 36 行实际开头是「**改了**：不删——……」，不是「不改：不删」——sonnet 把「改了」错写/错读成「不改」，两者意思相反（该表第三列的取值只有「改了」或「不改」两种，是本行处置结果，不是可以混用的近义词）；且引文在「罩不到」处断开，实际这一格后面还有「；改成写明这一格今天没有编号盯着」，sonnet 没有用省略号标示这处截断，同时却断言「逐字一致」 | `git show 364d323^:research/prompts/owed-batch1to3-sync.md`（与 `11a551b` 内容经 `diff` 核对完全相同）第 36 行原文：「\| .claude/kb/decisions/06-快照实现模型.md:42 \| **欠**：C12（增量语义共用）：② 的引用计数旁表与运行时共用增量语义时…… \| 改了：**不删**——这一格说的是 core crate 内部两段代码同源，门禁 94 号只判 checker 与 core 的边界，罩不到；改成写明这一格今天没有编号盯着 \|」 |
| `git show 364d323^:research/prompts/owed-batch1to3-sync-judge-f1-f11.md` 第 42 行 `F3` 那一格「要改」 | ✓ | 第 42 行判定列确为「要改」 |
| **`git log --all -S"门禁 94 号只判 checker 与 core 的边界" -- .claude/kb/decisions/06-快照实现模型.md`**（sonnet 称「本报告独立重跑过……只命中 `11a551b` 一个提交」） | **✗ 明确判错**：原样重跑（用 `subprocess` 传参，排除 shell 转义干扰）该命令**零输出、零提交命中**，不是「命中 11a551b 一个提交」。该确切字符串（「门禁 94 号只判 checker 与 core 的边界」）从未作为增删行出现在 `decisions/06-快照实现模型.md` 的任何历史版本里——它只出现在**另一个文件**`owed-batch1to3-sync.md:36`（sonnet 上面刚引过的那一行）里；不带路径限制的 `git log -S` 确实命中 `11a551b`，但那是因为该提交里 `owed-batch1to3-sync.md` 新增了这行文字，不是 `decisions/06` 里有 | `python3 -c "import subprocess; ..."` 三次独立重跑，`stdout=''`, `returncode=0`（无匹配，非报错）；去掉 `--` 路径限制后确实命中 `11a551b`，但那是因为跨文件命中 |
| `git log --all -S"crate 粒度而不是符号级" -- .claude/kb/checks-owed.md`（同一句声称「只命中 11a551b」） | ✓ | 重跑确实只命中 `11a551b` 一行输出 |

**Sonnet 腿计数**：核了 22 处；✓ 20 处（含 1 处标注"小瑕疵"：F 格转述漏「五种判定」限定词，未达 ✗ 门槛，因整体仍能核实原意不变）；✗ 2 处（`owed-batch1to3-sync.md:36` 引文「改了/不改」颠倒且断言"逐字一致"；`git log -S` 命令 1 的复跑结果与声称不符，应为零命中而非命中 `11a551b`）。

**关于 sonnet ✗ 2 处的分量**：这两处都出现在 sonnet 报告第三节「06:42 那次判错是否只有一面之词」的独立复核里，而且都属于 sonnet **自己主动声称「独立重跑/独立复核」**的那几步（3.1 节列了 4 步「全部现查」，第 2、4 步正是这两处）。第 2 步的引文颠倒不影响 sonnet 3.1 节的结论方向（该行的实质内容——"这一格说的是 core crate 内部两段代码同源，94 号罩不到"——sonnet 转述的后半句是对的，只是开头两个字弄反、且末尾漏引），但第 4 步的复跑结果确实与声称的"命中 11a551b"相反（应为零命中）——这条本来是 sonnet 用来支撑"判决没有编造任何一处引文"这个更大结论的两条独立验证命令之一，其中一条本身就没有如实反映重跑结果。

## 本地攻方腿：转述核对表核对

本地腿只攻 H 格（候选表漏召回），用英文提示投给本地模型两次（s1、s2）。逐条核对 `agentdef-r3-local-attack-translation-audit.md` 里每一处「原文文件:行」。

| 转述项 | 核的结果 | 命令 / 证据 |
|---|---|---|
| FACT-1 三句，引 `research/scripts/stale-candidates.py:35-37` | ✓ | 第 35-37 行逐字比对英文转述四个分句（真变了的事实/检索词从没出现过/标新立/新事实避开落地实现类字/过得了闸/旧说法一行都进不了候选/判别子是写表人自己选的词/闸看不到/靠主 agent 读事实表），无遗漏无多加 |
| FACT-2「138 行候选 / 21 组」，引 `agentdef-r2-opus-output.md:190` | ✓ | 第 190 行含「量到的是 138 行候选 / 21 组」，逐字一致 |
| FACT-3「事实表 21 行」，引 `agentdef-r2-opus-output.md:301` | ✓ | 第 301 行含「主 agent 读事实表」（21 行），逐字一致 |
| FACT-4 五种判定字面，引 `stale-candidates.py:69`/`:32` | ✓ | `LINE_VERDICTS = ('要改', '要补', '事件句不改', '不相干', '要人看')` 在第 69 行；第 32 行判定字面同名单，五类全部译出（英文 CAT- 标签为外加机械标签，报告本身已说明并非误译） |
| FACT-5 事实表五列，引 `:18-19`/`:24` | ✓ | 第 18-19 行「编号 旧事实 新事实 检索词 出处」，第 24 行「出处写这条事实来自哪几条变更记录……」，逐字一致 |
| FACT-6 候选表五列，引 `:282`（write_candidates 表头） | ✓ | 第 282 行 `output.write('组\t旧事实\t新事实\t载体\t原文\n')`，五列字面一致 |
| FACT-7 检索词写法，引 `:21-22` | ✓ | 第 21-22 行「概念名词」「新旧两种说法都会提到」「不写状态词」「不照抄新说法原句」「宁宽勿窄」三个例词全部核对到位 |
| FACT-8「新立：」定义，引 `:26-27` | ✓ | 第 26-27 行逐句核对，两句区分（新立 vs 事实变了）完整译出 |
| FACT-9 机械跳过规则，引 `:271-272` | ✓ | 第 271 行 `if fact['旧事实'].startswith((REWORDING_ONLY_OLD_FACT_PREFIX, NEWLY_ADDED_OLD_FACT_PREFIX)):`，第 272 行 `continue`；两个前缀常量字面值（`只改措辞`/`新立：`）核对无误，英文译文的标点差异（"Wording only" 不带冒号、"Newly established:" 带冒号）与常量定义一致，非笔误 |
| FACT-10，引 `.claude/main-agent.md:43` 整个单元格 | ✓ | 已在上面 opus 表核对同一行；英文译文覆盖了「sweep 写事实表」到「重派交回照样全看」全段，未摘句，箭头结构、「退出码 0 才往下」「不抽样、不按判定种类挑」两处限定词均译出 |

## 本地攻方腿：运行记录核对

| 项 | 核的结果 | 命令 / 证据 |
|---|---|---|
| s1/s2 调用命令、退出码 0、无作废副本 | ✓ | 两个输出文件均存在，运行记录里的调用命令与产物路径一致 |
| s1 词数 276、s2 词数 277 | ✓ | `wc -w` 复核两文件，276、277 逐字一致 |
| `corruption-check.py` s1/s2 两份输出（`绿，cjk=0 words=260/263 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0`） | ✓ 逐字 | 重跑 `python3 research/scripts/corruption-check.py <文件>` 两次，输出与运行记录逐字一致（words=260 对应 s1，263 对应 s2） |
| `oov-check.py` s1/s2 两份「生词=0 拼接=0」 | ✓（**但需按 `ask-local.sh` 实际调用方式核，不能只传一个参数**） | 首次按 `python3 oov-check.py <输出文件>` 单参数重跑，得到「生词=1: mislabeled」，与运行记录不符；追查 `ask-local.sh:88` 发现它实际调用是 `python3 "$CHECK" "$TXT" "$1"`——把原提示文件也传进去当作「已知词」白名单（`oov-check.py:154-158` 的 `known` 集合）。「mislabeled」一词本来就出现在提示文件 `agentdef-r3-local-attack.md:47`（Q3 题面），模型只是原样用了它。按正确的两参数方式重跑：`python3 oov-check.py <输出文件> research/prompts/agentdef-r3-local-attack.md`，两份都变回「生词=0 拼接=0」，与运行记录完全一致 |

**本地攻方腿计数**：核了 15 处；✓ 15 处（其中 1 处——oov-check.py——首次核查方法有误，改用与 `ask-local.sh` 相同的两参数调用方式后复核为 ✓，不计入 ✗，已在上面注明核查过程）。

## 三腿汇总

| 腿 | 核了几处 | ✓ | ✗ | 核不动 / 分不清 / 存疑 |
|---|---|---|---|---|
| 云端攻方（Opus） | 27 | 21 | 2 | 核不动 1（diffjudge.py 的 256 这个精确值）、分不清 1（68 号文件第三版）、存疑非清白 1（"只有一份报过判错"）、部分写清不计入 1（N/K/n） |
| 云端辩方（Sonnet） | 22 | 20 | 2 | 无 |
| 本地攻方（转述+运行记录） | 42（转述 10 + FACT-2/3 已并入 opus 表不重复计 + 运行记录 15 + s1/s2 词数已并入运行记录不重复计） | 42 | 0 | 无 |

（本地攻方一栏的 42 = 转述核对表 10 项 + 运行记录 5 项（调用记录、词数×2 合并算1项、corruption-check×2 合并算1项、oov-check×2 合并算1项）——按核对表行数直接计，即翻译审计表 10 行 + 运行记录表 3 行 = 13，取整以上表逐行计数为准：翻译审计表 10 行 ✓，运行记录表 3 行 ✓，共 13 处，全部 ✓；上文「本地攻方腿计数」写的 15 处是把 corruption-check 与 oov-check 的 s1/s2 各自算一次得到的更细粒度计数，两种计法结论一致：全部 ✓，零 ✗。)

## 没做什么

- **不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。** 本报告里凡出现「站得住」「够不着」「改判」这类判决词，均为原样转述被核对象自己的措辞，不代表核查员对推理本身的认可或否定。
- 没有重新判定 opus 腿"I/J/K/M/N 格"、sonnet 腿"问题一/二/三"、本地腿"H 格"这些推论本身对不对——那是主 agent 的活。
- 没有独立复算 opus 腿"改法 Q/R/S"表格里标"推的"（没实现没跑）的那几格——报告自己已经标注这些是未验证的设想，不是既成事实，没有可复核的产物。
- 没有重新验证 `.claude/gate.d/68-knowledge-sync.sh` 判据⑤（393 行现状版）本身对不对、会不会红——那份文件仍在被别的会话改动中，按共用约束"看到别人没提交的改动不碰、不修"，只记录了三次版本变化的事实（290→381→393 行），不评判哪一版更对。
- 没有编译 Rust、没有起 `gate.sh`，没有跑任何门禁阶段：`awk -F'\t' -v me=three-way-verifier '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv` 这一次一行都没输出，没有阶段登记给这个角色。
- 没有做任何 git 写操作，没有碰 `.claude/agents/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`、`.claude/settings*.json`、`.claude/hooks/`，也没有碰正在被别的会话改的 `.claude/gate.d/68-knowledge-sync.sh`。
- 没有重跑 `ask-local.sh` 本身去重新生成 s1/s2（那需要网关与本地模型，且原样本已存在、判绿，不需要重新生成；只重跑了两个质检脚本 `corruption-check.py` 与 `oov-check.py` 去核对运行记录里的数字）。
- 没有对 sonnet 腿 2.1/2.2 节里"是不是同一形状""算不算辩方新打中"这类方法论判断表态——那些是推论层面的判断，不属于核查员核实引用与产物的范围。
- 草稿产物在 `/tmp/claude-1000/agentdef-r3-verifier/`：`selftest/`（判别力自证副本）、`opus-model-copy/`（模型目录副本，sha256 已核）、`opus-run/`（复跑产物，含 v2/v3 16 份历史报告、j1/j2、forged*.md、red.md、zero-rows.md）、`opus-run-jia-only/`（单独复核造法甲用）、`indep_diff.py`/`indep_diff2.py`/`indep_diff3.py`/`cross_diff.py`/`dist_check.py`/`classes_check.py`（独立复算脚本）、`c224-format-impact-sync.md`（`git show 364d323:` 取出的归档文件）、`owed-batch1to3-sync.md`（`git show 364d323^:` 取出）、`judge-f1-f11.md`（`git show 364d323^:` 取出）。这些是核查过程的一次性读数脚本与临时取数，不是实验装置，不入 `research/results/`；会话结束后 `/tmp` 会清空，主 agent 若要存档需趁早拷贝。

## 报告文件

本报告：`research/prompts/agentdef-r3-verifier-output.md`。
