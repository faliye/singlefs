# C439（gate-ok 前移让同轮早先写的行不再被判） 第二轮攻方腿报告（2026-10-15）

攻击面：正文「三、格」K1–K5，只攻庚二（含「强制跑过不前移」那条守卫）与 wu_cf，避开第一轮攻方已攻过的形（绕过门禁、交错改暂存区、人给 `GATE_BASE`、stamp、豁免键撞上与表涨）。只在草稿目录 `/home/fy5090/.local/share/singlefs/work/c439-r2-opus/` 下的临时 git 仓里造历史；不起 `.claude/gate.d/` 下任何阶段，不跑 `gate.sh` / `gate-staged.sh` 本体（拷冻结副本进临时仓、里层换成假的 gate.sh 跑），不跑 cargo、重型测试。

## 各格判定一览

「新加条款 1 / 2」指正文「五、跑前条款」另加的第一、二行（落法出局；复用一侧「该重跑的被复用」），「第一轮条款 2」指第一轮正文「五、跑前条款」第二行（一行新增的错引文永久不在窗口里）。例子名是模型目录 `out/` 下的文件。

| 格 | 对象 | 判定 | 量过 / 推的 | 打中的条款 | 例子 |
|---|---|---|---|---|---|
| K1 | 庚二落法 A（里层从外壳那棵树建 worktree） | 没打中：8 种形（新页、`add -N`、符号链接、规范副本、`eol=crlf`、`ident`、gitlink、cargo 前缀）下临时树的索引树与外壳那棵逐字节相同（两步之间 HEAD 被挪那一形只造了今天的建法，落法 A 建树不读 HEAD，推的）；磁盘上只多出被 `.gitignore` 挡着、从工作区拷进去的规范副本（K2 用到它） | 量过 | — | `k1-tree-identity.txt` |
| K1 | 今天的建法（第 56 行取 diff、第 59 行建树、第 87 行套上） | 打中，第一轮 bing 同族、换了形：第 56 与第 59 行之间 `git reset --soft HEAD~1` 再挪回，临时树缺最后一个提交的整页，收尾比树相同照前移；落法 C（里层报它的索引树、外壳比）在这一形上比得出不同 | 量过 | 新加条款 1（这种落法出局；落法 A、C 照判） | `k1-tree-identity.txt` 末段 |
| K1 | 守卫「强制跑过不前移」 | 打中判据本身：外壳第 20 行的 preflight 把 `--force` 摘掉，第 33 行转给里层的参数里没有它，经外壳根本起不了强制跑；守卫照「参数里有 `--force`」写就永远不触发，要写就得按 `PREFLIGHT_FORCE_GIVEN` 判，且落法要先决定转不转 `--force` | 量过（探针脚本，外壳本体被重型闸拒跑） | 不触发条款（不漏判，是守卫写在了不存在的路上） | `k1-force-preflight.txt` |
| K1 | 守卫「人给 `GATE_BASE` 不前移」 | 打中：调用方环境里留着弄坏开关 `LIB_CHANGED_PATHS_BREAK=fallback`，`gate_diff_base` 不认 `GATE_BASE`，窗口收窄到 HEAD；格退 77、整轮退 0，守卫不看它，staged-green 前移到含错引文的树，之后永远不进窗口；对照臂（不设开关）两趟都判红 | 量过 | 第一轮条款 2 | `k1-break.txt` |
| K1 | 窗口判法本身（庚二与今天共用） | 打中：页里有一个 NUL 字节，`git diff` 当二进制，`gate_added_lines` 交不出它的新增行；格退 77，staged-green 前移；去掉 NUL 之后二进制→文本仍按二进制出 diff，永久；wu_cf 的全量判在同一棵树上判红 | 量过 | 第一轮条款 2（只打窗口类，wu_cf 不中） | `k1-binary.txt` |
| K2 | 复用：被 `.gitignore` 挡着的登记输入 | 打中：`checker-tier-lkmm.sh` 登记的 `.claude/singlefs-ai-sop/scripts/lib.sh` 不在任何一棵树里，被判的临时树里却有（第 91–93 行从工作区拷进去）；它从 v1 换成 v2，`admission.py gate-reuse` 照判可跳过。本仓登记表里这样的路径只有这一条 | 量过（`gate-reuse` 本体） | 新加条款 2 | `k2-reuse.txt` 头一段 |
| K2 | 复用：带本次未跑的一趟照样前移 | 打中：crates 变化之后那一趟有一条变异「本次未跑」（点名的测试打 `NOT_RUN_NO_GPU_ADAPTER`），整道退 0，庚二照前移；下一趟 `gate-reuse` 判可跳过，那一条在新输入上一次没判。对照臂（有本次未跑就不前移）下一趟判要跑 | 量过（`gate-reuse` 本体；阶段那一趟的结局照它文件头摆进去，没真跑阶段） | 新加条款 2 | `k2-reuse.txt` 后两段 |
| K2 | 复用：`GATE_REPLAY_FULL=1` 经外壳 | experiment-replay 格先问复用（第 337 行）再看 `GATE_REPLAY_FULL`（第 364 行）：经外壳要全跑，被上一趟只跑了 exact 快行的绿树复用掉 | 推的 | 新加条款 2（候补，没造仓） | — |
| K3 | 两会话交错（落法 A） | 没打中：两会话开跑 / 收尾 6 种先后 × 四个事件前各插一步（不做、暂存错引文、退出暂存、提交、暂存无关的）= 3750 段，staged-green 的树里一次都没出现没判过的错引文 | 量过 | — | `k3-sweep-summary.txt` |
| K3 | 红挡住前移时的旁路 | `GATE_BASE` 收窄被守卫挡住；`--force` 到不了里层；绕过门禁提交、不带 `--staged` 跑都不挪 staged-green；漏的只有 K1 的弄坏开关那一条，与手写 `update-ref`（推的） | 量过 + 推的 | 同 K1 第四行 | — |
| K3 | 代价 | 本仓 staged-green 停在 2026-09-22，到 c1e9b324 有 143 个提交；庚二此刻这一格判红 361 行 | 量过（真跑抽出的格判法） | 不触发条款（记代价） | `k3-real-window.txt` |
| K4 | 射程 | 整个 `gate_diff_base gate` 换成 staged-green：archive-past-rounds 格从 9287 份红变 0 份（「上一轮」那一批几乎没了，判法实际失效），改动路径 239 → 16359；只给 quoted-result-lines 换，doc-experiments「只取一次、六格共用」要拆成两份；不认 `GATE_DIFF_BASE` 的定案字面不动，C528 现状那两句要改 | 量过（克隆上） + 推的 | 第一轮条款 5 不触发 | `k4-scope.txt`、`k4-archive.txt` |
| K5 | wu_cf 重跑之后要回改的行 | 本仓全部 20 个重跑 / 换登记事件里登记表可比的 5 个：一次换登记（E142 r24→r25）让 E142 页 73 行在「只比登记产物」下变红、同一提交没回改；「全部产物加归档」下 20 个事件共 1 行（同一提交回改了）。存量 452 / 101，条款 3 不触发 | 量过 | 新加条款 3 不触发（记代价 73 行一次） | `k5-rerun.txt` |
| K5 | 上游规则改的射程 | 同一句「这次改动」的还有 results-cited 第三道（写明射程与 quoted-result-lines 同一条）与 doc-registries 的 invariant-anchors 格 | 量过（grep） | — | 正文 K5 一节 |

## 复跑

模型目录 `research/prompts/c439-r2-opus-model/`。一条命令从头造全部临时仓、拷工具副本、建 c1e9b324 的 `--shared` 克隆，跑完与 `out/` 逐字比（这一次挂钟 193 秒）：

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/c439-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c439-r1/frozen --work <新的草稿目录>
```

这一次的原样输出（模型目录拷进仓之后从仓里那一份起的第三次独立复跑；`k5-rerun.txt` 在第二次复跑时只差同一秒两个事件的先后，改成按「时刻、提交号」排之后逐字相同）：

```
name=compare file=k1-binary.txt same=yes
name=compare file=k1-break.txt same=yes
name=compare file=k1-force-preflight.txt same=yes
name=compare file=k1-tree-identity.txt same=yes
name=compare file=k2-reuse.txt same=yes
name=compare file=k3-real-window.txt same=yes
name=compare file=k3-sweep-summary.txt same=yes
name=compare file=k3-sweep.txt same=yes
name=compare file=k4-archive.txt same=yes
name=compare file=k4-scope.txt same=yes
name=compare file=k5-rerun.txt same=yes
name=compare file=tools-sha256.txt same=yes
```

`tools-sha256.txt` 比的是复跑时从仓里现拷的 `scripts/gate/admission.py`、`scripts/gate/lib_source_digest.py`、`.claude/scripts/{preflight.sh,project_preflight.py}`、规范副本的 `preflight.{sh,py}` 与 `comment-only-change.py`：这几份不在冻结副本里，仓里一改它就报 `same=no`，那时 K2 的数要重看。冻结副本那 10 份与本仓此刻的同名文件逐份 `cmp` 相同，所以下文引行号写仓里的路径，`cite-check.py --root <仓根>` 核的就是冻结那一版。

文件 sha256：

```
fa2cf06b19253fd5663b02c90a142ce5ce78cb50a1f28482acb61c434ab457ed  k1-binary.sh
438222de8e25004f52822bfdef12718374871cd1fd2c959dd858034024c5eba4  k1-break.sh
df38a96fb975c9e76a3d69035a87f83f94bd9e0d92bd271b92443380133a0199  k1-force-preflight.sh
2ff51ed700040c535c33c8a5af4c9cf2f2712e5b40620fa3dbf42feeb7d7f15d  k1-tree-identity.sh
436aaf6884fd3505843f6b68de6f7b6a59e33eb97b617d715cdb374422e9f531  k2-reuse.sh
b62779a8e9d8ca4df97e1899bdfa5d129510c1c49fb87de330552c8ac97f9422  k3-concurrent.sh
6faff9c98c74ddf9feeb344175a2331276a23206aff20c83f8e7045c3c27ce6a  k3-real-window.sh
41846d419fd3d721e4b60878a616d217707b8411c3f2cac1270613be7cb26542  k3-sweep.sh
ee77fdd6d2e450877e342d067d7f166a76bb20a3390c01b0d26ab84f4eb11ea5  k4-scope.sh
915a0ca6a3452f28abe90b3903258434ea04ce8da78f7bb550fd557f327eada0  k5-rerun.py
c9ccad6dbd833d7001d7d28dc585c8d9f272f713362008b89ce11bab872c8c1e  lib.sh
f5fd44b56861d53406031403bda61f64ac1a329f6c9ca656db5fb18d503c519f  tools/rerun.sh
814520ddb27c98933326795774ccf54ac2a80a13c9177d264a4477d18c89a9fb  out/k1-binary.txt
6b89c658c7ec1a7cade65b67074b3ca8cb545392605b5c22eb18b21bc731bb13  out/k1-break.txt
0191b13eaa818e5637e82db75d05ef23f7802bcd5769b66639f68019d4771541  out/k1-force-preflight.txt
1f7b4ca88f058820c483fc45269b22369706456e8a4066ca650b58c73287434b  out/k1-tree-identity.txt
cc04ee7ce6eff8895c575f9dbbc2f76d8fc70dc5e5d5d79d890ba9b00d884320  out/k2-reuse.txt
1aa004eec6affce0c303e13b083766ae01295a419e1931a6e4633e0dce67b790  out/k3-real-window.txt
14982347c153f25abfb1e8c02c37a240ecac5fbd72b33c2990333cdf9b673729  out/k3-sweep-summary.txt
388dfac06a31732a82cb52cacce68a71983f55c7ba911123b1428fc7e34bf861  out/k3-sweep.txt
92fe5280576a8032af0ea8d8283d2e9ce9f778cb96f5a29915192d1b97b2644a  out/k4-archive.txt
92dd1701b4de7a128519849a785bc60e148ddd8266439597ccf6d159d76b08b5  out/k4-scope.txt
aae077292033bd57c6941e801e95b896c1a92036690256397b52e68670bffbb4  out/k5-rerun.txt
0cadfd9ef5fb6d9d4c7938bcf1d8fe3ca18bf3b0cd289784946308c53999f539  out/tools-sha256.txt
```

### 模型怎么造、哪些是冻结代码

- 每段历史一个新临时仓，提交时刻写死（`lib.sh` 那一行带 `# clock-times:allow`）。仓里一份产物 `research/results/e900-x.out`（两行对的 `E7RESULT`）、一行 replay 登记、一页实验页；错引文是 `E7RESULT name=beta value=2`（产物里是 `value=1`）。
- quoted-result-lines 的判法不重写：冻结 `doc-experiments.sh` 第 1625–1796 行（那一格 python heredoc 的正文）原样抽出来跑，抽完核行数与首行；窗口照第 128–139、1617–1622 行用冻结 `changed-paths.sh` 的 `gate_diff_base gate` 与 `gate_added_lines` 取。
- 庚二照第一轮判决的定义：基准是 staged-green 那个提交（没有就空树，第一轮 `c439-model.sh` 第 75 行同一种取法）；落法 A：从外壳 write-tree 的那棵树建 detached worktree 判；退 0（含格退 77，gate.sh 第 631 行记成本次未跑、不记失败）且没人给 `GATE_BASE`、没带 `--force` 就前移。
- 整轮门禁只有这一格；别的阶段在这几形上判什么没逐个核（doc-text 对二进制的处置见 K1）。
- 外壳本体没跑：把冻结 `gate-staged.sh` 拷进临时仓、里层换成假 gate.sh 的那一版被重型闸按「整轮门禁」拒了，没换写法绕；改成探针脚本只跑它第 1–3、19–20 行同样的几句（`header_and_preflight_lines_match_frozen=yes` 核了逐字相同）。
- `gate.sh --staged` 建临时树那几行（第 55–65、87–99 行）原样抽出来 source，`die` 换成打一行返回。
- K2 的复用判定用 `admission.py gate-reuse` 本体；阶段本身不跑，阶段那一趟判出什么照它文件头摆进去（各段注释写明出处）。

## K1：庚二的落法与守卫

### 落法本身：临时树是不是外壳那棵（没打中，量过）

两种建法各比一次（`k1-tree-identity.sh`）：today 是冻结 `gate.sh` 第 55–65、87–99 行原样跑（第 56 行取暂存区 diff、第 59 行在 HEAD 上建 worktree、第 87 行套上、第 91–99 行拷规范副本与查 cargo 前缀）；geng2_a 是庚二落法 A（从外壳那棵树包成的提交建 worktree，再照第 90–99 行拷、查）。比两样：临时树 `git write-tree` 与外壳的树；临时树磁盘上的文件与外壳那棵树 `git archive` 出来的文件。原样输出（`out/k1-tree-identity.txt`）：

```
=== case=staged_new_page outer_tree=3013029dcecc
  today index_equal=yes files_equal=yes 
  geng2_a index_equal=yes files_equal=yes 
=== case=intent_to_add outer_tree=3013029dcecc
  today index_equal=yes files_equal=yes 
  geng2_a index_equal=yes files_equal=yes 
=== case=staged_symlink outer_tree=d18d4f68e3ed
  today index_equal=yes files_equal=yes 
  geng2_a index_equal=yes files_equal=yes 
=== case=ignored_family_copy_in_working_tree outer_tree=d18d4f68e3ed
  today index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
  geng2_a index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
warning: in the working copy of '.claude/kb/new.md', LF will be replaced by CRLF the next time Git touches it
=== case=eol_crlf_attribute outer_tree=4091ce1e0f03
  today index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
  geng2_a index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
=== case=ident_attribute outer_tree=ceb1022e2872
  today index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
  geng2_a index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
=== case=staged_gitlink outer_tree=c39a6d1494f1
  today index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
  geng2_a index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
=== case=untracked_cargo_prefix_in_working_tree outer_tree=ceb1022e2872
die: 源仓有 .claude/cargo-command-prefix，而 HEAD + 暂存区里没有它：--staged 这一轮的 cargo 会不经前缀跑
  today: 中途 die
  today index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
die: 源仓有 .claude/cargo-command-prefix，而 HEAD + 暂存区里没有它：--staged 这一轮的 cargo 会不经前缀跑
  geng2_a index_equal=yes files_equal=no diff=Only in <临时树>/.claude: singlefs-ai-sop;
=== case=head_moved_between_diff_and_worktree_add（today 那一建法；最后一个提交带 L（.claude/kb/bypass.md），暂存区只动另一份；git reset --soft HEAD~1 插在第 56 行与第 59 行之间，建完再挪回）
  write_tree_after=d4be866662b9（收尾比树：与开跑时 d4be866662b9 相同就前移）
  today index_equal=no files_equal=no diff=Only in <外壳的树>/.claude/kb: bypass.md;Only in <临时树>/.claude: singlefs-ai-sop;
```

读法：
- 「HEAD 被挪」之外的 8 形，两种建法的索引树都与外壳那棵相同；磁盘上只多一个 `.claude/singlefs-ai-sop`，是「mkdir -p "$staged_tree/.claude"; cp -r "$source_repository_root/.claude/$staged_family" "$staged_tree/.claude/"」（`.claude/singlefs-ai-sop/scripts/gate.sh:92`）从源仓工作区拷进去的规范副本（本仓 `.gitignore` 第 8 行挡着它，`git ls-files .claude/singlefs-ai-sop` 0 条）。它不在任何一棵树里，却被重阶段读：K2 第一形。
- `eol=crlf`、`ident` 两形磁盘上相同，是因为 `git archive` 与检出同样按属性转换；过滤器（`filter=` 带 smudge）没造，推的：同理两边都经检出。
- cargo 前缀那一形两种建法都在第 96–99 行当场 die，判红、不前移，不漏。
- 落法 B（外壳自己建树再起不带 `--staged` 的 gate.sh）没单独造：建树那一步与 A 相同，差在规范副本与前缀那两段要外壳自己补；不补就是「里层跑的是工作区那份共享脚本、临时树里没有规范副本」，与 A 的差只在 SOP 那一份，推的。

### 今天的建法：两步之间 HEAD 被挪（打中，第一轮 bing 同族、换了形）

历史（`k1-tree-identity.sh` 末段）：L 写进 `.claude/kb/bypass.md`、绕过门禁提交；暂存区只改另一份；外壳 write-tree（「staged_tree="$(git write-tree)"」（`scripts/gate/gate-staged.sh:28`））；里层第 56 行取 diff 之后、第 59 行建 worktree 之前，别的会话 `git reset --soft HEAD~1`（撤回上一个提交、留着暂存，常见写法），建完再挪回（`reset --soft` 回原提交，或 `commit -C ORIG_HEAD` 原样再提交，暂存区不变）。补丁只碰另一份，在 HEAD~1 上套得上；被判的树缺整页 `bypass.md`。收尾「now_tree="$(git write-tree)"」（`scripts/gate/gate-staged.sh:38`）与开跑时相同，照前移。上面输出末两行就是它：`write_tree_after` 与开跑时同一棵，`today index_equal=no`，差的正是 `Only in <外壳的树>/.claude/kb: bypass.md`。

四句：
- 分不分辨臂：分。打今天的建法与第一轮的 bing；落法 A 不中（判的就是那棵树）；落法 C（里层建完报它的 `write-tree`，外壳比）在这一形上比得出 `index_equal=no`，拦得住。
- 系统当时看不看得到：看得到，临时树建完一次 `write-tree` 就知道。
- 满足的分句：新加条款 1「staged-green 前移到的树里有被判的那棵没有的行」，这种落法（里层照旧重套、不比树）出局；第一轮已判 bing 出局，这一形只是多一种交错。
- 跑前条款的改法（换落法）：A、C 在这一格上都不中。

### 守卫「强制跑过不前移」写在了到不了的路上（量过）

外壳先跑自己的 preflight：「preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}」（`scripts/gate/gate-staged.sh:20`），preflight 把 `--force` 摘进一个变量：「if [[ "$preflight_argument" == --force ]]; then PREFLIGHT_FORCE_GIVEN=1; else PREFLIGHT_ARGUMENTS+=("$preflight_argument"); fi」（`.claude/singlefs-ai-sop/scripts/preflight.sh:31`）；转给里层的是摘过的参数：「bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"」（`scripts/gate/gate-staged.sh:33`）。探针脚本（文件头与外壳第 1–3 行、preflight 那两句与第 19–20 行逐字相同）原样输出（`out/k1-force-preflight.txt`）：

```
header_and_preflight_lines_match_frozen=yes
=== probe <根> --force
  after_preflight_args=[<草稿根>/repo-force] PREFLIGHT_FORCE_GIVEN=1
  forwarded_to_inner=[]
=== probe <根> --force --other
  after_preflight_args=[<草稿根>/repo-force --other] PREFLIGHT_FORCE_GIVEN=1
  forwarded_to_inner=[--other]
```

所以今天 `gate-staged.sh --force` 的里层一道都不会被强制跑：条件不满足的阶段照样判红、不前移，不漏判。守卫「这一趟有阶段强制跑过时不前移」照字面在今天的外壳下永远不触发；它要有意义，落法得先定「外壳转不转 `--force`」：

| 守卫放在 | 看得到 | 看不到 |
|---|---|---|
| 外壳 | 调用方的 `GATE_BASE`（在外壳自己设之前看）；`PREFLIGHT_FORCE_GIVEN`（不是参数，参数里已经没有 `--force`） | 里层哪一道强制跑过（只拿到退出码）；哪一道、哪几行本次未跑（K2）；环境里的弄坏开关（下一节） |
| 里层 `gate.sh` | 强制跑过的阶段（第 252 行 `GATE_OK_HELD_BY`）、本次未跑的汇总 | 外壳设的基准与调用方给的基准是同一个变量 `GATE_BASE`，分不开，要另起一个变量；staged-green 是外壳的 ref，里层写它要改上游 |

四句：不分辨臂（今天、丙、庚二一样）；不是漏判；满足的分句：没有，跑前条款不触发；改法：守卫按 `PREFLIGHT_FORCE_GIVEN` 判（被攻过零轮，推的）。

### 守卫漏掉的收窄：环境里的弄坏开关（打中，量过）

`gate_diff_base gate` 认 `GATE_BASE` 那一句带一个弄坏开关：「if [[ "${LIB_CHANGED_PATHS_BREAK:-}" != fallback && -n "${GATE_BASE:-}" ]] \」（`scripts/gate/changed-paths.sh:76`）。冻结 gate.sh、lib.sh、gate-staged.sh 里没有一处拒绝带弄坏开关跑（`grep -n BREAK` 三份 0 命中）。历史（`k1-break.sh`）：staged-green 挂在根上；L 绕过门禁提交；暂存一份无关的；带着 `LIB_CHANGED_PATHS_BREAK=fallback` 跑一趟庚二的 gate-staged；提交；再不带开关跑一趟。对照臂不设开关。原样输出（`out/k1-break.txt`）：

```
=== arm=control
step=introduce
step=commit_L_without_gate
step=stage_unrelated
  gate: judged_tree=d53374989ed3 base_tree=77c278f39bc6
    window_base=14fcc01b1843c1ddaa8096c93b3a137333d27813 added_E7RESULT_lines=1
      ✗ 这次改动新增或改写的 kb 正文与历史节里有 1 行整行抄的产物行，在 scripts/experiments/replay.sh 登记着的 1 份产物里一份都找不到：
         .claude/kb/experiments/900-x.md:5  E7RESULT name=beta value=2
         → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；
                   正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。
    cell_rc=1
  staged-green 不动（格 rc=1）
step=commit
step=stage_unrelated
  gate: judged_tree=46679b4b3bc1 base_tree=77c278f39bc6
    window_base=14fcc01b1843c1ddaa8096c93b3a137333d27813 added_E7RESULT_lines=1
      ✗ 这次改动新增或改写的 kb 正文与历史节里有 1 行整行抄的产物行，在 scripts/experiments/replay.sh 登记着的 1 份产物里一份都找不到：
         .claude/kb/experiments/900-x.md:5  E7RESULT name=beta value=2
         → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；
                   正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。
    cell_rc=1
  staged-green 不动（格 rc=1）
=== arm=break_switch_left_in_environment
step=introduce
step=commit_L_without_gate
step=stage_unrelated
  gate: judged_tree=d53374989ed3 base_tree=77c278f39bc6
    window_base=HEAD added_E7RESULT_lines=0
      ! 这次改动新增或改写的 kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判
    cell_rc=77
  staged-green -> d53374989ed3（格 rc=77，整轮按退 0 记）
step=commit
step=stage_unrelated
  gate: judged_tree=46679b4b3bc1 base_tree=d53374989ed3
    window_base=c43981681a970352eea52626e12b8eae3c2e4b15 added_E7RESULT_lines=0
      ! 这次改动新增或改写的 kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判
    cell_rc=77
  staged-green -> 46679b4b3bc1（格 rc=77，整轮按退 0 记）
```

带开关那一趟基准退成临时树的 HEAD，窗口里 0 行，格退 77（「print('  ! %s kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判' % scope)」（`.claude/gate.d/doc-experiments.sh:1691`）），gate.sh 把 77 记成本次未跑、不记失败（「record_stage "$local_stage_name" "$local_stage_exit_code" 77 "本次未跑：阶段报了这一轮无对象可判（退出码 77），原因见上方它的输出"」（`.claude/singlefs-ai-sop/scripts/gate.sh:631`）），整轮退 0；没人给 `GATE_BASE`、没带 `--force`，staged-green 前移到含 L 的树；下一趟窗口从那棵起，L 永远不在里面。对照臂两趟都判红、不前移。

四句：
- 分不分辨臂：分。打所有经 `gate_diff_base gate` 取窗口的候选（今天、庚二、丙）；wu_cf 全量判不取窗口，不中。
- 系统当时看不看得到：看得到，变量就在外壳的环境里。
- 满足的分句：第一轮条款 2「那一行一次都没进 `gate_added_lines` 的输出」。
- 跑前条款的改法：换落法（A / B / C）照中，收窄发生在里层取基准那一步，与树怎么建无关。改法（被攻过零轮，推的）：外壳见任何 `*_BREAK` 非空就不前移，或 gate.sh 整轮拒绝在设了弄坏开关时跑。
- 可达性：弄坏开关只给证红用，要有人在终端里 export 过它再跑 gate-staged，或一份脚本带着它起；子 agent 的 Bash 调用之间环境不留。偏低，推的。它与第一轮的「人给 `GATE_BASE`」同族：人想收窄窗口而 `GATE_BASE` 被守卫挡住时，这是一个没挡的等价写法。

### 窗口判法的盲区：git 当二进制的页（打中，量过）

`gate_added_lines` 只收 diff 里以 `+` 开头的行：「diff_output="$(git "${quote_option[@]}" diff --no-renames --unified=0 --no-color --no-ext-diff --src-prefix=a/ --dst-prefix=b/ "$base" -- "$@")" || return 1」（`scripts/gate/changed-paths.sh:127`）。一份页前 8000 字节里有一个 NUL，git 把它当二进制，diff 只出一句 `Binary files … differ`，一行 `+` 都没有；未跟踪的同样被第 147 行 `grep -Iq` 跳过。历史（`k1-binary.sh`）：staged-green 挂在根上；新页 `901-y.md` 第一行是 L、后面一行带 NUL，暂存，跑庚二；提交；把 NUL 去掉，暂存，跑；提交；改一份无关的，跑。每一趟后面在同一棵树上再跑一次全量判（wu_cf 的判法）。原样输出（`out/k1-binary.txt`）：

```
step=introduce staged-green=77c278f39bc6
step=stage_page_with_nul_and_L
  gate: judged_tree=d8b9e8d55749 base_tree=77c278f39bc6
    window_base=14fcc01b1843c1ddaa8096c93b3a137333d27813 added_E7RESULT_lines=0
      ! 这次改动新增或改写的 kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判
    cell_rc=77
  staged-green -> d8b9e8d55749（格 rc=77，整轮按退 0 记）
  full_judge（wu_cf 的全量判，同一棵树）：
    window_base=全量 added_E7RESULT_lines=0
      ✗ 全量 kb 正文与历史节里有 1 行整行抄的产物行，在 research/results/ 的 1 份产物里一份都找不到：
         .claude/kb/experiments/901-y.md:1  E7RESULT name=beta value=2
         → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；
                   正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。
    cell_rc=1
step=commit
step=stage_remove_nul
  gate: judged_tree=eff854ebc918 base_tree=d8b9e8d55749
    window_base=d25ddfddb99a2692639b8d4efcb925e32a7eff45 added_E7RESULT_lines=0
      ! 这次改动新增或改写的 kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判
    cell_rc=77
  staged-green -> eff854ebc918（格 rc=77，整轮按退 0 记）
step=commit
step=stage_unrelated
  gate: judged_tree=54f0bd0e7032 base_tree=eff854ebc918
    window_base=777aa6269343461a015552325008fada08d1876a added_E7RESULT_lines=0
      ! 这次改动新增或改写的 kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判
    cell_rc=77
  staged-green -> 54f0bd0e7032（格 rc=77，整轮按退 0 记）
  full_judge（同一棵树）：
    window_base=全量 added_E7RESULT_lines=0
      ✗ 全量 kb 正文与历史节里有 1 行整行抄的产物行，在 research/results/ 的 1 份产物里一份都找不到：
         .claude/kb/experiments/901-y.md:1  E7RESULT name=beta value=2
         → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；
                   正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。
    cell_rc=1
```

去掉 NUL 那一趟仍是 0 行：前一版是二进制，二进制到文本的 diff 照样只出一句 `Binary files … differ`。此后 L 在 staged-green 的树里，窗口永远不含它；同一棵树上全量判当场红。别的文本阶段也不接它：doc-text 照它文件头第 73 行「前 8 KiB 里有 NUL 的文件仍按二进制整份跳过，在「没判的」里报个数」（`.claude/gate.d/doc-text.sh:73`）只报个数、不判红。

四句：
- 分不分辨臂：分庚二与 wu_cf（窗口类全中，全量判不中）；不分今天与庚二。
- 系统当时看不看得到：看得到，diff 输出里有 `Binary files … differ` 那一句，`gate_added_lines` 的 awk 不认它。
- 满足的分句：第一轮条款 2。
- 跑前条款的改法：换落法照中。改法（被攻过零轮，推的）：`gate_added_lines` 见到二进制的 diff 就退非 0，调用方照第 109 行「取新增行失败时：results-cited 与 quoted-result-lines 退回全量判」退回全量；这是共用库的改动，所有窗口类候选一起受益，判决可能照「打中不分辨臂」把它另立一笔账。
- 可达性：要一份 kb 页里真有 NUL（粘进二进制输出、按 UTF-16 存盘）。低，推的；本仓此刻这一类页的个数没数。

## K2：庚二与 `stage-must-run.sh` 的复用

复用判据：「#   整轮全绿意味着每一道都在那棵树上绿过 ⇒ **一条 ref 就够**，不必按阶段各存一条；ref 是覆盖语义，存不下第二条。」（`scripts/gate/stage-must-run.sh:34`）。判法在 `admission.py` 的 `gate_reuse`，拿两棵树比登记路径：「changed_paths, same_by_digest = registered_paths_differences(root, green, staged_tree, input_paths)」（`scripts/gate/admission.py:1597`）。下面两形都是这句断言不成立的地方，庚二只前移「判过的那棵」也补不上。原样输出（`out/k2-reuse.txt`）：

```
=== shape=ignored_registered_input（checker-tier-lkmm 登记的 .claude/singlefs-ai-sop/scripts/lib.sh 被 .gitignore 挡着）
  step=introduce lib.sh=v1 tracked=0
  reuse stage=checker-tier-lkmm.sh rc=0 verdict=must_run reason=还没有过整轮全绿的暂存树（refs/sop/staged-green 不存在）
  staged-green -> 5721e219f0f5（整轮全绿：这一道在含 lib.sh v1 的临时树上判过）
  step=working_tree_lib.sh_v2（里层照冻结 gate.sh 第 91–93 行把它拷进临时树）; stage_unrelated
  reuse stage=checker-tier-lkmm.sh rc=10 verdict=may_skip reason=与上次整轮全绿那棵树（34124c25aae1）在这几条路径上逐文件源码摘要相同（注释与空白不算）：litmus/ .claude/scripts/lkmm.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/ .claude/gate.d/stage-inputs.tsv
=== shape=not_run_rows_then_reuse arm=geng2_as_written
  reuse stage=checker-tier-crates-mutation-replay.sh rc=0 verdict=must_run reason=还没有过整轮全绿的暂存树（refs/sop/staged-green 不存在）
  staged-green -> 5721e219f0f5（整轮全绿，变异表每条都抓到）
  step=stage_crates_change
  reuse stage=checker-tier-crates-mutation-replay.sh rc=0 verdict=must_run reason=这几条路径与上次整轮全绿那棵树不同（按源码摘要比，注释与空白不算）：crates/a.rs 
  stage_run: 1 条变异点名的测试打 NOT_RUN_NO_GPU_ADAPTER 记本次未跑，整道退 0、全绿标记不写（scripts/mutation/crates-mutation-rows.py 文件头「本次未跑」）；gate.sh 记 PASS、本次未跑进汇总
  staged-green -> b7c2ea496d2e（退 0、没人给 GATE_BASE、没带 --force）
  step=stage_unrelated（有卡的那一趟）
  reuse stage=checker-tier-crates-mutation-replay.sh rc=10 verdict=may_skip reason=与上次整轮全绿那棵树（cc3f31c9f6f0）在这几条路径上逐文件源码摘要相同（注释与空白不算）：crates/ Cargo.toml .claude/gate.d/stage-inputs.tsv .claude/gate.d/checker-tier-crates-mutation-replay.sh
=== shape=not_run_rows_then_reuse arm=stop_on_not_run
  reuse stage=checker-tier-crates-mutation-replay.sh rc=0 verdict=must_run reason=还没有过整轮全绿的暂存树（refs/sop/staged-green 不存在）
  staged-green -> 5721e219f0f5（整轮全绿，变异表每条都抓到）
  step=stage_crates_change
  reuse stage=checker-tier-crates-mutation-replay.sh rc=0 verdict=must_run reason=这几条路径与上次整轮全绿那棵树不同（按源码摘要比，注释与空白不算）：crates/a.rs 
  stage_run: 1 条变异点名的测试打 NOT_RUN_NO_GPU_ADAPTER 记本次未跑，整道退 0、全绿标记不写（scripts/mutation/crates-mutation-rows.py 文件头「本次未跑」）；gate.sh 记 PASS、本次未跑进汇总
  staged-green 不动（有本次未跑）
  step=stage_unrelated（有卡的那一趟）
  reuse stage=checker-tier-crates-mutation-replay.sh rc=0 verdict=must_run reason=这几条路径与上次整轮全绿那棵树不同（按源码摘要比，注释与空白不算）：crates/a.rs 
```

### 被 `.gitignore` 挡着的登记输入（打中，量过）

登记表那一行：`checker-tier-lkmm.sh` 的输入里有 `.claude/singlefs-ai-sop/scripts/lib.sh`（`.claude/gate.d/stage-inputs.tsv` 第 38 行），因为判法 `lkmm.sh` 读它：「source "$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/lib.sh"」（`.claude/scripts/lkmm.sh:46`）。这份文件被 `.gitignore` 挡着，不在 staged-green 的树里，也不在这一次的暂存树里，两棵树比永远「没变」；全绿标记那一侧按 `git ls-files --exclude-standard` 列文件（`scripts/gate/admission.py:603`），同样列不出它。而被判的临时树里有它（K1 第一节，gate.sh 第 92 行从工作区拷进去）。上面头一段：lib.sh 从 v1 换成 v2（规范副本升级），`gate-reuse` 判 `may_skip`。本仓登记表里逐条 `git ls-files` 一个文件都列不出的门禁行路径，数出来只有这一条（`key=checker-tier-lkmm.sh path=.claude/singlefs-ai-sop/scripts/lib.sh tracked_files=0 on_disk=1`）。

四句：
- 分不分辨臂：不分。今天、丙、庚二用的是同一条 ref、同一个 `gate_reuse`，一起中；病根在登记表与「只比树」，不在 C439 的窗口基准。
- 系统当时看不看得到：看得到。实验那一侧已经有同形的拦法：「"""登记的路径里 git 一个文件都列不出来的那几条（写错的路径在比对里永远算「没变」，要单独拦）。"""」（`scripts/gate/admission.py:639`），门禁行没接。
- 满足的分句：新加条款 2「一道重阶段读的路径在被判的树与 staged-green 的树之间不同，复用判据却判「没变」」——被判的树里是 v2，staged-green 判绿那一趟是 v1，判据说没变。
- 跑前条款的改法：条款只说记成庚二的漏、同等出局；照四句它属于共用前提，换任何一个 C439 候选都还在。改法（零轮，推的）：门禁行也过一遍「列不出文件就要跑」，或把规范副本的版本当 `environment=` 进比对。

### 带本次未跑的一趟照样前移（打中，量过判据一侧）

`crates-mutation-rows.py` 文件头写着一条变异在本机没跑到被改坏的那一处时怎么记：「整道不因它判红，成功句写明抓到几条、本次未跑几条，全绿标记不写；一条抓到的都没有、只有本次未跑的退 77（选行模式照旧退 3 / 1）。」（`scripts/mutation/crates-mutation-rows.py:129`），并许诺「只有「抓到」复用，本次未跑的下一趟照跑。」（`scripts/mutation/crates-mutation-rows.py:131`）。gate.sh 只在强制跑过时扣住前移（「if [[ -n "$STAGE_FORCED_SUMMARY" ]]; then GATE_OK_HELD_BY+=("$name（强制跑过）"); fi」（`.claude/singlefs-ai-sop/scripts/gate.sh:252`）），本次未跑不扣；外壳只看退出码。上面后两段：

1. staged-green 挂在 crates 旧版上（整张表都抓到过）。
2. 改 crates，这一趟 `gate-reuse` 判要跑；阶段跑了，一条变异的点名测试打 `NOT_RUN_NO_GPU_ADAPTER`（没卡的机器、卡被占或驱动不在），记本次未跑，整道退 0、全绿标记不写（照上面两句摆进去，没真跑阶段）；整轮退 0，庚二照前移（`geng2_as_written`）。
3. 下一趟在有卡的环境里跑，只改了一份文档：`gate-reuse` 先看全绿标记（不在），再比 staged-green 那棵树，crates 没变，判 `may_skip`。那一条变异在新 crates 上一次没判，第 131 行的许诺落空。
4. 对照臂 `stop_on_not_run`（有本次未跑就不前移）：第 3 步判 `must_run`。

同形的还有：checker-tier-research-build-and-replay 的 mutation-tables-compile 格逐行报「没判」（第 555 行写进 `GATE_NOT_RUN_FILE`）、experiment-replay 格负载高时整趟退 77（第 320–323 行）；这两处没造仓，推的。

四句：
- 分不分辨臂：不分。今天的外壳同样前移；庚二的两条守卫（`GATE_BASE`、`--force`）不管本次未跑。
- 系统当时看不看得到：里层看得到（本次未跑汇总、`GATE_NOT_RUN_FILE`），外壳只拿到退出码，看不到——守卫放外壳就得另开一个通道把它交出来。
- 满足的分句：新加条款 2：这一道读的 crates 在被判的树（v2）与 staged-green 判绿那一趟真判过的输入（v1）之间不同，复用判据判「没变」。
- 跑前条款的改法：换落法照中。改法（零轮，推的）：外壳在本次未跑汇总非空时不前移——但 quoted-result-lines 这类「无对象可判」的 77 也进同一张汇总，照这样改，大多数趟都挪不动 staged-green（窗口一直长）；要分开「无对象」与「环境不够没跑」两种，今天的退出码与汇总分不开。

### `GATE_REPLAY_FULL=1` 经外壳（推的，没造仓）

experiment-replay 格先问复用：「stage_run_or_skip "$STAGE_REPOSITORY/scripts/gate/stage-must-run.sh" "复用上一次整轮全绿的判定" \」（`.claude/gate.d/checker-tier-research-build-and-replay.sh:337`），之后才看「if [[ "${GATE_REPLAY_FULL:-0}" == 1 ]]; then」（`.claude/gate.d/checker-tier-research-build-and-replay.sh:364`）。默认档只跑 exact 快行（慢行与计时行列进「没派」，那一句不写进本次未跑汇总），这一趟整轮绿就前移；之后经外壳设 `GATE_REPLAY_FULL=1` 要全跑，复用判据只认 `SINGLEFS_GATE_FULL`，判可跳过，慢行与计时行一行没跑。第 374 行给的出路是直接跑 `gate.sh`（没有 `SINGLEFS_STAGED_TREE`，不复用），照出路走不漏；漏在经外壳走。

## K3：多会话

### 两会话同时跑庚二（没打中，量过）

`k3-sweep.sh`：一个暂存区、一个 HEAD、一条 staged-green；会话甲、乙各跑一趟庚二的 gate-staged（开跑：write-tree、读 staged-green；收尾：判那棵树，绿或无对象就前移到它，后写的覆盖先写的）。开跑 / 收尾的先后取全部 6 种，四个事件前各插一步别人的暂存区操作（不做、暂存错引文、退出暂存、原样提交、暂存一份无关的），末尾再跑一趟不交错的。共 6 × 5^4 = 3750 段，每段一个临时仓，31 并行，挂钟 27 秒（先跑 124 段估过：1 秒）。原样输出（`out/k3-sweep-summary.txt`）：

```
name=k3_sweep_count key=leak=yes count=0 of=3750
name=k3_sweep_count key=wrong_line_in_staged_green=1 count=0 of=3750
name=k3_sweep_count key=judged_red_ever=1 count=2118 of=3750
```

`wrong_line_in_staged_green=1` 0 段：staged-green 从没挂上含没判过的错引文的树。读法：每次前移的都是判过的那棵，判它时的基准是更早的一次 staged-green；覆盖只让基准变旧（窗口变宽），不让它跨过没判的行。

装置自己的错，照实记：第一次全格扫报了 193 段漏，查下来是模型里各进程共用同一份抽出来的格脚本，被并行重写到一半，python 跑了空文件退 0。改成每个进程各抽一份（抽完核行数与首行）之后重扫，0 段。第一次的结果留在草稿目录 `k3-sweep-racy-model.out`，不当证据。

### 红挡住前移时的旁路（量过 + 推的）

别的会话在暂存区留着一行红，谁跑庚二都红、staged-green 挪不动：
- 人给 `GATE_BASE` 收窄：庚二的守卫挡住前移，窗口照旧从老 staged-green 起，不漏（第一轮已量）。
- `--force`：到不了里层（K1），红照红。
- 绕过门禁提交、只跑不带 `--staged` 的整轮：都不挪 staged-green，下一次庚二的窗口包着它们，不漏。
- 把那一行先退出暂存、跑绿、再加回：被判的树里没有它，staged-green 挂的也没有，加回后照判（上面 3750 段里的 `U` 那一类），不漏，代价是动了别的会话的暂存。
- 环境里留着弄坏开关：漏（K1）。
- 手写 `git update-ref refs/sop/staged-green …` 解卡：守卫拦不住，推的，没造仓。

### 代价：挪不动时窗口有多宽（量过）

本仓此刻就是「挪不动」的样子：`k3-real-window.sh` 在 c1e9b324 的 `--shared` 克隆上（工作区干净）照庚二取窗口、真跑抽出的格判法。原样输出（`out/k3-real-window.txt`）：

```
name=k3_real head=c1e9b324c737 staged_green=e2ae1837615a staged_green_date=2026-09-22 commits_since=143
    window_base=e2ae1837615ad519759e356464ac634a0bd3db7a added_E7RESULT_lines=16038
      ✗ 这次改动新增或改写的 kb 正文与历史节里有 361 行整行抄的产物行，在 scripts/experiments/replay.sh 登记着的 241 份产物里一份都找不到：
    cell_rc=1
```

staged-green 停在 2026-09-22，之后 143 个提交；庚二此刻要判的窗口里 16038 行以 `E7RESULT` 开头的新增行（含 `research/prompts/` 下的，格判时再去掉），判红 361 行。第一轮在 f111c3f3 上用重写的比法量的是 355 行；这里是格判法本体，数不同，以这一次为准。头一趟引入庚二就要先清这 361 行或登记。

## K4：庚二的射程

三种基准在同一棵树（c1e9b324 的克隆）上的改动范围，与 archive-past-rounds 那一格的判定（真跑冻结 `scripts/kb/archive-past-rounds.py --check`，`GATE_BASE` 分别给两个基准）。原样输出（`out/k4-scope.txt`、`out/k4-archive.txt`）：

```
name=k4_scope label=today_N base=c1e9b324c737 changed_paths=0 archive_past_round_files=11095
name=k4_scope label=today_S base=ceac9b0fc09e changed_paths=239 archive_past_round_files=10864
name=k4_scope label=geng2 base=e2ae1837615a changed_paths=16359 archive_past_round_files=10
base=ceac9b0fc09e   ✗ 还留着上一轮及更早的实验记录 9287 份（本轮的与判决不算）
base=ceac9b0fc09e rc=1
base=e2ae1837615a   ✓ 没有上一轮留下的实验记录
base=e2ae1837615a rc=0
```

`archive_past_round_files` 一列是只按「已跟踪、相对基准没改、不是判决与 abandoned-rounds」粗数的，没减「还被代码当输入」与「被登记表点名」；判定以 `k4-archive.txt` 那几行为准。

| 射程 | 动什么 | 漏改的那条路留下什么 |
|---|---|---|
| 只给 quoted-result-lines 换基准 | doc-experiments 开头「改动范围在这一道里只取一次、六格共用」（`.claude/gate.d/doc-experiments.sh:12`）要拆成两份；code-tooling 的 change-range-single-source 格要认第二种取法（推的） | 别的五格照旧按 HEAD~1 / HEAD 判，绕过门禁的提交在它们那里照第一轮 H1 永远判不到；C439 只修了这一格 |
| `gate_diff_base gate` 整个换 | 用它的格全跟着换（正文第二节列的那一串） | archive-past-rounds：判「上一轮」的那一批是「相对基准没改的」（「dirty = set(p for p in git("diff", "--name-only", base_of(root), "--", *DIRS, root=root).splitlines() if p)」（`scripts/kb/archive-past-rounds.py:111`）），基准退到 2026-09-22，那之后进仓的记录全算本轮，此刻 9287 份红变 0 份，归档规则实际不判；multipath-registry 的只缩不涨「与基准提交比」（`.claude/gate.d/doc-experiments.sh:1834`），基准变旧之后，在老 staged-green 之后删掉的登记行可以再加回来不红（此刻两棵树上表一样，0 行，推的） |
| 不带 `--staged` 那条路改不改 | 不改：两条路窗口差得更远（此刻 0 条路径 对 16359 条） | 不漏：不带 `--staged` 的整轮不挪 staged-green，之后的庚二照判；改：`changed-paths.sh` 没设 `GATE_BASE` 时读 staged-green，回退次序（第 10、76–84 行）变了 |

C528（两条门禁路判的改动窗口不同） 现状那两句要改：「#           diff_base，不带 --staged 的整轮用这里的回退次序；全推出去了时前者是 HEAD~1、后者是 HEAD，两条路判的窗口不同（C528）。」（`scripts/gate/changed-paths.sh:13`）。用户 2026-09-24 的定案「#           不认上游 gate.sh 导出的 GATE_DIFF_BASE（用户 2026-09-24 定）：带 --staged 那条路在里层把 GATE_BASE 设成上游」（`scripts/gate/changed-paths.sh:12`）：庚二由外壳经 `GATE_BASE` 交基准，不用 `GATE_DIFF_BASE`，字面不动；但「带 --staged 那条路在里层把 GATE_BASE 设成上游 diff_base」这半句的事实变了（推的）。第一轮条款 5 不触发。

## K5：wu_cf 的两处没量的

### 产物重跑一轮之后要回改多少行（量过）

取样：本仓克隆上改了 `scripts/experiments/replay.sh` 或原地改了 `research/results/*.out` 的每一个提交（全取，不抽样，20 个）。对父提交树里正文的整行抄产物行（射程照格第 1630–1644 行，全量判下历史节整节不判），按两种对照集各判一次「在父提交找得到、放到这个提交的对照集里找不到」。`registered` 只比 replay 登记着的产物（加登记着而已出树的最后一版），`all` 比全部产物加全部归档。`replay.sh` 在 503b80e2 才搬到 `scripts/experiments/`，之前的父提交登记表读不到（`registry_rows_at_parent=0`），这些事件的 `registered` 列不作数，作数的 5 个。原样输出（`out/k5-rerun.txt`）：

```
name=k5_event commit=f79f1f013f59 date=2026-08-30 quoted_body_lines=0 products_touched=15 registered_red_before=0 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=eeddad1c193f date=2026-08-31 quoted_body_lines=0 products_touched=11 registered_red_before=0 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=05f64d97d46c date=2026-08-31 quoted_body_lines=0 products_touched=11 registered_red_before=0 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=507e9ad3e14d date=2026-09-05 quoted_body_lines=0 products_touched=5 registered_red_before=0 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=04b4f3e5cfac date=2026-09-06 quoted_body_lines=0 products_touched=5 registered_red_before=0 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=648745b8c61d date=2026-09-07 quoted_body_lines=0 products_touched=7 registered_red_before=0 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=3419de718638 date=2026-09-10 quoted_body_lines=0 products_touched=13 registered_red_before=0 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=119f6c151148 date=2026-09-10 quoted_body_lines=26 products_touched=3 registered_red_before=26 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=d704051dc39e date=2026-09-18 quoted_body_lines=564 products_touched=3 registered_red_before=564 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=8186d5b59753 date=2026-09-21 quoted_body_lines=596 products_touched=6 registered_red_before=596 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=67f2c78518da date=2026-09-24 quoted_body_lines=705 products_touched=42 registered_red_before=705 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=110 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=f669c521da56 date=2026-09-25 quoted_body_lines=786 products_touched=65 registered_red_before=786 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=80 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=bc57af7a9894 date=2026-09-27 quoted_body_lines=857 products_touched=34 registered_red_before=857 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=1 all_fixed_in_same_commit=1 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=dbcc0d2f51d9 date=2026-09-28 quoted_body_lines=857 products_touched=34 registered_red_before=857 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=0 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=503b80e2a4fc date=2026-10-11 quoted_body_lines=1704 products_touched=0 registered_red_before=1704 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=101 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=0
name=k5_event commit=6aa74a765d4f date=2026-10-11 quoted_body_lines=1702 products_touched=67 registered_red_before=379 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=101 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=240
name=k5_event commit=e0bddcba7922 date=2026-10-11 quoted_body_lines=1702 products_touched=0 registered_red_before=379 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=101 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=240
name=k5_event commit=f778192d3c29 date=2026-10-11 quoted_body_lines=1702 products_touched=14 registered_red_before=379 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=101 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=240
name=k5_event commit=1b6f98657c2f date=2026-10-14 quoted_body_lines=1702 products_touched=0 registered_red_before=379 registered_turned_red=73 registered_fixed_in_same_commit=0 registered_left_red=73 all_red_before=101 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=240 registered_turned_red_by_page=.claude/kb/experiments/142-新池新建文件的干跑.md:73
name=k5_event commit=30c245b098a5 date=2026-10-14 quoted_body_lines=1702 products_touched=17 registered_red_before=452 registered_turned_red=0 registered_fixed_in_same_commit=0 registered_left_red=0 all_red_before=101 all_turned_red=0 all_fixed_in_same_commit=0 all_left_red=0 registry_rows_at_parent=241
name=k5_total events=20 registry_valid_events=5 registered_turned_red=73 all_turned_red=1
```

读法：
- 作数的 5 个事件里一次换登记（1b6f9865，E142 从 r24 的产物换到 r25 的，同一提交里 r25 的产物还没进仓，下一个提交 30c245b0 才进）让 E142 页 73 行在登记对照下变红，同一提交没回改（`registered_fixed_in_same_commit=0`）。wu_cf 下表不许涨，这 73 行要在那一趟当场回改，或者那一趟一直红。
- `all` 对照下 20 个事件一共 1 行变红，同一提交就回改了。
- 存量：登记对照下 452 行（30c245b0 父提交那一行 `registered_red_before=452`，与第一轮的 455 同一量级）、全部对照下 101 行。一次换登记 73 行 < 存量，新加条款 3 不触发；记代价：每换一次登记，引那份产物的整页当场回改。
- 历史被压成了大批提交（`git log -- scripts/experiments/replay.sh` 全史 3 个），一个提交里常常换了好几轮，每「轮」的量分不出来；5 个作数的事件太少，73 是一次的数，不是上界。

### 与共享规则冲突：改上游规则的射程（量过 grep，判断是推的）

共享规则那一句：「⇒ **正文引产物的检查按「这次改动」判，不按全仓判。** 当天写进正文的数要和当天的产物逐字对得上；」（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:38`）。项目里照这一句只判「这次改动」的，除了 quoted-result-lines（第 1601 行射程 4），还有：
- doc-experiments 的 results-cited 第三道，写明跟它同一条：「  # 只判这次改动新增或改写的点名行（射程与 quoted-result-lines 格同一条）：早先的点名是历史；别的会话还没提交产物的实验页也不由这一次判。」（`.claude/gate.d/doc-experiments.sh:346`）。
- doc-registries 的 invariant-anchors 格（第 2721 行起）：「# 这次改动新增或改写的行：diff 的 `+` 侧。基准那一版没有这个文件时（新建，含未跟踪），整份都算新写。」（`.claude/gate.d/doc-registries.sh:2774`）。它判的是决策锚点，不是引产物，不在那一句的字面射程里。

所以 wu_cf 要走，上游那一句至少要给「整行抄的产物行」开一个口子；results-cited 第三道写明与它同射程，跟不跟要一起定，不跟就是两格对同一批正文用两种射程（推的）。

## 自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的格上 |
|---|---|---|
| 庚二用落法 A 或落法 C，不用今天的重套 | K1 两步之间 HEAD 被挪 | 量过：A 不中（判的就是那棵）；C 比得出 `index_equal=no` |
| 外壳的守卫按 `PREFLIGHT_FORCE_GIVEN` 判，并先定外壳转不转 `--force` | K1 强制跑过那条守卫 | 推的 |
| 外壳见任何 `*_BREAK` 非空不前移（或 gate.sh 整轮拒跑） | K1 弄坏开关收窄 | 推的；`k1-break.txt` 的对照臂只证明不设开关时判得到 |
| `gate_added_lines` 见二进制 diff 退非 0，调用方退回全量 | K1 二进制页 | 推的 |
| 门禁行的登记路径列不出已跟踪文件就判要跑（照实验那一侧第 638–640 行） | K2 被挡着的登记输入 | 推的 |
| 有「环境不够没跑」的本次未跑就不前移 staged-green | K2 本次未跑之后复用 | 量过：`stop_on_not_run` 臂下一趟判 `must_run`；代价没量：今天的汇总分不开「无对象」与「没跑」 |

## 没打中的形状

| 对象 | 试过的形 | 取样 | 结果 |
|---|---|---|---|
| 落法 A 的树 | 新页、`add -N`、符号链接、被挡着的规范副本、`eol=crlf`、`ident`、gitlink、cargo 前缀 | 每形一段 | 索引树全同；cargo 前缀判红不漏 |
| 庚二两会话 | 6 种先后 × 5^4 插法 | 3750 段全格 | 0 漏 |
| 庚二的旁路 | `GATE_BASE`、`--force`、绕过门禁提交、不带 `--staged`、退出暂存再加回 | 上面全格与第一轮的量 | 只有弄坏开关漏 |
| K4 只缩不涨 | multipath-registry 的滞后表在 staged-green 与 HEAD 之间删过的行 | 本仓两棵树 | 此刻 0 行，形是推的 |
| 没造仓、推的 | 「## 历史版本」标题挪到错引文下面（历史行变正文，diff 里只有标题那一行，窗口类判不到、全量判得到）；页是指向 `research/prompts/` 的符号链接（diff 里是链接目标的文字）；新克隆里没有 staged-green（照第一轮取空树，全量判，只是代价）；手写 `update-ref` 解卡 | — | 前两形与二进制页同类，没量 |

## 这条腿自己的限度

- 外壳本体没跑（重型闸拒），`--force` 那一格是探针；`gate.sh --staged` 只跑了建树那几行；整轮门禁只建模了 quoted-result-lines 一格，别的阶段在这几形上判什么没核（二进制那一形只核了 doc-text 的文件头）。
- K2 的阶段一侧（变异本次未跑、lkmm 读 lib.sh）照文件头摆进去，没真跑阶段；`admission.py` 等 7 份是从仓里此刻拷的，不在冻结副本里，仓里改了要重跑。
- K3 只有一行错引文、两个会话各一趟、四个插点；三个会话、每会话几趟没扫。
- K5 作数的事件只有 5 个（历史被压成大批提交），73 行是一次的数；「全部产物」对照没管归档前就被覆盖、从没进过仓的产物。
- 可达性的判断（弄坏开关、NUL 字节）都是推的，没在本仓历史里数过发生过几次。

## 没做什么

- 不判辩方那几格（K6 与辩方替庚二、wu_cf 答的 K1、K4），不替主 agent 采纳；副本与模型上的数不算入库装置上的数。
- 没起任何门禁阶段、`gate.sh`、`gate-staged.sh`，没跑 cargo、重型测试。
- 要主 agent 跑的命令：没有必须的；要复核照「复跑」一节那一条（挂钟约 3 分钟）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/c439-r2-opus/` 里的临时仓、克隆与复跑目录都已删；留着的是工具副本（`tools/`）、模型草稿（`model/`，与仓里模型目录同一份）、各次输出与日志（含不当证据的 `k3-sweep-racy-model.out`）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/c439-r2-opus-output.md --root /home/fy5090/code/singlefs` 末行（退出码 0；「没判」的 10 处是模型输出里临时仓的路径与带嵌套「」的整行引文，后者引号里套引号、脚本按裸行号认，行号是现查的）：

```
  ✓ 核了 24 处引文、10 处裸行号，对上 24 处，没判 10 处（其中裸行号没有引文可比 6 处；逐处列在上面）
```
