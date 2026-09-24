# gate-fix-forks-r3 云端攻方（Opus）：T1、T5、T6、T7

<!-- doc-lint:not-numbers T1 T3 T4 T5 T6 T7 T8 T11 M0 M1 M2 M3 M4 M5 M6 M7 M8 N1 N2 N3 N4 N5 N6 N7 N8 N9 N10 -->

写于 2026-09-24 02:25 JST（2026-09-23 17:25 UTC）。只攻正文第四节分给这条腿的四格（T1、T5、T6、T7），攻的是改法自己引入的放过与误拦；T3、T4、T8 与第二轮判决的数没碰。被判的代码一律按快照树 `/tmp/claude-1000/gate-fix-forks-r3-snapshot-tree/` 读（105 份，开工时对 `research/prompts/gate-fix-forks-r3-snapshot/sha256sums.txt` 全部 OK），行号取自快照树。

## 复跑

模型全在 `research/prompts/gate-fix-forks-r3-opus-model/`。被判的阶段、共用脚本、52 号的脚本与样本取快照树；真仓只用来 `git show HEAD:` 取改动之前那一版作对照、读 git 历史与 kb 现状（只读）。现场都建在第三个参数给的目录下的临时 git 仓里（`GIT_CONFIG_GLOBAL=/dev/null`、`GIT_CONFIG_NOSYSTEM=1`，与本机 git 配置隔开）。在仓根上跑：

```
M=research/prompts/gate-fix-forks-r3-opus-model; S=/tmp/claude-1000/gate-fix-forks-r3-snapshot-tree; R=/tmp/claude-1000/gate-fix-forks-r3-opus/run
nice -n 19 python3 $M/t1-titles.py      .              # T1：真仓实验页标题按 75 号两个判法归类
nice -n 19 python3 $M/t1-mention-forms.py .            # T1-b：真仓决策里 doc-lint 认、75 号不认的实验号写法
nice -n 19 bash    $M/t1-scenes.sh      $S . $R/t1     # T1：⑤ 后一半与 ⑨ 的放过、误拦（F 那一族归 T5）
nice -n 19 python3 $M/t5-moved-pages.py .              # T5：真仓哪几页一搬家 88 / 40 号就判到对不上的旧行
nice -n 19 bash    $M/t5-states.sh      $S . $R/t5s    # T5：七处判退出码之后，五种仓状态
nice -n 19 bash    $M/t5-config-amend.sh $S . $R/t5c   # T5：61 号在 color / 外部 diff 配置下；40、88 号与 git commit --amend
nice -n 19 bash    $M/t5-64.sh          $S $R/t5-64    # T5：64 号三条判据在以后写法上的漏判与误拦
nice -n 19 bash    $M/t6-mutants.sh     $S . $R/t6     # T6：check-segment-registry.py 的十个新变异
nice -n 19 bash    $M/t6-fixture-v2.sh  $S . $R/t6v2   # T6 改法：十行加两句提示的红样本，再跑一遍变异
nice -n 19 bash    $M/t7-invocations.sh $S . $R/t7     # T7：九种调用方式
nice -n 19 bash    $M/t-fix.sh          $S . $R/fix    # 本腿提的改法打在快照树的副本上，重跑 t1-scenes、t5-config-amend、t5-states 与样本自检
```

各 `.out` 是照上面命令跑的原样输出（2026-09-23 17:22–17:25 UTC）。sha256：

```
d7d168153baacd8f53d89da481cb29c1fdf05e8cb97611eec615993136971b78  t1-scenes.out
33d6c5b23b7843a94d3a27ab6b943280662c414f928cb85b76651a5e4c4baa49  t1-scenes.sh
b9487cff5f42aaed4d09cedc242780b8cadfa1e9295cfaa08582adb2edeea83f  t1-mention-forms.out
431b7ea46484ed89c34f3868cf3435fd15a99bc1dc80dae930a3e7fbc1fcffea  t1-mention-forms.py
923df9296fd650d759d63845ade440e200a4c60d399b16b1be455ac42eb71f9b  t1-titles.out
2036220a7852c0ffedb09a659317050dd805c3d075c585cc0447897a86c8eea9  t1-titles.py
7cecf97901d00a3a0b8b4cf2554ff41694b3d9c869bc8947e1ab11896148b05c  t5-64.out
17ca933d2566e6e1fb0350767f79f5c276d7b46aceeaf27cc77ca9f3f55bfcb7  t5-64.sh
0be3896476e4bfd98cad3dc4a740d5847945c777d8f319e21e7898da203cfcb5  t5-config-amend.out
a9c803e2bc86de1f99860ca6cf8df211f9c4c6f1465ee9e4b8dc02e713153d2f  t5-config-amend.sh
f26dc162fb1c6cb6b351fecc83b730cdd838d8bcb15eb3587e7440baf2abae8d  t5-moved-pages.out
33c8fa2465cab5a1c8a771a99bd9c00336e09726764d1eae37c5b4da5018b1fd  t5-moved-pages.py
73a9c91eb8304ea76a92e75bda362119263d7eda74fd921e9eda5bf36abdd011  t5-states.out
9b4a8dd78f69b024c58b910d0eb6637b2c24854be066211590499835a0a6c157  t5-states.sh
c8a21b9b50ad1eea1463ec50ea84c1fd272846a23de67b1a1105b85fa7705f9b  t6-fixture-v2.out
2bd92bda9892e4781b8433ab3c698be6afe7aef6074084c26e0abef77940a317  t6-fixture-v2.sh
b3b7388daec5eb963a122952a96d5efa5762280d4211aa9ba7e5f29045572ce0  t6-mutants.out
6533e8d7dec90965af44154ea9478903d1c9b63a4e2fd53b211f72af81079e2c  t6-mutants.sh
c41be95c11f46320dfd9bdf9b15d15a5aae3b2e5036e64f974de394151650411  t7-invocations.out
7307c8e4bff287e30557c5b0740d2a2df8c603a299c65f2b7352d7a11ebaa5ed  t7-invocations.sh
7c63fb727dd96cab85c868b4495a19b63bc4ac59b63c85812bd1c15a0d88bf02  t-fix.out
d8c5e38925c04f3754b9c32d4741ae6a6b5b65e749f0877e9cd766d0174a8e40  t-fix.sh
```

**快照漂移**：收工时（17:25 UTC）主树对快照清单 `sha256sum -c` 有 3 份 FAILED：`.claude/rules/implementation-workflow.md`、`.claude/scripts/gen-decision-items.py`、`research/scripts/replay.sh`。模型没读主树的这三份：t6 的 `replay.sh` 与 E142 产物名取快照树（`from()` 先找快照），t7 用的生成器是快照树的拷贝；本报告引的行号也都取快照树。t6 从主树拷的 E142 产物与第八节点名的 4 份 `.rs` 不在快照清单里（别的会话正在改 `crates/`），它们在跑那一刻的 sha256 印在 `t6-mutants.out` 开头。

## 各格判定一览

判据字面用正文 `research/prompts/_gate-fix-forks-r3-body.md:79`「- 一个**具体可复现**的失效（给得出文件、提交或输入写法，现跑确实如此）⇒ 打中。」；「消不掉」用 `:13` 的共用问句。这一轮每格只有一个被判形态，「分辨臂」一栏写的是它分不分得开「改法之前（HEAD）/ 改法之后（快照）」。

| 格 | 被判的东西 | 判定 | 打中的东西（一句） | 改法前后分得开 | 本腿改法（零轮） |
|---|---|---|---|---|---|
| T1-a | 75 ⑤ 后一半只认状态段恰好「已跑」 | **打中（放过）** | 未跑改成「已跑两轮」「已测」「首轮已跑」「**已跑**」「已跑 (…)」「已跑：…」「已跑，…」「已跑。」，欠账块等它的决策没回看，75 退 0（t1-scenes A）；真仓今天就有 8 页用这类状态（E80「已跑两轮」、E21「三段全部已测」等），40 号把它们认作跑过 | 否（两版都放过；是新形态的射程） | 推的：与 40 号共用一个「跑过了」的谓词 |
| T1-b | 75 ⑤ 后一半只认 `E<号>（` | **打中（放过）** | 决策写 `**E2**（…）`、`` `E2`（…） ``、`E2 （…）`、`E2(…)`（doc-lint G 都认）时，E2 改成已跑、决策没回看，75 退 0（B） | 否 | **量过**：另起一条认这四种写法的正则，四格转红、基线仍绿 |
| T1-c | 「决策文件在这一批里」就算回看了 | **打中（放过）** | 同一批里 D1 只有一处与 E2 无关的改动（术语改名这类），75 退 0（C1）；别的会话在工作区改了 D1 没暂存，直接跑退 0、`--staged` 退 1（C2） | 否 | **量过**：收成「这一批在那份决策里新增或改写的行点到了 E<号>」，C1、C2 转红 |
| T1-d | 同上，基准那一版的标题按路径取 | **打中（误拦，消得掉）** | 早就已跑的 E5 只是 `git mv` / `mv` 换了文件名，75 报「这次改动把 E5 改成已跑」退 1；HEAD 版 75 退 0（D1、D2）。补一行表才消（D3） | **是**（新红） | **量过**：基准里这个路径没有页时按实验号找原来那页的标题，D1、D2 转绿 |
| T1-e | ⑤ 后一半只在「改成已跑」那一刻触发 | **打中（放过，射程）** | 已跑的 E5 重跑、结论翻了，只回看表里 D3，欠账块等 E5 的 D1 没动，75 退 0（D4） | 否 | 推的 |
| T1-f | 10 号撤掉、并进 ⑨ | **打中（放过）** | 标题整行里有「退役」「作废」就不判 ⑨：「E9 退役盘的重建代价 —— 已跑」没有对应决策，75 退 0、HEAD 10 号第 4 段红（E）；真仓 E43、E56、E103、E104 四页结论照样成立而被豁免 | **是**（HEAD 10 红、快照 75 绿） | 推的 |
| T5-a | `--no-renames` 让搬家整页算新增 | **站住** | 真仓 204 份正文 616 行 E7RESULT、256 份被点名产物，搬家后多判的行里对不上的 0 行 | — | — |
| T5-b | 40、88「HEAD 里还在、工作区删了就算找得到」 | **打中（窄，放过）** | 误删的产物经 `git commit --amend` 从所有提交里抹掉：amend 之前 40、88 快照版退 0（HEAD 版都退 1），amend 之后两道退 1（H）；不 amend、另起一次提交则 git 历史里找得到、两道绿（H2） | **是** | 推的 |
| T5-c | 64 号三条判据 | **打中（漏判 9 种、误拦 5 种）** | 漏：`os.getenv('GATE_BASE')`、写死 `origin/master`、`HEAD^`、`HEAD~`、`'HEAD' + '~1'`、`git status --porcelain`、`--numstat`、`\|\| true`、同一行的 `\|\|` 属于后一条命令；拦：折行后 `\|\|` 在下一行、下一行取 `$?`、`set -e`、python 多行文档字符串、heredoc 用法说明（t5-64）。今天的 82 份阶段 64 号判绿 | 否 | 推的 |
| T5-d | 七处判共用取法的退出码 | **打中（窄，消不掉的红）** | 刚 `git init` 还没提交、或切到 orphan 分支（HEAD 未出生）：11、56、68、69、97 与 61 号报「取不到这次改动碰了哪些路径」退 1，只有提交一次才消；HEAD 版 69、97、61 退 0、56 退 77；75 号先判 HEAD 在不在、不红。浅克隆、`--staged` 临时 worktree 不红 | **是** | **量过**：HEAD 未出生时基准取空树，六道都不再报取不到 |
| T5-e | 61 号定案判定改用共用脚本、碰没碰仍自己 `git diff`；75 号自己的新增行函数 | **打中（61 误拦、消不掉；75 放过）** | 使用者配了 `color.ui=always` 或 `diff.external`：61 号每条未定项都报「一个字都没动」退 1（HEAD 版退 0）（G）；75 号 ⑤ 整段失效、该红的批退 0（F） | 61 **是**；75 否（HEAD 版同样放过） | **量过**：两处 `git diff` 带 `--no-color --no-ext-diff`，G 转绿、F 转红 |
| T6 | 52 号五行红样本 | **打中** | 十个新变异（N1–N10）各让仓里的脚本在一处真输入的错上判不出（原 52 号退 1、变异后退 0），47 号 `--selftest` 退 0、五行红样本判对（t6-mutants） | 否（样本内容，不是位置） | **量过**：红样本扩成十行加两句提示，N1–N9 与第二轮的 M1 都抓到；N10 抓不到 |
| T7 | 31、52 号在 cd 之前取位置 | **站住** | 九种调用方式里没有一种悄悄用了别处的脚本或生成器：符号链接、`source`、标准输入、带相对路径的 `CDPATH` 四种都退 1 并点名找不到的路径；HEAD 版这四种退 77 | — | — |

## T1　75 号 ⑤ 后一半、10 号并进 ⑨

### 被判的原文（快照树，行号现查）

- `.claude/gate.d/75-decision-experiment-links.sh:17` `#      它指的决策文件要在这次改动里。这次改动把实验的标题状态改成「已跑」（状态段恰好是这两个字）时，`
- `.claude/gate.d/75-decision-experiment-links.sh:18` ``#      决策正文（历史版本之前）用 `E<号>（` 引了它的每条决策都要回看：决策文件在这次改动里，或者表里那条决策的一行改过。``
- `.claude/gate.d/75-decision-experiment-links.sh:389` `    return bool(status) and re.sub(r'（.*$', '', status.group(1)).strip() == '已跑'`
- `.claude/gate.d/75-decision-experiment-links.sh:68` `E_REF = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')`
- `.claude/gate.d/75-decision-experiment-links.sh:444` `        if body_changed and ran_status(title_line) and not ran_status(base_title(rel)):`
- `.claude/gate.d/75-decision-experiment-links.sh:394` `    shown = git('show', f'{base}:{rel}')`
- `.claude/gate.d/75-decision-experiment-links.sh:450` `                if info['path'] not in changed and decision not in reviewed:`
- `.claude/gate.d/75-decision-experiment-links.sh:26` `#      标题状态里写着「作废」或「退役」的实验不判（它们留着是为了记下那条路不通）。`
- `.claude/gate.d/75-decision-experiment-links.sh:287` `    voided = re.search(r'作废|退役', head_line) is not None`

### 基线世界（t1-scenes.sh）

D1（已定，已瘦身）已定项 1 的「**依据**」引 E1，「**欠**」块写「等 E2（样本实验二） 跑完再核射程；E5（样本实验五） 的结论也在这里等着复核。」；D3 已定项 1 写「无实验：…」。E1 已跑、表里 D1 支撑；E2 未跑、表里只有 D3 备料；E5 已跑、表里只有 D3 备料。基线提交，快照 75 退 0。每一格从这份基线拷一份、做一批改动再跑。

### T1-a　状态写法（A）

批：E2 标题状态从「未跑」改成下表的写法，表里只把 D3 那一行的回看日期改成当天；D1 一个字不动。

| E2 的状态段 | 快照 75 |
|---|---|
| `已跑（2026-09-24）` | 退 1，[没回看引用它的决策]（对照格，⑤ 后一半起作用） |
| `已跑两轮（…）`、`首轮已跑（…）`、`已测（…）`、`**已跑**（…）`、`已跑 (…)`（半角括号）、`已跑：结论不碰甲`、`已跑，结论不碰甲`、`已跑。` | 八格都退 0 |

每一步许可它的那一句：`:389` 只在状态段去掉全角括号之后逐字等于「已跑」时认；`:444` 不认就不进后一半。真仓今天（`t1-titles.out`）157 页里状态段恰好是「已跑」的 133 页；40 号认作跑过（`.claude/gate.d/40-results-cited.sh:53` `    cur=$2; done=($0 ~ /已跑|已测/); cited=0; excused=0; next`）、75 号后一半不认的 8 页：E7「首轮已跑」、E8「层 A 已跑」、E16「记账模型这一半已跑」、E19「建模层已跑」、E21「三段全部已测」、E34「算术 + 真设备两半都已跑」、E74「轴 ① 与轴 ② 的结构那一半已跑」、E80「已跑两轮」。E80、E21 是跑完了的写法，不是部分。

四句：改法前后都放过（HEAD 版没有后一半），不是新引入的放过、是新形态射程之外的一类；判别子看得到（标题就在页里）；满足 `:79` 字面（给得出输入写法，现跑退 0）；跑前条款没给改法。第二轮判决写明「部分已跑、已跑但作废不算」是有意的，这一格打的是「跑完了、写法不同」那几种，不是那两种。

### T1-b　决策用别的形态提到实验（B）

批：同 A 第一格（E2 改成「已跑（2026-09-24）」、只回看 D3），D1 欠账块里 E2 分别写成：

| D1 里的写法 | 基线 | 改成已跑 |
|---|---|---|
| `E2（样本实验二）` | 退 0 | 退 1 |
| `**E2**（样本实验二）` | 退 0 | 退 0 |
| `` `E2`（样本实验二） `` | 退 0 | 退 0 |
| `E2 （样本实验二）` | 退 0 | 退 0 |
| `E2(样本实验二)` | 退 0 | 退 0 |

许可它的句子：`:68` 要编号紧跟全角括号。doc-lint 的 G 检查认这四种写法（`.claude/singlefs-ai-sop/scripts/doc-lint.sh:900` ``            if (match(rest, /^[*`]+/)) { pre += RLENGTH; rest = substr(rest, RLENGTH + 1) }``、`:901` `            if (match(rest, /^ +[（(]/)) {               # 英文写法 `O2 (name)``），所以这几种写法在 kb 里过得了 doc-lint。真仓今天的决策正文里没有这四种写法提到、而同一份里又不带标准写法的实验号（`t1-mention-forms.out`：28 份决策里这类写法 0 处），这一格今天没有真对子。

四句：改法前后都放过；看得到；满足 `:79`；改法量过（见 `t-fix.out` 的 B 段：四格都转成退 1，基线四格仍退 0）。

### T1-c　「决策文件在这一批里」（C）

- C1：同 A 第一格，另把 D1 的「**定案**：样本规则。」改成「样本准则。」（与 E2 无关，形同全仓术语改名）。快照 75 退 0。
- C2：同 A 第一格暂存；别的会话在工作区改了 D1 那一句、没暂存。工作区直接跑退 0；照 `gate.sh --staged` 的做法在临时 worktree 上跑退 1。

许可它的句子：`:450` 只问决策文件在不在 `changed` 里，不问改的是不是那一句。主 agent 这一轮正在做的全仓术语改名就是 C1 这种批：改名碰到的每一份决策都自动算「回看过」，同批把哪个实验改成已跑，后一半都不说话。

四句：前后都放过（HEAD 版没有后一半）；判别子看得到（这一批在 D1 里新增或改写了哪几行，git 给得出，`added_lines` 已经在同一个文件里）；满足 `:79`；改法量过（`t-fix.out` C 段：C1、C2 工作区直接跑都转成退 1）。代价：人改了 D1 的欠账块但把 E2 那几个字删掉了，D1 就不再引 E2、后一半不再判它，不会误拦；改了 D1 别处又没碰引 E2 的那一句，会被拦（这正是要的）。

### T1-d　搬家被读成「改成已跑」（D，误拦）

- D1：E5（早就已跑）的页 `git mv .claude/kb/experiments/05-样本5.md .claude/kb/experiments/05-样本五.md`，一个字不改。快照 75 退 1：「[没回看引用它的决策] .claude/kb/experiments/05-样本五.md：这次改动把 E5 改成已跑，.claude/kb/decisions/01-甲.md 的正文引了它」。HEAD 版 75 退 0。
- D2：同上但用 `mv`、不暂存：快照退 1、HEAD 退 0。
- D3：在 D2 上给 E5 的表补一行 `| D1（甲） | 不影响 | 2026-09-24 不受影响：只是搬家 |`：退 0。

每一步许可它的那一句：`:394` 按新路径去基准里取标题，搬家之后基准里没有这个路径 ⇒ 取到 None ⇒ `:444` 的 `not ran_status(None)` 为真 ⇒ 当成「这一批把 E5 改成已跑」。搬家本身是 `.claude/rules/path-moves.md:6` 许可的：`**文件与目录的路径不在冻结范围内**：一个文件或目录搬了家、改了名，全仓指向它的地方一次改成新路径——`research/prompts/`、`research/results/`、`records/`、变更史与历史版本节、别的会话的记录都算，不留一处旧路径。`

四句：分得开改法前后（HEAD 退 0、快照退 1，是后一半新引入的红）；判别子看得到（基准那棵树里有 `## E5 ` 开头的页，只是路径不同）；满足 `:79`，但按 `:13` 的字面这道红**消得掉**（D3 补一行表，或碰一下 D1）——它不属于「消不掉的红」，属于「报的理由是假的、逼人为一次搬家去改内容」；改法量过（`t-fix.out` D 段：D1、D2 快照转成退 0，D3 仍退 0）。

真仓今天会不会撞上：要一个已跑实验、它被某条决策正文用 `E<号>（` 引、而它的表里没有那条决策。第二轮攻方现扫这种对子有 25 对（`gate-fix-forks-r2-opus-output.md` 的 T1 一节），其中已跑的那些页一搬家就中；这一轮没重扫逐对的状态（见「这条腿自己的限度」）。

### T1-e　重跑翻结论不触发（D4，射程）

E5 已跑，正文改一句「重跑之后结论翻了」、表里只回看 D3，标题照旧「已跑」：快照 75 退 0，D1 欠账块里等 E5 复核的那句没人动。`:444` 只在「基准不是已跑、现在是已跑」时进后一半。这一格是后一半的触发条件本身，75 号头部为什么立这一道写的就是重跑（`.claude/gate.d/75-decision-experiment-links.sh:30` `# 一个实验重跑、结论变了之后，没有任何东西逼人回去看它撑着的那几条决策。`）。四句：前后都放过；看得到；满足 `:79`；推的改法：后一半的触发条件从「改成已跑」放宽成「已跑的页正文改了」，代价是每次改已跑实验的正文都要回看引它的全部决策，没量。

### T1-f　10 号并进 ⑨ 之后，⑨ 放、10 号原来抓的一类（E）

批：新加一页「## E9 退役盘的重建代价 —— 已跑（2026-09-24）」，正文提到 D3，表里只有 `| D3（丙） | 不影响 | … |`，没有一条决策引 E9。快照 75 退 0；HEAD 版 10 号第 4 段「✗ E9 已跑，但决策正文一次都没引用它——它的结论悬空了」。同一页只把名字换成「坏盘的重建代价」：快照 75 退 1，[没对应决策]。

许可它的句子：`:287` 在**标题整行**里找「作废|退役」，而 `:26` 说的是「标题状态里写着」。真仓今天（`t1-titles.out`）标题里命中这两个词的 7 页：E45、E61、E69 是结论作废；E43（「上界 A 作废」）、E56（「前三轮因 harness 缺陷作废」）、E103（「只数了叶子，作废」）、E104（「容器退役记录」）四页结论照样成立，⑨ 对它们永远不判。四页今天各有支撑行，所以今天没有漏判的页；10 号第 4 段撤掉之后，这四页将来丢了对应决策，再没有一道检查说话。

四句：分得开（HEAD 10 红、快照 75 绿）；看得到；满足 `:79`；推的改法：「作废 / 退役」只认紧挨着「结论」的那种（`结论[^，。：]{0,3}(作废|退役)`，今天 7 页分得开：E45、E61、E69 中，E43、E56、E103、E104 不中——这一句只拿这条正则对真仓 7 个标题现跑核过，命令与输出在「这条腿自己的限度」），没写进 75 号、没跑样本。

正文 `:39` 举的那个例子（备料只点名一笔欠账、不点名决策）不是 ⑨ 放、10 号抓：10 号第 4 段认「**备料**：」点名 C<号> 就放行，⑨ 要表里有支撑 / 推翻 / 备料的一行，⑨ 更严。

## T5　改法自己引入的

### T5-a　`--no-renames` 与 88、40 号：站住

`research/scripts/changed-paths.sh:99` `  diff_output="$(git "${quote_option[@]}" diff --no-renames --unified=0 --no-color --no-ext-diff --src-prefix=a/ --dst-prefix=b/ "$base" -- "$@")" || return 1` 让搬家后的页整页算新增。`t5-moved-pages.out`：照 88 号的取法，204 份正文里整行抄的 E7RESULT 616 行，在树里、`git log --all --diff-filter=D` 删掉的产物、HEAD 里有而工作区没有的产物里都找不到的 0 行；照 40 号的取法，实验正文点名的产物 256 份，树里、HEAD、git 历史里都没有的 0 份。今天任何一页搬家都不会让 88、40 号多出一道红，更谈不上消不掉。什么会推翻：将来有一行早先抄的 E7RESULT 或一个点名的产物在归档里找不到（例如产物从来没提交过），那一页一搬家就红；那时改法是改那一行（写「原始输出未留存」），消得掉。

### T5-b　「HEAD 里还在、工作区删了就算找得到」与 `git commit --amend`（H）

历史（t5-config-amend.sh H 段）：c0 已推（有上游）；本地 c1 新加 E2 的页（点名 `e2-new-2026-09-24.out`、整行抄一行 `E7RESULT name=new v=2`）与那份产物；收拾工作区时产物被误删、`git rm` 进暂存区，准备 amend 进 c1。

| 时刻 | 快照 40 | HEAD 40 | 快照 88 | HEAD 88 |
|---|---|---|---|---|
| amend 之前 | 退 0 | 退 1 | 退 0 | 退 1 |
| `git commit --amend` 之后 | 退 1 | — | 退 1 | — |

amend 之后 `git log --all --diff-filter=D` 找到 0 条、HEAD 里没有：产物在哪个提交里都没有过。许可 amend 之前那一次放行的句子：`.claude/gate.d/40-results-cited.sh:119` `  git -c core.quotepath=false cat-file -e "HEAD:$RES/$name" 2>/dev/null && continue` 与 `.claude/gate.d/88-quoted-result-lines.sh:118` `    # 正在归档的：HEAD 里还在、工作区里已经删了的产物（这一批照归档规则删上一轮的产物，删除还没提交，` 那一段。放开用户动作（H2）：同一段历史、误删之后不 amend 而另起一次提交，删除进了历史，两道都退 0 且是对的——所以只有 amend、`reset --soft` 再提交、rebase 压提交这类改写本地提交的动作中。

四句：分得开（HEAD 两道 amend 之前就红）；判别子看得到（HEAD 是不是已经推出去的提交，`@{upstream}` 给得出）；满足 `:79`，但窄：放过只在 amend 之前那一次，amend 之后下一次门禁就红（那几行仍在窗口里）；推的改法：只在 HEAD 已经是上游的祖先时才把「HEAD 里还在」算找得到，没实现。

### T5-c　64 号的三条判据（t5-64）

每一格一个临时根，`.claude/gate.d/` 下只放一份样本阶段，跑快照树的 64 号。第二轮攻方报过的 11 种写法不再列。

| 写法 | 该 | 实测 | 放过或拦下它的句子 |
|---|---|---|---|
| python `base = os.getenv('GATE_BASE') or 'HEAD'` | 红（①） | 退 0 | `:38` 只认 `$GATE_BASE`、`${GATE_BASE`、`environ` |
| `git -c core.quotepath=false diff --name-only origin/master --` | 红（①） | 退 0 | `:38`、`:40` 不认写死的远端分支 |
| 同上换成 `HEAD^` | 红（①） | 退 0 | `:40` 只认 `HEAD~\d` |
| 同上换成 `HEAD~`（就是 HEAD~1） | 红（①） | 退 0 | 同上 |
| python `'HEAD' + '~1'` | 红（①） | 退 0 | 同上 |
| `git status --porcelain \| cut -c4-` 列路径、不带 quotepath | 红（②） | 退 0 | `:43` 只认 `--name-only`、`--name-status`、`ls-files` |
| `git diff --numstat "$base" -- \| cut -f3` | 红（②） | 退 0 | 同上 |
| `changed="$(gate_changed_paths …)" \|\| true` | 红（③） | 退 0 | `:52` 见 `\|\|` 就算判过 |
| `changed="$(gate_changed_paths …)"; [[ -n "$changed" ]] \|\| echo …` | 红（③） | 退 0 | 同上，`\|\|` 属于后一条命令 |
| 调用折行、`\|\| { …; exit 1; }` 在下一行 | 绿 | 退 1（③） | `:52` 只看调用所在那一行 |
| 下一行 `status=$?`、再下一行 `if (( status != 0 ))` | 绿 | 退 1（③） | 同上 |
| `set -euo pipefail` 下 `changed="$(gate_changed_paths …)"` | 绿 | 退 1（③） | 同上 |
| heredoc 里的 python 多行文档字符串，中间一行提到 `gate_added_lines` | 绿 | 退 1（③） | `:45` 的文字前缀只认 `#`、echo、引号开头 |
| heredoc 用法说明里写 `changed=$(gate_changed_paths <基准> untracked)` 与 merge-base | 绿 | 退 1（①③） | 同上 |

被引的行（快照树 `.claude/gate.d/64-change-range-single-source.sh`）：`:38` `BASE_PATTERN = re.compile(r'merge-base|@\{u(pstream)?\}|gate-ok|\$\{?GATE_(DIFF_)?BASE\b|environ[^\n]*GATE_(DIFF_)?BASE')`；`:40` `BASE_IN_GIT_PATTERN = re.compile(r'\bHEAD~\d|\S\.\.\.(\s|$|[^.])')`；`:43` `LIST_PATTERN = re.compile(r'--name-only|--name-status|\bls-files\b')`；`:52` `STATUS_HANDLED = re.compile(r'\|\||^\s*(if|elif|while|until|!)\b')`；`:15` ``#   ③ 调共用取法（gate_changed_paths、gate_added_lines）要判退出码：同一行带 `||`，或放在 if / while 里。``（③ 的判据字面就写着「同一行带 `||`」，所以折行、`$?`、`set -e` 三格是判据本身写窄了，不是实现偏离判据）。

今天：快照 64 号对主树 82 份阶段判绿。今天的 `.claude/gate.d/30-decision-history.sh:52`（主树，不在快照清单里）`numstat="$(git diff HEAD --numstat -- "${KB[@]}")" || git_failed "diff --numstat" "$?"` 正是 64 号看不见的 `--numstat` 不带 quotepath；它只把行数加起来、不拿路径比，今天判不错。

四句：不分改法前后（64 号是新立的）；看得到（都是源码里的字面）；满足 `:79`；推的改法：① 加 `os.getenv`、`HEAD^`、`HEAD~(?!\d)`、`origin/` 与 `@~`；② 加 `status --porcelain`、`--numstat`、`--stat`、`--raw`；③ 把「`|| true` / `|| :`」算没判，并按语句（接续行合并之后）而不是按行判——都没实现，折行与 `set -e` 两格改了判据字面才修得到。

### T5-d　七处判退出码之后的仓状态（t5-states）

每一格一个临时仓，放一套最小的文件（决策、实验目录、不变量页、`crates/demo/src/lib.rs`、从 HEAD 取的 `knowledge-sync-triggers.tsv`）。「*」= 阶段报「取不到这次改动碰了哪些路径 / 新增了哪些行」退出；没有「*」的退 1 是内容上的红（触发文件没登记、crates 改动没有判决这类），改工作区消得掉。

| 仓状态 | 快照 11 / 56 / 68 / 69 / 75 / 97 / 61 | HEAD 同七道 |
|---|---|---|
| 刚 `git init`、还没有提交（不放 crates） | 1* / 1* / 1* / 1* / 0 / 1* / 1* | 1 / 77 / 1 / 0 / 0 / 0 / 0 |
| 一个提交、没有上游 | 1 / 1 / 1 / 0 / 0 / 0 / 77 | 1 / 1 / 1 / 0 / 0 / 0 / 0 |
| 同一个仓 `git checkout --orphan`（HEAD 未出生） | 1* / 1* / 1* / 1* / 0 / 1* / 1* | 1 / 77 / 1 / 0 / 0 / 0 / 0 |
| 浅克隆（`--depth 1`，有上游） | 1 / 1 / 1 / 0 / 1 / 0 / 77 | 1 / 1 / 1 / 0 / 1 / 0 / 0 |
| 浅克隆、上游又前进两步（merge-base 取不到） | 1 / 1 / 1 / 0 / 1 / 0 / 77 | 1 / 77 / 77 / 0 / 1 / 0 / 0 |
| 照 `gate.sh --staged` 建的临时 worktree | 1 / 1 / 1 / 77 / 1 / 0 / 77 | — |

许可它的句子：HEAD 未出生时 `research/scripts/changed-paths.sh:55` `        echo "HEAD"` 交出一个解析不了的基准，`:86` `    git "${quote_option[@]}" diff --name-only "${filter_option[@]}" "$base" -- || [[ "${LIB_CHANGED_PATHS_BREAK:-}" == swallow ]] || exit 1` 失败，调用方照判了退出码的写法退 1（例 `.claude/gate.d/11-batch-scope.sh:50` `changed="$(gate_changed_paths "$(gate_diff_base head)" untracked)" || {`）。上游对同一个状态有明写的处置：`.claude/singlefs-ai-sop/scripts/lib.sh:220` `  # 空仓（还没有任何 commit）时 HEAD 不存在，diff 会失败——只看未跟踪文件`；75 号先判 HEAD 在不在（`in_git`），也不红。

四句：分得开（69、97、61 从退 0、56 从退 77 变成退 1*）；判别子看得到（`git rev-parse --verify HEAD`）；满足 `:79`，而且是 `:13` 说的「改工作区里任何文件都消不掉的红」——只有提交一次才消；这个仓今天有提交，只有新仓、新接这套门禁的项目、`--orphan` 分支撞得上，所以记「窄」；改法量过（`t-fix.out` 的 t5-states 段：空仓与 orphan 两行六道都不再报「*」，退出码回到与 HEAD 版一致；共用脚本自证在改过的树上退 0；样本自检两棵树逐条相同）。

### T5-e　使用者的 git 配置：61 号新引入的误拦，75 号没修的放过（G、F）

61 号现在一半用共用脚本（`gate_added_lines` 带 `--no-color --no-ext-diff`，`changed-paths.sh:99`），一半仍自己跑：`.claude/gate.d/61-settled-same-file.sh:86` `    touched="$(git diff -U0 "$BASE" -- "$f" \`。

- G：D1 这一批新加「### 已定（2026-09-24，用户定案）」小节，同时在它唯一那条未定项上补了一句「2026-09-24 复核过，仍然开着，因为样本理由」（该绿）。不配：快照 61 退 0。配 `color.ui=always` 或 `diff.external=<一个外部 diff 程序>`：快照 61 退 1「✗ 01-甲.md:13 本次新增了「已定」小节，而这条未定项一个字都没动」；HEAD 版 61 在同样配置下退 0（它的定案判定也被颜色码吃掉，看不见定案）。前一半认得出定案、后一半一行都读不出「碰过的行」，两半的取法不一致就是这道红的来源。
- F：同 T1-a 第一格（该红），只多一条 `color.ui=always` 或 `diff.external`：快照 75 退 0。75 号自己那份函数 `.claude/gate.d/75-decision-experiment-links.sh:368` `    diff = git('diff', '--no-renames', '-U0', base, '--', rel).stdout` 不带 `--no-color --no-ext-diff`，hunk 头认不出来，⑤ 的前后两半与「写了改了」那条一起静默。第二轮判决改这个函数时只补了文件头与 `--no-renames`。

四句：61 分得开（HEAD 退 0、快照退 1），75 分不开（HEAD 版同样放过）；判别子看得到；61 满足 `:13` 的「消不掉」——配置在 `.git/config` 或用户的全局配置里，不在工作区；改法量过（`t-fix.out`：两处 `git diff` 加 `--no-color --no-ext-diff` 之后 G 两格转成退 0、F 两格转成退 1）。

## T6　52 号的五行红样本（t6-mutants）

每个变异打在快照树脚本的拷贝上（旧串断言恰好命中一次），跑三样：47 号那条 `--selftest`、上游 `stage-selftest.sh` 跑 52 号的 red / green、52 号对着一份埋了一处错的真输入（快照树的 layout 与 replay.sh、主树的 E142 产物与第八节点名的 4 份 `.rs`）。第二轮的 M0–M8 不再报，只留 M1 作对照。

| 变异（改的那一行，`research/scripts/check-segment-registry.py`） | --selftest | 样本自检 | 埋的错 | 原 52 → 变异 52 |
|---|---|---|---|---|
| M1 产物行不比种类（第二轮，对照） | 退 0 | 退 1（判错 2 条） | 取号行种类 | 1 → 0 |
| N1 `:235` `        crash_state_count=merged_crash_state_count,` → `None` | 退 0 | 退 0 | 整条流 262165 → 262166 个状态 | 1 → 0 |
| N2 `:358` `        if array_count == 0:` → `if False:` | 退 0 | 退 0 | 第二条流第 3、4 段对调（写数与闭式都不变） | 1 → 0 |
| N3 `:342` `        if sum(segment_lengths) != claimed_operation_count:` → `if False:` | 退 0 | 退 0 | 第二条流 279 → 280 次写 | 1 → 0 |
| N4 `:360` `        if str(claimed_crash_state_count) not in normalized_test_text:` → `if False:` | 退 0 | 退 0 | 钉第二条流的用例里 2104413 改成 2104414 | 1 → 0 |
| N5 `:264` ``        return [f'{entry.row_label}：出处指的用例文件 {file_match.group(1)} 不存在']`` → `return []` | 退 0 | 退 0 | 钉 mkfs 的用例文件搬走（kb 没跟着改） | 1 → 0 |
| N6 `:269` ``        return [f'{entry.row_label}：出处指的用例文件里找不到 `fn {function_match.group(1)}`']`` → `return []` | 退 0 | 退 0 | 钉 mkfs 的用例函数改名 | 1 → 0 |
| N7 `:478` `            if pinned is None:` 之下先 `continue` | 退 0 | 退 0 | mkfs 行出处里丢了用例路径 | 1 → 0 |
| N8 `:484` `        if entry.segment_sequence_text is None:` 之下先 `continue` | 退 0 | 退 0 | 取号行 `` `2` `` 丢了反引号 | 1 → 0 |
| N9 `:518` `        if product_entry['step_kinds_text'] is None:` 之下先 `continue` | 退 0 | 退 0 | 产物取号行丢了 `kinds=` | 1 → 0 |
| N10 `:218` `    if len(remaining_paths) != 1:` → `< 1` | 退 0 | 退 0 | 八节开头的声明多出一条 path | 1 → 0 |

为什么两道都够不着：`--selftest` 的三刀（`:583` `        (mutated_layout_text, mutated_row_label,`、`:585` `        (kind_mutated_layout_text, kind_mutated_row_label,`、`:587` `        (pinned_mutated_layout_text, original_pinned_text,`）只改段序列、种类、第二条流的最后一项，而最后一项加 1 同时打坏写数、闭式与数组三处，任拿掉其中一处比对它都照红；前两刀落在 mkfs 行，走的是「钉活代码的用例」那一支，从不碰产物比对。五行红样本（`.claude/gate.d/52-segment-registry.sh:20` `#   一句整条流）、replay.sh 里 E142 那一行、产物里几行 name=segments。red 五行各埋一种错（段序列、种类、操作数、状态数、`）只测表格行对产物的五个分支，整条流那句的状态数、第二条流、钉活代码的用例、没段序列、产物缺 kinds、声明集合这几支一处都没有。

四句：分不开改法前后（第二轮的五行样本比原来一行多抓四支，这十支两版都抓不到；病在样本内容，不在「按阶段位置调仓里的脚本」那一半）；判别子看得到（每一处埋的错，原脚本都判红）；满足 `:79`（给得出变异与输入，现跑 1 → 0）；跑前条款没给改法。N1 最重：整条流那句是层 0 枚举吃的那一条（layout 第八节「整条流那一行才是层 0 枚举吃的」），它的状态数抄错一位，脚本里那一道比对拿掉了也没有东西说话。

**改法（量过，零轮）**：`t6-fixture-v2.sh` 把红样本扩成十行加两句提示，每条比对分支各一处错：原五行，加「没写段序列」「产物这一行没有 kinds」「钉住的用例文件不在」「用例里没有那个函数」「出处既无 path 也无用例」五行，整条流那句的状态数写错，另加一句第二条流（`2+1`、4 次写、5 个状态，钉它的样本用例里是 `[1, 2]`、不含 5）一句同时打坏写数、数组、用例里的状态数三处；expect 由原脚本对它的原样输出逐条生成（共 14 处不符、15 条 want）。重跑变异表（`t6-fixture-v2.out`）：基线样本自检退 0；M1 与 N1–N9 样本自检都退 1（各判错 2 条）；N10 仍退 0——声明集合剩不下恰好一条走的是「结构性错误、退 2」那一支，与逐行比对的错放不进同一份红样本，要么另开一种样本，要么进 `--selftest` 的第四刀，都没做。样本里多了一份 `crates/demo/tests/pin.rs`（住在 `fixtures/52-segment-registry.sh/red/` 下），别的门禁扫不扫到它这一层没核（推的：56、80 号按 `crates/` 开头的路径认，样本路径以 `.claude/` 开头）。

## T7　31、52 号在 cd 之前取位置（t7-invocations）：站住

现场：A = 快照树的拷贝；B = 诱饵拷贝，脚本与生成器换成只打「DECOY」的；F31、F52 = 两道阶段自己的 green 样本当外来项目根。对的结果是用 A 的脚本判 F、判绿。被判的句子：`.claude/gate.d/52-segment-registry.sh:27` 与 `.claude/gate.d/31-blocking-verdict.sh:59` 都是 `REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"`。

| 调用方式（都另给项目根 F） | 31 快照 | 31 HEAD | 52 快照 | 52 HEAD |
|---|---|---|---|---|
| 绝对路径 | 用 A、判绿 | 用 A、判绿 | 用 A、判绿 | 找不到，退 77 |
| 在 A 里用相对路径 | 用 A、判绿 | 找不到，退 77 | 用 A、判绿 | 找不到，退 77 |
| 在 B 里用 `../A/…` 的相对路径 | 用 A、判绿 | 用 A、判绿 | 用 A、判绿 | 找不到，退 77 |
| 经 `bin/` 下指向阶段文件的符号链接 | 找不到生成器，退 1 | 找不到，退 77 | 找不到，退 1 | 找不到，退 77 |
| 经指向 A 的目录链接 | 用 A、判绿 | 用 A、判绿 | 用 A、判绿 | 找不到，退 77 |
| `bash -c 'source …'` | 找不到生成器，退 1 | 找不到，退 77 | 找不到，退 1 | 找不到，退 77 |
| 从标准输入喂给 `bash -s` | 找不到生成器，退 1 | 找不到，退 77 | 找不到，退 1 | 找不到，退 77 |
| 导出 `CDPATH=B`，在 A 里用相对路径 | 找不到生成器，退 1 | 找不到，退 77 | 找不到，退 1 | 找不到，退 77 |
| 导出 `CDPATH=B`，绝对路径 | 用 A、判绿 | 用 A、判绿 | 用 A、判绿 | 找不到，退 77 |

九种里没有一种让快照版悄悄用了诱饵 B 或判了别的仓：取错位置的四种（文件符号链接、`source`、标准输入、带 `CDPATH` 的相对路径）都退 1 并在输出里点名找不到的那个路径，门禁记失败、不记通过；HEAD 版在同样四种里退 77（记「本次未跑」）。`CDPATH` 那一格核过细节：bash 的 `cd` 用上 `CDPATH` 时把目标目录打到标准输出，`$( … && pwd)` 于是拿到两行（都是 B），拼出的脚本路径带换行、找不到——它确实先解析到了诱饵，是换行让它失败了，不是位置取对了。门禁自己调阶段走的是 `$ROOT/.claude/gate.d` 下的文件路径（`.claude/singlefs-ai-sop/scripts/gate.sh:433` `    stage_rc=0; GATE_IN_STAGE=1 bash "$f" "$ROOT" || stage_rc=$?`，`$f` 由 `find "$GATE_D"` 给；样本自检的 `$stage` 由 `"$GD"/*.sh` 给），不经符号链接、`source`、标准输入；`CDPATH` 那一格只在调用方自己导出了它、又用相对路径调时才有。什么会推翻：找到一种调用方式让快照版退 0 而用的是 B 的脚本或生成器。

## 本腿的改法各修哪一格（都被攻过零轮）

「量过」= 打在快照树副本上、重跑同一批现场的原样输出在 `t-fix.out` / `t6-fixture-v2.out`；「推的」= 按代码推、没实现没跑。副本上的数不算入库装置上的数。

| 改法 | 修的格 | 在打中的格上 | 别的格有没有变 |
|---|---|---|---|
| 75 号另起 `MENTION_REF`（认粗体、反引号、空格、半角括号），只用于「决策正文引了它」 | T1-b | 量过：四格由退 0 转退 1 | 量过：四份基线仍退 0；75 号样本 red / green 仍判对 |
| 「决策文件在这一批里」收成「这一批在那份决策里新增或改写的行点到了 `E<号>`」 | T1-c | 量过：C1、C2 工作区直接跑由退 0 转退 1 | 量过：A 第一格、B 标准写法仍退 1；基线仍退 0 |
| 基准里这个路径没有页时，`git grep` 按实验号取基准那一版的标题 | T1-d | 量过：D1、D2 由退 1 转退 0 | 量过：E9 新页两格结果不变 |
| 与 40 号共用「跑过了」的谓词 | T1-a | 推的 | — |
| 已跑页正文改了也触发后一半 | T1-e | 推的（代价没量） | — |
| ⑨ 的豁免只认「结论…作废 / 退役」 | T1-f | 推的（正则对真仓 7 个标题现跑核过，没写进 75 号） | — |
| 只在 HEAD 已是上游祖先时认「HEAD 里还在」 | T5-b | 推的 | — |
| 64 号三条判据补写法、③ 按语句判 | T5-c | 推的 | — |
| 共用脚本：HEAD 未出生时基准取空树 | T5-d | 量过：空仓、orphan 两行六道不再报「*」 | 量过：另四种仓状态逐格不变；共用脚本 `--selftest` 退 0 |
| 61、75 号自己的 `git diff` 带 `--no-color --no-ext-diff` | T5-e | 量过：G 两格由退 1 转退 0，F 两格由退 0 转退 1 | 量过：不配时两道结果不变 |
| 52 号红样本十行加两句提示 | T6 | 量过：M1、N1–N9 由样本判对转为判错 | 量过：基线样本自检仍退 0；N10 修不到 |

改过的树上的样本自检（`t-fix.out` 末段）与原快照树逐条相同：40、52、61、64、75、80、88 号七道的 red / green 都判对；10 号两棵树一样判错 2 个——它读的 `lib-owed.py` 不在快照清单里，拷出来的树缺它，与改法无关（主树上 10 号现跑末行「  ✓ kb 腐化审计通过」）。

## 没打中的形状

- T1：一批里先改成已跑再改回——净 diff 为零，基准标题与现在相同，后一半本来就不该说话（推的，没跑）。备料只点名一笔欠账（C<号>）——⑨ 比 10 号第 4 段严，不是放过。待回填清单里的已跑实验——清单今天 0 行、只减不增，走不到。真仓决策里 doc-lint 认、75 号不认的实验号写法——0 处（`t1-mention-forms.out`）。「部分已跑」「已跑但作废」——形态有意不算，没攻。
- T5：搬家后 88、40 号多判的旧行——真仓 616 行、256 份产物里 0 行对不上（`t5-moved-pages.out`）。浅克隆（有上游、上游前进两步）与 `--staged` 临时 worktree——七道都不报「取不到」（`t5-states.out`）。决策文件搬家让 61 号的索引页「待议」一节误红——主树 `decisions.md` 今天 0 个 `## 待议`，没建现场。带引号路径让 `gate_added_lines` 退非 0——88、40 号退回全量判，按 T5-a 的扫数今天全量判也是绿，没建现场。
- T7：九种调用方式，见上表，快照版没有一种用错了仓而退 0。
- T6：十个新变异全中；`--selftest` 自己的三刀被改坏（例如 `mutate_one_segment_number` 选到别的行）没试。

## 这条腿自己的限度

- T1-f 那条正则只在真仓 7 个标题上现跑核过，命令与原样输出：

```
$ for f in .claude/kb/experiments/*.md; do l=$(grep -m1 '^## E[0-9]' "$f"); if grep -qE '作废|退役' <<<"$l"; then n=$(grep -oE '^## E[0-9]+' <<<"$l"); m=$(python3 -c "import re,sys; print(bool(re.search(r'结论[^，。：]{0,3}(作废|退役)', sys.argv[1])))" "$l"); echo "$n $m"; fi; done
## E103 False
## E104 False
## E43 False
## E45 True
## E56 False
## E61 True
## E69 True
```

- T1-d 在真仓上会撞到哪几页没逐对重扫：第二轮攻方的 25 对里哪些实验今天已跑，这一轮没算。
- T1、T5 的现场都是合成的最小仓，每一格只放开了几步用户动作（T1-c 改 D1 的哪一句、暂不暂存；T1-d `git mv` 与 `mv`、补不补表行；T5-b amend 与另起一次提交）；别的后缀（例如 amend 之后立刻推）没扫。
- T6 用的 E142 产物与 4 份 `.rs` 取自主树（不在快照清单里，别的会话在改 `crates/`），sha256 印在 `t6-mutants.out` 开头；换一版输入，埋错的那几处替换可能命中 0 次（脚本会断言失败，不会静默）。
- 本腿的改法只在这份模型上量过、被攻过零轮；改过的树只重跑了 t1-scenes、t5-config-amend、t5-states 与 16 道阶段的样本自检，没跑整轮门禁。
- 旁见（不在这四格里，两版都一样）：给上游 `stage-selftest.sh` 传相对路径的阶段目录时，它先 `cd` 到样本临时目录再 `bash "$stage"`，每个样本都退 127——本腿第一次对改过的树跑样本自检时撞上，改成绝对路径重跑；`gate.sh <相对项目根>` 会把相对的 `$ROOT/.claude/gate.d` 交给它，没建现场核。

## 没做什么

- 没碰 T3、T4、T8 与第二轮判决的数；没判正推、辩方那几格；不替主 agent 采纳。
- 没跑 `gate.sh`、没跑 54、55、57、59、87 号，没编译 Rust。开跑前 `ps` 看到别的会话的 5 个 `cargo test` 进程（`second_transaction_step_three_acquisition_barrier_layer0`、`second_transaction_supplement_three_random_history`、`first_transaction_step_seven_layer0`、`e142-first-txn-dry-run` 两个），没有 qemu / fio / vm-bench；本腿不碰 cargo，没有等锁。
- 没改仓里任何被判文件；写的只有这份报告与 `research/prompts/gate-fix-forks-r3-opus-model/` 下 22 份文件，草稿与临时仓在 `/tmp/claude-1000/gate-fix-forks-r3-opus/`（`run/`、`fix/tree/` 是改过的快照副本，不入库）。
- 副本上的数没在入库装置上重做。
