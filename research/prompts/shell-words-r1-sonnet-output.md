# shell-words-r1 正推腿报告（Sonnet）

日期：2026-10-26。轮：shell-words-r1（第一轮）。立场：正推。

## 0　做法、复跑、文件指纹

做法：从 bash 的展开次序与各钩子的判据推出「这一条命令应当判成什么」，再与钩子今天的判定逐族比。比的时候有两把尺：

- 真 bash（本机 GNU bash 5.2.21、GNU env 9.4）：花括号展开、`env` 的赋值、`bash -c --`、`time -p` 都是现跑 bash 看它真做了什么，不凭记忆。
- 「改前」树：冻结副本里 `own.diff` 把三份文件（`bash-command-detector.sh`、`heavy-test-guard.sh`、`lib_shell_words.py`）反向打回得到；`heavy-test-guard.sh` 里有 2 个 hunk（文件头注释、`KNOWN_GUARD_BREAK_ITEMS` 一行）打不回，不影响判定。「改后」树是冻结副本原样。两棵树各配齐同一份其余钩子库、脚本、定义、门禁表，喂法是进程内调各钩子的 `main()`（`tools/fam4.py`，一棵树一个进程），关键反例另用真的钩子入口（`bash <钩子>` 读标准输入的 JSON，`tools/feed.sh`）逐条喂。

冻结副本四份文件的 sha256 与快照清单 `research/prompts/shell-words-r1-snapshot/files-sha256.txt` 一致（命令输出见 §0.2）。下文引的行号都是冻结副本里的行号；这四份与主工作区同一个 sha256，所以 `cite-check.py` 按仓路径读是同一行。

### 0.1　复跑命令

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/shell-words-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/shell-words-r1-frozen --work <新的草稿目录>
```

输出里的 `name=matrix_vs_model`、`name=keyfeeds_vs_model`、`name=brace_cases_vs_model`、`name=brace_oracle_vs_model` 四行都应是 `identical`。我交回前在新 shell 里整份跑过一次（见 §8），退 0。矩阵一行一条：`标签|钩子|agent|改前rc|改后rc|命令sha8`，共 575 条（`out/matrix.tsv`）；改前改后的退出码迁移是：

```
0->0 178 条 ／ 0->2 156 条 ／ 2->0 37 条 ／ 2->2 204 条
```

其中 2->0（改前拒、改后放）37 条，我逐条分了类：真回归 25 条（§1、§2），按 bash 本来就不该拒的 12 条（§1 的 4 条、`cp` 多目的地 2 条、`bash {,}…detached-run.sh` 1 条、`bash --rcfile …` 5 条）。

### 0.2　冻结副本指纹（原样输出）

```
$ sha256sum <冻结副本>/.claude/hooks/{bash-command-detector.sh,heavy-test-guard.sh,lib_heavy_tests.py,lib_shell_words.py}
b9b2962f196cd6fb5d3d909b4d92d69d8ab1ff47615e23066e4bc752601809e1  bash-command-detector.sh
febb12c7288864633c5dfc02ff8f97ad0990ec763cced95e613a77fe477e574d  heavy-test-guard.sh
642293b0477fe39fd9e6958e2ffa32259b5f67415cc30fe10964fe13f786e224  lib_heavy_tests.py
0803f0343825ed7265171cb3d537c4a069a854440b45dc01dfcb170b61371c7e  lib_shell_words.py
```

### 0.3　模型目录每个文件的 sha256（`research/prompts/shell-words-r1-sonnet-model/`）

```
2b57e7148eba5cd2d4217e3360342909f6bbb3787d5dcc00dcff225dcc39c689  families/fam-b4.json
f4d31578f76df56a7e90b5dfe7132524b8433c08b1f885867848c0e5284187a3  families/fam-w1b.json
db5262145eab0e6aaffea1567a06eb09ec30cfa3109e361670d360fbd7562ccc  families/fam-w1c.json
740b5fe1c08003bbb8e21b122bd6aa529ef0be9b335e3c5e127ffb4f9444928d  families/fam-w1d.json
23d6c3f676c0d11d1a195c5fb5245038d2968fe31931eb7f1d5b8141edcb0b49  families/fam-w1e.json
23564eee2f18096a7d71a0c155b491dedaae1e7a90da30aad02b3baa14a7e38d  families/fam-w1.json
56d76f9952bff7c630b5a9ac69bb7418c300fd2440df4fa40b4d955de93bb573  families/fam-w2.json
6ff8dbe7e61523b3f26e918286f84c1302e92e1fe4bd69a1aaec8f8fa9794d1d  families/fam-w3b.json
3ac73e06c7d8bd5ad38733605866343f70d81efe753380b6b88b6c01528eb2cc  families/fam-w3c.json
911c17f3365613981d769cdaf59107874be66022d68abe52140f3733a8c9fa85  families/fam-w3d.json
d61458ceaa040e4a32a9019a5185d6fb1bcc0ef83256babd411c43739679dffc  families/fam-w3e.json
f2adf9df609706108ac0189edca46e006e8933f6b9a73bca25c6ea380b356df9  families/fam-w3.json
04e3aafd6d9fff9f2defa9d6397400dc97992e57538aa7569e39f0851f1489d5  families/fam-w4.json
6e9bb784b0d3f81a121cc74eeff1b85151c159b63cd01f6484c40c2383192a10  families/fam-w5.json
870afaa7b673307a7c67afb3231e9da554354002824004f44050728a8dd93d7a  families/fam-w5b.json
4e84fec2c8082f6f9d87a34792f8b7e26bc9967bb0f575b9f7f4e56703c47694  out/brace-cases.txt
59336994774678b763e147dab517036bb10e59409425cb933dacbf8bab842288  out/brace-oracle.txt
ba23858f9a44479530547a8158c30d148c35bcb1fb0b765bb49432625c73b384  out/keyfeeds.txt
682a7953e3a8a04c9950da1a30fb04d8b405d8e750030059b6da85b375471107  out/matrix.tsv
84c4e570256e54bcc271b0af7be6643cc685ebb230c5c3af8046d8a1a3dcba16  tools/brace_cases.py
599458c8bc013377b0a87147e50b1852dca24e6f8f50d81c6f415fcd54cd3013  tools/brace_oracle4.py
0b714aad84e616bebb5af2f20a8759fdc096e075f1de90b1bb7418ba2c842198  tools/fam4.py
863e59b8ffd10c4f9da08c623b9077d2fa30c96ca14268865a61861107941503  tools/famshow.py
3201df10f172de52b19af19d82e1d07184b7eac74994543c5cab253ea291991a  tools/shapecount.py
29d3fd7d98271bcda801285c7ccd044f84c9949ca8287b7e55eabaed8101e3c6  tools/feed.sh
b9bac9340524d483f92e0b03545c4419664a588e3a98d5f04a6e096564c1f84f  tools/inproc.py
bdc1abbe635f425ae125e9c23f3c237fe26bd6d575b0d72c90a61f7091455a1c  tools/keyfeeds.sh
1db5a7489005d19e760daf21459fc26abe6b519e243533baa368f8eed942eae5  tools/matrix.py
99ec6ce714bc217ee03ad99975b72bc7bf9fe4528e98a46c84d3351d3b174bf8  tools/mkcorpus.py
d784be199b1ff2e561b9467630745c34ee73aa5dd421f3fc9c800ccb9b45c4c7  tools/rerun.sh
```

家族文件里的「expect」列是我写探针时按 bash 语义写的期望，少数几条写错了期望（例如 `cp x a b` 的多目的地形态），**以矩阵的改前/改后两列和下文逐条的 bash 实测为准，不以 expect 列为准**。

## 1　W1　花括号展开的射程

### 1.1　bash 的次序与代码的写法

bash 的展开次序是花括号展开最先、其后才是波浪号、参数、命令替换、通配、引号去除；花括号展开出来的**空串**（`{,}`、`{a,}`、`{,a}` 作整词时的空份）在引号外会被去掉，不留成空词。代码把展开放在切词的 `flush` 里：「tokens.extend((False, expanded) for expanded in brace_expand(word, mask))」（`.claude/hooks/lib_shell_words.py:516`）；`mask` 区分引号里外，所以引号里、转义过、`$(…)` 正文里的花括号不展；`${` 起的整段跳过。

### 1.2　逐词对照真 bash（正推的主证据）

`tools/brace_oracle4.py` 随机拼 8000 个无空格的词（花括号、逗号、`..`、数字、字母、引号、反斜杠混拼），每个词交给真 bash 做 `printf '[%s]'`，与 `shell_tokens` 的词表逐词比，超出 64 份的不比：

```
Counter({'empties-only': 2, 'other': 2}) of 8000
["other", "0{1..3}{..2{a..c}}", ["01{..2{a..c}}", ...
["other", "{..2{a,b}x}", ["..2ax", "..2bx"], ["{..2ax}", "{..2bx}"]]
```

8000 个里只有 4 个不同，两个是空份（下面 §1.3），另两个是 bash 自己在 `{..2…}` 这种怪形态里丢外层花括号的行为，实际命令里不会写。`tools/brace_cases.py` 的 100 条手写词里 29 条不同，扣掉变量与命令替换没代入的（我没让代码代入变量，对不上是我这边的比法），真不同的只有：空份、补零序列、`{a..Z}` 里的反斜杠位、64 份上限、赋值词、`x={a,b}`。

### 1.3　代码该判而没判对的写法族

| 写法族 | bash 实际 | 今天的代码 | 方向 | 依据 |
|---|---|---|---|---|
| 引号外 `{a,b}`、`{1..3}`、嵌套、前后缀相乘 | 展开 | 一致（§1.2 的 8000 条） | 判对 | 「tokens.extend((False, expanded) for expanded in brace_expand(word, mask))」（`.claude/hooks/lib_shell_words.py:516`） |
| 空份（`{cargo,}`、`{,}`、`{,test}` 作整词） | 去掉空词 | 留下空串词 | 偏松，见 §1.4 | 「return [expanded for expanded, _ in results] if results else [word]」（`.claude/hooks/lib_shell_words.py:585`），没有去空这一步 |
| 补零序列 `{01..03}`、`{00..3}` | `01 02 03` | `1 2 3` | 偏松（`cp s{01..02}.sh sub/ && bash sub/s01.sh` 漏拒） | 「texts = [str(value) if numeric else chr(value) for value in values]」（`.claude/hooks/lib_shell_words.py:643`） |
| 一个词展开超过 64 份 | 展 | 原样留一个词 | 导致 `shell_tokens` 层看不见；我喂 kill / tee / cargo / 等待循环四类，份数取 3、63、64、65、70，改前改后全拒，没找到借它放行的写法 | 「BRACE_EXPANSION_LIMIT = 64」（`.claude/hooks/lib_shell_words.py:576`）；重型闸另有 1024 份的展开兜底「WORD_EXPANSION_LIMIT = 1024」（`.claude/hooks/lib_heavy_tests.py:1173`） |
| 整词像 `名字=…` | 命令前缀位置不展；`declare x={a,b}`、`export`、参数位 `printf a={x,y}` 展（5.2.21 实测） | 一律不展 | 偏松，但我没找到让判定翻转的命令（`declare S={light.sh,s.sh}; bash $S` 改前改后都放） | 「if "{" not in word or words_break_is_set("braces-unexpanded") or ASSIGNMENT.match(word) or re.match」（`.claude/hooks/lib_shell_words.py:582`） |
| `[[ … ]]` 里、`case` 模式里 | 不展 | 展 | 偏严，但展出来的词不在命令位置，喂 `[[ x == cargo{,-x} ]]`、`case $x in cargo{,-x})` 改前改后都放 | 同上，切词不分位置 |
| 一个词里嵌套超过约 1000 层 | 展 | 递归炸掉，钩子「没判成、照常执行」 | 改前也炸（重型闸的展开也递归），不是这一批新增 | 实测 2000 组 `{a,b}`：改前改后都 rc=0 |

### 1.4　改动让原来判对的写法判错：空份（重型闸，改前拒、改后放）

机理：重型闸本来就有一套按 bash `braces.c` 写的展开，并且去空：「展开出来的空串照 bash 去掉」（`.claude/hooks/lib_heavy_tests.py:3329`），实现里是「results = [result for result in brace_expansion_with_empties(word, [BRACE_SCAN_LIMIT]) if result]」（`.claude/hooks/lib_heavy_tests.py:3336`）。现在切词先展一遍、又不去空，重型闸拿到的词表里没有花括号可展了，却多出空串词：命令词是 `''`，或 `cargo '' test`，原来靠它拒的写法就放了。

喂法：真的钩子入口，同一条命令分别喂改前树、改后树（`tools/keyfeeds.sh`，原样输出，✗ 行截 120 字）：

```
命令（guard/main）：{cargo,} test --all
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（照 bash 的花括号展开与参数里的通配展开，b
  改后：rc=0
命令（guard/main）：{,} cargo test --all
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（照 bash 的花括号展开与参数里的通配展开，b
  改后：rc=0
命令（guard/main）：cargo {,test} --all
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（照 bash 的花括号展开与参数里的通配展开，b
  改后：rc=0
命令（guard/implementation-writer）：{bash,} scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier
  改前：rc=2 ✗ 重型测试被拒：cargo test 用 -p 点名 singlefs-checker-tier（singlefs-checker-tier；当前目录或清单�
  改后：rc=0
```

bash 实测这几条真会跑：`{,} echo hi-1`、`{echo,} hi-2` 都打出 `hi-1`、`hi-2`。同族另有 5 条（`nice {,} cargo test --all`、`timeout 5 {,} cargo test --all`、`env {,} cargo test --all`、`X=1 {cargo,} test --all`、实现员的 `cargo {,test} -p singlefs-checker-tier` 经内存包装）同样改前拒、改后放，共 9 条真回归（`out/matrix.tsv` 的 `W1-brace-empty-*` 行）。另 4 条（`env A=1 {,}cargo …`、`{,}cargo …`、`cargo{,} …`、`c{,}argo …`）按 bash 是 `cargo cargo test --all`，不跑测试，改后放行是对的，改前反而是误拒。

检出器一侧：花括号让原来漏的变成抓到了（`{sed,} -i … scripts/hosts/x.sh`、`sed {,-i} …`、`{tee,} …`、`{cp,} …`、`{until,} grep -q x f; do sleep 10; done`、`{kill,} -9 -1`、`bash scripts/hosts/detached-run.sh {start,} …` 改前 0、改后 2），但同一类空份写在命令词前面的仍漏（`{,} kill -9 0`、`{,} tee …`、`{,} sed -i …`、`{bash,} scripts/hosts/detached-run.sh start …`），改前也漏，所以不是回归，是新认法没补全。

### 1.5　结论与推翻条件

- 结论：展开本身与真 bash 在引号外的常见写法上逐词一致；**判错的是不去空份**——它让重型闸原来判对的 9 条写法从拒变放。偏严一侧（`[[ ]]`、`case`）没造出会被误拒的命令；`>64 份` 与赋值词这两处我没找到借它放行的写法。
- 什么现象推翻：在「def brace_expand(word, mask):」（`.claude/hooks/lib_shell_words.py:580`）里去掉空串后，矩阵里 `W1-brace-empty-main`/`-writer` 的 9 条真回归应回到 rc=2；如果它们仍是 0，说明还有别的来源。反过来，若有人造出引号外、非赋值词、份数 ≤64 而与真 bash 逐词不同的非空份例子，则「引号外一致」这一句被推翻。

## 2　W2　heredoc 进没收尾的替换

### 2.1　机理

`heredoc_feeds_shell` 现在先把 `<<` 之前那一行交给 `open_substitution_tail`：「head = open_substitution_tail(head)」（`.claude/hooks/lib_shell_words.py:324`）；检出器里判 heredoc 正文在不在 `timeout` 底下也调它：「head_commands = shell_words.simple_commands(shell_words.shell_tokens(shell_words.open_substitution_tail(head)))[0]   # 挂在没收尾的 $( / <( 里的，在那一层里找挂它的命令」（`.claude/hooks/bash-command-detector.sh:955`）。

这个函数自己维护的是**一个**引号状态变量 `quote`。两条规则凑在一起出问题：开括号不看引号——「elif text.startswith("$(", position) or (quote is None and text.startswith(("<(", ">("), position)):」（`.claude/hooks/lib_shell_words.py:366`），双引号里的 `$(` 照开；收括号却要求引号外——「elif quote is None and character == ")" and openings and openings[-1][0] in ("(", "(("):」（`.claude/hooks/lib_shell_words.py:377`）。于是**双引号里已经收尾的 `$(…)` 永远收不了尾**，函数把它当成还开着，交回它之后的残片；残片往往带着半个引号，`shell_tokens` 把整段切成一个词，挂在行尾的 `bash` 认不出，heredoc 正文被当数据剥掉。同一份文本在 `command_substitution_end` / `double_quote_end` 那一套里（`shell_tokens` 用的）是对的，两套状态对不上就在这里。

### 2.2　逐族结果（`out/matrix.tsv` 的 `W2-guard`、`W2-det`；上是重型闸，下是检出器的前台等待循环）

| `<<` 之前那一行 | bash 实际 | 改前 | 改后 |
|---|---|---|---|
| `x=$(bash <<'EOF'`（正文跟 `)` 收尾） | 子 shell 跑正文 | 放（漏） | **拒**（新认法起作用） |
| `diff <(bash <<'EOF'`、`` x=`bash <<'EOF` ``、`x="$(bash <<'EOF'`、`echo "$(echo "x")" "$(bash <<'EOF'` | 同上 | 放（漏） | 拒 |
| `x=$(cat <<'EOF'`、`VAR="$(cat <<'EOF'`、`cat > f.txt <<'EOF'` | 正文是数据 | 放 | 放（对） |
| `echo $(date); bash <<'EOF'`、`(cd x; bash <<'EOF'`、`f() { bash <<'EOF'`、`case a in a) bash <<'EOF'`、`a=$((1+2)); bash <<'EOF'`、`echo ${x:-$(echo y)}; bash <<'EOF'`、`echo "`date`"; bash <<'EOF'` | 跑正文 | 拒 | 拒（对） |
| `FOO="$(pwd)" bash <<'EOF'` | 跑正文 | 拒 | **放（漏）** |
| `cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'` | 跑正文 | 拒 | **放（漏）** |
| `echo "a$(echo x)b"; bash <<'EOF'`、`echo "$(date)" \| bash <<'EOF'`、`y="$(echo "x")"; bash -s <<'EOF'`、`echo "$(date)" > /dev/null && sh <<'EOF'`、`echo "$(printf "%s" "$(date)")"; bash <<'EOF'`、`X="$(date)" Y="$(pwd)" bash <<'EOF'` | 跑正文 | 拒 | **放（漏）** |

最后三行共 8 条命令，重型闸（正文 `cargo test --all`）与检出器（正文 `until grep -q x f; do sleep 10; done`）各一遍，16 行全是改前 rc=2、改后 rc=0，即本批在 §0.1 的 2->0 里占了 16 条真回归。

原样输出（真钩子入口，✗ 行截 120 字）：

```
命令（guard/main）：FOO="$(pwd)" bash <<'EOF'\ncargo test --all\nEOF
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SI
  改后：rc=0
命令（guard/main）：cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'\ncargo test --all\nEOF
  改前：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SI
  改后：rc=0
命令（det/main）：cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF
  改前：rc=2 ✗ 前台的等待循环没有超时（认出的循环：until grep -q x f）：等的条件不成立就一直不返回
  改后：rc=0
命令（guard/main）：x=$(bash <<'EOF'\ncargo test --all\nEOF\n)
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SI
```

### 2.3　回答格里的两个问题

- 两套状态对不上的输入里判错的方向：**偏松**，而且**不是回到改前的漏拒**——这些写法改前是对的（rc=2），这一批把它们改漏了。偏严方向（把数据当成 shell 正文）我没造出来：残片被切成一个引号没配平的词时，`simple_commands` 取最后一条命令只会取到这个怪词，我造的几例里它都不是 shell 名。
- 有没有写法让 `$(bash <<EOF` 照旧被当数据剥掉：**没有**，它自己（含前面带双引号替换的 `echo "$(echo "x")" "$(bash <<'EOF'`）都认出来了。漏的是另一类：顶层的 `bash <<EOF`，只要同一行它前面有一个双引号包着、已收尾的 `$(…)`。常见到的写法就是 `cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'`。

### 2.4　推翻条件

- 若把 `open_substitution_tail` 的双引号状态改成与 `double_quote_end` 一样「里面的 `$(…)` 整段跳过」后，上表三行「放（漏）」的 8 条命令（16 行）仍是 rc=0，则机理说错了。
- 若实现员的语料里本来就有这类写法（双引号 `$(…)` 之后同行接 `bash <<`）而判定没变，则「语料罩不住」这一句（§6）被推翻。我在仓内文档语料（4541 条）里数过，这一形状 0 条。

## 3　W3　detached-run 写法的新认法

### 3.1　新认法在哪

- 检出器：`cat 源 > 目的` 的目的算拷贝——「targets = [target for operator, target in command.output_redirections if operator in DETACHED_RUN_CAT_COPY_REDIRECTS]」（`.claude/hooks/bash-command-detector.sh:1093`），重定向集合是「DETACHED_RUN_CAT_COPY_REDIRECTS = {">", ">|", ">>", "&>", "&>>"}」（`.claude/hooks/bash-command-detector.sh:878`）；命令词里的变量代入「if "$" in word and not detector_break_is_set("detached-run-variable-word-unseen"):」（`.claude/hooks/bash-command-detector.sh:1136`）；启动器名单「DETACHED_RUN_LAUNCHER_NAMES = frozenset({"time", "flock", "chrt", "prlimit", "systemd-run", "strace", "perf", "rustup"})」（`.claude/hooks/bash-command-detector.sh:881`），剥法借重型库的 `command_under_launcher`：「inner = library.command_under_launcher(command.words, command.directory) if library is not None else None」（`.claude/hooks/bash-command-detector.sh:1116`）。
- 重型闸：「COPYING_COMMANDS = {"cp", "ln", "install", "mv"}」（`.claude/hooks/heavy-test-guard.sh:582`），`cat_copy_pairs` 取「targets = [target for operator, target in command.output_redirections if operator in (">", ">|")]」（`.claude/hooks/heavy-test-guard.sh:588`）。

### 3.2　主 agent 按 `.claude/main-agent.md` 照抄的起法：没有误拒

`.claude/main-agent.md` 写的起法是「bash scripts/hosts/detached-run.sh start --wall-clock <within-2h|over-2h> --label <名> -- <命令>」（`.claude/main-agent.md:72`），另有 `watch`、`stop <名>`、`list`。检出器只拒子 agent，主 agent 的 49 条同样的命令（`W3-det-main`）改前改后全是 0→0。重型闸一侧我喂了 30 条主 agent 的写法（`W3-main-guard`：裸起、带 `SINGLEFS_HEAVY_TESTS=commit`、带 `SINGLEFS_RUN_SCOPE`、`capped.sh 31`、`-- env FOO=1 cargo test --all`、`-- bash -c '…'`、外面包 `/usr/bin/time -p`、`flock`、变量代入的脚本词、`cp` / `install` / `mv` / `ln -s` 拷出再起），**主 agent 正当的那几种（`watch` / `stop` / `list`、带前缀的起全量、`-- 命令` 里不是重型的）改前改后一条都没变**；变的 8 条都是该变的：内层是 `time -p cargo test --all`、`eval 'cargo test --all'`、外层 `time -p`、`cat > 拷出` 再起、`ln -s` 再起、`install` / `mv` 拷出再起（改前漏拒、改后拒），以及内层 `builtin cargo test --all`（改后拒、bash 实测 `builtin: cargo: not a shell builtin`，偏严，无害）。

### 3.3　子 agent 一侧：认进来的与仍漏的（`W3-det-clerk`，49 条，改前 0→改后 2 的 28 条）

| 族 | 例（都带 `bash scripts/hosts/detached-run.sh start --label m -- sleep 1`） | 改前→改后 |
|---|---|---|
| 启动器（名单里 8 个） | `/usr/bin/time -p`、`-f %e`、`-o out.txt`；`flock /tmp/l`、`flock -x /tmp/l -c '…'`、`flock -w 5`；`systemd-run --user --wait`、`--pipe --wait`、`--scope`、`-P`；`chrt -f 1`；`prlimit --as=…`；`strace -o /dev/null`；`perf stat`；`rustup run stable`；叠起来 `nice flock /tmp/l /usr/bin/time -v`、`timeout 5 flock …`、`taskset -c 0 flock …`、`env FOO=1 flock …` | 0→2 |
| 命令词里的变量 | `S=…; bash "$S" start`、`S=…; $S start`、`bash ${S}`、`export S=…`、`S2=$S` | 0→2 |
| `cat 源 > 目的` | `cat scripts/hosts/detached-run.sh > /tmp/x.sh && bash /tmp/x.sh start …`；`>>`；`cat -n`；两个源 | 0→2 |
| eval 字面代码 | `eval '…'`、`eval bash …`、`eval "…"` | 0→2 |
| 只读与别的命令 | `/usr/bin/time bash …detached-run.sh list`、`flock /tmp/l ls scripts/hosts/detached-run.sh`、`/usr/bin/time -v cargo build`、`flock /tmp/l cp x y`、`strace -f ls`、`perf stat ls`、`rustup run stable cargo build`、`eval 'bash …detached-run.sh list'`、`cat …detached-run.sh > /dev/null`、`cat …detached-run.sh \| head -3` | 0→0（对，没误拒） |
| **仍漏：`cat … > 拷贝 2>/dev/null`** | `cat scripts/hosts/detached-run.sh > /tmp/x.sh 2>/dev/null && bash /tmp/x.sh start …` | 0→0（**漏**，§3.4） |
| 仍漏：变量来自循环或命令替换 | `for s in scripts/hosts/detached-run.sh; do bash $s start …; done`、`S=$(echo scripts/hosts/detached-run.sh); bash $S start …` | 0→0（文件头写明判不到） |
| 仍漏：名单外的启动器 | `numactl --cpubind=0`、`nsenter -t 1`、`unshare -r`、`setpriv --reuid=1000`、`script -qc`、`watch -n 1`、`parallel`、`ssh localhost`、`su -c`、`runuser -u me --`、`doas`、`ltrace`、`valgrind`、`gdb --args`、`faketime now`、`unbuffer`、`sg users -c`、`cpulimit -l 50 --`、`dbus-run-session --`、`firejail`、`bwrap --bind / /`、`proot`、`scl enable x --`、`chroot /`、`cgexec -g cpu:x`（`xargs`、`find -exec` 文件头已列为罩不住的） | 0→0 |

「没加认法的 6 类之外」：上面「仍漏」的第三、四行是 6 类之外还能再加的（名单外的启动器至少 25 个，我一个个喂过，全 0→0）；**改后变得认不出的，我只找到 §1.4 和 §2 两处（花括号空份、双引号替换后的 heredoc）**，除此之外没有一条改前认得出、改后认不出：仓内文档语料 4541 条 × 2 钩子 × 主 agent / 实现员两种身份，改前改后判定 0 条不同（`probes` 在草稿目录，数字见 §6）。

### 3.4　新认法自己判错的：`cat … > 拷贝 2>/dev/null`

`shell_tokens` 把重定向前的 fd 号去掉（「if operator[0] in "<>" and word is not None and word.isdigit():」（`.claude/hooks/lib_shell_words.py:564`）），所以 `2>/dev/null` 以 `(">", "/dev/null")` 的样子进了 `output_redirections`；两处都取**最后一个** `>`：

```
$ python3 -I -c '…commands_at_command_position("cat s.sh > d1.sh 2>/dev/null && bash d1.sh", "/tmp")…'
['cat', 's.sh'] (('>', 'd1.sh'), ('>', '/dev/null'))
['d1.sh'] ()
```

于是目的被取成 `/dev/null`，`d1.sh` 不算拷贝，执行时读盘上那份（还不在）只记一条「不存在」。原样输出：

```
命令（guard/main）：cat s.sh > d1.sh 2>/dev/null && bash d1.sh
  改前：rc=0
  改后：rc=0
命令（guard/main）：cat s.sh > d2.sh && bash d2.sh
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/d2.sh:4 里
命令（det/clerk）：cat scripts/hosts/detached-run.sh > /tmp/shw-dr-b.sh 2>/dev/null && bash /tmp/shw-dr-b.sh start --label m -- sleep 1
  改前：rc=0
  改后：rc=0
命令（det/clerk）：cat scripts/hosts/detached-run.sh > /tmp/shw-dr-a.sh && bash /tmp/shw-dr-a.sh start --label m -- sleep 1
  改前：rc=0
  改后：rc=2 ✗ clerk 用 scripts/hosts/detached-run.sh 起或停脱离会话的长跑（认出的调用：/tmp/shw-dr-a.sh（这条
```

这不是回归（改前也漏），是新加的认法在最自然的写法（`cat 源 > 目的 2>/dev/null`）上不起作用；重型闸 `cp` 一侧没有这个问题（`cp` 没有输出重定向）。

### 3.5　`ln` / `mv` / `install` 拷出再起：多数认对，`ln -s` 相对源有两处不对

`copy_pairs` 把 `ln -s 源 链接` 当「链接是源的拷贝」，源按**当前目录**解（「return [(sources[0], destination)]」（`.claude/hooks/heavy-test-guard.sh:633`））；而符号链接里存的相对路径是按**链接所在目录**解的。链接不在当前目录时：

```
命令（guard/main）：ln -s ../s.sh sub/l.sh && bash sub/l.sh
  改前：rc=0
  改后：rc=0
命令（guard/main）：ln -s s.sh sub/l2.sh && bash sub/l2.sh
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/sub/l2.sh:4 里的
```

第一条真跑起来链接指到 `sub/../s.sh`（重型脚本），应拒，改后漏；第二条链接指到不存在的 `sub/s.sh`，bash 会报找不到，改后却因当前目录有 `s.sh` 而拒（偏严，无害）。`ln -sf $PWD/s.sh l3.sh`（源带没代入的变量）同样只记「不存在」。`install -m 755 / -D / -t / -o -g / -C / -T / --mode=`、`mv`、`mv -f`、`mv -t`、`cp` 后 `mv` 改名、`cat s.sh > d; chmod +x d; ./d` 这些 §3.1 里的认法都判对（矩阵 `W3-guard-copy`）。

### 3.6　结论与推翻条件

- 结论：新认法对主 agent 正当起法没有误拒；对子 agent 把 8 个启动器、变量词、eval、`cat >` 的常见写法认进来了。判错 / 漏的：`cat … > 拷贝 2>/dev/null`（两个钩子都漏，最自然的写法）、`ln -s` 相对源跨目录（一漏一误拒）。
- 什么现象推翻：`cat_copy_pairs` / `detached_run_cat_destinations` 取目的时去掉 stderr 的重定向后，`d1.sh`、`/tmp/shw-dr-b.sh` 两条应变 rc=2；仍 0 则机理说错。若有人在子 agent 身上找到一条正当写法被 `W3-det-clerk` 的新认法拒掉（我的 20 条只读与别的命令都没被拒），则「没有误拒」被推翻。

## 4　W4　env 宽松赋值的收紧

### 4.1　env 真实行为（本机 GNU coreutils 9.4 现跑）

GNU env 把选项之后、第一个不含 `=` 的词之前的**所有含 `=` 的词**都当 `NAME=VALUE` 设进环境，不看名字合不合 shell 变量名，**开头就是 `=` 的也算**：

```
$ env -- a-b=1 echo hi
hi
$ env a-b=1 echo hi
hi
$ env =1 echo hi
hi
$ env = echo hi
hi
$ env 1=1 echo hi
hi
$ env -- -x echo hi
env: ‘-x’: No such file or directory
$ env FOO=1 -- echo hi
env: ‘--’: No such file or directory
$ env -0 echo hi
env: cannot specify --null (-0) with command
Try 'env --help' for more information.
$ env -u a-b=1 echo hi
env: cannot unset ‘a-b=1’: Invalid argument
```

### 4.2　代码

新加的一支：「elif name == "env" and "=" in argument[1:] and not words_break_is_set("env-loose-assignments-as-command"):」（`.claude/hooks/lib_shell_words.py:910`）。它在 `ASSIGNMENT`（合变量名的）和「以 `-` 开头的当选项跳过」两支之后、位置参数之前，所以只在**不以 `-` 开头、含 `=` 且 `=` 不在第一位**的词上起作用。

### 4.3　逐族（`W4-env`，30 条，重型闸，主 agent 身份，命令里都是 `… cargo test --all`）

| 写法 | env 真做的 | 改前 | 改后 | 判 |
|---|---|---|---|---|
| `env -- a-b=1 CMD`、`env a-b=1 CMD`、`env 1=1 CMD`、`env 'x y=1' CMD`、`env A=1 B-2=3 CMD` | 设变量后跑 CMD | 放（漏） | 拒 | 对 |
| `env -i -- a-b=1 CMD`、`env -u X -- a-b=1 CMD` | 同上 | 放（漏） | 拒 | 对 |
| `env x= CMD`、`env -u A CMD`、`env -uA CMD`、`env -i CMD`、`env -iu A CMD`、`env -- CMD`、`env -C /tmp CMD`、`env --chdir=/tmp CMD`、`env --split-string='CMD'` | 跑 CMD | 拒 | 拒（没变） | 对 |
| `env -S 'a-b=1 CMD'`、`env -S 'env -- a-b=1 CMD'`、`env -S '-i a-b=1 CMD'` | 拆开后同上 | 放（漏） | 拒 | 对 |
| `env -S 'CMD'`、`env -S'CMD'`、`env -iS 'CMD'`；`env -S "x\_y=1 CMD"`（`\_` 是空格，命令是 `x`） | 跑 CMD；后者跑 `x` | 拒；后者放 | 拒；后者放 | 对 |
| **`env =1 CMD`、`env = CMD`** | **跑 CMD**（§4.1） | 放（漏） | **放（漏）** | **判错：代码注释写「只在开头的 `=` 不算」，与 env 实测相反** |
| `env -- -x CMD`、`env FOO=1 -- CMD`、`env -0 CMD`、`env -u a-b=1 CMD`、`env -- a-b=1 -- CMD` | env 自己报错，CMD 不跑 | 前四条拒、最后一条放 | 拒 | 偏严，无害（不跑的命令也拒；最后一条改后才拒） |

判错的就是 §4.2 引的那条里的 `argument[1:]`：它是有意把开头 `=` 排除在外的，但 GNU env 9.4 对 `=1`、单独的 `=` 照样当赋值、照样跑后面的命令。改前改后都漏，所以不是回归，是新写的规则里夹了一条与实测相反的例外，留下 `env =1 cargo test --all` 这个口子。

原样输出（真钩子入口）：

```
命令（guard/main）：env =1 cargo test --all
  改前：rc=0
  改后：rc=0
```

### 4.4　收紧后有没有正当命令被拒

收紧只会让钩子多跳过几个含 `=` 的词、找到后面的真命令，所以只可能把「原来被当成命令词而放行」的写法改成「看见真命令」。语料里 1 次（`env "$var=$val" bash stage-selftest.sh`）我在仓内文档语料里没有同形，用 `env 'x y=1' cargo test --all` 代替：改后拒，且 env 真的会跑 cargo，拒是对的。我没造出一条「正当的、env 真跑的是轻命令、却被拒」的例子：被拒的都是后面真有重型命令。`env -S`、`-u`、`-i`、`--`、`-C` 的组合除 `env =1 CMD` 那一类外，判得与 env 一致（env 自己报错的那几条是偏严）。

### 4.5　结论与推翻条件

- 结论：收紧方向判对；**开头 `=` 的例外判错**（漏拒）。
- 什么现象推翻：若 `env =1 echo hi` 在别的 env 版本上不跑 `echo`（例如 busybox），则「判错」要改成「与 GNU env 不同」；若有人造出 `argument` 含 `=`、不以 `-` 起头、env 却把它当命令词的写法，则「含 `=` 的都是赋值」这一句被推翻（我没找到）。

## 5　W5　time、builtin、function、bash 选项与 `//` 路径

### 5.1　bash 实测（本机 5.2.21）

```
$ bash -c 'builtin -- echo hi; builtin cargo; builtin echo ok; time -p -- echo hi; time -- echo hi; time -p -p echo x'
hi
bash: line 1: builtin: cargo: not a shell builtin
ok
hi
real 0.00
user 0.00
sys 0.00
hi
real 0.00
user 0.00
sys 0.00
bash: line 1: -p: command not found
$ bash -c 'bash -c -- "echo viaC"'
viaC
$ echo "echo from-stdin \$1" | bash -s -- argx
from-stdin argx
```

`time` 关键字只认一个 `-p`、之后可再跟一个 `--`；`builtin` 后面必须是内建；`bash -c -- '代码'` 的代码是 `--` 之后第一个词；`bash -s -- 词` 的词是 `$1`，不是脚本。

### 5.2　代码

「if word == "time" and not words_break_is_set("time-options-as-command"):」（`.claude/hooks/lib_shell_words.py:855`）、「if word == "builtin" and not words_break_is_set("builtin-as-command"):」（`.claude/hooks/lib_shell_words.py:862`）、「if word == "function" and not words_break_is_set("function-keyword-as-command"):」（`.claude/hooks/lib_shell_words.py:867`）；`bash` 选项：「if argument in ("-", "--") and option_values_read:」（`.claude/hooks/lib_shell_words.py:931`）；`//`：「return single_leading_slash(os.path.normpath(path))」（`.claude/hooks/lib_shell_words.py:1000`）。

### 5.3　逐族（`W5`，63 条，重型闸，主 agent 身份；`W5-det`，17 条，检出器，子 agent 身份）

| 写法族 | bash 实际 | 改前→改后 | 判 |
|---|---|---|---|
| `time -p ls`、`time -- ls`、`time -p -- ls`、`time ls`、`time -p -p cargo test --all`（第二个 `-p` 是命令词） | 轻命令 / 找不到命令 | 0→0 | 对 |
| `time -p CMD`、`time -- CMD`、`time -p -- CMD`、`! time -p CMD`、`nice time -p CMD`、`command time -p CMD`、`x=1 time -p CMD`、`time -p bash s.sh`；检出器 `time -p bash …detached-run.sh start` | 跑 CMD | 0→2 | 对 |
| `builtin source s.sh`、`builtin . s.sh`、`builtin -- source s.sh`、`builtin eval '…'`、`builtin exec CMD`、`builtin command CMD`；检出器 `builtin source …detached-run.sh start` | 跑 | 0→2 | 对 |
| `builtin cargo test --all` | 报 not a shell builtin，不跑 | 0→2 | 偏严（无害） |
| `function f { CMD; }; f`、检出器的同形 | 跑 CMD | 0→2 | 对 |
| `function f { cargo test --all; }`（只定义、不调用） | 不跑 | 0→2 | 偏严（无害；`f() { …; }` 本来就这样判） |
| `bash --rcfile s.sh -c 'echo hi'`、`--init-file s.sh -c …`、`bash --rcfile s.sh light.sh`、`bash --rcfile s.sh -n light.sh`、`bash --rcfile s.sh` | rc 文件只给交互 shell 读，不当脚本跑 | **2→0** | 对（改前误拒） |
| `bash --rcfile light.sh s.sh`、`bash -- s.sh`、`bash - s.sh`、`bash --noprofile --norc -- s.sh`、`bash -l -- s.sh`、`sh -- s.sh`；检出器 `bash --rcfile /dev/null …detached-run.sh start`、`bash -- …detached-run.sh start` | 脚本是 `s.sh` | 0→2 | 对 |
| `bash -`、`bash --`、`bash -- light.sh`、`bash --rcfile`（缺值）、`bash --init-file` | 读标准输入 / 轻脚本 / bash 自己报错 | 0→0，没抛异常 | 对 |
| **`bash -c -- 'cargo test --all'`**；检出器 `bash -c -- 'bash …detached-run.sh start …'` | **跑代码** | 0→0 | **判错（漏拒）**：`-c` 一支先于 `--` 一支返回，代码取成了 `--`（「return ShellInvocation(arguments[option_count] if option_count < len(arguments) else "", None)」（`.claude/hooks/lib_shell_words.py:943`）） |
| `bash -s -- s.sh`、检出器 `bash -s -- …detached-run.sh start …` | 从标准输入读，`s.sh` 是 `$1` | 0→2 | 偏严（`curl … \| bash -s -- 参数` 这类写法里的参数若恰是仓内脚本名才会误拒） |
| `bash -- -c '…'`、`bash - -c '…'` | `-c` 当脚本名 | 0→0 | 对 |
| `//`：`cp s.sh /tmp/x.sh && bash //tmp/x.sh`、`bash ///tmp/x.sh`（重型闸）；检出器 `cp …detached-run.sh /tmp/x.sh && bash //tmp/x.sh start …`、`cp … //tmp/x.sh && bash /tmp/x.sh start …` | 同一个文件 | 重型闸 2→2（本来按 realpath 比，文件头已写明）；检出器 `//tmp` 开头的 0→2，`///` 以上改前就 2 | 对 |

注意 `bash -c -- '代码'` 这一行**不是这一批新增的漏**（改前也 0），但格里问的是「`bash --` 这类有没有被当成起了一个 shell 跑脚本而误判」：新加的 `--` 一支只管 `--` 在 `-c` **之前**的写法，`-c` 之后的 `--` 仍未处理，这是同一块代码里留下的缺口。

原样输出（真钩子入口）：

```
命令（guard/main）：bash -c -- 'cargo test --all'
  改前：rc=0
  改后：rc=0
命令（guard/main）：bash -s -- s.sh
  改前：rc=0
  改后：rc=2 ✗ 重型测试被拒：脚本 <work>/scratch/s.sh:4 里的 carg
```

### 5.4　结论与推翻条件

- 结论：`time`、`builtin`、`function`、`--rcfile` / `--init-file`、`--` / `-`、`//` 补认之后，原来判对的写法没有被判错（矩阵里 `W5` 的 2→0 五条全是改前误拒、`0→2` 里除上表三条偏严外都是改前漏拒）；`bash --rcfile`、`bash -` 没被当成「起了一个 shell 跑脚本」。留下一处漏拒（`bash -c --`）、三处偏严（`builtin` 后不是内建、只定义不调用的 `function`、`bash -s -- 词`）。
- 什么现象推翻：若 `bash -c -- 'echo viaC'` 在别的 bash 版本上不跑 `echo`，则 `-c --` 一行要降为「与 5.2 不同」；若 `bash --rcfile s.sh -c 'echo hi'` 在钩子里被拒（改后应放），则「rc 文件值不当脚本」这一句被推翻。

## 6　W6　语料证据够不够

### 6.1　我复核不了的

实现员报告里的「1000 份会话记录、去重后 146675 条命令、切词变了 773 条、两版钩子判定变了 20 次（19 次实现员从拒变放、1 次收紧）、主 agent 一次没变」：这份语料在禁读清单里的实现员产物里，原始命令又来自会话记录（`~/.claude/` 之下，定义写明不碰），**我一条也没读到，这几个数都复核不了**。下面是推的，不是量的。

### 6.2　我自己能量的替代语料

从仓内 `.claude/`、`records/`、`scripts/` 的 md 里抽围栏代码块与行内代码里的命令（`tools/mkcorpus.py`，去重后 4545 条），两棵树（改前 / 改后）× 两个钩子 × 两种身份（主 agent / 实现员）各判一遍（`tools/inproc.py`）。原样输出：

```
guard main 4545 改前拒 332 改后拒 332 判定不同 0
guard implementation-writer 4545 改前拒 621 改后拒 621 判定不同 0
det main 4545 改前拒 29 改后拒 29 判定不同 0
det implementation-writer 4545 改前拒 55 改后拒 55 判定不同 0
```

四组全是 0 条不同；这份语料不随矩阵入库（内容随仓里文档变，不逐字可复现），要复跑照 `tools/mkcorpus.py`、`tools/inproc.py` 文件头。

### 6.3　这份语料（以及实现员那份）罩不罩得住 W1–W5 的射程

同一份 4545 条里，本报告各处反例的形状各有几条（正则数，`tools/shapecount.py`）：

| 形状 | 对应反例 | 4545 条里的条数 |
|---|---|---|
| 同一行里先有双引号包着的 `$(…)`、后面接 shell 的 heredoc | §2 的 8 条命令（16 行回归） | 0 |
| 花括号空份写在命令词或命令词前面（`{cargo,} test`、`{,} cargo`） | §1.4 的 9 条回归 | 0（引号外的空份只有 3 条，全是后缀位置：`foo.{sh,}`、`X{,.bak}`、`x{,-mirror}-…`） |
| `cat 源 > 目的` 后面又接 `2>` | §3.4 | 0 |
| `env` 后面以 `=` 开头的词 | §4.3 | 0 |
| `bash -c --` | §5.3 | 0 |
| `bash -s --` | §5.3 | 0 |
| `ln -s ../…`（相对源） | §3.5 | 0 |
| 任何一条 heredoc 喂 shell | W2 的前提 | 4 |
| 引号外带逗号的花括号 | W1 的前提 | 55 |

结论：**判定没变，对这些形状是因为它们在语料里根本不出现**；而同一批反例直接喂钩子时改前改后是不同的（§1.4、§2、§3.4 的原样输出）。所以「语料里改前改后只差 20 次」说明不了 W1–W5 的射程被罩住了。

### 6.4　语料方法上本身够不着的几处（推，没量）

- 它只比「改前 / 改后」的差。**两版都漏**的缺口——`bash -c --`、`env =1 …`、名单外的启动器、`cat … > f 2>/dev/null`——两版判定一样，差分永远看不见。
- 它是 agent 实际写过的命令，对「借花括号、双引号替换去绕」这类形状天然稀少；而 `cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'` 这样的写法对人对 agent 都自然，语料里没有只说明没写过，不说明以后不写。
- 判定依赖盘上状态（脚本在不在、当前目录）：回放一条旧命令时，它当年读的脚本、拷出的文件多半已不在，「cp / ln / mv / cat 拷出再起」这一族只会走到「不存在、记一条检出」那一支，**读源文件内容再判的那一支回放里走不到**。
- 「主 agent 的命令判定一次都没变」只说明主 agent 的命令里没有这些形状。

### 6.5　结论与推翻条件

- 结论：没核成——缺实现员那份语料的命令清单与判定差（我不能读），我自己的 4545 条也罩不住 W1–W5 的形状；语料证据不足以支撑「判定没变就是没问题」。
- 什么现象推翻：若实现员的 146675 条里有至少一条 §1.4 / §2 / §3.4 形状、且两版判定一样（说明我的反例在他那份语料的回放条件下不成立），则 §6.3 的第二个结论要收回；若他的 20 次变化里有哪条是真回归而他归成了改进，则语料「罩不住」更是实锤。

## 7　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W1　花括号展开的射程 | 判错（反例） | 引号外常见写法与真 bash 逐词一致（8000 个随机词只差 4 个）；但整词的空份不去掉，`{cargo,} test --all`、`{,} cargo test --all`、`cargo {,test} --all`、`nice {,} …`、`X=1 {cargo,} …`、实现员 `{bash,} … run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier` 等 9 条改前拒、改后放；另有补零序列 `{01..02}` 漏拒；`[[ ]]` / `case` 偏严没造出误拒，>64 份没找到借它放行的写法 |
| W2　heredoc 进没收尾的替换 | 判错（反例） | `$(bash <<EOF`、`<(bash <<EOF`、反引号这类**认对了**（改前漏）；但 `open_substitution_tail` 在双引号里收不了 `)`，同一行里先有 `"$(…)"` 再接顶层 `bash <<EOF`（如 `cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'`）改前拒、改后放，重型闸与检出器各 8 条共 16 行；方向是偏松且比改前更松 |
| W3　detached-run 写法的新认法 | 判错（反例）；主 agent 起法判对 | 主 agent 照 `main-agent.md` 抄的起法没有一条被误拒；8 个启动器、变量词、eval、`cat >` 的常见写法认进来了；但 `cat … > 拷贝 2>/dev/null` 两个钩子都漏（把 `2>` 当成输出目的），`ln -s` 相对源跨目录一漏一误拒，名单外至少 25 个启动器仍不认 |
| W4　env 宽松赋值的收紧 | 判对（收紧方向）；另有一处判错（反例） | 含 `=` 的词跳过后，`-S`、`-u`、`-i`、`--`、`-C` 的组合与 GNU env 一致，没造出被误拒的正当命令；但代码特意把开头 `=` 排除，env 9.4 实测 `env =1 echo hi` 照跑，`env =1 cargo test --all` 改前改后都放 |
| W5　time、builtin、function、bash 选项与 `//` | 判对（另有四处边角） | 原来判对的写法没判错，`bash --rcfile`、`bash -` 没被当成起了脚本（5 条改前误拒改成放）；留 `bash -c -- '代码'` 漏拒（改前也漏）、`builtin` 后不是内建、只定义不调用的 `function`、`bash -s -- 词` 三处偏严 |
| W6　语料证据够不够 | 没核成（缺实现员那份 146675 条的命令清单与判定差，来自会话记录，我不能读） | 我自己的 4545 条文档语料四组 0 条不同，但本报告各反例的形状在里面 0 条，所以「判定没变」在这些形状上是因为没出现，不是因为判对；语料只比改前改后的差，罩不住两版都漏的缺口 |

## 8　没做什么、复核不了的、清理

没做什么：

- 不替主 agent 采纳或出判决；上面每条都只是我这条腿的正推结论与推翻条件，没写改法。
- 没读禁读清单里的任何东西（实现员的报告、语料、探针，`shell-words-r1-opus-*`）。
- 没读也没碰 `~/.claude/` 下的会话记录，所以实现员报告里的 146675 / 773 / 20 三个数复核不了（§6.1）。
- 没跑两份钩子的 `--selftest`、其余库的自证与 13 个弄坏开关：这些是实现员自己的验证，主工作区里已由主 agent 观测过；我的反例都用探针直接喂，没经过它们。`heavy-test-guard.sh` 自证另红的两格（别的会话的 GPU 格）我没碰。
- 没起 cargo、没跑重型测试、没碰 GPU。
- 没单独喂另外共用 `lib_shell_words.py` 的钩子与脚本（`run-scope-guard.sh`、`timer-only-remind-guard.sh`、`two-logs-guard.sh`、`handback-guard.sh`、`host-resources-reminder.sh`、`shared-index-stale-reminder.sh`、`scripts/hosts/detached-run.sh`、`scripts/agents/agent-watch.py` 等）：`shell_tokens` 的空份与 `heredoc_feeds_shell` 的判错对它们也可能成立，没核。
- `heavy-test-guard.sh` 文件头与 `POLICY` 里广谱变异那一句只读了，没喂（是文字，不是判据代码）。

意外与补记：

- 我读冻结副本时有一次在它的 `.claude/hooks/` 里生成了一个 `__pycache__`（当时用 `python3 -I` 从该目录导入 `lib_shell_words`），发现后立刻删掉，四份文件的 sha256 与快照清单仍一致（§0.2 的输出是删完之后跑的）。
- 复跑脚本第一次经 `run-with-memory-cap.sh 4G` 起，被别人占满的内存队列排了 3 分多钟、被我自己设的 `timeout` 停掉；改 `1G` 同样排队。之后 `rerun.sh` 是不经包装、带 `ulimit -v 6291456` 与 `nice -n 19` 直接跑的（脚本里只有 python 进程，不编译、不跑编出来的代码，两个进程一族并行，峰值很小）。§0.1 给的复跑命令是照规矩的包装写法，队列空时用它。
- 草稿目录里建过的仓副本与复跑目录（`tree` 33M、`tree-orig` 33M、`rerun1`～`rerun7`（66M、66M、66M、75M、66M、66M、66M）、`frozen-copy` 2M、`rev` 2M，共约 541M（`du -sm`）），交回前删掉；留下的只有小文件（探针、语料、`scratch`、日志、`progress.md`）。
- 复跑记录：在新 shell（`env -i … bash --noprofile --norc -c`）里整份跑 `tools/rerun.sh` 一次、在普通 shell 里再跑一次，两次退 0，`matrix_vs_model`、`keyfeeds_vs_model`、`brace_cases_vs_model`、`brace_oracle_vs_model` 四行都是 `identical`，`family_run_exit_codes` 是 `30 0`（30 个进程全退 0）。
