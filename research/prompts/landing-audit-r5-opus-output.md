# landing-audit-r5 攻方（Opus）：K2 第十批的修补本身、K3 产生结论的方法与没看过的地方（2026-10-23）

正文 `research/prompts/_landing-audit-r5-body.md`，第一轮判决 `research/prompts/landing-audit-r4-main-verification.md`，第十批交回 `research/prompts/subagent-landing-audit-2026-10-22/b10-*.md`。模型目录 `research/prompts/landing-audit-r5-opus-model/`。下文提的改法都是这条腿自己提的：只在我的模型上量过、被攻过零轮。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r5-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的空草稿目录>
```

- 它做的：从冻结副本拷钩子、定义、`scripts/agents`、`scripts/kb`、`scripts/hosts` 进 `tree-cur`（改后）；`tree-orig` 是同一份盖上 `orig/` 里第十批之前的五份原文（存成 `<文件名>.orig`，sha256 与 b10-hooks 交回的 `originals/manifest.tsv` 逐个相同，见 `out/orig-sha256.txt`），再去掉新库（改前）；`tree-fix` 是 `tree-cur` 加我提的 ⑰ 改法（`probes/make_fix.py`）。
- 真会话记录只读：按 `out/corpus-agents.txt` 那 3343 份子 agent 记录现抽语料（`tools/extract_corpus.py`，抽不齐退 1），跑 18 个探针、与 `out/` 逐个比；`watch_progress` 取此刻的文件系统，不比。`probes/durations.py` 不在 rerun 里，单跑（它扫全部记录，数会随新记录涨）。
- 我从 `/tmp` 起、用仓里这一份模型目录跑过一次：18 行「逐行相同」，退 0。冻结副本就是主工作区（开工时 `sha256sum -c research/prompts/landing-audit-r5-snapshot/judged-sha256.txt` 27 份 OK）。
- 语料抽于 2026-10-23；这一轮两条腿自己的进行中记录（a77dffd2065a0ba7e、ab7f880bc2d28e03a）没收。
- 跑了什么、没跑什么：只起 python 与 bash 探针、钩子的自证与 hook 模式，不编译；`naming-lint.sh` 只在 4000 行的造出来的小树上起过两次。

模型目录每个文件的 sha256：

```
0ca8de769e5c47c03a57a1d3bf7f0d42d2bfd196c19d4c1f9c40a5abd5be27ef  orig/completion-cleanup-guard.sh.orig
bab11748f599621eaf13ef5fd47364b537513a32de04fecae9ad6a169fa07548  orig/handback-guard.sh.orig
be47321288e64d3509f51312898febd05ca8a72a658d876bf45751ad9053a3bf  orig/lib_three_way_legs.py.orig
ce5cc2616c2667979836c6ddf2dfd460f358350cae533b17992af3a1e953c6a1  orig/runner-dispatch-guard.sh.orig
6f5afccac6427a6c62f9ce03067d7fa1fd79b9d77c06fe3ce32c7222e79f0991  orig/selftest-registry.tsv.orig
140f4941a89eb2dd01544d99283f721cc6cde2002b9cafc0bbecb18330f4a753  out/adhoc_recent.out
c782c4c3418f65e42c717da89f033a89e1485a9b0f9faf6d03656d03980b6a45  out/corpus-agents.txt
f9ecd215cccfd9622e555230ca321bd0aeec32e80034d12955286e024cc49db6  out/dump_naming.out
519aa39694e593b0db7d48317e6c75cf142321ca4d3580463832270489be1ed5  out/durations.out
9417ae70741294799e4dbf9809f94329a3b6b922e35843ba8e9d6156891e0875  out/echo_status.out
26fa3b28795b8c45bc595273d15866242ab1b4608a9b7baf9745614368aa3f1f  out/feed_heavy.out
81ab94d6fc0423dc120f021d41d2c46a441ce985ec63c4ae539838623ac8b498  out/fence_rules.out
09dcbca0338a0052cfeded1c3b12bd3625f6008160bf6f6841ac66271e06a059  out/flip_sample.out
1c357902f1a173b1d18fff80711366c40f509fbc9286ce606cc59b87fe0783e7  out/import_break.out
0a69a93bd4f9719ee5a8474975b47533814534d20822d72df948e2cfc2c4d166  out/leg_forms.out
a1a0514986195222ac4b4ff49f6e4fa262bbde98fcccd0e67ac74de1eeea3529  out/leg_handbacks.out
cac259ede2dcea4c1d24105883373fce70689275f8d8791dc885868e442a5b00  out/naming_e2e.out
8d6a3b438d453209b0fd66e4ffa73b22079480c855eb816668dda02c8cbf2be0  out/naming_masks.out
4efe65801c764ec40aa4de9d3c8ac8a0e523a4949ebc956509b37e83128c73fa  out/naming_real.out
a271e05d691ea5c50c576c5ebbf372c4331d885fecdf55fd033c315f9a36f25b  out/orig-sha256.txt
00398b701e9b25a49b814531a50874197c9c4fd0bfd4c6e1de067f2323c3aeaa  out/real_cells_fix.out
1ef461c6465b8625b8d863881d455d7a47696797432930c148a9e995701659d3  out/real_prompts_dir.out
10793e185d02a4eac80bbd14185d6fb3ce06ee7e40785533cbb6d6e718cb08aa  out/report_verdicts_fix.out
286184812441b5889cca359a6eda3df99ce4be43ac7eda9a65198036f5eca798  out/report_verdicts.out
03f588cad4c198fcbff67ed242b324044d2607723f731dddc54777030502516e  out/scratch_lines.out
62b0bbf884437303f0e85f0a0a390dd7afaa4a833240205a14fb2bc43c1f4b4a  out/sigpipe.out
5aed8d565dc259bdac4d909369aed52b6bb46b75cdebf6e2901a2ce785656bd4  out/watch_progress.out
592e0c32926ccdfde9337369e36527a60939b072f183e193a24c7066d3859f94  probes/adhoc_recent.py
2958d6fdd4e3430a186a33a26b11a69941d334d5a485655f74e4c2ee9952ac4d  probes/dump_naming.py
f398d1dbb4fe465043f534e2705b33d9e938f66cf560eb9fc3c5b2455a6fb1b8  probes/durations.py
1de7f88ca7bae4f042d613224dd33339e86aa9d193c57627d4238ee06f4c505d  probes/echo_status.py
bdd8b187156afecb89ff6d35f341897951f4e5603f073d2e6069deb3bf166d4e  probes/feed_heavy.py
12a46c1d6edafa5619c875b4bde4d95b0f122ce05d089b98f805a02161a139d0  probes/fence_rules.py
9581bc6958665c342cf6a2e8dd159f4e5ce2338de94fe517b2a0bdc1ebadad4b  probes/flip_sample.py
3fded8a65970b2bdd0596311c9d2d11c62a5dd2344011d440b7bf7db0dbd9277  probes/hookload.py
408c204d0391e96b9bcb27c224d7e0c4b1a754a92e0d05ed27d7ae0783dea573  probes/import_break.py
72e039f51ed6bb33fcc10bf6a9fb7c70fcf518e97172c029fb6bec53a4a758cc  probes/leg_forms.py
21c2f1b084bb56a13537e6a149ab7a9fc1eaf3464f104499f988ffc945de2ec7  probes/leg_handbacks.py
13cecd5400ff977634aa17f9f03f0d2ed475f0d05744346d2de33f9a1d65568d  probes/make_fix.py
b8c191a6d0a30dbd8d61453800d229560b56d394e8f11c4a1785b8ffdbd26fe0  probes/naming_e2e.py
959c4403ff6a36c8badcc907dcacc4eb3fd755724498be757b13bf8f523daebc  probes/naming_masks.py
5a08be1b6c6c8412a55001c23d7921809bd585759d53839ef3b2ef3ccdeeb1d1  probes/naming_real.py
fdb30b3c1a8dabcba7d5882776095f267b26bd764d93667f148b7c7469d7c890  probes/real_cells_fix.py
a714b16599228f96aecd576dfa12c2443bb4afd22c46e7e87615252f9ff16ffe  probes/real_prompts_dir.sh
3921225029638e1732925488a7a9e2c82f47a71fd0d1aa14287aac8c8c664a69  probes/report_verdicts.py
f8145484165a823edabce97a225275767e756b5ef85e91aedb101ed63ef3510e  probes/scratch_lines.py
8a10a1df305dcd96ad53e36fea1a3ef965f41414f69a015faab4a856eddf18a5  probes/sigpipe.sh
29cdabb5ee1af00293b330bbe841c45b03e9d76ff70f98a363d7fc8caaa60e84  probes/watch_progress.py
fb7b6f524e3e13b268fb97cc22ffd9b03b7d5d3f79f83d637ebad1cbc79f7715  tools/extract_corpus.py
c1afb312ac744fc77d53417ec29db8e9263bfa53095c832f3188e27ad70efbff  tools/rerun.sh
```

## 各格判定一览

| 格 | 条 | 判 | 一句话 |
|---|---|---|---|
| K2 新库与四处调用 | K2-1a | 打中（重） | ⑰ 收成只认行首一行之后，真派发提示里有 402 份从拒变成放（2026-10-14 之前 388、10-14 到 10-21 14、10-22 起 0）：全是「自己的 report.md 写在自己的草稿目录下、草稿目录写在句中或列表里」，抽 12 份全是自己的；从拒变放的转述纠正 0 份。⑳ 那一格把同一类的 a9717cf07fb6e7c21 钉成「放行」 |
| | K2-1b | 打中（轻） | 新库逐行认围栏，派发闸 ⑭⑮⑰ 照 CommonMark 认；语言名带反引号的那一行两边不一致。真提示 0 份 |
| | K2-1c | 打中（轻） | `scratch_directories` 用 `os.path.abspath`，相对路径按钩子起时的目录解：同一句「草稿目录：<相对路径>」在工作根下起认出一个目录、在仓根起认不出 |
| | 与骨架那一行逐字一致吗 | 没打中（量了出入） | 新库认的是骨架那一行的超集；骨架自己 `--check` 的 `DRAFT_LINE` 与新库在 10-22 起有 6 份不同，都是 `DRAFT_LINE` 把「（先」粘进路径，新库对 |
| K2 交回闸 ⑤ | K2-2a | 打中（重） | 2026-10-22 起改了要判的 `.rs`、改前放行的 4 份真记录，改后全拒；4 份都跑完了（`rc=1` 打在输出里）。病根是「包着 timeout、又不是整条最后一条」不收，而真记录里套 `timeout 230` 再 `; echo rc=$?` 是最常见的写法 |
| | K2-2b | 打中（中） | 工具退出码不是命名检查自己的那几种写法照收：接 `\| head` 被 SIGPIPE 杀掉（`PIPESTATUS=141 0`）、`; echo rc=$?` 打出 `rc=2`、`\|\| true`，改前改后都放 |
| K2 三方腿报告路径与 ⑳ | K2-3a | 打中（中） | ⑳ 读的 14 份真提示放在 `research/` 下、路径在代码里拼，归档规则认不出它是输入；下一次提交归档删掉之后，派发闸自证退 0、「查了 493 种」，⑳ 只剩一行「!」 |
| | K2-3b | 打中（轻） | 两侧 13 种写法逐个一致；但两侧都把仓根之外的 `…/research/prompts/<名>-output.md` 认成仓里那一份，交回闸再拿仓里那一份去核引文。真交回 46 次是这样写的（第十批之前就这样） |
| K2 例外说法 | K2-4a | 打中（轻） | 四处例外只罩「merge-and-verify.sh 起的那一步」，而 merge-and-verify.sh 自己的四处出路打的是单跑那两格的整条命令；照第 3 条，主 agent 跑它要弹窗 |
| | 有没有写宽 | 没打中 | 五处都限在「那一步」，没有一处写成整段合入后验证不弹窗 |
| K2 复跑格按字节收 | K2-5a | 打中（轻） | ㉛ 要「容错解码……别的照判」，做的是严格解、坏字节那一条直接返回，它点名的项不比；另有两处与改前不逐条相同（推的，此刻 0 份受影响）。超时与退出码的收法没变 |
| K3 真记录里出现几次 | K3-1 | 量了 | 第一轮 K3-1 那五种在 142 条真命名检查里 0 次；K3-2 的转述误拦 0 份；第十批收严之后新拒的：⑤ 4 份（全是误拒），② 0 次，⑯ 少建的锁 76 份（全在 10-14 之前） |
| K3 两轮都没看过的 | K3-2a | 打中（中） | ㉚：看门狗按提示里点名的全部工作根路径取最晚的 progress.md；10-22 起此刻找得到 progress.md 的 58 份里，21 份取到的不在本提示草稿目录下 |
| | K3-2b | 打中（轻） | `subagent_transcript.py` 的守卫只管直接执行，import 它的三处平常跑照认开关；续做闸 hook 模式下环境里有开关值就从放行变成拒（造的记录） |

数法统一写在 K3 一节。

## K2-1　新库 `lib_prompt_scratch_line.py` 与四处调用

### K2-1a　⑰ 收严丢掉了自己的 report.md（重）

哪一处：
- `.claude/hooks/runner-dispatch-guard.sh:1393`：「一行点名的（prompt_scratch_directories）」——⑰ 认的本提示草稿目录只剩行首那一行。
- `.claude/hooks/lib_prompt_scratch_line.py:15`：「照骨架写的提示都有那一行；没有那一行的提示，四处各按自己的规矩办：派发闸 ⑰ 只判不带目录的裸文件名」。
- 第一轮判决 `research/prompts/landing-audit-r4-main-verification.md:49`：「所以收严不丢罩住的对象」。
- ⑳ 的期望 `.claude/hooks/runner-dispatch-guard.sh:1989`：「("a9717cf07fb6e7c21.md", "6ffe7b19d753d8ea", "tooling-writer", False, False,」——第一个 False 是「报告文件名不拒」。

怎么复出来：`probes/report_verdicts.py` 拿真派发提示各喂改前（`tree-orig`）与改后（`tree-cur`）的 `report_file_name_verdict`、`three_way_report_verdict`，`out/report_verdicts.out` 原样（每行截到例子之前）：

```
报告文件名：改前拒、改后放：2026-10-14 之前：没有行首那一行：本提示自己的草稿目录写在列表、引用或粗体起头的「草稿目录：」行：18 份
报告文件名：改前拒、改后放：2026-10-14 之前：没有行首那一行：本提示自己的草稿目录写在句中不带冒号的「草稿目录 <路径>」：282 份
报告文件名：改前拒、改后放：2026-10-14 之前：没有行首那一行：本提示自己的草稿目录写在句中带冒号的「…；草稿目录：<路径>」：88 份
报告文件名：改前拒、改后放：2026-10-14 到 10-21：没有行首那一行：本提示自己的草稿目录写在句中带冒号的「…；草稿目录：<路径>」：14 份
```

- 翻了的 402 份没有一份带行首「草稿目录：」一行（带那一行、改前因为转述而拒的：0 份）。三方腿形态那一判一份都没翻。
- 是不是自己的：`probes/flip_sample.py` 固定种子抽 12 份，`out/flip_sample.out` 里 12 份都是「报告：<目录>/report.md；草稿目录：<同一目录>」这一类，例 a0a7455a41c3ecf07：「报告：`~/.local/share/singlefs/work/impl-gpu-judge-gaps-and-injection-reds/report.md`；草稿目录：`~/.local/share/singlefs/work/impl-gpu-judge-gaps-and-injection-reds/`。」
- a9717cf07fb6e7c21 本身：`research/prompts/landing-audit-r3-opus-model/prompts/a9717cf07fb6e7c21.md:1`：「草稿目录 `~/.local/share/singlefs/work/tooling-two-stage-followup/`（自己建，排他），报告 `~/.local/share/singlefs/work/tooling-two-stage-followup/report.md`」。这是工具员自己的 report.md，Claude Code 自带的拦截不让子 agent 用 Write 建它；⑳ 那一格把这个误放写成了期望。
- 第一轮攻方的三条探针（`research/prompts/landing-audit-r4-opus-model/probes/scratch_prompts.sh`）在改后树上：relay-comma 0、relay-list 0、own-zai 0、own-control 2，与 b10-hooks 交回的「改后」一致。

四句：
1. 分不分辨改法：分辨。同一份提示改前拒、改后放，只差第十批这一处。
2. 判它的东西看不看得到：看得到。草稿目录与报告路径就在同一份提示、多半在同一行。
3. 满足字面的哪一句：判决 K2-4 那一格的「收严不丢罩住的对象」——第九批 ⑰ 罩住的这 402 份，第十批之后不罩了。
4. 判决给的改法在这一格上还中不中：判决给的就是「取骨架那一种（行首「草稿目录：」）」，照做就是这一格的成因，对这一格不起作用。

现时的暴露：10-22 起的 216 份提示都带行首那一行（`out/scratch_lines.out`「骨架那一行……2026-10-22 起 216」，「没有行首「草稿目录：」一行、却在别处写了工作根下的草稿目录……2026-10-14 之前 764、2026-10-14 到 10-21 14」），所以照骨架派的时候这一格是 0；主 agent 手写提示时又会回来。

改法（只在我的模型上量过、被攻过零轮）：⑰ 先照新库认行首那一行；一行都没有时，退回第九批 ⑰ 的认法（前面不连着字、冒号可有可无、不在引号与代码块里）。⑳ 里 a9717cf07fb6e7c21 的期望改成报告文件名拒。
- 量过（`tree-fix`）：`out/report_verdicts_fix.out`「两判改前改后一份都没翻」（对第九批的判法，402 份全回来）；第一轮三条探针在 `tree-fix` 上读数与改后树相同（relay-comma 0、relay-list 0、own-zai 0）；`out/real_cells_fix.out`「改法：ran，14 格，与期望不同 1 格」，那一格就是 a9717cf07fb6e7c21。
- 推翻条件：主 agent 现查这 402 份里有转述别的子 agent 的 report.md、而那份提示没有自己的草稿目录的；或主 agent 定 ⑰ 只管照骨架写的提示（那这一格降成记录，⑳ 的期望另写依据）。

### K2-1b　新库认围栏的办法与派发闸别处不一致（轻）

哪一处：`.claude/hooks/lib_prompt_scratch_line.py:47`：「FENCE_OPENING = re.compile(r"^ {0,3}(`{3,}|~{3,})")」——逐行配、不看语言名里有没有反引号；同一个派发闸里 ⑭⑮ 与 ⑰ 的报告行经 `without_code_blocks` 照 CommonMark 认（markdown-it-py）。

`probes/fence_rules.py`，`out/fence_rules.out` 原样（截到「；⑯ 建锁」之前那一段）：
```
语言名里带反引号的那一行（CommonMark 不当围栏开头）：新库认 []；照 CommonMark 遮过之后认 ['/home/fy5090/.local/share/singlefs/work/r5-fence-mine']；⑯ 建锁 []
真派发提示里两种认法认出的「草稿目录：」值不同的：0 份
```
列表项里的围栏块、引用块里的围栏块、顶格的闭合围栏块三种两边一致；缩进 4 格的代码块两边都认（都不当代码块）。同一份提示里，这一种写法让 ⑯ 不建锁、⑰ 不认本提示的草稿目录，而 ⑭⑮ 照常读。四句：分辨（只在新库里）；看得到；满足的是新库文件头「闭合了的围栏代码块……里的行不认」与派发闸文件头「照 CommonMark」两句不是同一种认法；判决没给这一格的改法。改法（零轮、推的）：新库认围栏改调派发闸那一份（或搬进共用库），两处一种认法。推翻条件：主 agent 定「草稿目录：」一行不走 CommonMark 也行，因为真提示 0 份。

### K2-1c　相对路径按钩子起时的目录解（轻）

哪一处：`.claude/hooks/lib_prompt_scratch_line.py:113`：「path = os.path.normpath(os.path.abspath(work_roots.expand_written_path(value)))」。完工清理闸那一侧同样 `os.path.abspath`（`.claude/hooks/completion-cleanup-guard.sh:319` 下一行）。

复出来（一次性命令，原样输出）：
```
$ cd ~/.local/share/singlefs/work && python3 -I -c '…m.scratch_directories("草稿目录：landing-audit-r5-relative-probe\n")…'
起在工作根： ['/home/fy5090/.local/share/singlefs/work/landing-audit-r5-relative-probe']
起在仓根： []
```
我自己的探针也因此在两个起点数差了 1（账表那一格 95 与 94），现在探针一律先 `os.chdir("/")`。四句：分辨（改前 ⑯ 认法只收工作根前缀起头的，相对路径一律不认）；看得到；满足的是新库文件头「展开成绝对路径（normpath）」那一句，它没说按谁的当前目录；判决没给改法。改法（零轮、推的）：展开之后还不是绝对路径的一律不认。推翻条件：钩子的当前目录恒是仓根（那么相对路径永远认不出，只是没有误认）。

### 与骨架那一行逐字一致吗：没打中（量了出入）

骨架打的是第 0 列、全角冒号：`python3 scripts/agents/dispatch-prompt.py three-way-attack` 打出「草稿目录：<填 草稿目录>」。新库认的是它的超集（行首空白、半角冒号、冒号前空白都认）。骨架自己 `--check` 用的 `scripts/agents/dispatch-prompt.py:228`：「DRAFT_LINE = re.compile(r"^草稿目录：\s*(\S+)", re.M)」路径到空白为止。`out/adhoc_recent.out` 里 10-22 起 6 份不同，全是这一种：
```
骨架 a2d0dfec1f73ac422 DRAFT_LINE ~/.local/share/singlefs/work/m5-translate-en-2（先 新库 ['/home/fy5090/.local/share/singlefs/work/m5-translate-en-2']
```
新库截在括号前是对的；同一份提示交回闸 ⑥ 改前也把「（先」粘进路径，改后对了（`out/scratch_lines.out`「交回闸 ⑥ 认的草稿目录改前改后不同……2026-10-22 起 6」就是这 6 份）。完工清理闸的 139 份不同全是「改前把路径后面的字粘进路径、改后截在标点」。⑯ 少建的锁 76 份全在 2026-10-14 之前（写者锁那一天才有），10-14 起 0 份——b10-hooks 第八节「推出来的，没量过」的那一条，量出来是 0。

## K2-2　交回闸 ⑤ 新判法

### K2-2a　套 timeout 再 `; echo rc=$?` 的真记录全被拒（重）

哪一处：
- `.claude/hooks/handback-guard.sh:187`：「NAMING_TRUNCATING_PREFIXES = frozenset({"timeout"})」。
- `.claude/hooks/handback-guard.sh:650`：「if item["timeout"] and not status_carried:」——退 0 时包着 timeout、又不是整条最后一条的不收。
- 判决 `research/prompts/landing-audit-r4-main-verification.md:58`：「外层不套会截断的 `timeout`」。
- 子 agent 这样写的缘由：`.claude/agent-common.md:69`：「前台命令的 `timeout` 不超过 240000 毫秒」。

怎么复出来：`probes/naming_e2e.py` 把 4 份真子 agent 记录原文件逐字节拷进假主会话目录，喂两棵树里的 `handback-guard.sh`。`out/naming_e2e.out` 原样（截到冒号后 40 字）：
```
a7aa1608c74c3e21e（tooling-writer，记录 sha256 3e90a0855150a648…）：改前退 0；改后退 2：✗ tooling-writer 改了 crates/ 或 scripts/ 下的 .rs
aa87dd9fe761d727f（tooling-writer，记录 sha256 1d25a002905f1740…）：改前退 0；改后退 2：✗ tooling-writer 改了 crates/ 或 scripts/ 下的 .rs
aa68d151c2fa72bb7（tooling-writer，记录 sha256 372e60e5f98ff551…）：改前退 0；改后退 2：✗ tooling-writer 改了 crates/ 或 scripts/ 下的 .rs
ab6dd5f35228122b7（tooling-writer，记录 sha256 18a52406f8ccb57e…）：改前退 0；改后退 2：✗ tooling-writer 改了 crates/ 或 scripts/ 下的 .rs
```
它们真跑完了：`out/dump_naming.out` 里 a7aa1608c74c3e21e 的命令是「… timeout 230 nice -n 19 bash .claude/gate.d/code-source-discipline.sh --check … /home/fy5090/code/singlefs > $D/naming-code-source.txt 2>&1; echo rc=$?; tail -n 12 …」，工具结果开头「rc=1⏎  ✗ 数据形态的名字不合命名纪律：768 处违规」；另三份同形（`echo "rc=$?"`、`echo "csd rc=$?"`，aa68d151c2fa72bb7 的那一条是 `timeout 230 … 2>&1 | tail -8 | cut -c1-300`）。

全量：`out/naming_real.out`：
```
改了要判的 .rs（主工作区或此刻还带 .git 的树）的那几份，按派发日期：
  2026-10-22 之前：改前拒、改后拒：144 份
  2026-10-22 起：改前拒、改后拒：4 份
  2026-10-22 起：改前放、改后拒：4 份
```
即 10-22 起改了要判的 `.rs` 的 8 份里，照新 ⑤ 一份都交不回去；新 ⑤ 多拒的 4 份全是误拒。逐条看，`out/naming_real.out`「包着 timeout、又不是整条最后一条（后面接 ; echo $?、管道这类，退出码不是它的）：23 条」。造的对照，`out/naming_masks.out`：
```
误拒：真会话里的写法 timeout 230 包着、输出落文件、; echo rc=$?（rc=1 是判红跑完），工具退 0：改前退 0，改后退 2（判得对的应当是 0）
对照：同一写法去掉 timeout，工具退 0：改前退 0，改后退 0（判得对的应当是 0）
```
b10-hooks 第八节列的认不出的写法（`bash -c` 只收一层、子 shell、`$(…)`、`set -e`）里没有这一种；它的形状探针「timeout 包着、不是最后一条」应当 2，是有意的。

四句：分辨（同一份记录改前放、改后拒）；看得到（交回闸读的是整份记录，`rc=1` 就在那一条的工具结果里，它只看工具退出码）；满足的是判决那一句「外层不套会截断的 timeout」——这里的 timeout 没截断（打出来的是 1，不是 124），实现把「套了 timeout」当成「会截断」；判决的改法照做就是成因。

另量了一样（`probes/durations.py`，单跑，`out/durations.out`）：
```
code-source-discipline.sh --check（主工作区为根、只起它一条、2026-10-15 起）：17 次，中位 1 秒，最长 16 秒，超过 120 秒的 0 次
naming-lint.sh（主工作区为根、只起它一条、2026-10-15 起）：57 次，中位 3 秒，最长 112 秒，超过 120 秒的 0 次
```
不套 timeout 照拒绝说明里的两条重跑，都在前台 240 秒以内。

改法（只在我的模型上量过、被攻过零轮）：
1. 交回闸 ⑤ 遇到「命名检查后面紧跟 `; echo <标签>$?`」的那一条，从工具结果里按那个标签读它打的数，读到 0 或 1 就当这一条的退出码收（包着 timeout 也收，打出来的数说明它没被截断）；读不到照现在不收。推的：`out/echo_status.out` 按这条规矩给改后少认的真调用分类，紧跟 `; echo …$?` 且工具结果开头读得到 0 或 1 的 27 条（`rc=` 13、`exit=` 12、`csd rc=` 1、`naming_rc=` 1），4 份误拒里 3 份能回来，aa68d151c2fa72bb7 那一条是管道、回不来；没在钩子上实现。
2. 拒绝说明的出路加一句：两条不套 timeout、不接管道（量的：最长 112 秒）。推的。
推翻条件：主 agent 现查这 4 份里有一份的 `rc=` 不是那条命名检查打的；或定「套 timeout 一律不收」是有意的代价（那就该写进第八节那张认不出的表，并在共用约束「命名检查交回前跑」一条写明不许套 timeout）。

### K2-2b　工具退出码不是命名检查自己的，照收（中）

哪一处：`.claude/hooks/handback-guard.sh:624`：「退 0：都收；同一段里前面有 cd 或命名检查的，要在顶层最后一段」——退 0 只看工具结果的退出码；在管道里、后面还接着命令的那一条，退的 0 不是它的。

复出来：`probes/naming_masks.py`（造的记录，工具结果照真跑时的样子写），`out/naming_masks.out` 原样：
```
误放：两条都接 | head -2，命名检查被 SIGPIPE 杀掉（真跑是 PIPESTATUS=141 0），工具退 0：改前退 0，改后退 0（判得对的应当是 2）
误放：两条都退 2（用法错）而后面接 ; echo rc=$?，工具退 0：改前退 0，改后退 0（判得对的应当是 2）
误放：两条都接 || true，真退 3（naming-lint 没有 .rs 可查）也退 0：改前退 0，改后退 0（判得对的应当是 2）
```
SIGPIPE 那一种是真跑出来的：`probes/sigpipe.sh` 在 4000 行的小树上起 `naming-lint.sh`，`out/sigpipe.out`：
```
整条跑完：退 1，输出 4007 行
接 | head -2：PIPESTATUS=141 0（naming-lint 退 141 是被 SIGPIPE 杀掉，整条的退出码是 head 的）
```
真记录里这类写法改后照收的（`out/naming_real.out`，一条调用可带几个标签）：「改后照认；退 0；接管道到 head（head 先退、命名检查可能被 SIGPIPE 截断）：21 次」「改后照认；退 0；接管道到 tail：13 次」「改后照认；退 0；后面接「; echo」打退出码：30 次」。

四句：不分辨（改前改后都放，第十批没让它变坏，但也没做到判决要的「真跑完」）；管道那一种判别子观测不到——交回闸看得到的只有整条的退出码，`; echo` 那一种看得到（打在输出里）；满足的是判决「只认工具结果退出码 0、且命令里那两条处在顺序执行位置」的字面，不满足它的用意；判决给的改法（只认 0）在这几格上照中。改法（零轮、推的）：管道里的命名检查只在命令里有 `set -o pipefail` 或紧跟 `echo ${PIPESTATUS[…]}` 且读得到 0 或 1 时收；`; echo $?` 照 K2-2a 改法 1 读数；`||` 左侧的不收。代价：上面 21 + 13 次管道写法会从收变成不收（量的次数，改后会怎样是推的）。推翻条件：主 agent 定「交回闸只防没起、不防起了被截断」。

## K2-3　`lib_three_way_legs.py` 两个函数与 ⑳ 的 14 份真提示

### K2-3a　14 份真提示下一次提交就会被归档删掉（中）

哪一处：
- `.claude/hooks/runner-dispatch-guard.sh:1974`：「REAL_PROMPTS_DIRECTORY = os.path.join("research", "prompts", "landing-audit-r3-opus-model", "prompts")」。
- `.claude/hooks/runner-dispatch-guard.sh:1999`：「不在（第三轮攻方模型目录归档了的，去 git log 里取回再跑）」。
- `scripts/kb/archive-past-rounds.py:64`：「代码里拼出来的路径」——归档按字面完整路径认「还被代码当输入」，拼出来的看不见、不保护。

怎么报（`probes/real_prompts_dir.sh` 在 `tree-cur` 的拷贝里放、改那份目录），`out/real_prompts_dir.out` 原样：
```
目录不在：自证退 0；自检通过（查了 493 种）；本次未跑行 2；✗ 行 0
目录在、14 份原样：自证退 0；自检通过（查了 507 种）；本次未跑行 0；✗ 行 0
目录在、a9717cf07fb6e7c21.md 末尾多一个空格：自证退 1；；本次未跑行 0；✗ 行 1
  ✗ 自检：真提示:a9717cf07fb6e7c21.md（tooling-writer，草稿目录只写在句中、没有行首「草稿目录：」一行，只
```
改了一个字节：红、点名那一份（这一半对）。被删：退 0，绿行照打、少 14 种，只有一行「!」与 `GATE_NOT_RUN_FILE` 里一行。

会不会被删（推的，照 `scripts/kb/archive-past-rounds.py` 文件头的判据）：那 14 份在 HEAD 里（`git ls-files research/prompts/landing-audit-r3-opus-model` 有它们），这一轮没碰；`GATE_BASE=HEAD python3 scripts/kb/archive-past-rounds.py --check` 打的「留下 68 份还被代码当输入的产物」里 `landing-audit-r3-opus-model/prompts` 0 处。所以下一次 `--apply` 会删掉，⑳ 那 14 格从那以后一直是本次未跑。

四句：分辨（新加的格）；看得到（它报了本次未跑）；满足九十节 ⑳ 那一行「自证拿第三轮攻方 … 里那几份真提示当样本」的只有这一次提交之前；判决没给这一格的改法。改法（零轮、推的）：把 14 份连同 sha256 搬进跟着钩子放的样本目录（如 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`），本次未跑那一支只留给样本目录本身不在。推翻条件：主 agent 现查归档时这 14 份被别的登记保下（`--apply` 之后它们还在）。

### K2-3b　两侧一致，但都把仓根之外的报告认成仓里那一份（轻）

`probes/leg_forms.py` 13 种写法两侧逐个一致（`out/leg_forms.out`：裸写、反引号、句号、仓根绝对路径、`./`、`:12`、中文名、名字带点、`「」` 里的两侧都认；`.bak`、子目录、`<轮>` 两侧都不认）。不一致的地方不在两侧之间，在库的说明与实现之间：
- `.claude/hooks/lib_three_way_legs.py:20`：「仓根起头的绝对路径里的那一段照认」——实现的前瞻只要求前面不连着字母数字 `_ . -`，任何目录起头的都认。
- `out/leg_forms.out`：「/home/fy5090/.local/share/singlefs/work/r5-leg-copy/research/prompts/x-r1-opus-output.md：派发闸 ⑰ 放行；交回闸 ② 认出 ['research/prompts/x-r1-opus-output.md']」。
- 交回闸再拿仓里的同名文件去核引文；仓里没有就是 `.claude/hooks/handback-guard.sh:271`：「引文这一条没判（放行）」。
- 真交回（`probes/leg_handbacks.py`，`out/leg_handbacks.out`）：「改后认出的那一段前面接着仓根之外的目录：46 次」，例「报告：/home/fy5090/.local/share/singlefs/work/m4-predev/research/prompts/m4-trunc-r2-opus-output.md」；同一文件「两边相同：715 次」「两边不同：4 次」（4 次是改前多认了 `<轮>` 占位，改后照认那份真路径），新拒 0 次。

四句：不分辨（第十批之前的正则同样认）；看得到；满足的是库文件头那一句之外的写法；判决 K2-5 只要求一处认法，没说仓根之外。改法（零轮、推的）：`leg_report_mentions` 前面那一截只许是空、非路径字符、`./` 或恰好仓根；代价是那 46 次会改成按没点名报告拒。推翻条件：主 agent 定三方腿可以把报告写在仓外副本里（那么交回闸应当按点名的绝对路径去核，而不是仓里同名的那一份）。

## K2-4　`MERGE_VERIFY_QUICK_STEP` 与几处例外说法

### K2-4a　例外比 merge-and-verify.sh 自己给的出路窄（轻）

几处写的是同一个例外，都限在「merge-and-verify.sh 起的那一步」：
- `.claude/main-agent.md:22`：「合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步除外，不弹窗」
- `.claude/rules/implementation-workflow.md:45`：「合入后验证里 `scripts/gate/merge-and-verify.sh` 起的 checker 档快档那一步除外：不弹窗」（第 47 行同一句带前缀）
- `.claude/agent-common.md:58`：「起的 checker 档快档那一步是例外：同样只由主 agent 跑，不弹窗」
- `.claude/hooks/heavy-test-guard.sh:847`：「MERGE_VERIFY_QUICK_STEP = "" if guard_break_is_set("merge-verify-quick-step-dropped")」（同一句原文，主 agent 的「照抄」那一行也拼进它）

而 merge-and-verify.sh 自己在那一步本次未跑（`--peer`、`--copy`）、没带前缀、红了的时候，出路给的是单跑那两格的整条命令：`scripts/gate/merge-and-verify.sh:697`：「或只补这一步：$(checker_quick_rerun_command」；那条命令的声明 `scripts/gate/merge-and-verify.sh:662`：「为什么=合入后验证这几格红了，修完只重跑它们」。这条命令不是 merge-and-verify.sh 起的，照第 3 条就要弹窗。主 agent 不带前缀起它时，重型闸的「照抄」那一行又把例外原句贴一遍（`out/feed_heavy.out`）：
```
main 退 2："SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-tier-crash-point-replay.sh 的 crash-case-repla"　→ 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户；合入后验证里 `scripts/gate/merge-and-verify.sh` 带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步不另弹窗，跑法照 .cl
```
四句：分辨（新写的说法）；看得到（命令就是脚本打的）；用户定案「合入后验证跑受影响的快档」罩得住这条重跑，四处说法的字面罩不住；判决 K2-1 要的是「写出这个例外，指到定案」，四处照派发提示指到了验证段第 1 步（b10-defs-sync「顺手看到」第 3 条已报这一偏差，这里不重复），这一格它没碰。改法（零轮、推的）：四处的「那一步」后面加「（含 merge-and-verify.sh 打出的只重跑那两格的命令）」，或让 merge-and-verify.sh 的出路只给「带 `--skip-apply` 重跑整条」。推翻条件：主 agent 定单跑那两格本来就该弹窗。

### 有没有一处写宽了：没打中

查了 `.claude/main-agent.md` 第 7、22、29、52、96 行，`.claude/rules/implementation-workflow.md` 第 45、47 行，`.claude/agent-common.md` 第 58 行，`.claude/hooks/heavy-test-guard.sh` 第 847、861、959 行与 merge-and-verify.sh 文件头那一段（`grep -rn '合入后验证'` 全列过）。没有一处写成「合入后验证」整段不弹窗；第 45 行少「带 `SINGLEFS_HEAVY_TESTS=user-request`」几个字，但那一行的场合就是带这个前缀的，不算宽。第 52 行命令段不带前缀、b10-defs-sync 已列，不重复。

## K2-5　break-switch-replay 格按字节收

### K2-5a　做的是严格解，不是「容错解码、别的照判」（轻）

- ㉛ 那一行：`records/2026-09-16-subagent拆分提案.md:1502`：「复跑格按字节收、容错解码，坏字节那一份记一条红并点名路径与开关，别的照判」。
- `.claude/gate.d/code-tooling.sh:4217`：「return "坏字节", (f"{stream} 里有不合法的 UTF-8」——解不了就整条返回，那一条点名的项不再比；要等改了字节、下一趟才看得到它是不是也「不符」。这一份的点名比法被坏字节盖住了。
- 与改前不逐条相同的另两处（读代码，推的）：一是改前 `text=True` 会把单独的 `\r` 与 `\r\n` 当换行，按字节解之后单独的 `\r` 留在行里，`appears()` 在一行里找几段时可能把原来分在两行的几段接上；登记表 22 份脚本里只有 `.claude/hooks/runner-dispatch-guard.sh` 的源码里出现 `\r`，都是处理输入的，此刻 0 份受影响。二是坏字节的判定在「守卫」判定之前（`.claude/gate.d/code-tooling.sh:4222` 那一行在后），守卫拒绝说明里带坏字节的会报成坏字节。
- 超时与退出码：没变。`subprocess.TimeoutExpired` 那一支原样（「超时」），`code = done.returncode` 原样，守卫、没红、没点名、不符、对五种的判法与次序原样（对 b10-replay-decode 草稿目录 `edits/attempt.old` 与 `attempt.new`）。
四句：不分辨（坏字节那一份改前整格崩、改后这一条红，都不是绿）；看得到；满足 ㉛「记一条红并点名路径与开关」，不满足「容错解码……别的照判」里对这一份的那半句；判决没给改法。改法（零轮、推的）：先 `decode("utf-8", errors="replace")` 照常比点名项，再在有坏字节时另记一条「坏字节」红；两种红同一趟都报出来。推翻条件：主 agent 定「坏字节那一份不比点名项」是有意的。

## K3　产生结论的方法与没看过的地方

### 数法

- 语料：`~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/agent-*.jsonl` 全部 3343 份（除这一轮两条腿进行中的两份），号单在 `out/corpus-agents.txt`；不是抽样，是全数。按第一条记录的日期分三段：2026-10-14 之前 2398 份、10-14 到 10-21 729 份、10-22 起 216 份（`out/report_verdicts.out` 头一行）。
- 派发提示：每份记录的第一条 user 记录；「份记录」一份算一份，「份不同提示」按 sha256 去重（3343 份记录、3341 份不同提示）。
- 命名检查：每条 Bash 调用一次（「次」），一条调用里每条命名检查各一次（「条」）；只数改前认法认得出的（119 次调用、142 条）。
- 交回：每次 SubagentHandback 调用一次，被拒重交的各算一次。
- 判法用的是两棵树里的真函数（`probes/hookload.py` 把钩子里那段 python 载成模块，不改一个字），不另抄一份；分类标签（「接管道到 head」这类）是探针按命令文字打的，一条调用可带几个标签。
- 「一条腿只抽一次样不算一次观测」：这是全数，不是抽样；但 0 次这类「没打中」照规则仍要另一次复跑才记，`tools/rerun.sh` 就是那一次的起法。

### K3-1　第一轮打中的形状与第十批新拒的形状，在真记录里出现几次

| 形状 | 真记录里 | 出处 |
|---|---|---|
| 第一轮 K3-1 那五种（`false &&`、`true \|\|`、`if false`、只定义不调用的函数、`timeout` 被杀退 124）包着命名检查 | 0 条（142 条里：`\|\|` 右侧 0、退 124 0；落在 if 或函数体里的 3 条都是 a427eccd6824832fb 在 `&&` 后面起的子 shell，真跑了） | `out/naming_real.out` 逐条原因 |
| 第一轮 K3-2 的转述误拦（行首有「草稿目录：」、另在句中或列表里转述别处的 report.md，改前拒） | 0 份 | `out/report_verdicts.out`：翻了的 402 份全没有行首那一行 |
| 第一轮 K3-2 的「草稿目录在 <路径>」 | 0 份 | `out/scratch_lines.out` 没有这一行形状 |
| 第十批新拒：交回闸 ⑤ | 4 份（全是误拒，见 K2-2a） | `out/naming_real.out`、`out/naming_e2e.out` |
| 第十批新拒：交回闸 ② | 0 次 | `out/leg_handbacks.out` |
| 第十批新拒：派发闸 ⑰ 三方腿形态（占位、子目录） | 0 份 | `out/report_verdicts.out` 没有这一判的行 |
| 第十批新放：派发闸 ⑰ 报告文件名 | 402 份（10-14 之前 388、10-14 到 10-21 14、10-22 起 0） | K2-1a |
| 第十批少建：⑯ 写者锁 | 76 份，全在 10-14 之前 | `out/scratch_lines.out` |
| 列表、粗体起头的「草稿目录：」行 | 列表 66 份、粗体 11 份，全在 10-14 之前 | `out/scratch_lines.out` |

所以第一轮两条打中，在真派发与真交回里都是 0 次；第十批照它们收严之后，真记录里多拒的只有 ⑤ 那 4 份，而且全拒错了；多放的 402 份是同一次收严丢掉的。第一轮的结论是在造出来的输入上得的，第十批的自证也只核了造出来的输入与 14 份真提示；两样都没碰到真记录里最常见的写法（套 timeout 的命名检查、手写在一行里的「报告：…；草稿目录：…」）。

### K3-2a　㉚：同一份提示，四处认出的草稿目录不同（中）

- 骨架 `--check`（`DRAFT_LINE`）：10-22 起 6 份与新库不同，见 K2-1 末节；2026-10-14 之前 182 份、10-14 到 10-21 1 份。
- 账表（`scripts/agents/agent-cost.py:763`：「draft = re.search(r"草稿目录[：:]\s*(" + PATH_CHARS + "+)", prompt or "")」）：只要冒号紧跟、不认反引号；与新库不同的 163 + 94 份全在 10-14 之前，10-14 起 0 份。
- 看门狗（`scripts/agents/agent-watch.py:1651`：「取改得最晚的那一份，没有返回 None。」）：不认「草稿目录：」一行，提示里点名的工作根路径都算候选。`out/scratch_lines.out`「看门狗 progress_file_of 的候选里另有新库认的草稿目录之外的路径……2026-10-22 起 136」；照它的写法在此刻文件系统上取（`probes/watch_progress.py`，10-22 起）：
```
取到的不在本提示草稿目录下（看门狗告警附的是别人的 progress.md）：21 份
取到的在本提示草稿目录下：37 份
此刻一份 progress.md 都找不到：158 份
```
（每行截到例子之前；这一格随文件系统变，rerun 不比。）例 ac39106511365e719：本提示草稿目录是 `…/m4-quarantine-mark-gpu-grouping`，提示里叫它先读上一个实现员的报告 `…/m4-quarantine-mark-writer/report.md`，取到的是 `…/m4-quarantine-mark-writer/progress.md`。告警那一刻取哪一份看那一刻的修改时刻，这里是推的。
- 四句：分辨（新库只管四处钩子，三处脚本各认各的，第十批没碰也没说）；看得到；满足九十节 ㉚ 记的那件事；判决没给改法。改法（零轮、推的）：看门狗先取新库认的草稿目录下的 progress.md，没有再退回现在的取法；骨架 `DRAFT_LINE` 与账表改调新库。推翻条件：主 agent 现查看门狗告警里附的 progress.md 从没出过别人的。

### K3-2b　`subagent_transcript.py` 的守卫对 import 它的三处（轻）

- 守卫只在直接执行的 `main()` 里（`scripts/agents/subagent_transcript.py:333`：「refused = refuse_break_outside_selftest(arguments)」），被 import 时照认开关（b10-replay-decode 第二条写明这是给调用方自证用的）。三处调用方自己的自证 b10-replay-decode 交回里写了都退 0（我没重跑）；守卫不在 import 那条路上，平常跑时对它们没有作用。
- 但开关值漏进环境时，三处平常跑都会照它改判，不拒。续做闸 hook 模式（`probes/import_break.py`，造的记录：交回被交回闸拒、之后没再交），`out/import_break.out` 原样：
```
续做闸（hook 模式）不设开关：退 0
续做闸（hook 模式）环境里有 SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered：退 2；✗ 子 agent abreakprobe0000001 已经交回过（它的会话记录里有一次送达的 SubagentHandback），交回之后不续做，新活一律新
```
- 四句：不分辨（第十批之前一样）；看得到（环境变量就在那里）；㉛ 只要求直接执行时拒，做到了；判决没给改法。改法（零轮、推的）：三处调用方在不是自己的 `--selftest` 时看到 `SUBAGENT_TRANSCRIPT_BREAK` 就拒（或 import 前把它清掉）。推翻条件：主 agent 定环境里不会漏进这个值。

## 改法一览（都是这条腿提的、被攻过零轮）

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| ⑰ 没有行首「草稿目录：」时退回第九批认法；⑳ a9717 期望改拒 | K2-1a | 量过：`out/report_verdicts_fix.out`「两判改前改后一份都没翻」；第一轮三条探针读数不变；`out/real_cells_fix.out` 只差 a9717 那一格 |
| 新库认围栏改调 CommonMark 那一份 | K2-1b | 推的 |
| 展开之后不是绝对路径的不认 | K2-1c | 推的 |
| ⑤ 按 `; echo <标签>$?` 打出的数收 | K2-2a | 推的（`out/echo_status.out` 是分类，没在钩子上实现） |
| ⑤ 拒绝出路写「不套 timeout、不接管道」 | K2-2a | 推的（112 秒是量的） |
| ⑤ 管道里的只在 pipefail 或 PIPESTATUS 读得到时收 | K2-2b | 推的 |
| 14 份真提示搬进钩子的样本目录 | K2-3a | 推的 |
| `leg_report_mentions` 只认仓根或无前缀 | K2-3b | 推的 |
| 例外写成含 merge-and-verify.sh 打出的重跑命令 | K2-4a | 推的 |
| 坏字节那一份先容错解码照比、再另记一条红 | K2-5a | 推的 |
| 看门狗先取新库认的草稿目录 | K3-2a | 推的 |
| 三处调用方平常跑见到开关就拒 | K3-2b | 推的 |

## 没打中的形状

- 新库：全角半角冒号、行首空白、反引号、括注截断、`<填 …>` 占位、闭合围栏（顶格、列表项里、引用块里）、几行去重、`~` 与 `$HOME` 展开、根本身不算——造的输入与真提示都对；真提示里新库认出两个以上草稿目录的 0 份（四处不会为一份提示各建几把锁）。
- 四处调用：⑯、⑰、完工清理闸、交回闸 ⑥ 都调同一个函数（钩子自证的替身格之外，我在真提示上逐份比了改前改后）；完工清理闸只认工作根、不认过渡期旧根，与改前一样。
- 交回闸 ⑤：`bash -n`、写在 `$(…)` 里、`for` / `until` / `case` 里、`exit 0` 之后、放后台、退 2 / 77 这几种拒得对；真记录里没有一条退 124 或退 77 的命名检查。
- 三方腿报告路径：派发闸 ⑰ 与交回闸 ② 13 种写法逐个一致；交回闸 ② 在真交回上改后没有新拒的。
- ⑳ 改了一个字节：红、点名那一份。
- 例外说法：没有写宽的。
- 复跑格：超时、退出码、五种判法的次序没变；真仓上的读数没跑（那一道归主 agent）。
- 骨架那一行：骨架打的每一种都被新库认出。

试的范围：真提示 3343 份（全数）、真命名检查调用 119 次、真三方腿交回 719 次、真记录端到端 4 份；造的提示 5 种围栏形状、13 种报告路径写法、5 种命名检查退出码遮掩写法、3 种 ⑳ 目录状态。

## 这条腿自己的限度

- 语料里的工具结果只存了开头 300 字；退出码只看「Exit code N」那一开头，够用；`echo_status` 的分类只看这 300 字，读不到的 11 条可能其实读得到。
- 「改前」是 b10-hooks 备份的五份原文盖在今天的树上；别的会话这几天改过的库（`lib_heavy_tests.py`、`lib_shell_words.py` 等）两棵树用的是同一份今天的，改前读数不是第十批之前那一刻的原样。
- 树带不带 `.git` 按此刻的文件系统判：当时的 git worktree 副本多半删了，那几份按 rsync 副本不判，⑤ 那几行的份数因此偏少。
- 「是不是自己的 report.md」只抽了 12 份看；402 份的分法是按「提示里有没有行首那一行」机械分的。
- 看门狗那一格按此刻的文件系统取，告警那一刻取哪一份是推的。
- 改法只有 ⑰ 那一条在草稿树上量过；别的都是推的。
- 我自己的探针一开始没固定当前目录，出过一次 95 / 94 之差（K2-1c）；现在全部先 `os.chdir("/")`，从 `/tmp` 起复跑逐行相同。

## 没做什么

- 没编译、没起 cargo、没跑重型测试与门禁阶段；没改主工作区的任何文件（只写了这份报告与模型目录）。
- 没跑 break-switch-replay 格在真仓上的读数、没跑 code-tooling 样本档（归主 agent，b10-replay-decode「要主 agent 跑的命令」已列）。
- 没重跑 import `subagent_transcript.py` 的三处调用方的自证。
- K1 那一格（判决够不够得着）不归这条腿，没判；派发闸自证读的 14 份真提示只在副本里拷、改过。
- 草稿目录清理：删了 `tree-cur`（14M）、`tree-orig`（11M）、`tree-fix`（14M）、语料 `corpus.jsonl`（559M）、`run1` 到 `run6`（各 128K）、`r4probe-cur` 与 `r4probe-fix`（各 80K）、`sigpipe-root`（96K）、`orig`（592K）、草稿里的 `model`（912K，仓里那一份留着）；删之前 `du -sh` 记的这几个大小。剩下 356K 是读数与日志文本，没有仓副本、编译目录或库拷贝。

## 要主 agent 跑的命令

- 复跑：本文开头那一条（工作目录随意，`--work` 给新的空目录），看末尾 18 行都是「逐行相同」。
- 那 4 份误拒的端到端复核在 rerun.sh 里（`probes/naming_e2e.py`，读真记录原文件，输出比 `out/naming_e2e.out`）；rerun.sh 跑完会删掉它建的三棵树，单跑这一个探针要先照 rerun.sh 的头一段建 `tree-cur` 与 `tree-orig`。
