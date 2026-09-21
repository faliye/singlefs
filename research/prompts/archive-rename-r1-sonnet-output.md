# 云端正推腿（Sonnet）交回：archive-rename-r1

分到的格：J3、J4、J6。立场：逐条核第二节代码事实与事故因果；把归档判据在边界上推到底。

## 材料现状核实（先写在前面，因为影响后面每一格怎么读）

`research/prompts/_archive-rename-r1-diff.md` 第 1806-1911 行给的
`research/scripts/archive-past-rounds.py` 那段 diff（blob `93435d9..ed6b610`），
与 `git diff --cached -- research/scripts/archive-past-rounds.py` 现在跑出来的
真实 diff（blob `93435d9..93fb60d`）**不是同一份**：附录里那份是
`gates_losing_input()` / `stale_exclusions()`（按文件名子串扫 `.claude/gate.d/*.sh` 与
`research/scripts/*`）的版本；仓里现在真正暂存的是 `still_an_input()` /
`missing_code_inputs()`（按 `INPUT_TREES` 扫非注释行、`missing_code_inputs` 只认
`include!` 一族）的版本，157 行插入、2 行删除，`git hash-object` 与索引项的 blob
（`93fb60dc20b2c84d302c01ab68372eb1200238e6`）逐字节一致。

`什么现象会推翻它`：若主 agent 能证明 `_archive-rename-r1-diff.md` 生成之后没有人再改过暂存区
（例如给出生成时刻与此后没有 `git add` 的记录），这条就不成立，附录与现状应当一致。
我现在只核到「两者不一致」这个事实，没有核到「谁在什么时刻改的」。

派发提示明确要求 J3 读工作区里现在的 `archive-past-rounds.py`（「现读那一份」），
下面 J3 全部按这份现状分析，不按附录二里那段过时的 diff。

## J3：归档删完之后还有没有别的检查失去输入而没人报

### ① 两个函数的射程之间还有第三类漏掉：`.claude/agents/` 里的字面路径引用，两道自检与门禁 23 号都够不到

`research/scripts/archive-past-rounds.py:29` `SCAN = (".claude/kb", "records", ".claude/rules", "briefs")`
是 `rewrite_links()`（29-33 行；47-82 行是 `still_an_input`，93-121 行是 `rewrite_links`）唯一会去改写引用的目录集合。
`.claude/agents/` **不在这张表里**。

实测：`.claude/agents/three-way-materials.md:20` 与 `:28` 两行都写着
`` `research/prompts/_m2-code-r1-diff.md` ``（反引号里的完整路径，「形态照」example）。

```
git log --oneline --all -- "research/prompts/_m2-code-r1-diff.md" | tail -3
9de47f9 收尾：提交 crates/ 之外的全部改动（规范、kb、门禁、三方论证材料）
3cff909 上一轮及更早的实验记录归档进版本库；系统配置拆四类可改性；术语改名做成会红的门禁
```

这份文件恰恰是在**这一轮判的这次归档提交（3cff909）**里被删掉的（`ls research/prompts/_m2-code-r1-diff.md`
现在报 `No such file or directory`）。三道防线依次核过，全部够不到它：

1. `still_an_input()`（47-82 行）：`.claude/agents` 不在 `INPUT_TREES`（32 行：
   `(".claude/gate.d", ".claude/scripts", "research", "crates")`），且 agent 定义本来就不该被当成
   「代码输入」保护——它不是编译期读取，保护它没有意义，这一步不该也没有报。
2. `rewrite_links()`：`.claude/agents` 不在 `SCAN`，所以这个引用没有被改写成
   `.claude/agent-common.md:24` 说的「只写文件名、不写路径」那种形态——它现在还是一条
   **看起来完整、其实指空的路径**，跟已归档文件的正常形态（裸文件名）不一样，读的人/ 子 agent
   如果真去 `Read` 这条路径会直接扑空，没有任何信号提示「这是正常的、已归档」。
3. 门禁 23 号（`.claude/gate.d/lib-link-targets.py`）：archive-past-rounds.py 自己的成功提示
   （217-221 行）教用户「下一步：跑门禁 23 号（文档指向）确认没有指空的链接」，但 23 号只判
   markdown 链接语法 `[text](path)`（`lib-link-targets.py:80` 的正则
   `r'\[([^\]]*)\]\(([^)\s]+)\)'`），而且 `mask_code()`（54-61 行）在扫之前先把反引号包住的片段
   整段涂空——这条引用是反引号形态，不是 `[]()` 链接，**23 号连看都看不到它**。

三层防线各自的设计边界都各自成立（不当输入不该保护、agents 不是「记录」类目、23 号只管 markdown
链接），但叠在一起就形成了一条没人管的缝：**agent 定义文件里用反引号写的、指向具体已归档文件全路径的
「形态照」引用**，删除之后既不报错也不改写，静静地变成一条指空的路径。

`什么观测会让它触发（已经触发）`：任何 subagent 或人照着 `three-way-materials.md:20/28` 的指引去
`Read research/prompts/_m2-code-r1-diff.md`，会拿到「文件不存在」，而不是「这是已归档材料，去
git 历史查」的提示——这正是已经发生的状态，我在上面用 `ls` 与 `git log` 现查过。

除此之外没有找到第三类：
- `build.rs`：`find . -name "build.rs" -not -path "*/target/*"` 零命中，仓里没有这类文件。
- 变异表的「原文」列：`crates/mutations.tsv` 与 `research/mutations/*.tsv` 的原文列逐一核过，
  指的都是 `.rs` 源码片段（`awk -F'\t' '!/^#/ {print $2}' … | grep -v '\.rs$'` 之外的行都是数值/表达式，
  不是 `research/results` 或 `research/prompts` 路径），`grep -rl "research/results\|research/prompts"
  research/mutations/ crates/mutations.tsv` 零命中。
- shell 把产物 `cat` 出来比：`research/scripts/replay.sh:39` 的 `TABLE` heredoc 里逐行写着字面文件名
  （如 106 行 `e100-system-configuration-slot-2026-09-03.out`），这些是 `.sh` 文件的非注释文本，
  `still_an_input()` 确实抓得到——实测跑一遍 `still_an_input(root, 现有的229个候选文件)`，命中 13 个，
  其中 9 个正是 `replay.sh` 的 `TABLE` 行、4 个是 `include_str!` 链（e109/e133/e134/e136），
  没有漏项。

`站得住 / 站不住`：这一条判「站不住」——两个函数各自的射程都有清楚边界，但没人把
`.claude/agents/` 这类「给 agent 读的、里面混着具体归档文件路径」的目录纳入 `SCAN`，
导致这唯一一处真实存在的字面路径引用漏在外面。

### ② 收窄到 `include` 一族的理由，站得住

`missing_code_inputs()` 的注释（132-135 行）说：按「非注释行出现过这个名字」扫，
「49 处命中里只有 0 处是真的」，会命中 fixtures 假路径、模板占位串
`eNN-xxx-YYYY-MM-DD.out`、`println!` 里的复跑说明。我不能重跑出那个「49」这个具体数字
——那次扫的是已经删掉的历史候选集，现在的环境里已经没有那批文件名可供复现——但三类假阳性
逐一在仓里独立核实存在：

1. 模板占位串：`.claude/gate.d/85-repro-command.sh:29` 原样写着
   `echo "    原始输出 \`research/results/eNN-xxx-YYYY-MM-DD.out\`。"`，这是 `.sh` 文件非注释行，
   若用「名字出现即算命中」的宽口径，任何形如 `eNN-xxx-YYYY-MM-DD.out` 的候选名都不会真的等于
   这条占位串，但如果宽口径是拿真实候选名去反过来扫全仓文本、而这条占位串本身长得像一个产物名，
   两种扫法的方向不同——这里我只核实了这条占位串**存在**，没有反推出它在哪种具体扫法下会造成
   误报，这一条按「转述」记，不算坐实。
2. `println!` 复跑说明：`research/e7-index-bench/src/bin/e131_livelist_carrier.rs:136`
   （非注释、真代码）`println!("判据写死在 research/prompts/e131-preregistration.md（写于本装置之前）");`
   —— `research/prompts/e131-preregistration.md` 这份文件本身已经被这次归档删掉
   （`ls` 报不存在），但这行 `println!` 只是在**运行时打印**这句话，不读那个文件，删掉它
   完全不影响编译或运行。实测：
   ```
   still_an_input(root, ["research/prompts/e131-preregistration.md"])
   -> {'research/prompts/e131-preregistration.md': ['research/e7-index-bench/src/bin/e131_livelist_carrier.rs']}
   ```
   `still_an_input()` 会把它算成「被当输入」（因为它设计上宁可多留、不许错删，这一步不是 bug，
   是有意选择的安全方向）；但如果 `missing_code_inputs()` 也用同样宽的口径去判「删完之后代码点没点名
   一个不存在的产物」，这一行会被误报成「crate 编不过」，而实际上 `cargo build` 不会因为这一行报错
   ——这正是注释里说的那类假阳性，且是用真实存在的代码验出来的，不是虚构。
3. fixtures 假路径：`.claude/gate.d/fixtures/69-evidence-in-repo.sh/red/setup.sh`、
   `.claude/gate.d/fixtures/85-repro-command.sh/red/.claude/kb/experiments/99-测试样本.md` 等
   文件里确实放着形如产物路径的假数据，专门用来让门禁自证判红；`still_an_input()` 与
   `missing_code_inputs()` 都显式跳过路径含 `fixtures` 的目录（54、61、140 行的
   `if "fixtures" in directory.split(os.sep): continue`），这条本身已经被设计规避，
   跳过逻辑现读代码确认存在。

三类假阳性来源两类已用真实代码坐实（模板占位串、`println!` 复跑说明），一类（fixtures）确认
存在且已被显式排除。收窄到 `include`/`include_str!`/`include_bytes!` 这一族的理由——
「这些形态的脚本读不到文件时自己会失败（运行期），编译期输入不会（build 期就塌，且没有中间
可观察的失败点）」——在此站得住：`include!` 一族命中的都是 **编译期硬绑定**，其余形态命中的都是
**运行期字符串**，两者对「crate 编不过」这个具体判据的因果链完全不同，窄而准的取舍是合理的。

## J4：「删上一轮、留本轮」这条判据在边界上怎么样

`past_round_files()`（41-44 行）：
```
tracked = git ls-files DIRS
dirty   = git diff --name-only HEAD -- DIRS
doomed  = tracked − dirty − KEEP
```

在 `/tmp/claude-1000/archive-rename-r1-sonnet/j4test` 建了一个真实 git 仓逐条测：

| 边界情形 | 实测结果 | 判定 |
|---|---|---|
| `git mv` 改名（新名字这一轮才起） | `git ls-files` 只报新名；`git diff --name-only HEAD` 也只报新名（新名对 HEAD 是「新增」）⇒ `dirty` 含新名 ⇒ 不进 `doomed`，**留** | 站得住：`git diff HEAD`（不带 `--cached`）比较的是「工作树 vs HEAD」，改名之后的新路径在 HEAD 里不存在，天然算「dirty」 |
| 部分暂存（`git add -p` 只暂存一半改动，工作区还有未暂存的差异，status 显示 `MM`） | `git diff --name-only HEAD -- DIRS` 照样报出该文件（这条 diff 不区分暂存与未暂存，两者只要有一个与 HEAD 不同就算 dirty）⇒ **留** | 站得住 |
| 软链接（`research/results/` 下的符号链接，指向仓外，且自 HEAD 起未改） | `git ls-files -s` 报 `120000` 模式；不在 `dirty` 里 ⇒ 判 `doomed`；`os.remove()` 对符号链接本身生效，只删链接不删目标，无异常 | 站得住，行为与普通文件一致，没有特殊风险 |
| 子模块 / gitlink（`research/results/` 下挂了一个 `160000` 模式的 gitlink，指向内部另一个 git 仓，自 HEAD 起未改） | `git ls-files -s` 报 `160000`；不在 `dirty` 里 ⇒ 判 `doomed`；`os.remove()` 对着一个**磁盘上是目录**的路径调用，直接抛 `IsADirectoryError`，`run()`（177-222 行）里 `for relative in doomed: os.remove(...)` 这段循环没有 try/except，整个 `--apply` 未捕获异常中止 | **站不住**：该删的没删完，而且崩溃发生在 `os.remove` 循环内部，`stale_exclusions()`、`missing_code_inputs()`（206-207 行，在 remove 循环之后才跑）**全部不会执行**——不是「多删或少删一个」，是这一次 `--apply` 直接以未捕获异常收场，之前已经跑完的 `rewrite_links()` 造成的文本改写留在半提交状态 |

gitlink 场景是在临时仓里现造现测的：

```
git update-index --add --cacheinfo 160000,<sha>,research/results/nested-repo
git commit -qm "add gitlink round"
git diff --name-only HEAD -- research/results   # 空，未改
python3 -c "import os; os.remove('research/results/nested-repo')"
# IsADirectoryError: [Errno 21] Is a directory: 'research/results/nested-repo'
```

仓里现在没有子模块（`.gitmodules` 不存在，`git ls-files -s research/results research/prompts`
现查全部是 `100644`/`100755`），所以这是**构造出来的边界，不是仓里正在发生的事故**；
但按 J4 的要求「构造一个状态：该删而不该删，或该留而被删」，这个构造确实成立：
它不是「误删」而是「该删的因为撞上一个未处理的异常类型而删不干净、还带崩连带效果」。

`什么观测会让它触发`：`research/results/` 或 `research/prompts/` 下出现任何一个
`160000` 模式（gitlink）条目、且自 HEAD 起未修改，`--apply` 就会在那一项上抛
`IsADirectoryError` 中止。

`什么现象会推翻这条`：若 `run()` 的删除循环外面套了 try/except 并优雅跳过目录类型的条目
（我读到的 177-222 行没有），这条就不成立；我只核了现在这一份代码。

## J6：门禁 88 号收成「按本次改动判」之后，漏判了什么

现读 `.claude/gate.d/88-quoted-result-lines.sh`。判据在 4-22 行注释里写死，
第 4 条（19-22 行）「只判这次改动新增或改写的行……拿不到 diff 基准时退回全量判」是
2026-09-21 这一次收窄的核心。机制（34-92 行 python）：

- `body`（42-43 行）＝ kb 正文里 `\n## 历史版本` **之前**的部分，`quoted`（44-47 行）
  只从这个 `body` 里收集整行 `E7RESULT ` 开头的行。
- `BASE`（28-32 行）：`GATE_BASE` 环境变量优先；没设就依次尝试
  `refs/singlefs/gate-ok`、`@{upstream}`。
- 有 `BASE` 时（50-69 行）：对每份 kb 文件跑 `git diff --unified=0 $BASE -- $path`，
  把 diff 里以 `+` 开头、内容是 `E7RESULT ...` 的行收进 `added` 集合（按
  `(path, 内容)` 配对，不按行号），再把 `quoted` 过滤成只剩 `(path, 内容)`
  在 `added` 里的那些。

在 `/tmp/claude-1000/archive-rename-r1-sonnet/j6test` 建仓复现了两种漏判，都是真的跑出来的：

**漏判一：「改的是历史节里的行」**——即便 diff 基准已经推到最新（不存在「跨基准」问题），
只要错误的 `E7RESULT` 行写进了 `## 历史版本` 一节（哪怕是**这次改动当天新写的**历史条目），
它就完全不出现在 `quoted` 里，88 号直接报「无对象可判」（退出码 77）：

```
cat > .claude/kb/page.md <<'EOF5'
# 页

正文

## 历史版本

### 2026-09-21
- 这次改动新写的历史条目，里面抄错了一行：
E7RESULT freshly_wrong=2026
EOF5
git commit -qm "在历史版本一节里新写一行错的产物引用"
bash .claude/gate.d/88-quoted-result-lines.sh "$(pwd)"
# ! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判
# exit=77
```

88 号注释里第 3 条「历史节里的旧行不判」的本意是保护**旧**产物已归档的**旧**记录，
但代码判据只看「在不在 `## 历史版本` 之后」，不看这行是不是**这次改动才写的**——
第 3 条与第 4 条叠在一起，第 4 条本该新增的判力被第 3 条无条件排除历史节直接抹掉了。

**漏判二：「改动跨了 diff 基准」**——`refs/singlefs/gate-ok` 会被 `gate-triage`
在共享 `gate.sh` 全绿时前移（`.claude/agents/gate-triage.md:31`：
「共享 `gate.sh` 全绿时 `update-ref refs/singlefs/gate-ok`」）。这个 ref 前移
与「这一行 E7RESULT 有没有被 88 号真正核对过」是两件不相关的事——`gate.sh`
可能因为 88 号那一次跑刚好取不到 diff 基准而全量判过、或 88 号那一次刚好因为
其他原因判过，也可能因为别的阶段不相关而整体判绿，`gate-ok` 就会前移到那个提交，
从此那个提交里已经存在的任何一行 `E7RESULT`（不管当时有没有真被验证过内容）都从
「新增或改写」的判定范围里永久掉出去。实测：

```
round0（gate-ok 指到这里）
  → commitA：写下 E7RESULT wrong_value=999（产物里没有这一行）
  → 模拟 gate-triage 把 gate-ok 前移到 commitA
  → commitB（同一轮继续）：再写 E7RESULT another_wrong=1000

bash .claude/gate.d/88-quoted-result-lines.sh "$(pwd)"
# ✗ 这次改动新增或改写的 kb 正文里有 1 行整行抄的产物行 …一份都找不到：
#      .claude/kb/page.md:5  E7RESULT another_wrong=1000
```

只报出了 `commitB` 那一行，`commitA` 里同样错误的 `wrong_value=999` 完全没有出现在
报告里——不是因为它被验证过并通过，是因为 `gate-ok` 已经越过了写下它的那个提交，
`git diff --unified=0 $BASE` 从这一点看不到它是「新增」。这与「一轮的提交不止一次」
这个现实直接冲突：只要中途有一次 `gate.sh` 全绿（不管是不是因为 88 号那次刚好没测到
这一行），这一行就再也进不了 88 号的判定范围，永久归类成「历史参考」，即使它从来没被
真正核对过内容。

`什么观测会让这两条触发（都已经在上面用真实命令触发过）`：
① 任何一个 `E7RESULT` 错行被写进当天新增的 `## 历史版本` 小节；
② 任何一轮的提交跨越了一次 `refs/singlefs/gate-ok` 的前移。

`什么现象会推翻它们`：
① 若把「历史节里的行不判」改成「历史节里**这次改动之前**就存在的行不判、这次改动新写进历史节的行照判」，
   这条漏判就会消失——但我读到的 42-47 行没有做这个区分。
② 若 `gate-ok` 只在**整轮收尾**（而不是任何一次全绿）时才前移，或者 88 号的
   `BASE` 改成锚定「这一轮真正的起点」（例如这一轮的第一个 commit，而不是
   随时可能前移的共享 ref），这条漏判也会消失——我读到的 28-32 行目前两者都没做。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J3① | 打中，找到第二处 | `.claude/agents/three-way-materials.md:20/28` 反引号里写着已归档文件 `research/prompts/_m2-code-r1-diff.md` 的全路径，`still_an_input`/`rewrite_links`/门禁 23 号三层都够不到（agents 不在 `SCAN`，23 号只判 `[]()` 链接且反引号会被 `mask_code` 涂掉），这条现在就是一个指空的路径 |
| J3② | 站得住 | `missing_code_inputs()` 收窄到 `include!` 一族的理由——运行期字符串（`println!`、模板占位串）与编译期硬绑定是两类不同的因果链——用仓里真实存在的 `e131_livelist_carrier.rs:136` 与 `85-repro-command.sh:29` 验证成立；「49 处命中」这个具体数字核不动 |
| J4 | 站不住（构造边界） | `git mv`、部分暂存两种边界情形实测都被现有逻辑正确处理；软链接无异常；`research/results/`（或 `prompts/`）下出现未改动的 gitlink（`160000`）会让 `--apply` 在 `os.remove()` 上抛未捕获的 `IsADirectoryError` 中止，仓里现在没有子模块，这是构造出来的边界而非现存事故 |
| J6 | 打中两处 | 「历史节里的旧行不判」不区分新旧，把当天新写进历史节的错行也一起排除在外；`refs/singlefs/gate-ok` 会在任意一次 `gate.sh` 全绿时前移，导致同一轮内、前移点之前写下的错行永久脱离「新增或改写」的判定范围 |

## 没做什么

- 不判 J1、J2、J5、J7、J8（分给别的腿）。
- 没有重跑 `gate.sh` 全量、没有编译 `crates/` 或 `research/`（本次改动全是字符串字面量与脚本，
  与我分到的格无关，没有必要）。
- J3①「49 处命中」这个具体数字核不动——当时扫的是已经删掉的历史候选集，现在的仓里已经没有
  那批文件可供复现，只独立核实了三类假阳性来源里的两类（模板占位串、`println!` 复跑说明）
  确实存在，第三类（fixtures 假路径）确认存在且已被两个函数显式排除。
- J4 的 `git mv`、部分暂存、软链接、gitlink 四种边界都在 `/tmp/claude-1000/archive-rename-r1-sonnet/j4test`
  的临时仓里现造现测，没有改动真实项目仓的任何文件；这个临时仓与 `/tmp/claude-1000/archive-rename-r1-sonnet/j6test`
  测完后留在原地，没有清理（草稿目录，按定义不影响仓）。
- 没有核实"两个函数当前射程"之外，`.claude/agents/` 这条缝是否是唯一的第三类；只穷举了任务列出的
  四个候选形态（shell cat、python 读、build.rs、变异表原文列）并逐一给出结论，没有对整个仓做更大范围的
  穷举式扫描。
