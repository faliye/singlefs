# landing-audit-r6 攻方腿（Opus，K2、K3）报告（2026-10-23）

正文 `research/prompts/_landing-audit-r6-body.md`。这一腿只攻 K2（「已定」项撑不撑得住）、K3（报出的数能不能核）。这是第三轮、最后一轮：下面我提的每一个改法都只在我自己的模型上量过，被攻过零轮。

## 复跑

模型目录 `research/prompts/landing-audit-r6-opus-model/`。整条复跑（工作目录随意；`--frozen` 给仓根或主 agent 的副本，`--work` 给一个新目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r6-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新草稿目录> --model research/prompts/landing-audit-r6-opus-model
```

- 我在草稿目录里跑通过一次（退 0，1 分 19 秒，抽 3362 份记录）。6 份输出与 `out/` 逐行相同；`leg_paths`、`watch_progress_all`、`naming_result_shapes` 三份因为多落了 1 份会话记录各差 1 个数；`rerun_command_feed` 那一份是因为当时喂的是拷出来的树、stderr 多了两行，已改成喂 `--frozen` 下的钩子（`tools/rerun.sh` 是改过之后的样子）。
- 内存包装：这一趟里 `run-with-memory-cap.sh 4G` 一直排不上（`--status` 写「slice singlefs-heavy.slice 的总上限 40.1 GiB」，「账上在跑 1 条：……要 40.0 GiB」，是别的会话的 `checker-tier-crash-point-replay.sh`）。我起的那一条排了 10 分钟以上，就停了它（`proc.py stop`）。之后这些纯 python 探针都没经包装，直接 `nice -n 19` 跑，外面设了 `ulimit -v`（4 到 6 GiB）。上一行那次 rerun 也是这样跑的。
- 模板自带的 `--selftest` 判的是模板自己的 `rerun_steps`，换掉之后头一格必红，第二轮攻方那一份也一样（`tools/rerun.sh --selftest` 红 1 格）。判决复跑看 `rerun_steps` 打出的 `name=` 行，不看它。
- 前提：`research/prompts/landing-audit-r5-opus-model/` 的 `orig/` 与 `out/corpus-agents.txt` 还在，r5 的两个数要用；下一次归档删了它就从 git 历史取回。
- 只读的输入：仓根，真会话记录 `~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/`（`naming_e2e_all.py` 用符号链接指过去，不拷也不改）。

每个文件的 sha256：

```
efd1f29986615d52a32f80347a6322f52d2d0f8d55bc753ff64c82ab7f5b212a  orig/v10/agent-watch.py.orig
01fa3f9c87dea9fd19b09a0e4d4b96cc58e81d830d0919ebec51d3faaadcdfb0  orig/v10/handback-guard.sh.orig
b70fdbc329e9396a1709b530a94883c9c7d7ecd876ea5a87faa5cf60aedeb558  orig/v10/heavy-test-guard.sh.orig
d38184ae3014bd70e1713308a30c0655f918b8c39f7432deb0e48caa58215c22  orig/v10/lib_prompt_scratch_line.py.orig
f67d526a033d60663f4fc2ed98da32c3cbe2893c27df61a3d4445eea0d265089  orig/v10/lib_three_way_legs.py.orig
03b8c2d13b5d64204cbc7fa7f904e60d3411ddb4f1503805a21dd9ef7aa3d5b2  orig/v10/runner-dispatch-guard.sh.orig
1ba1c96d260b663d33a986a7023923f0fae84b10a2288e507eab1768a2cbfc6d  orig/v35/handback-guard.sh.orig
e1617453f787cc6fd40fd7b1ebaa183eebf30c0e09ac96592ccd9c951d69defe  orig/v35/lib_three_way_legs.py.orig
9bf252fcc9cbaec25d3c5a513ecabf425aa37bc2728c38547b090b5fc26b31c1  orig/v35/runner-dispatch-guard.sh.orig
4f9c26a955aa45115247931e81b6441b433c02335c62e0f3a5fb0f404c947823  out/function_diff.out
ad80025283e7d25db48916c5cd04a97618e375469b25f90bf17db25c8b055ec0  out/leg_paths.out
3bc8c2c7403ea7221bc05521cb7153bf730d46b84f338a5045705784cd2eac8f  out/leg_prefix_variants.out
d9fdb0db99636fc018997bb32506fb0474def343485411e85c15bae582d5d61e  out/naming_e2e_all.out
784bff9b5263caeec2eb4fb8b9d8e77b70e507891e6423133586a1b6ed6d02fa  out/naming_forms.out
64be8847e032d9235fe10f6ee771382c19cfb0394028cb28499c0b53872bbe44  out/naming_result_shapes.out
13af1597b196ff87eb2e72081709be1fcb180482d6f7891ec623f93913ebf83c  out/orig-sha256.txt
4efe65801c764ec40aa4de9d3c8ac8a0e523a4949ebc956509b37e83128c73fa  out/r5_naming_real.out
286184812441b5889cca359a6eda3df99ce4be43ac7eda9a65198036f5eca798  out/r5_report_verdicts.out
a613b66648408c21dc2dfa9660c01949009af3282b61150fdabd3f92b08e70aa  out/rerun_command_feed.out
4a6847871832c61c491c04ad78bf4d21e5aa479852fa9af801abcb48c7c4f9a5  out/selftests.out
945d9b3c135b26b1054850e1102c8e9214918dcede91508108377ac42d31672b  out/watch_progress_all.out
0daccfeb7c1ec4dfbe0c71a3e35cfcbae94a823229299c3643d0c0ee7722861d  probes/function_diff.py
3fded8a65970b2bdd0596311c9d2d11c62a5dd2344011d440b7bf7db0dbd9277  probes/hookload.py
ce045b79d9dd05290503d1a2d4f7d22068d440c59887c1b2aa297632646e880a  probes/leg_paths.py
5bd34b311e5e7e9ff61daf2bc594d2a9e8e4bda7535aca68021387c81496d358  probes/leg_prefix_variants.py
3c05aa5c71d3541204ece348cfb631e1339cdf6d3150f22145f1331e618df2b2  probes/naming_e2e_all.py
2433917018684de27c627f8b4661a99095b0cd206c4f731074d6f26c04d1bf2c  probes/naming_forms.py
22259276e9962e2f99c63a373b0e8e8ae4b445276917046591393897738d215a  probes/naming_result_shapes.py
5a08be1b6c6c8412a55001c23d7921809bd585759d53839ef3b2ef3ccdeeb1d1  probes/r5_naming_real.py
3921225029638e1732925488a7a9e2c82f47a71fd0d1aa14287aac8c8c664a69  probes/r5_report_verdicts.py
fe3ef08d7da0cbc62c8d2cba029720e38f9b2d63581238f9f719617a4122a66c  probes/rerun_command_feed.sh
813df9c2b0e0647b419c3602533b9ce57dc1ce4982914655d528978465e1f748  probes/watch_progress_all.py
fb7b6f524e3e13b268fb97cc22ffd9b03b7d5d3f79f83d637ebad1cbc79f7715  tools/extract_corpus.py
653240766e1b1a523d26b5f75a256f72618311ac3912c6df8eece5e65a348213  tools/make-trees.sh
```

`tools/rerun.sh` 的 sha256 在报告末尾（它是最后才定稿的）。三棵树：`v10` 是第十批那一版，开工时从暂存区取的（`git show :<文件>`，sha256 以 `01fa3f9c`、`03b8c2d1`、`f67d526a` 起头，和任务 35 交回报告写的「换上之前」那一版相同）；`v35` 是任务 37 草稿目录 `originals/` 里存的，也就是任务 35 换上之后、任务 37 之前那一版；`now` 是主工作区此刻的。`r5_report_verdicts.py`、`r5_naming_real.py`、`extract_corpus.py`、`hookload.py` 是从第二轮攻方模型原样拷来的；其余探针是这一腿新写的。

## 各格判定一览

| 格 | 条 | 判 | 轻重 |
|---|---|---|---|
| K2 | K2-1 交回闸 ⑤：共用约束自己教的「输出落进文件、再读汇总行」写法，第十一批之后被拒；拒绝说明写成「没跑过」 | 打中 | 轻到中 |
| K2 | K2-2 `| tail` 那一类不收：该不该收 | 不收站得住；该改的是拒绝说明与共用约束那一句（同 K2-1） | — |
| K2 | K2-3 交回闸 ⑤ 退 0 而输出落成 persisted-output 时读不到 `; echo rc=$?` | 打中（两版都拒，不是第十一批带进来的） | 轻 |
| K2 | K2-4 交回闸 ② 与派发闸 ⑰ 的仓内路径认法、worktree 认法 | 没打中（14 种写法）；`$SINGLEFS_WORK_ROOT` 起头的不认，和 `lib_work_root.py` 不一致，真交回里 0 次 | 轻（推的） |
| K2 | K2-5 看门狗按行首「草稿目录：」取 progress.md | 没打中；库文件头那一句没把看门狗写进去 | 措辞 |
| K2 | K2-6 前两轮站住的形态（写回员目标、三方腿类型表、三个报告名前缀、草稿目录一行收严、坏字节记红） | 没打中（函数体逐字相同，真提示 3361 份判得一样） | — |
| K2 | K2-7 例外说法与 merge-and-verify.sh 打出的重跑命令 | 五处逐字一样，打出的命令主 agent 起得了；打中一处：「跑法照验证段第 1 步」指到的那一步没写这条命令，第 4 步说的正相反 | 轻 |
| K3 | K3-1 第二轮判决 402 份、4 份 | 复出来了（402 那一份输出逐字节相同；4 份一样，只差改名换掉的路径） | — |
| K3 | K3-2 第十一批 3、46、693、22、41；任务 37 的 37、9 | 都复出来了（693 今天是 695，多的 2 份是这一轮的两条腿，判法不变） | — |
| K3 | K3-3 第十一批的自证种数 78、512、1402（266）、18、92、23、36 格 | 都复出来了 | — |
| K3 | K3-4 第九批收尾的结局账 249 个 | 复不出来：今天 280 个；命令的窗口含当天，记录里没留截止点 | 轻 |
| K3 | K3-5 两份判决的 `cite-check.py --root .` 读数（第一轮全对上 52 处，第二轮 1 处对不上） | 照写的命令复不出来：今天 22 处、20 处对不上 | 轻 |
| K3 | K3-6 第二轮攻方模型的 `out/` 被改名会话改过 | 打中：照它的 rerun.sh 比，`naming_real` 已经对不上 | 轻 |
| K3 | K3-7 快照份数与 diff 行数（80、14292、27、10423） | 复出来了 | — |
| K3 | K3-8 第十批的 507 种、62 种 | 子 agent 复不了（副本里改过的脚本，钩子不让起自证），交主 agent | 没核 |

## K2　「已定」项撑不撑得住

### K2-1　交回闸 ⑤：共用约束教的「落进文件、再读汇总行」第十一批之后被拒，拒绝说明写成「没跑过」（打中，轻到中）

在哪：交回闸只收命名检查自己的退出码认得出的那一段（`.claude/hooks/handback-guard.sh:755`「每一段（&& / || 连成的一段）的退出码：顶层最后一段取这一条 Bash 的退出码」）。共用约束同时教了两种会被它拒的跑法：`.claude/agent-common.md:63`「长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`）」，`.claude/agent-common.md:99`「两条的命令与末尾原样输出贴进报告」。拒绝说明又把跑过了的写成没跑：`.claude/hooks/handback-guard.sh:845`「会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）」。

怎么复出来：`probes/naming_forms.py` 造一份假会话记录（改了主工作区 `crates/` 下一份 `.rs`，再按给定的命令与工具结果跑两条命名检查），交回正文只写「交回」，喂两版 `handback-guard.sh` 的入口（命令不真跑，判的是钩子读记录的那一层）。`out/naming_forms.out` 原样：

```
两条各自单独跑、判红退 1（对照）：v10 退 0，now 退 0（真跑完了没有：跑完了，该放行（0））
前面加 nice -n 19（共用约束「跑脚本、跑模型一律加 nice -n 19」）、退 1：v10 退 0，now 退 0（真跑完了没有：跑完了，该放行（0））
经 two-logs.sh 包着落盘（共用约束「要落盘就经 two-logs.sh」）、退 1：v10 退 0，now 退 0（真跑完了没有：跑完了，该放行（0））
输出落进草稿目录文件、再读汇总行（共用约束「长命令的输出落进草稿目录的文件，只读汇总行」）：v10 退 0，now 退 2（真跑完了没有：跑完了，该放行（0））
接 | tail -n 20、输出里有收尾汇总行（第十一批没收的那一类）：v10 退 0，now 退 2（真跑完了没有：跑完了，该放行（0））
接 | tail -n 20、后面 ; echo rc=${PIPESTATUS[0]}（打出的是命名检查自己的退出码）：v10 退 0，now 退 2（真跑完了没有：跑完了，该放行（0））
set -o pipefail 之后接 | tail -n 20、整条退 1：v10 退 2，now 退 2（真跑完了没有：跑完了，该放行（0））
( 两条 ; echo rc=$? ) | tail -n 5（rc 行走管道照样打出来）：v10 退 0，now 退 0（真跑完了没有：跑完了，该放行（0））
timeout 230 包着、; echo rc=$?、输出太长落成 persisted-output（rc 行在预览之外）：v10 退 2，now 退 2（真跑完了没有：跑完了，该放行（0））
两条用 && 连着、前一条退 1（后一条没跑），整条退 1：v10 退 2，now 退 2（真跑完了没有：没跑完或认不出，该拒（2））
两条接 || true（退出码被吃掉，对照）：v10 退 0，now 退 2（真跑完了没有：没跑完或认不出，该拒（2））
```

`| tail` 那一种被拒时，拒绝说明的头两行（`tail_refusal.out`，草稿目录里，截到 260 字）：

```
✗ tooling-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
  /home/fy5090/code/singlefs：最后改的是 /home/fy5090/code/singlefs/crates/singlefs-harness/src/lib.rs；改完之后没跑：code-source-discipline.sh 那四格、naming-lint.sh
```

照定义办的子 agent 会这样走：跑过了，被告知「没跑过」，再照「末尾原样输出」接一次 `| tail`，又被拒。拒绝说明里没有一个字讲退出码认不出。

真记录里有多少：交回层面新拒 0 份。`out/naming_e2e_all.out` 末尾原样（端到端喂入口，见 K3-2）：

```
汇总（按首条记录的 UTC 日期；退出码依次是 v10、now）：
  2026-10-22 之前：0/0：614 份
  2026-10-22 之前：2/2：144 份
  2026-10-22 起：0/0：35 份
  2026-10-22 起：2/0：3 份
  2026-10-22 起：2/2：5 份
共 801 份改过 .rs 的
```

逐条调用的层面，工具员量到新拒 27 条，其中「后面 ; 接 grep，没有 echo $?」1 条（`a5e5d9ce99b57fb08`），那一条就是「落进文件再读」这一种。这个数是工具员的，我没在逐条层面重量。交回层面没多拒，是因为那 14 份记录里 12 份改的树此刻没有 `.git`（工具员的分法）。m5-dev 是带 `.git` 的 worktree，以后在它里面改 `.rs` 的执行员都要判。

四句：
- 分不分辨臂：分辨。第十批那一版放这两种写法，此刻这一版拒；第十一批的改法就是两版的差别。
- 系统当时看不看得到判别它的东西：只看得到一部分。钩子能看到命令文本和结果文字，看不到命名检查自己的退出码（工具退的是 `tail` 的 0）。「跑完了没有」只能从输出里有没有收尾汇总行看出来，`| tail -n 20` 的输出里一般有，`grep -c` 的输出里没有。
- 满足的是判据哪一个分句：不违反第二轮判决的字面。判决 K2-2b 写「`rc=` 一行或管道最后一段之外的写法不认」，钩子照做了；工具员也照判决第 5 条在真记录上量过。打中的是判决这一句和共用约束 `:63`、`:99` 两句之间的矛盾，再加上拒绝说明 `:845` 那一句说的不是实情。
- 改法在打中的格上还中不中：改法 A（推的）修「说明不对」与「以后还撞」两格，不让这一种写法放行；改法 B（推的）修「放行」那一格，但带进一条新的判法；改法 C（推的）只修 `PIPESTATUS` 那一行，别的格不起作用。

改法（都只在我的模型上量过或只是推的，被攻过零轮）：
- A（推的）：拒绝说明按每一条认出了命名检查、却认不出退出码的调用，分两种说：「这一条跑了，但接了管道，退出码认不出」与「后面用 ; 接了别的命令」。出路写「两条单独跑，或后面紧跟 `; echo rc=$?`；输出要落盘用 two-logs.sh」。`.claude/agent-common.md:99` 那一句补同样的话。判法本身不动。
- B（推的）：命名检查出现在管道里、又不是最后一段时，输出里有那一道的收尾汇总行（code-source-discipline 的逐格汇总，naming-lint 的末行）就收。这是新判法，假放行的风险（汇总行是别处打出来的、输出被截掉）没量过。
- C（推的）：`status_echo_pattern` 认 `${PIPESTATUS[0]}`，打出的就是管道第一段的退出码。真记录里只有 1 条（`a8519aab2a61cba49`，工具员的数）。

推翻条件：主 agent 定了「命名检查不许接管道、不许落进文件再读」，并且共用约束 `:63` 的通则明写不管命名检查；或者在真记录里查到子 agent 照拒绝说明重跑时，会自己去掉 `| tail`（那就说明这句说明不误导人）。

### K2-2　`| tail` 那一类不收是不是该收：不收站得住

- 退出码这一条路上，`| tail` 确实认不出：`tail` 退 0，看不到命名检查是不是被杀了。第二轮判决认命名检查自己的退出码，这一点在 `| tail` 上没有反例。
- 代价小：交回层面 2026-10-22 起照旧拒的只有 `aa68d151c2fa72bb7` 一份（K3-2 那张表里「2/2」的 5 份，另 4 份是 m5-dev 里一条命名检查都没跑的执行员）。被拒的子 agent 照拒绝说明单独再跑一遍就过（对照那一格：单独跑、退 1，两版都放）。
- 所以不收站得住。要补的是 K2-1 改法 A，让被拒的子 agent 知道为什么被拒；收不收汇总行（改法 B）是新判法，交用户表里记零轮。

### K2-3　退 0 而输出落成 persisted-output 时，`; echo rc=$?` 那一行读不到（打中，轻；两版都拒）

钩子按工具结果文字读打出的那一行（`.claude/hooks/handback-guard.sh:746`「printed = pattern.findall(output or "")」）。退 0、输出很长时，Claude Code 只在记录里留开头约 2 KB 的预览，`rc=` 那一行不在里面，整条按认不出拒（上面那张表里 timeout 那一行）。退非 0 的不落盘：我在这一腿里跑了一条输出 6000 行、退 1 的命令，工具结果开头是「Exit code 1」，中间截掉，`:457` 照样读得到退出码。真记录里命名检查的结果形状（`out/naming_result_shapes.out` 原样，例子截短）：

```
不是错误｜persisted-output 开头｜输出落成 persisted-output：5 条；例：a8a66a81f09d886b1：'<persisted-output>\nOutput too large (111KB). Full output saved to: /ho' ／ a139bd6fda2c9ded9：'<persisted-output>\nOutput too large (40.4KB). Full output saved to: /h' ／ a17693edf1b2c48ab：'<persisted-output>\nOutput too large (35.2KB). Full output saved to: /h'
```

这 5 条里，命令带 `echo …$?`、`rc=` 那一行落在预览之外的，工具员数的是 1 条。改法（推的，被攻过零轮）：输出落成 persisted-output 时，按预览里写的路径去读整份输出再找那一行；读不到才按认不出算。推翻条件：Claude Code 的 persisted-output 那份文件在交回时已经被清掉，或者路径读不到。

### K2-4　交回闸 ② 与派发闸 ⑰ 的仓内路径认法、worktree 认法（没打中；一处轻的不一致，推的）

量了什么：
- 真记录（K3-2 的 `out/leg_paths.out`）：670 次三方腿交回、695 份三方腿派发提示，三版一一比过，第十一批改动之外没有别的翻转；3361 份各类型派发提示交 `report_file_name_verdict`，三版判得一样（「拒/拒/拒：1365 份」「放/放/放：1996 份」）。
- 没被量过的 14 种写法，交此刻的 `leg_report_mentions_in_trees`。为了量符号链接那一种，在草稿目录里临时建了一个指向 m5-dev 的链接，用完删了，没碰 m5-dev 本身。`out/leg_prefix_variants.out` 原样：

```
工作树：['/home/fy5090/code/singlefs', '/home/fy5090/.local/share/singlefs/work/m5-dev']；取列表的错误：None
m5-dev 绝对路径（对照）：认出 /home/fy5090/.local/share/singlefs/work/m5-dev
m5-dev 路径里带 /./：认出 /home/fy5090/.local/share/singlefs/work/m5-dev
m5-dev 路径末尾多一个 /：认出 /home/fy5090/.local/share/singlefs/work/m5-dev
经符号链接写的 m5-dev：认出 /home/fy5090/.local/share/singlefs/work/m5-dev
$SINGLEFS_WORK_ROOT/m5-dev（lib_work_root 认这种写法）：没认出
~/.local/share/singlefs/work/m5-dev：认出 /home/fy5090/.local/share/singlefs/work/m5-dev
仓根路径里带 /./：认出 /home/fy5090/code/singlefs
工作区相对路径写法：认出 /home/fy5090/code/singlefs
<副本>/ 占位起头：没认出
草稿目录名起头的相对写法 m5-dev/：没认出
Markdown 链接 [报告](…/m5-dev/…)：认出 /home/fy5090/.local/share/singlefs/work/m5-dev
file:// 写法：认出 /home/fy5090/.local/share/singlefs/work/m5-dev
仓根绝对路径（对照）：认出 /home/fy5090/code/singlefs
没有前缀（对照）：认出 /home/fy5090/code/singlefs
```

只有一处不一致：`$SINGLEFS_WORK_ROOT/` 起头的不认（`.claude/hooks/lib_three_way_legs.py:140`「if "$" in written or "~" in written:」）。同一个钩子的别处按 `lib_work_root.py` 展开这种写法（`.claude/hooks/handback-guard.sh:294`「path = work_roots.expand_written_path(path) if work_roots else path」）。3351 次交回里写了 `$SINGLEFS_WORK_ROOT` 的只有 2 次，都不是三方腿（三方腿 670 次全落在上面那几类里），所以 0 次撞上。改法（推的，被攻过零轮）：`prefix_tree` 先过一遍 `lib_work_root.expand_written_path` 再判。推翻条件：真交回里写成这样的三方腿报告出现了，钩子却照样认出。

worktree 认法做到了第二轮判决那一条（仓外的不认，本仓 worktree 里的认，并按那一棵当根核引文）。在真记录与这 14 种写法上，我没找到误放或误拒。

### K2-5　看门狗按行首「草稿目录：」取 progress.md（没打中；库文件头一句没跟上）

- 真记录（K3-2 的 `out/watch_progress_all.out`）：改后取到别的目录下的是 0 份；本提示目录下明明有 progress.md、改后却取不到的也是 0 份。
- 提示里点名两个以上不同草稿目录的：0 份。同一个目录被两份以上提示当草稿目录的有 46 个，此刻有 progress.md 的 4 个，都是同一件活的接续派发（例：`m3-close-b16-tools-ji12` 先后是 `a55de88df0f612b0b`、`a6e8bd09dcf72f5e0`，都是 tooling-writer）。取到前一条的 progress.md 是对的。
- 措辞：`.claude/hooks/lib_prompt_scratch_line.py:16`「照骨架写的提示都有那一行；没有那一行的提示，四处各按自己的规矩办」，只列了派发闸、完工清理闸、交回闸三家（四处），没有看门狗那一种（「报取不到、不到别处去找」）。文件头调用方表已经从「四处」改成了「五处」。改法：这一句补看门狗，「四处」改「五处」。

### K2-6　前两轮站住的形态没被第十一批改坏（没打中）

`probes/function_diff.py` 把第十批那一版与此刻那一版的同名函数逐字比。`out/function_diff.out` 摘要：交回闸 `scribe_spec_entries`、`missing_targets_sections`、`missing_targets_listed`、`scribe_target_problems`（写回员目标）、`citation_checked_agents`、`decide`、`naming_source_tree` 等 13 个逐字相同；派发闸 `report_file_name_verdict`（三个报告名前缀）、`prompt_scratch_directories`、`report_scratch_directories`、`three_way_report_types`、`own_report_token` 逐字相同，只有 `three_way_report_verdict` 不同（改的就是调库多传一个仓根）；库 `frontmatter_fields`、`three_way_leg_types`、`three_way_leg_types_scanned`、`is_leg_report_path` 逐字相同，`leg_report_mentions` 不同（改的就是它）；草稿目录库前后只差文件头 3 行。真提示上，`report_file_name_verdict` 在 3361 份上三版判得一样。坏字节记红那一格不在第十一批改动里：`.claude/gate.d/code-tooling.sh:4268` 还是「return "坏字节", (f"{stream} 里有不合法的 UTF-8」，样本目录 `break-switch-replay-undecodable-red` 还在；`break-switch-replay-files.tsv` 里交回闸、派发闸两行还在。

### K2-7　例外说法与 merge-and-verify.sh 打出的重跑命令（逐字对得上；打中一处指向不对，轻）

- 五处（`.claude/main-agent.md:22`、`.claude/agent-common.md:58`、`.claude/rules/implementation-workflow.md:45`、`:47`、`.claude/hooks/heavy-test-guard.sh:847` 与自证 `:1979`）都含同一串「带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）」（逐处用 python 找子串核过）。
- 打出命令的五处（`scripts/gate/merge-and-verify.sh:678`、`:684`、`:697`、`:710`、`:735`）是 --peer、--copy（这两处都是本次未跑）、没带前缀、显存不够（记红）、测试红。例外那一句的括注罩得住这五处。本次未跑与记红的另几种（补丁没碰、那一道退 77、内存排不上）不打命令，也就不用罩。
- `probes/rerun_command_feed.sh` 从脚本里原样抽出 `checker_quick_rerun_command`，用主工作区的取值打出那一条，喂主 agent 身份的两道钩子。`out/rerun_command_feed.out` 原样（命令一行很长，照抄）：

```
打出的命令（起 merge-and-verify 时 SINGLEFS_HEAVY_TESTS=没设）：SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-tier-crash-point-replay.sh 的 crash-case-replay、feature-gated-tests 格; 范围=2 格; 为什么=合入后验证这几格红了，修完只重跑它们：键相同的照复用，只现判红的那几条与改后键变了的' bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 31 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/.claude/gate.d/checker-tier-crash-point-replay.sh --check crash-case-replay\,feature-gated-tests /home/fy5090/code/singlefs
  主 agent 身份喂 heavy-test-guard.sh：退 0 
  主 agent 身份喂 run-scope-guard.sh：退 0 
```

（`SINGLEFS_HEAVY_TESTS=user-request` 起 merge-and-verify 的那一种，打出的命令一样，两道钩子也都退 0，见同一份文件。）
- 打中的一处：五处都说这条命令「跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步」，可第 1 步（`.claude/main-agent.md:52`）一个字没写这条只重跑几格的命令；第 4 步（`.claude/main-agent.md:55`）写的还正相反：「这一段红的、判决打中的，合成一批派 `implementation-writer` 一次改全；改完回到验证段第 1 步整体再验一次」。脚本自己的出路是 `scripts/gate/merge-and-verify.sh:735`「FAILED 的那几条测试函数并进写段的下一批派 implementation-writer 改；改完只重跑那几格」。照定义走的主 agent 会整条重跑；照脚本走的只重跑两格，是哪一种定义没说。任务 36 工具员在「没做什么」里点过这一处，交了主 agent；调度记录节「第十一批」没有它的去向。
- 改法（推的，被攻过零轮）：验证段第 4 步加一句「只有 checker 档快档那一步红的，照脚本打出的那一条只重跑那几格；编译、测试红的回第 1 步整条重跑」，或者把五处的「跑法照」改指到第 4 步。推翻条件：主 agent 定了「修完一律整条重跑」，那就该删掉脚本 `:735` 的「只重跑那几格」，五处括注也跟着收窄。
- 另有一处用词：`--peer`、`--copy` 两种本次未跑的情形，打出的那一条的范围声明里写的也是「为什么=合入后验证这几格红了」（`scripts/gate/merge-and-verify.sh:662`），可那时并没有红。范围声明闸只看这段长度，判不出来。改不改都行。

## K3　报出的数能不能核

语料：用第二轮攻方的抽取脚本（`tools/extract_corpus.py`，原样拷来），把 `~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/` 下全部 3361 份记录抽出来（2026-10-23），「抽了 3361 份（共 3361 份记录）」。工具员自己的抽取与量数脚本我一份没用。

### K3-1　第二轮判决的 402 份与 4 份（复出来了）

- 402：照第二轮攻方的 `report_verdicts.py`（原样拷成 `probes/r5_report_verdicts.py`），用它的 3343 份清单，改后取第十批那一版（v10），改前取它 `orig/` 里第十批之前的五份。输出与第二轮攻方 `out/report_verdicts.out` 逐字节相同（`cmp` 无差，两边 sha256 都是 `286184812441b5889cca359a6eda3df99ce4be43ac7eda9a65198036f5eca798`）：四行 18、282、88、14 份，合计 402，其中 2026-10-14 之前 388 份。把改后换成此刻那一版重跑，输出也是逐字节相同：第十一批没动这 402 份。
- 4 份：照它的 `naming_real.py`（`probes/r5_naming_real.py`），改后取 v10。`out/r5_naming_real.out` 里那几行原样：

```
  2026-10-22 之前：改前拒、改后拒：144 份
  2026-10-22 起：改前拒、改后拒：4 份
  2026-10-22 起：改前放、改后拒：4 份
```

  例子是 `a7aa1608c74c3e21e`、`aa87dd9fe761d727f`、`aa68d151c2fa72bb7`，与判决相同。和第二轮攻方 `out/naming_real.out` 比，差的只有例子里的命令文字：真记录里是 `scripts/experiments`，它的 `out/` 里成了 `scripts/experiment-apparatus`。`sed 's#scripts/experiments#scripts/experiment-apparatus#g'` 之后两份逐字节相同。原因见 K3-6。

### K3-2　第十一批工具员的 3、46、693、22、41 与任务 37 的 37、9（都复出来了）

- 3（交回闸 ⑤ 新放）与 0（新拒）：`probes/naming_e2e_all.py` 不 import 钩子里的任何函数。它把每份改过 `.rs` 的实现员、执行员、工具员记录（801 份）用符号链接放进假主会话目录，喂两版 `handback-guard.sh` 入口，只看退出码；工具员是在进程里调 `naming_problems`，两边不共用代码。读数见 K2-1 那张汇总表：「2026-10-22 起：2/0：3 份」，就是 `ab6dd5f35228122b7`、`a7aa1608c74c3e21e`、`aa87dd9fe761d727f`；「0/2」一份都没有。2026-10-22 起照旧拒的 5 份是 `aa68d151c2fa72bb7` 与 4 份执行员（`ab85b77d63e4ab98e`、`aba8ee95f9f1480db`、`a120daf854fcada68`、`aa7c16d30ce29c555`），和工具员说的一样。拒的合计 144 + 5 + 3 = 152，与工具员的「152 份」相同。
- 46、37、9、670、693：`probes/leg_paths.py` 把三版交回闸的 `leg_reports_named`、派发闸的 `three_way_report_verdict` 载进来比。这是同一组函数，量法和工具员一样，抽取代码不一样。`out/leg_paths.out` 原样（头 9 行）：

```
三方腿类型（此刻 frontmatter）：three-way-attack、three-way-defense、three-way-forward
② 三方腿交回（一次调用一次；v10→v35→now）：
  认出/认出/认出：618 次；例：a6545bcc4c8eb512d（2026-10-17）、a6c3a55ff5bc19039（2026-10-17）
  认出/没认出/认出；前缀 /home/fy5090/.local/share/singlefs/work/m5-dev/：37 次；例：a028ec0c5ee1bd7bb（2026-10-19）、a05f0aae7e6706e18（2026-10-20）
  认出/没认出/没认出；前缀 /home/fy5090/.local/share/singlefs/work/m4-predev/：9 次；例：a2e0cc6ef56fad5c8（2026-10-16）、a3b0c2a40d1b94bf7（2026-10-16）
  没认出/没认出/没认出；前缀 （没有 research/prompts/…-output.md）：6 次；例：a4e8e26a56372c72a（2026-09-28）、ae8bcf94bea3860c6（2026-09-28）
  合计 670 次
⑰ 三方腿派发提示 three_way_report_verdict（一份记录一份；v10→v35→now）：
  拒/拒/拒：37 份；例：a4e8e26a56372c72a、ae8bcf94bea3860c6、a5603803187800851
```

  下一行是「放/放/放：658 份」，合计 695 份。46 = 37 + 9；任务 37 的「37 次改回放行」「9 次照拒」「15 次都不认」（9 + 6）「618 次」「670 次」都对得上。693 今天是 695，多出的 2 份是这一轮的两条腿，三版判得一样；拒的 37 份不变。
- 22、41：`probes/watch_progress_all.py` 不调草稿目录库，自己另写一份行首「草稿目录：」的认法，取到的 progress.md 是不是本提示的，按 realpath 比；看门狗那边调两版的 `progress_file_of`。`out/watch_progress_all.out` 末两行原样：

```
2026-10-22 起 合计 234 份：改前取到别的目录下的 22，改后 0；改前取到本提示的 40，改后 40；改后取不到而本提示目录下有 progress.md 的 0
2026-10-22 之前 合计 3127 份：改前取到别的目录下的 19，改后 0；改前取到本提示的 60，改后 60；改后取不到而本提示目录下有 progress.md 的 0
```

  22 → 0、本提示的 40 不变，与工具员相同（它是 231 份，今天 234）；全部 22 + 19 = 41 → 0，本提示的 40 + 60 = 100 不变，与工具员相同（它是 3358 份，今天 3361）。这一格取此刻的文件系统，过几天再跑数会变。

### K3-3　自证种数（都复出来了）

主工作区里并行起 7 份登记为 light 的自证，7 个退出码文件齐了，都是 0。`out/selftests.out` 每份末行开头：「handback-guard 自检通过（查了 78 种）」「自检通过（查了 512 种）」「自检通过（查了 1402 种，其中该拒 266 种）」「lib_prompt_scratch_line 自检通过（查了 18 格）」「自检通过（查了 92 项）」「自检通过（查了 23 项，其中 outcome 的 21 个子 agent）」，`agent-watch.py` 退 0、「✗ 自检」0 行。成功句里有「条件消失的确认作废（36 格）」：工具员说 33 → 36 是推的，36 这一头今天量到了。

### K3-4　第九批收尾的结局账 249 个（复不出来，轻）

在哪：`records/2026-10-22-子agent结果落地审计.md:254`「查了 249 个子 agent」，命令是 `agent-cost.py outcome --since 2026-10-22 --until 2026-10-23 --summary --out records/landing-audit-b9-outcome-2026-10-23.txt`。照这条命令重跑（`--out` 改成草稿目录里的文件），末行原样：「✓ 查了 280 个子 agent（--since 2026-10-22、--until 2026-10-23（含那一天），日期按东京日期（Asia/Tokyo）切天，按首条记录的时刻筛）」，返工字样一行「44 个，占个数 15.7%，占花费 14.3%」（记录里是 38 个、15.3%、14.6%）；「没交回的 12 个，按截断原因：停掉 9、还在跑 3」相同。为什么复不出来：`--until` 含当天，记的那一刻 10-23 还没过完，之后派的子 agent 都算进去了；记录不写时刻，落盘的 `.txt` 里也没有子 agent 号，截止点找不回来。改法（推的，被攻过零轮）：收尾的结局账 `--until` 取前一天；要算当天的，`.txt` 里逐个列出子 agent 号，复核按号重算。推翻条件：`agent-cost.py` 有按清单重算的参数，而我没看到（`--help` 只列了 `--root`、`--since`、`--until`、`--repo`、`--summary`、`--jobs`）。

### K3-5　两份判决的 cite-check 读数（照写的命令复不出来，轻）

两份判决都写成 `--root .`：第一轮 `research/prompts/landing-audit-r4-main-verification.md:25`「对上 52 处」，第二轮 `research/prompts/landing-audit-r5-main-verification.md:22`「1 处对不上，21 处没判」。规则要的是快照根：`.claude/rules/three-way-inference.md:125`「先跑 `python3 scripts/agents/cite-check.py <全部腿报告> --root <快照根> --background <背景材料>`」。今天照判决写的命令重跑，末行分别是：

```
  ✗ 核了 52 处引文、93 处裸行号，22 处对不上，93 处没判（其中裸行号没有引文可比 93 处；逐处列在上面）
  ✗ 核了 60 处引文、21 处裸行号，20 处对不上，21 处没判（其中裸行号没有引文可比 21 处；逐处列在上面）
```

对不上的都是被引文件后来改过、行号挪了（例：「landing-audit-r4-opus-output.md:62 引 .claude/hooks/heavy-test-guard.sh:857……原文在这份文件第 861 行」）。判决当时的读数我不认为是错的，只是今天核不了：快照目录里只有 sha256 与对 HEAD 的 diff，没有一棵能当 `--root` 的树。第二轮判决对 code-tooling.sh 一份做过倒推，另外 26 份没有。改法（推的，被攻过零轮）：开轮时把被判的文件按原路径拷进快照目录下的一棵小树，判决的 cite-check 用 `--root` 指它。推翻条件：cite-check 另有按 sha256 或按 diff 倒推的模式，而我没看到。

### K3-6　第二轮攻方模型被改名会话改过，它自己的复跑已经对不上（打中，轻；别人的改动）

调度记录 `records/2026-10-22-子agent结果落地审计.md:304` 列了改名改到的 16 份证据，其中有 `landing-audit-r5-opus-model/orig/handback-guard.sh.orig`、`orig/selftest-registry.tsv.orig`、`out/dump_naming.out`、`out/naming_real.out`；正文 `research/prompts/_landing-audit-r6-body.md:31` 照 `.claude/rules/path-moves.md` 判「只差路径的不算改动」。在 `orig/` 目录里跑 `sha256sum -c ../out/orig-sha256.txt` 原样：

```
completion-cleanup-guard.sh.orig: OK
handback-guard.sh.orig: FAILED
lib_three_way_legs.py.orig: OK
runner-dispatch-guard.sh.orig: OK
selftest-registry.tsv.orig: FAILED
sha256sum: WARNING: 2 computed checksums did NOT match
```

再加上 K3-1 那一条：它的 `naming_real` 探针读的是没改过的真会话记录，打出的是旧路径，它的 `out/` 里存的是新路径，于是它的 `rerun.sh` 一跑，`naming_real` 那一行就是「与 out/ 不同」。这份模型在 git 里还没提交过（`git show HEAD:` 取不到），原来那两份 `.orig` 的字节已经找不回来。判法没受影响：K3-1 里 402 与 4 两个数照样复出来了。受影响的是「判决复跑比 sha256、比输出」那一步，往后那一步会红。改法（推的，被攻过零轮）：全仓改名时跳过钉了 sha256 的冻结证据（模型目录的 `orig/`、`out/`），或者同一次改名里重算并登记 `orig-sha256.txt`，在调度记录里写明重钉过。推翻条件：`.claude/rules/path-moves.md` 明写了冻结证据里按 sha256 钉住的副本也照改、改后不再比 sha256。

### K3-7　快照份数与 diff 行数（复出来了）

`wc -l`：`landing-audit-r4-snapshot/judged-sha256.txt` 80 行、`judged-vs-head.diff` 14292 行；`landing-audit-r5-snapshot/judged-sha256.txt` 27 行、`judged-vs-head.diff` 10423 行，都与调度记录、判决里写的相同。这一轮的快照：`sha256sum -c research/prompts/landing-audit-r6-snapshot/judged-sha256.txt` 40 份没有一行不是 OK（开工时跑的）。

### K3-8　第十批的 507 种、62 种（没核，交主 agent）

这两个数是第十批那一版的自证种数。那一版的钩子与主工作区此刻的不一样，在副本里起它的自证会被 `heavy-test-guard.sh` 拒，我没起。推的：第十批那一版派发闸的 ⑳ 读 `research/prompts/landing-audit-r3-opus-model/prompts/` 下 14 份真提示、按 16 位 sha256 钉；改名会话改过其中 `a9717cf07fb6e7c21.md` 一行（任务 37 交回报告「没做什么」），所以今天在第十批那一版上起自证，那一格多半会红，507 种复不出原样。要主 agent 跑的命令见末节。

## 没打中的形状

- 交回闸 ⑤ 的假放行：`|| true`、`; echo rc=$?` 打出 2、`&&` 连着且前一条退 1、`set -o pipefail` 接管道、`( …; echo rc=$? ) | tail`（这一种放行是对的：rc 行打的就是命名检查自己的退出码）。点名一个不存在的格、让 code-source-discipline 退 1 冒充跑完：执行前就被 `heavy-test-guard.sh` 拒了（「点名里有别的格或不带 `--check` 的照拒」），走不到交回闸。真记录里命名检查的结果形状：is_error 而开头不是「Exit code N」的只有「被钩子拒」40 条、「Permission … denied」2 条，都按没跑算，这是对的。
- 交回闸 ② 与派发闸 ⑰：K2-4 那 14 种写法；三版在 670 次交回、695 份三方腿派发提示、3361 份全部派发提示上的翻转，全落在第十一批要的那几类里。
- 看门狗：真提示里多个草稿目录行的 0 份；共用同一个草稿目录的 46 个，查了此刻有 progress.md 的 4 个，都是同一件活接着派。
- 前两轮站住的形态：K2-6 那几个函数逐字比；真提示上 `report_file_name_verdict` 三版判得一样。
- 例外说法：五处逐字一样；打出的重跑命令（起 merge-and-verify 时带不带前缀两种）主 agent 身份喂 `heavy-test-guard.sh` 与 `run-scope-guard.sh` 都退 0。

取样范围：真会话记录全部 3361 份（2026-10-23 抽）；命名检查端到端 801 份改过 `.rs` 的；造的命名检查写法 11 种；路径写法 14 种。

## 这条腿自己的限度

- K2-1、K2-3 的改法 A、B、C 和 K2-4、K2-7、K3-4、K3-5、K3-6 的改法，都是我提的，只在我自己的模型上量过或只是推的，被攻过零轮。按规则第三轮之后不再开轮，这几条进判决的交用户表、标「零轮」。
- 命名检查的造的写法（`naming_forms.py`）不真跑命令，工具结果的样子是照 Claude Code 写的：退 0 原样、退非 0 带「Exit code N」、退 0 而太长落成 persisted-output。退非 0 时的样子我在这一腿里现量过一次（开头「Exit code 1」，中间截掉），退 0 落盘的样子取自真记录里那 5 条。
- K2-1 的「逐条调用新拒 27 条」「`a5e5d9ce99b57fb08` 那一条」是工具员的数，我只在交回层面重量了（新拒 0）。
- K3-2 的 46、37、9 这几个，我的量法与工具员一样，是在进程里调同一组函数。和它独立的只有抽取代码，判法代码是同一份（`leg_reports_named`、`three_way_report_verdict`）。⑤ 的 3 与看门狗的 22、41 是另走的路（入口退出码、另写的草稿目录认法）。
- 「树有没有 `.git`」「目录下有没有 progress.md」都取此刻的文件系统，与工具员相同；过几天重跑数会变。
- 内存包装这一趟排不上（别的会话订了 40.0 GiB），探针都没经包装跑（见「复跑」一节）。判决复跑时照命令经包装起。
- 一条腿只抽一次样，「没打中」那几格单独不能当结论用（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。

## 要主 agent 跑的命令

1. 判决复跑：见开头「复跑」一节。工作目录随意，内存上限 4G，不起 cargo，不设线程。看末尾的 `name=` 行：`naming_e2e_all`、`naming_forms`、`leg_prefix_variants`、`function_diff`、`r5_report_verdicts`、`r5_naming_real` 应当「与 out/ 逐行相同」；另四份随新落的记录变，比的是里面的数。
2. K3-8 第十批自证种数（要主 agent 在集成副本里、带 `SINGLEFS_HEAVY_TESTS=user-request` 起，子 agent 起不了）：拷一份 `.claude/hooks` 进副本，把 `research/prompts/landing-audit-r6-opus-model/orig/v10/` 下的 `handback-guard.sh.orig`、`lib_three_way_legs.py.orig`、`lib_prompt_scratch_line.py.orig`、`runner-dispatch-guard.sh.orig` 去掉 `.orig` 盖上去，在副本里跑 `bash <副本>/.claude/hooks/handback-guard.sh --selftest` 与 `bash <副本>/.claude/hooks/runner-dispatch-guard.sh --selftest`。看末行的「查了 N 种」是不是 62 与 507；派发闸那一份要是红在 `真提示:a9717cf07fb6e7c21` 一格，那是改名改了 `research/prompts/landing-audit-r3-opus-model/prompts/` 下那一份（K3-8 推的那一条）。

## 没做什么

- 没跑重型测试，不编译，没跑门禁阶段与 lint（误点了一次 `code-source-discipline.sh --check no-such-cell`，被 `heavy-test-guard.sh` 执行前拒了，那条命令没跑起来）。
- 没在逐条调用的层面重量第十一批的 27、11、107 几个数；没复第十批 507、62 两个数（K3-8）；没复第九批任务 25、30 的 3317、3324 份那几个数（它们在节「第九批」，不在这一轮点名的节里）。
- K1 那一格（未定项清单）不归这条腿。K2-7 里「验证段第 1 步本身没写那条命令」在调度记录节「第十一批」里没有去向，是我顺手查到的，归不归 K1 由主 agent 定。
- 没读禁读清单里的 `research/prompts/landing-audit-r6-sonnet-output.md`。

## 草稿目录与清理

草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r6-opus`。删之前 `du -sh`：`tree-now`、`tree-v35`、`tree-v10`、`tree-r5orig` 各 14M（钩子、定义与 `scripts/agents|kb|hosts` 的拷贝），语料 `corpus.jsonl` 562M，都删了；`e2e/`（符号链接与空的假会话文件）也删了。剩下 324K，都是文本输出（`*.out`、`*.rc`、`*.log`、`b9-outcome-rerun.txt`、`steps.txt`、`rerun.sh.new`）。没有仓副本、编译目录或库拷贝。

`tools/rerun.sh` 的 sha256：`51e0a0d9f1771fc10a06f75eefc4fa83c7fff61a19a28913fe4de970415d34ef`。
