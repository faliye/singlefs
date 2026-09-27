# sync-local-legs-r1 附录二：被判两份定义相对 HEAD 的 diff（2026-09-27 现取）

```diff
diff --git a/.claude/agents/three-way-local-attack.md b/.claude/agents/three-way-local-attack.md
index e412f70e..37686dbb 100644
--- a/.claude/agents/three-way-local-attack.md
+++ b/.claude/agents/three-way-local-attack.md
@@ -26,9 +26,9 @@ required-inputs: 禁读清单, 草稿目录, 提示文件
 
 1. 把攻击面写成英文提示：自足、不用任何 markdown 强调、答案按编号、每条答复写「什么现象会推翻它」。能落成数的问题给写死的事实表（数整行抄，核对表里写来源文件:行），要模型按表格逐格填，不许只答 yes / no。提示里明令答复不写代码行号与文件行号（按函数名、表格行号指），核对表与运行记录里把样本自带的行号一律标「模型自给、未核」。
 2. 逐句核转述：每一句英文与原文并排，缺一个限定词就补上；英文比原文多出来的限定词、括注也单列一行，写明为什么加。核对表写进 `research/prompts/<轮>-local-attack-translation-audit.md`（英文项 / 原文文件:行 / 首稿缺的 / 定稿）。
-3. 前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`。
-4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，沿用同一个号重跑；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5、或网关不通：停下，报「本地腿缺席」。
-5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本>` 看它列出的生词：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
+3. 取号：`ls <前缀>-output-s*.md` 看已用到几号，取下一个没用过的 `<n>`（重定向到一个已有样本的号会把它整份盖掉）。前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`；单次请求最长 `ASK_LOCAL_TIMEOUT`（默认 900 秒）超过 Bash 前台上限时，改用 Bash 的 `run_in_background` 起同一条命令，起完结束本轮等完成通知，退出码取通知里的。
+4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，先确认它大小为 0，再用 `>|` 重定向到同一个号重跑（`>|` 在开没开 `noclobber` 时都写得进）；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5（包括 6：损坏检测器没跑成或找不到，这一份没验过）、或网关不通：停下，报「本地腿缺席」。
+5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本> <提示文件>`（带上提示文件，提示里的专名才不算生词）看它列出的生词，生词表只打前 300 个字符，打满了就不以它为全、逐段通读样本：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
    5b. 提示里有事实表题的：主 agent 另给一份算好的答案表的路径（放在草稿目录之外、提示里不引，本地模型读不到）；收回样本后用一段 python 逐格比数，只打印对不上的格号、不打印答案，不一致的格在运行记录里标「算术对不上」；主 agent 没给答案表的，运行记录写「没有答案表，算术没比」。
 6. 攒到至少两份干净样本为止；连续五次调用（判红作废的也算）拿不到两份干净的，停下照实报。
 
diff --git a/.claude/agents/three-way-local-defense.md b/.claude/agents/three-way-local-defense.md
index b0fbc8fb..788aa845 100644
--- a/.claude/agents/three-way-local-defense.md
+++ b/.claude/agents/three-way-local-defense.md
@@ -10,7 +10,7 @@ required-inputs: 禁读清单, 草稿目录, 提示文件
 
 # 本地辩方腿（three-way-local-defense）
 
-开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入」「做什么」「写范围」「产出」四节：做法与它逐条相同，只有下面几处不同。
+开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入（主 agent 必须给）」「做什么」「写范围」「产出」四节：做法与它逐条相同，只有下面几处不同。
 开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节。
 
 ## 与本地攻方不同的地方
```
