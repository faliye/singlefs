# d137-gov-r1 正推腿（Sonnet）报告

轮名 d137-gov-r1，第一轮。立场：照真 bash 与钩子、脚本此刻的代码逐条对照。草稿目录 `~/.local/share/singlefs/work/d137-gov-r1-sonnet`；模型目录 `research/prompts/d137-gov-r1-sonnet-model/`。
日期 2026-10-29。

## 0 开工核对

- 开工快照 41 份：`sha256sum -c research/prompts/d137-gov-r1-snapshot/files-sha256.txt` 此刻全部 OK（量的，命令输出里没有一行非 OK）。所以下面引的行号对应的 sha256 就是快照里的那一份。
- 各件 own.diff 的 sha256 前 8 位与正文第二节列的逐个对上（量的：`sha256sum` 17 份）。
- 喂法：把钩子、`scripts/`、`.claude/` 拷进 `~/.local/share/singlefs/work/d137-gov-r1-sonnet/tree`（73M，无 `.git`，无 `kb`），`CLAUDE_PROJECT_DIR` 指它，`AGENT_HOOK_DETECTIONS` 指草稿目录里的 jsonl，cwd 放草稿目录里的 `sandbox`；每条样本 `setsid --wait bash <钩子>` 放进自己的进程组，超时杀整组。主工作区一个文件没改。
- 复跑命令（模型目录 `research/prompts/d137-gov-r1-sonnet-model/`；`--frozen` 给任何一棵与开工快照逐字节相同的树，主工作区此刻就是，只读）：
  `bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/d137-gov-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <仓根或冻结副本> --work <新的草稿目录>`
  我这一趟包装排不上（`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=45` 试了一次，退 252，队里是别的会话的 8G 与 24G 的活），按交付要求改用 `( ulimit -v 8388608; nice -n 19 bash …/rerun.sh … )` 直接跑，每步并行起一个 python 进程。前台 180 秒被工具转后台，两趟都跑完：第一趟（`--work …/rr2`）摘要与旧期望差在新加的几格，换成这一趟的摘要作 `out/expected-summary.txt`；第二趟（`--work …/rr3`）退 0、摘要与期望逐行相同、留下的样本进程 0。每步的仓副本 64M，共 8 份，跑完脚本自己删（删前大小在脚本输出里）。
  每个文件的 sha256（`sha256sum tools/* out/*`，在模型目录里取）：
  32d22728ee2a9440a34048223fa67673b2a788a8953688b848b66eeb9cef44c1  tools/addfrag.py
  cafd81ae6dd8e6dcf7b2906d6308d6752c22eeef2dcb15f894874afc53c88231  tools/battery_g2.py
  8ae3b53f6fe57f33ec1db23a26c123f3f9b9696508d1fbd9b87cd0b820dcb394  tools/battery_other.py
  d7996bff6a3baec393b610c1969a5ed90ce28b70a9c621deaec94ba969e0d9b6  tools/cases_fuzz2.py
  d8e33aa0715a43c90b953f6d1b889fd8817f84c2dfb4b4ce3321b887f49d6309  tools/cases_fuzz.py
  10aff58b27def6ecefdb734e385bd8fd26110a60537a701f6d7973ac4f5358f4  tools/cases_heavy2.py
  241067ddf97d6acb76565db45efb43c1d024e0464b81f9fa33e1328d7a1ebe11  tools/cases_heavy.py
  bbc9c9d977ba31025eb8dae91e1c9a5bc4a1c9f90544648e12729bbb54e67705  tools/chkrefs.py
  409473f396e1e7218e23c659d9c1ca3680452bbf8e7036ef2a3af6840da3ba30  tools/dispatch_probe.py
  ff8e6e7e8f9098b9782efef89927a3a88ee864323969fa0f92c37e819810c539  tools/feed.py
  d2bb8d30bc0ae230c1518bfa6c21484f6483d0d4751496a7eeedf38576c847a1  tools/garbage_input.py
  71cd6c29b81af584215beb02bff6db8222c4756bd4072caef8b18148d088bb37  tools/hits_g1.py
  af6073f334658ba859a7500c5f447445e81bb07812c184ce89695f974faf967b  tools/opts_fuzz.py
  f30c04751890c5984acc19f92ce19ec02497fdde83f75c54b128c279f53976e2  tools/overblock.py
  eafaec41aa3dbbbb754b088a3f560d02c03d40cf2ea1ff52d80b5797fa40d28f  tools/raw_g1.py
  e47686ed4a48d4de191b2a6b93d87d99b677661e45ccd8f9d228b03aa365fdaf  tools/rerun.sh
  34cebd124c68fa7bb482262463ca99f011d7ab1342575f6c61f8d4359d410132  tools/run_fuzz.py
  9e16e6a79ca9e90c29096be5517d8940c80a4da727606916b99ac8b464348336  tools/run_heavy.py
  a0eee1d7d503c556bec1ebaa4e164bf4eb0df39a82894ec2da5aa49c81c0d833  tools/sandbox_reset.py
  f6db63341bdf3489939e0f3f401ba85a951104b2b6ca66a1ceee5f0e2a32ae39  tools/setup_sandbox.sh
  ab91a85bffab14d1e442ac9e6ca1b852c95b1221e7385ef00ab60a6643dde0e7  tools/summarize.py
  263bc0b6a90b05b7aa925e2748f8972a29fdfb476ade0bb305453de91a74474c  tools/truth.py
  79ad16c65f0dfc45c0398112c17103d6fbfa15ba2efe85d700c274c354edb6ab  out/expected-summary.txt
  c2277c4281814d40c99d93d8d7b802b438f5d48232d6dfb44ad761ae767c0e78  out/hits-g1.tsv
  ec94a2e2afeef1a90728a277d823324474e7c1b29b7d652dd0f97ebb8f34fd77  out/new-sentences.txt
  ddbfcb5a485e33c12a040ac1443434e296e793197f34ed4ff7e657941ef2b7e2  out/raw-g1.txt

## G1　照用户口径，钩子判得对不对

### G1.0　口径与方法

用户口径（逐字，`records/2026-10-29-一百三十七节那一轮调度记录.md` 第 13、14 行）：
- 第 13 行：`认不出的一律拒（偏严）；三种静默漏拒（命令替换里的 case 模式与注释、`coproc`、Z15/Z16）照改成拒`
- 第 14 行：`cat 不看源文件在不在、重定向目标花括号展开成几个词：照拒，拒绝说明改贴切；函数体外引号里的花括号：放（切词带出引号信息）`

方法（判法自己写，不拿钩子自证当依据）：
1. 每条命令先喂拷来的钩子（`heavy-test-guard.sh`，sha256 前 8 位 83aacdab；`bash-command-detector.sh` 1a7647f6；`runner-dispatch-guard.sh` 472145f2；`write-guard.sh` 89be5574），退出码与 stderr 原样记；喂之前、之后都把沙箱复位（先跑真 bash 留下的文件会让钩子读得到，第一趟就因此虚高过，已改成钩子在先）。
2. 再把同一条命令放进真 bash（`bash -c`，沙箱里 `cargo` 是只记调用的垫片，`setsid --wait` 起、超时杀整组），看垫片收到什么。「钩子放行而真 bash 起了 `cargo test`」记成**漏拒**；「钩子拒而真 bash 没起」记成**误拒**。载荷都是 `cargo test --all`，所以对子 agent 不论从哪条理由都该拒，放行就是没看见。
3. 期望（该拒 2 / 该放 0）在看钩子输出之前按用户口径、钩子文件头和真 bash 行为写在 `tools/cases_heavy.py`、`cases_heavy2.py`、`cases_fuzz.py`、`cases_fuzz2.py` 里；主 agent 不带前缀与带前缀各有一组。

### G1.1　撑得住的（量的：`tools/run_heavy.py` 输出 复跑的 `<work>/out/heavy.tsv`，171 行）

| 项 | 喂了几条 | 符合期望 | 说明 |
|---|---|---|---|
| 认不出一律拒（子 agent）：变量、命令替换、位置参数、进程替换拼命令词或脚本路径，`eval "$c"`，`for f…; bash $f`，`cargo $(echo test --all)`，`cargo "$@"`，`${X:---all}`，`bash g*.sh`，`cat f \| bash`，`bash -s < f`，`BASH_ENV=… bash -c true`，xargs / find -exec 起 bash，转发函数，写出再起 | 26 | 26 | 拒的理由多数写「认不出起什么的写法」，变量赋过字面值的（`c=cargo; $c test --all`、`F=--all; cargo test $F`）照展开后按「全量」拒，也算对 |
| 三种静默漏拒改成拒：命令替换里不加括号的 case 模式、命令替换里注释的 `)`、`coproc`（含 `coproc X { … }`）、位置参数与变量拼 cargo 旗标 | 7 | 7 | 子 agent 与主 agent 不带前缀都拒（主 agent 组 M16–M18 三条同） |
| 字面对照：同一载荷的 77 种字面写法（引号、反斜杠、`$'…'`、花括号 `{cargo,test,--all}`、env / nice / timeout / flock / setsid --wait / stdbuf 等前缀、`bash -c`、heredoc、here-string、管道进 sh、子 shell、`&&`/`\|\|`、循环、case、命令替换、进程替换、`cargo +nightly`、`cargo t`、后台 `&`） | 77 | 77 | 全部拒（`cargo t --all` 真 bash 里垫片收到 `t`，是别名，不算起 test） |
| 主 agent 前缀规矩：不带前缀拒；`SINGLEFS_HEAVY_TESTS=commit` 放；分号隔开的赋值、`env -i` 之后、`true;` 之后、管道前一段的前缀不算；`export` 过的算；值写错不算；带前缀时 `$c`、`BASH_ENV=…` 放行 | 18 | 18 | 与「认不出的一律拒，主 agent 带前缀放行」一致 |
| 子 agent 的正当命令（ls、git、grep、sed -n、python3 -I、for/while/case/函数、`$(date)`、`ps … "$(id -u)"`、`nice`/`timeout` 包 harness 测试、`run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib` 及它的 `bash -c` 形式、`capped.sh 31` 再包它、`cargo fmt/build/--no-run`、`find … -exec grep`、`xargs grep`、写 `cargo test --all` 字样进 `/dev/null` 的 echo/heredoc、`echo {a,b}`、`echo "{a,b}"`、`echo '{cargo,test,--all}'`） | 43 | 42 | 唯一不符是 G1.2 的 H10 |
| 边界：本仓内存包装与逐字节相同的拷贝按本仓那一份认（放，rc=0）；尾部多一行的拷贝按读正文判（拒，rc=2）；`capped.sh` 同（`tools/battery_other.py`「拷贝边界」组） | 5 | 5 | 改过的拷贝先报「没判全」再拒 |
| 撤旧根（写闸，Write 工具）：实现员与无定义的 agent 往 `/tmp/claude-1000/…` 写被拒（实现员报「写范围不含」、无定义的报「只许写工作根」），写工作根放；主 agent 不判（`battery_other.py`「旧根:写闸」组） | 8 | 8 | |
| 撤旧根（检出钩子，Bash）：无定义 agent 的 `>`、`tee` 写会话临时目录根拒；项目定义的实现员与主 agent 放（仓外不判，`agent-common.md` 第 38 行只说无定义 agent，一致）（「旧根:检出」组） | 6 | 6 | |
| 撤旧根（检出钩子 ⑦ 仍管会话临时目录根）：`echo x > /tmp/claude-1000/…/old.sh`（已存在的脚本）无定义 agent、实现员、主 agent 三种身份都退 2，同目录 `sed -i` 三种身份都放（⑦ 不含 sed -i，一致）（「旧根:检出⑦」组） | 6 | 6 | `grep -rn 'legacy_root\|LEGACY_ROOT\|过渡期'` 在被判的钩子、定义与脚本里 0 命中；完工清理闸、`copy-originals.py`、`clean-session-scratch.py`、`agent-watch.py` 都改取 `claude_session_temporary_root`（`grep -rln` 命中这些文件，我没单独喂它们） |
| 撤旧根（派发闸）：「草稿目录：」在 `/tmp/claude-1000/…`、工作根本身、相对路径、`/tmp/x`、仓外别处都退 2；`$HOME/…/work/…` 写法放；报告行或「自证在哪跑：」点名会话临时目录根退 2；只读引用该根下路径放（「草稿目录」组） | 9 | 9 | 与 `agent-common.md` 派发闸那一句对得上 |
| 准入退码（假的 `admission.py` 退各个码，`tools/dispatch_probe.py`）：0 放；2 拒；3 拒；77 拒；1、4、5、124、250 与卡住超时放行并记「没判全」检出；2、3 有「准入已判：」行放；非重跑登记放；`admission.py` 不在（python 自己退 2）被当成「登记里没有」拒——见 H12 | 15 | 14 | 与用户口径「退 2、退 3 照拒，别的码判不了放行并记检出」一致 |
| 检出钩子 ⑫（实现员写范围，cwd 在拷来的树里）：仓内范围外的 `>`、`tee`（含花括号）、`cp`、`sed -i`、python 写、`for` 写死词与 `$(ls)` 的循环变量、变量代入、`$PWD`、`cd` 之后、`../` 回爬、`eval` 字面、`bash -c`、`find -exec`、`xargs`、`parallel` 全拒；范围内放；`mktemp -d` 赋的变量放；仓外不判（「⑫写范围」组） | 24 | 24 | 与 `agent-common.md` 说的一致 |
| 检出钩子 ②④①⑮（主 agent，run_in_background）：`setsid cargo build &` 拒；`setsid --wait` 放；`setsid bash -c` 放（并记检出）；`( … ) & p1=$!; … wait $p1; wait $p2` 放；`sleep 1 & sleep 2 & wait $!` 拒；`coproc`、`nohup … &` 拒；看门狗自己的输出或 stderr 丢进 `/dev/null`、花括号整组丢进去拒，别的命令的 `2>/dev/null` 放；`git reset`、`git restore --staged .`、`git stash`、`g=reset; git $g` 拒，`git stash push -- 路径` 放（「②④①⑮」组） | 16 | 16 | 主 agent 前面被误记的 `( … ) & p1=$!` 那条此刻不记 |
| 检出钩子 ⑬：`claude -p`、`BASH_ENV` 文件里起 claude、`--rcfile` 文件里起 claude 拒；`BASH_ENV` 文件无害放（「⑬」组） | 4 | 4 | |

推翻「撑得住」这一栏的现象：同一批喂法在别的内存或负载下换结论；或在草稿目录以外的主工作区钩子上同样的喂法结果不同（我只复核了拷来的、sha256 与快照一致的那一份，主工作区的此刻文件与快照一致已用 `sha256sum -c` 核过）。

### G1.2　打中的

读法：「子 agent」= 派发身份 `implementation-writer`，「主 agent」= 输入里没有 `agent_type`、命令不带前缀；两者此刻判法相同。载荷都是 `cargo test --all`，真 bash 里垫片收到 `CARGO test --all`（`tools/hits_g1.py` 输出 `out/hits-g1.tsv`，原样 stderr 在 `out/raw-g1.txt`）。
下面 H1–H8 在子 agent 与主 agent 身份下的钩子退出码都是 0，真 bash 都真起了 `cargo test --all`；每条给最短喂法。

**H1　`bash -euo pipefail -c '…'` 一类带值的短选项合写：整条 `-c` 代码没判（最常见的写法，最重）**
- 喂法：`bash -euo pipefail -c 'cargo test --all'`
- 钩子（退 0，只记检出）：`! heavy-test-guard.sh：bash / sh 起的脚本 …/sandbox/pipefail 不存在，里面的命令没判；这条命令照常执行，记了一条检出`
- 同类（都放、真起）：`bash -eo pipefail -c …`、`bash -Eeuo pipefail -c …`、`bash -euxo pipefail -c …`、`bash -ouc pipefail '…'`（退 0、零检出）、`bash -euo pipefail heavy.sh`、`env bash -euo pipefail -c …`、`timeout 5 bash -euo pipefail -c …`；对照 `bash -o pipefail -c …`（分开写）与 `bash -ec …` 照拒。差分表：复跑的 `<work>/out/opts.tsv`（105 种选项写法，漏拒 14，其余 66 符合、25 条 sh/dash 误拒是 dash 不认 `-o pipefail` 与 `-l` 重置 PATH 的垫片假象，不算）。
- 原因：`.claude/hooks/lib_shell_words.py:1479`（sha256 前 8 位 0ea57581）`if argument in ("-o", "+o", "-O", "+O") or (argument in SHELL_LONG_OPTIONS_WITH_VALUE and option_values_read):` 只认整词 `-o`、`-O`；`-eo`、`-euo` 里的 `o` 要吃下一个词，钩子没吃，`pipefail` 被当成脚本路径，`-c` 的代码没进判。
- 口径对照：第 13 行「认不出的一律拒」；`.claude/hooks/heavy-test-guard.sh:201`（83aacdab）把「读不到或看不全的脚本」列为放行并记检出，这条正是落在那里——但它不是脚本读不到，是钩子把选项值当了脚本。
- 推翻它：把上面任一条喂回去，退出码是 2。

**H2　`trap '…' EXIT|ERR|DEBUG|0`：字符串当 shell 代码执行，钩子不判**
- 喂法：`trap 'cargo test --all' EXIT`；`trap 'bash heavy.sh' EXIT`；`trap 'cargo test --all' DEBUG; true`（退 0、零检出，真起）。对照 `eval 'cargo test --all'` 退 2。`RETURN`、`SIGTERM` 两种这条命令里没触发，真 bash 没起。
- 原因：`.claude/hooks/lib_shell_words.py:1917` 的 `FUNCTION_CALLING_WORDS = frozenset({"eval", "source", ".", "trap", "compgen", "declare", "typeset", "set", "alias", "enable"})` 只用于判「函数有没有被调用」，`trap` 的参数不当命令切。同族共 11 种（复跑的 `<work>/out/fuzz.tsv` 里 Z015、Z016、Y001–Y004、Y007–Y011）。

**H3　`alias` + `shopt -s expand_aliases`：别名展开成重型命令**
- 喂法：`alias cargo='cargo test --all'` 换行 `shopt -s expand_aliases` 换行 `cargo`；`shopt -s expand_aliases` 换行 `alias ll='bash heavy.sh'` 换行 `ll`（退 0、零检出，真起）。`lib_heavy_tests.py` 只展开 cargo 自己的 `--config alias.` 别名，不认 shell 别名。7 种写法。

**H4　标准输入这一族：钩子文件头说「喂给 shell 的标准输入不是字面的」认不出就拒，下面几种漏**
- 喂法（都退 0、零检出，真起）：`source /dev/stdin <<< 'cargo test --all'`；`bash /dev/stdin < heavy.sh`；`bash /dev/fd/0 <<< '…'`；`exec < heavy.sh; bash`；`exec 0< heavy.sh; bash -s`；`{ bash; } < heavy.sh`；`( bash ) < heavy.sh`；`while true; do bash; break; done < heavy.sh`。对照 `bash < heavy.sh`、`bash -s < f`、`cat f | bash` 退 2。共 16 种。
- 原因（推的，没逐行追到代码）：`bash` 后面没有脚本路径时，钩子只看 `bash` 自己命令上的 `<`；`/dev/stdin`、`/dev/fd/0` 当成一份读得到的设备文件；复合命令（`{ …; } <`、`( … ) <`、`while … done <`）与 `exec <` 上的重定向没传给里面那条 `bash`。

**H5　`cd` 失败、`cd` 目标算不出之后的相对路径脚本**
- 喂法：`cd missing_d137 2>/dev/null; bash heavy.sh`（退 0，检出 1：`…/sandbox/missing_d137/heavy.sh 不存在，里面的命令没判`）；`cd missing_d137 || true; bash heavy.sh`；`cd ' '; bash heavy.sh`；`cd "$(echo missing_d137)" 2>/dev/null; bash heavy.sh`（退 0、零检出）。12 种。真 bash 里 `cd` 失败、留在原目录，`heavy.sh` 起了。
- 原因：钩子把 `cd` 当成必成功；脚本路径不存在只记检出放行（`.claude/hooks/heavy-test-guard.sh:1564`：`notices.append(Notice(here, f"{missing}，里面的命令没判"))`，前一格 `if missing and missing.endswith(PATH_UNRESOLVED_SUFFIX)` 才拒）。最后一种是文件头 `.claude/hooks/heavy-test-guard.sh:267` 列的「cd 到展开不了的变量路径之后……相对路径的脚本」仍放行；用户口径第 13 行点名 `⑤ heavy-test-guard「看不见的」` 要一律拒，这一项还在。

**H6　`~+`（波浪号加号）**：`bash ~+/heavy.sh`、`cd . ; bash ~+/heavy.sh` 退 0、检出 1（「…/~+/heavy.sh 不存在」）；`BASH_ENV=~+/heavy.sh bash -c :` 退 0、零检出；三条真 bash 都起了。`~+` 即 `$PWD`；`$PWD/…`、`${PWD}/…`、`$(pwd)/…` 钩子都认，只漏这一个写法。

**H7　`cat` 的源在盘上不在：「照拒」只落在一种写法上**
- 用户口径（第 14 行）：`cat 不看源文件在不在 … 照拒，拒绝说明改贴切`。钩子文件头 `.claude/hooks/heavy-test-guard.sh:190`：`几个源的 cat 里有盘上不在的源（`cat s.sh "2"> d.sh && bash d.sh`）照把在的源接起来判、照拒（偏严，用户 2026-10-29 定）`。代码：`.claude/hooks/heavy-test-guard.sh:1141` 记 `missing_cat_sources`，`:1841` 起只是在**已经拒了**的消息后面加一段点名不在的源。
- 所以：单源 `>` 且不在：拒（`cat nonexist_src.sh > dd7.sh && bash dd7.sh` 退 2，认不出）；但盘上不在的源是同一条命令里前面刚写出来的，就放：
  `echo 'cargo test --all' > m1.txt; cat src_ok.sh m1.txt > dd1.sh; bash dd1.sh`（退 0、零检出，真起）
  `echo 'cargo test --all' > m9.txt; cat m9.txt >> dd9.sh; bash dd9.sh`（`>>`，退 0）
  `echo 'cargo test --all' > m10.txt; cat m10.txt | tee dd10.sh > /dev/null; bash dd10.sh`（`tee`，退 0）
  对照 `… > m3.txt && cat m3.txt > dd3.sh && bash dd3.sh` 退 2。
- 另：`cat src_ok.sh nonexist_src.sh > dd8.sh && bash dd8.sh`（在的源无害、不在的源真没有）退 0——文件头那句「照拒」按字面是无条件拒，按代码是「在的部分拒才拒」，两处说法不一致，我取代码的读法，未列打中。

**H8　改名的 shell**：`cp /bin/bash ./mysh_d137; ./mysh_d137 heavy.sh`、`ln -sf /bin/bash ./mysh_d137b; ./mysh_d137b -c '…'`、`rbash -c '…'`、`/bin/rbash heavy.sh`、`busybox sh -c '…'`（都退 0，真起；本机 `command -v rbash busybox` 都有）。钩子按命令词名字认 shell；不认拷贝、符号链接与 rbash / busybox。属用户列的「已知没认」里「别的程序」的邻居，这里按同类报。

**H9　已知没认的同类（只列名字，不展开）**：`tclsh <<< 'exec cargo test --all'`、`gawk 'BEGIN{system(…)}'`、`perl -e 'system(…)'`、`sed -n '1e cargo test --all' f`、`git -c alias.x='!cargo test --all' x`、`tar --to-command=…`、`ssh -o ProxyCommand=…`、`GIT_EXTERNAL_DIFF=… git diff`、`python3 -c`、`make -f /dev/stdin`、`env python3 -c os.execvp`：全退 0、零检出、真起（共 15 种，复跑的 `<work>/out/fuzz.tsv`）。`git -c core.pager=…`、`less +'!…'` 这次没触发，真 bash 没起，不算。

**H10　引号里的花括号作命令词被当花括号展开（误拒）**
- 喂法：`'{cargo,test,--all}'`（单引号）与 `"{cargo,test,--all}"`：钩子退 2，说「照 bash 的花括号展开……bash 真起的那一条是 cargo test --all」；真 bash 里引号里的花括号不展开，命令词就是字面的 `{cargo,test,--all}`，「command not found」，垫片没收到调用。
- 用户口径第 14 行「函数体外引号里的花括号：放（切词带出引号信息）」：`echo '{cargo,test,--all}'` 放（对），命令词位置的没放。影响小：没有正当命令长这样。

**H11　引号里的重定向目标花括号被拒，拒绝说明说「分不出」（说明过时）**
- 喂法：`echo x > "out{1,2}.txt"`（实现员，cwd 在工作根）退 2；真 bash 写一个名叫 `out{1,2}.txt` 的文件，无 ambiguous redirect。
- 拒绝说明（`.claude/hooks/bash-command-detector.sh:1940`，1a7647f6）写「这道闸切词时分不出花括号在不在引号里」，而 `lib_shell_words.py` 的 `ShellWord.unquoted_braces` 此刻带着引号信息（`echo '{cargo,test}' | cat` 放行，推的是靠它）。`.claude/agent-common.md:86` 写「引号里的同样」，所以文档与实现一致，但与用户口径第三类的精神不一致，拒绝说明也已不真。低。

**H12　`admission.py` 不在或 python 自己退 2，被当成「登记里没有」拒**
- 喂法：`RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT` 指到不存在的文件，设计员写重跑登记：退 2，`✗ E142 的重跑准入判不了（scripts/gate/admission.py experiment 退 2：准入登记里没有这个实验键，或这个实验的登记有错）`。python 找不到脚本、argparse 用法错都退 2，与「没登记」同码。用户口径「别的码判不了放行并记检出」，这里 2 是不可分辨的码。低（真仓里 `admission.py` 在）。

**H13　python3 不在 PATH 时，钩子退 78、什么都不记（文件头与 `.claude/agent-common.md:76` 说照常执行并记一条检出）**
- 喂法：`PATH` 指到一个不含 `python3` 的目录，喂 `heavy-test-guard.sh`（`cargo test --all`）与 `bash-command-detector.sh`（`claude -p hi`）：两个都退 78、检出 0 条，stderr 首行 `✗ …/heavy-test-guard.sh：没有 python3，判不了准入与运行条件，拒绝执行…`（`tools/battery_g2.py` 末尾「bash层」组，4 格全不符）。
- 对照文件：`.claude/hooks/heavy-test-guard.sh:767` 起一段写「钩子 bash 这一层起不来时也记一条检出（一百三十七节 ⑤）：python3 起不来、……」并有 `hook_record_bash_layer_failure`；但 `:763` 的 `source …/scripts/preflight.sh`（接着 `preflight` 调用）在它之前，先按 `run-condition: command python3` 拒了，退 78，走不到那个函数。`.claude/agent-common.md:76`：`钩子 bash 那一层起不来` 照常执行并记检出。
- 退 78 不是 2，Claude Code 照放，所以「照常执行」成立，「必记检出」不成立：这一种看门狗读不到。影响面窄（本机有 python3），先报；没有查 preflight 对其余钩子是否同样处理（上面只喂了两份）。

### G1.3　偏严的代价、边界与核不动的

偏严的代价（量的，`tools/overblock.py`，20 条子 agent 平常会写的、本身不起重活的命令，期望按「本身不重」写，结果 7 条被「认不出」拒、13 条放）：被拒的是 `$EDITOR CLAUDE.md`、`"$PY" x.py`、`${CARGO:-cargo} fmt --all -- --check`、`$(which python3) -c …`、`exec "$@"`、`bash "$SCRIPT"`、`x=$(command -v git); $x status`；放的有 `$HOME/.cargo/bin/foo`、前面赋过字面值的 `PY=python3; $PY …`、`bash -n $f` 循环、`find … -exec bash -n {} \;`、`xargs -n1 bash -n`、`xargs -I{} sh -c "echo {}"`。7 条拒绝都是用户定的「认不出的一律拒（偏严）」本意，说明点名了变量并给改法，不算误拒；列出来给主 agent 估这一条的日常代价。

核不动的：
- zsh：本机没有 zsh（`command -v zsh` 空），`ZDOTDIR=… zsh -c true` 的判法只喂了钩子，没有起真 shell 对照；文件头自己也写了「本机没有 zsh，这一项没起真 shell 实测」。
- 主工作区钩子此刻与我喂的拷贝逐字节相同这件事，只核了 sha256（快照 41 份全 OK）；没在主工作区本体上喂（交付要求不改主工作区）。
- 看门狗（`agent-watch.py`）、内存包装、峰值库、`clean-session-scratch.py`、`copy-originals.py` 的判法：G1 只喂了钩子；这几份的「会话临时目录」改动我只做了 `grep` 级核对（命中 `claude_session_temporary_root`），没起它们的自证（自证在腿的限额里不跑重的，也不碰本机别的会话的内存队）。
- 四份全量复跑读数：按提示这一趟腿不引。
- 读不到正文的 `ssh` 起的远端命令：钩子照本机那一条判（文件头说偏严），喂了 `ssh localhost cargo test --all` 拒，没有登上第二台。

推翻 G1.2 里任一条：把该条喂法原样喂回去，退出码是 2（H10、H11、H12 反过来：H10 退 0、H11 退 0 才算推翻）。

## G2　定义、共用约束、规则与钩子和脚本实际做的一致吗

方法：从 `git diff HEAD --word-diff` 抽出这一轮新写的句子（`agent-common.md` 36 处、`main-agent.md` 12 处、五份定义、`verification.md`、`commit-stages.md`、`CLAUDE.md`；别的会话的块不判），逐句拿钩子与脚本此刻的行为对：能喂的写成 `tools/battery_g2.py` 的 60 格（每格的「句子」列写的是被验的那句，期望照句子写）。「只写怎么做」按 `.claude/singlefs-ai-sop/rules/rules-discipline.md:13` 那张表：`| 怎么做：步骤、次序、判据、阈值 | 为什么这么定：论证、取舍、推导 |`。

### G2.1　撑得住的（量的）

| 项 | 数 | 结果 |
|---|---|---|
| 九份定义「开工先读」里 `文件「小节」` 引用，小节标题逐字在被引文件里（`tools` 外的小脚本，对九份定义共 74 处） | 74 | 71 处逐字命中；3 处是正则被嵌套的「」截断（`implementation-workflow.md` 的「复用重型门禁的判定……」、`test-discipline.md` 的「阴性结果要能和「代码没跑到」分开」、`investigator.md` 指向 `implementation-writer.md`「开工先读：」那一个行标签），我对着 `grep -n '^#'` 逐个看过，标题存在 |
| 进程组那一句（`setsid --wait` / `start_new_session=True` / 脚本里对整组发信号 / 命令行里停进程照旧一次点名一个）在 `agent-common.md`、`three-way-forward.md`、`three-way-attack.md`、`three-way-defense.md` 四处逐字相同 | 4 | 相同（`python3` 对子串 `count`：各 1） |
| `detached-run.sh` 文件头「认得出 / 认不出由重型闸拒 / 罩不住」三段（20 格：bash/source/直接执行/改名拷贝、cp·install·ln·mv 再起、变量代入、eval 字面、`/usr/bin/time`·flock·systemd-run --wait·xargs 后的字面路径、`eval "$c"`·未赋值变量·`xargs -I`·`find -exec` 拒、删签名行的拷贝与 python subprocess 放） | 20 | 20 格都与文件头（`scripts/hosts/detached-run.sh:120` 起）一致 |
| `agent-common.md` 检出 ②④：带 break、`while read`、`kill -0` 的循环前台照拒、套 `timeout` 放；`wait`、函数里的 `wait`、`wait -n`、`tail --pid`、`kill -0`、`wait $p1 $p2` 的数法 | 15 | 15 格一致 |
| 检出 ⑦ 的 `rsync --inplace`；⑬ 名单外启动器、here-string、`env -S`、`bash -i` 读 bashrc、`ENV` 文件、`--norc` 不读；⑮ 的 `git rm --cached`、`git checkout [<提交>] -- <整棵树>`、`git stash --keep-index`、`reset --soft` | 17 | 17 格一致 |
| 重型闸「放行靠参数的三份入口只认本仓那一份」：本仓 `--quick` 放，别处拷贝与符号链接读正文拒，本仓不带参数拒 | 4 | 4 格一致 |
| 「判不了的（输入不是 JSON 对象……）照常放行并记一条检出（只记）」：十份钩子（重型闸、检出钩子、派发闸、写闸、交回正文闸、弹窗闸、续做闸、完工清理闸、范围声明闸、定时闸）× 三种坏输入（非 JSON、`[]`、空） | 30 | 30 格都是退 0、检出恰好加 1 条（`tools/garbage_input.py`） |
| 派发闸准入句（`.claude/agent-common.md:96`）：退 77 / 2 / 3 拒；0、2、3、77 之外的码与「跑不了」放行并记检出 | 14 | 14 格一致（唯一不一致是 H12 的 python 自己退 2） |
| 会话临时目录各处认法：`lib_work_root.py` 文件头、`agent-write-scope.tsv` 注释、写闸、检出钩子 ⑦、派发闸、完工清理闸（`completion-cleanup-guard.sh` 改取 `claude_session_temporary_root`）、`agent-watch.py`、`copy-originals.py`、`clean-session-scratch.py` | — | G1.1 的喂法与 `grep -rln claude_session_temporary_root` 对上；旧名 `legacy_root` / `LEGACY_ROOT` / 「过渡期」在被判文件里 0 命中（`grep -rn` 零输出） |
| `.claude/main-agent.md:72` 的 `--ack` 四种键，与 `scripts/agents/agent-watch.py:16` 的四种键逐字相同；看门狗「只报本会话单元、别的会话的、排内存队的、声明了串行段的、登记成轻的自证底下的只在报告里记一行」与 `agent-watch.py` 文件头 200 行上下的分类、`detached-run.sh`、`core_usage.py` 文件头同口径 | — | 三处互相一致（读文，没起看门狗自证） |
| `.claude/rules/verification.md:35` 快档那一格「不带特性那一趟与特性里没有 gpu 的那一趟经 test-targets-parallel.py 并行起」 | — | 与 `.claude/gate.d/checker-tier-crash-point-replay.sh:175` 起的文件头一致（读文，没起这道门禁） |
| `.claude/skills/crash-test/commit-stages.md:26` 的对端密钥句（取不到退 2、核不过退 1、一行不写） | — | 与 `scripts/mutation/broad-mutation.py:120` 一致（读文） |
| `CLAUDE.md:60` 「四篇项目本地规则不 @ 常驻：派出去的 agent 由定义点名读」 | 4 | 四篇各自被至少一份定义点名（`grep -l`：mutation-sampling 四份、test-script-discipline 五份、path-moves 两份、directory-layout 两份）；`^@.claude/rules/` 行数 8，与调度记录第 114 行同 |

### G2.2　打中的

**S1　`.claude/agent-common.md:81` 一句理由写进了共用约束**
- 原文（那一行的后半）：`只记不拒（检出 ②）：run_in_background 里命令位置上不带 `--wait`（`-w`）的 `setsid`（前缀与 `capped.sh` 这类包装剥开）记一条检出，它是进程组组长时会 fork、自己马上退出，完成通知当场发出；写成 `setsid --wait`。`
- 对照：`.claude/agent-common.md:4` `共用约束与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进`……；`.claude/singlefs-ai-sop/rules/rules-discipline.md:13` 把「为什么这么定：论证、取舍、推导」放进「不写」一栏。「它是进程组组长时会 fork、自己马上退出，完成通知当场发出」解释为什么要写 `--wait`，是论证；删掉这半句读的人不少知道任何该怎么做的事（`写成 setsid --wait` 已经够）。
- 这一轮 `git diff HEAD` 新增行里命中「会 fork」「当场发出」的只有这一行（`grep -c` 本轮新增行：agent-common 1，其余 12 份 0）；日期、「用户 20…」「records/2026」的新增行 0。
- 推翻它：说明那半句在别处查不到、删掉会让人写错 `setsid`——`setsid --wait` 这一条写法本身已在同一句里。

**S2　`lib_heavy_tests.py` 文件头两处说 case 模式行被切成命令，`lib_shell_words.py` 已经折叠它**
- `.claude/hooks/lib_heavy_tests.py:218`（sha256 前 8 位 ce3b4387）：`不带通配的 case 模式行（`scripts/x.sh) …`）照旧被切成一条命令（lib_shell_words.py 不认 case，老口子）。`；`:256`：`接受的误拒：脚本里 case 的模式行被切成命令（lib_shell_words.py 不认 case` ……
- 此刻 `.claude/hooks/lib_shell_words.py:939`（0ea57581）`fold_cases = not words_break_is_set("case-patterns-as-commands")`：case 的模式默认折叠（弄坏开关 `case-patterns-as-commands` 才退回把模式词当命令词）。
- 喂法（实现员，cwd 在拷来的树）：`case x in` 换行 `  scripts/crash/crash-point-replay.sh) echo hi ;;` 换行 `esac`；同样的 `scripts/mutation/mutate.sh)`、`.claude/gate.d/checker-tier-crash-point-replay.sh)`、`.claude/gate.d/lib/*.py)`、`scripts/*)`：全部退 0、零检出，没有被当成命令去判。文件头说的「老口子」此刻不存在，「接受的误拒」那条也不会发生。
- 推翻它：弄坏开关打开时才有那个行为；平常喂上面的 case 退 2，才算文件头对。

**S3　`.claude/hooks/heavy-test-guard.sh:190` 说「照拒」，代码只在已经拒了的消息后面加说明（与用户口径第 14 行的「照拒」是两回事）**
- 文件头：`几个源的 cat 里有盘上不在的源（`cat s.sh "2"> d.sh && bash d.sh`）照把在的源接起来判、照拒（偏严，用户 2026-10-29 定）`。代码 `:1141` 只往 `missing_cat_sources` 记，`:1841` 的 `if state.missing_cat_sources and not guard_break_is_set("cat-missing-source-unexplained")` 在**已经决定拒**之后给消息加一段。
- 所以「照拒」只是「在的部分该拒才拒、顺带点名不在的源」；`cat src_ok.sh nonexist_src.sh > dd8.sh && bash dd8.sh` 退 0。另见 G1 的 H7：同一条命令里先写出再被 cat 的源，多源、`>>`、`tee` 三种都放。文字说「照拒」，行为是「有条件拒」，实现与用户口径差一截。

**S4　`.claude/hooks/bash-command-detector.sh:1940` 的拒绝说明说「分不出花括号在不在引号里」，而共用切词此刻分得出**（G1 的 H11）：说明已不真；`.claude/agent-common.md:86` 写「引号里的同样」，所以文档与实现一致，与说明、与用户口径第 14 行第三类的精神不一致。

**S5　同一个目录两个名字**
- `.claude/agent-common.md:39` 写 `Claude Code 给会话的临时目录 `/tmp/claude-<uid>/``，`.claude/agent-common.md:44` 与 `.claude/rules/three-way-inference.md:164` 写 `不放会话共用的暂存目录`。用户选定的名字是「会话临时目录」（`records/2026-10-29-一百三十七节那一轮调度记录.md` 第 99 行：`scratchpad 另立常设的「会话临时目录」`）。同一份 `agent-common.md` 里「会话临时目录」（39、96 行）与「会话共用的暂存目录」（44 行）并存，读的人分不清是不是同一个目录；规则文件 `.claude/rules/three-way-inference.md:164` 也还用旧叫法。低。
- 另：本次派发环境的系统提示里有一句 Scratchpad 目录「always use it for temporary files」（观测，来自我这次的运行环境，不是仓里的文件），与 `.claude/agent-common.md:44` 的「草稿放派发提示给的草稿目录」相反，仓里没有一处写明两句冲突时先听哪句；写闸又拒子 agent 往那里写。低；没追会话提示的出处。

**S6　`.claude/agent-common.md:96`（派发闸）「准入的 `admission.py` 跑不了……照常放行并记一条检出」与 H12 不符**：脚本不在时 python 自己退 2，被钩子按「登记里没有」拒。

**S7　`.claude/agent-common.md:76`「钩子 bash 那一层起不来」照常执行并记检出：python3 不在 PATH 时是 H13**。

**没打中、但值得一提的两处读起来像矛盾、核下来不是**
- `.claude/agent-common.md:38` 一句「不在任何子 agent 的写范围里：Write / Edit 写进去由写闸拒，没有项目定义的 agent 在 Bash 里写进去由检出钩子拒」——项目定义的实现员用 Bash 往该目录写 `>`、`tee`、`cp` 是放行的（battery「旧根:检出」组 `implementation-writer` 两格退 0）。句子没有说它们也拒，范围限定词齐全，不算过度外推；但读句子的子 agent 容易读成「任何子 agent 都写不进去」。
- `main-agent.md` 说声明了串行段的告警只记不叫醒、「读到时核理由」：看门狗确实只写报告一行，主 agent 读不到报告就核不到。这是 G3（攻方腿）的题，这里不判。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 认不出的一律拒（变量、命令替换、位置参数、进程替换、eval、启动文件、转发函数、xargs / find -exec） | 一致 | 26+77 条喂法与 18 条主 agent 前缀喂法全对；子 agent 与主 agent 不带前缀同判 |
| G1 三种静默漏拒改成拒（命令替换里无括号 case 与注释、coproc、位置参数与变量拼旗标） | 一致 | 7 条全拒 |
| G1 偏严三类：cat 不看源在不在 | 冲突 | `>` 单源拒；多源、`>>`、`tee`、同命令里先写出的源放（H7，.claude/hooks/heavy-test-guard.sh:190 与代码不符，S3） |
| G1 偏严三类：重定向目标花括号 | 一致 | 展开成几个词的拒；引号里的也拒（H11，说明过时） |
| G1 偏严三类：函数体外引号里的花括号放 | 部分冲突 | 参数里放；命令词位置的 `'{cargo,test,--all}'` 被拒（H10） |
| G1 撤旧根两样 | 一致 | 工作根过渡期放行撤掉；会话临时目录根仍归 ⑦、写闸、派发闸、完工清理闸、看门狗等（`battery_other.py` 29 格对） |
| G1 准入退码 | 一致（一处小冲突） | 2、3、77 拒，别的码放行并记；python 自己退 2 的情形被误当「登记里没有」（H12） |
| G1 边界（本仓脚本与逐字节相同拷贝不判，改过的读正文） | 一致 | 内存包装与 `capped.sh` 的拷贝 5 格对；脚本正文里 `"$@"`、`$cmd` 的边界 6 格对 |
| G1 认不出一律拒有没有放过真重型 | 冲突 | 81 种同载荷写法放过且真起：H1 组合选项最重，H2 trap，H3 alias，H4 标准输入，H5 cd 失败，H6 `~+`，H8 改名 shell；H9 是已知没认的同类 |
| G1 有没有把正当命令拒掉 | 一致 | 20 条里 7 条按偏严口径拒，没有一条本身起重活的被放；A 组 43 条里只 H10 一条 |
| G2 新写的句子与钩子实际判法 | 大体一致 | 56 格喂法 + 30 格坏输入 + 74 处小节引用都对；不一致 4 条：S1、S2、S3、H13 |
| G2 只写怎么做（不写理由、经过） | 冲突 | S1：`.claude/agent-common.md:81` 一句理由；日期、用户语、records 引用的新增行 0 |
| G2 几份文件之间同一件事 | 大体一致 | 进程组句四处相同、`--ack` 四种键三处相同；叫法 S5、钩子文件头 S2 互相不一致 |

## 没做什么

- 不判攻方腿的格 G3（过度外推、内部矛盾与循环依赖、最脆弱的一条）；H1–H8 里哪条是「最脆弱」不在这里排。
- 不替主 agent 采纳或出判决；上面的「打中」都附了复跑命令，由主 agent 现查。
- 没起任何重型测试、cargo、GPU；探针都是钩子文件（只读 JSON 输入）与垫片 `cargo`。没起看门狗、内存包装、`clean-session-scratch.py`、`copy-originals.py` 的自证。
- 没读禁读清单里的文件（攻方腿的报告与模型目录）。
- 没喂的钩子：`lib_prompt_scratch_line.py`、`lib_snapshot_conflicts.py`、`handback-guard.sh` 的交回正文内容判法（只喂了坏输入）、`lib_write_scope.py` 单独的判表（经 ⑫ 与写闸间接喂过）；`mutation-triage.md` 这一轮没有改动（diff 0 行），没逐句对。
- G2 没逐句覆盖的新句子：`verification.md` 里别的会话改的 gpu / Verus 段、`agent-common.md` 里其余 20 处对重型闸和派发闸的改写中不能用一条命令喂出的（写者锁、快照清单、线程登记读不到那几句）。
- 主工作区一个文件没改。完工清理（量的，`du -sh`）：删前草稿目录里 `tree` 73M（喂钩子用的仓副本）、`rr2` 1.5M、`rr3` 1.5M、`nopy` 40K，共 78M；删后草稿目录 1.5M（脚本、日志、检出记录）。复跑时每步的仓副本 64M×8 由 `rerun.sh` 自己删。`ps` 核：样本进程（`mysh_d137`、沙箱里的 `heavy.sh`、各步 python）0 个。

## 引文核对表

下面每条都是对着被引文件 `grep -nF` 命中后生成的（原文在那一行里逐字出现，行号是命中的行号），交给 `scripts/agents/cite-check.py` 逐处比对；正文里其余只写行号的位置，行号同样取自这张表或 `grep -n` 现查。

- 「if argument in ("-o", "+o", "-O", "+O") or (argument in SHELL_LONG_OPTIONS_WITH_VALUE and option_values_read):」（`.claude/hooks/lib_shell_words.py:1479`）
- 「FUNCTION_CALLING_WORDS = frozenset({"eval", "source", ".", "trap", "compgen", "declare", "typeset", "set", "alias", "enable"})」（`.claude/hooks/lib_shell_words.py:1917`）
- 「fold_cases = not words_break_is_set("case-patterns-as-commands")」（`.claude/hooks/lib_shell_words.py:939`）
- 「几个源的 cat 里有盘上不在的源（`cat s.sh "2"> d.sh && bash d.sh`）照把在的源接起来判、照拒（偏严，用户 2026-10-29 定）」（`.claude/hooks/heavy-test-guard.sh:190`）
- 「读不到或看不全的脚本、脚本正文里喂给 shell 的不是字面的。」（`.claude/hooks/heavy-test-guard.sh:201`）
- 「#   cd 到展开不了的变量路径之后（变量不在这段命令文本里赋值」（`.claude/hooks/heavy-test-guard.sh:267`）
- 「source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"」（`.claude/hooks/heavy-test-guard.sh:763`）
- 「钩子 bash 这一层起不来时也记一条检出（一百三十七节 ⑤）」（`.claude/hooks/heavy-test-guard.sh:767`）
- 「notices.append(Notice(here, f"{missing}，里面的命令没判"))」（`.claude/hooks/heavy-test-guard.sh:1564`）
- 「state.missing_cat_sources.append((source, destination))」（`.claude/hooks/heavy-test-guard.sh:1141`）
- 「if state.missing_cat_sources and not guard_break_is_set("cat-missing-source-unexplained"):」（`.claude/hooks/heavy-test-guard.sh:1841`）
- 「这道闸切词时分不出花括号在不在引号里」（`.claude/hooks/bash-command-detector.sh:1940`）
- 「不带通配的 case 模式行（`scripts/x.sh) …`）照旧被切成一条命令（lib_shell_words.py 不认 case，老口子）」（`.claude/hooks/lib_heavy_tests.py:218`）
- 「接受的误拒：脚本里 case 的模式行被切成命令（lib_shell_words.py 不认 case」（`.claude/hooks/lib_heavy_tests.py:256`）
- 「Claude Code 给会话的临时目录 `/tmp/claude-<uid>/`」（`.claude/agent-common.md:39`）
- 「草稿放派发提示给的草稿目录，不放会话共用的暂存目录」（`.claude/agent-common.md:44`）
- 「它是进程组组长时会 fork、自己马上退出，完成通知当场发出」（`.claude/agent-common.md:81`）
- 「重定向目标上展开成几个词的花括号（bash 报 ambiguous redirect、一个字节都不写；引号里的同样）」（`.claude/agent-common.md:86`）
- 「不放会话共用的暂存目录」（`.claude/rules/three-way-inference.md:164`）
- 「不带特性那一趟与特性里没有 `gpu` 的那一趟经 `python3 scripts/gate/test-targets-parallel.py --jobs <每台默认重活线程数> -- <cargo test 的参数>` 按测试目标并行起」（`.claude/rules/verification.md:35`）
- 「# 快档两趟按测试目标并行起」（`.claude/gate.d/checker-tier-crash-point-replay.sh:175`）
- 「键四种：`进程:<pid>`、`<agent 号>:<告警名>`、`单元:<名>`」（`.claude/main-agent.md:72`）
- 「键四种：进程:<pid>、<agent 号>:<告警名>、单元:<名>、类:<告警名>」（`scripts/agents/agent-watch.py:16`）
- 「#   罩不住的写法：删掉签名行的拷贝；python 的 subprocess 起的；命令词不带斜杠、从 PATH 里找到的拷贝」（`scripts/hosts/detached-run.sh:120`）
- 「取不到对端密钥退 2、有一行用对端密钥核不过整表退 1，两种都一行不写」（`.claude/skills/crash-test/commit-stages.md:26`）
- 「取不到（不在、权限不对、内容写坏、落在库目录里）拒导入（退 2，一行都不写）」（`scripts/mutation/broad-mutation.py:120`）
- 「| 怎么做：步骤、次序、判据、阈值 | 为什么这么定：论证、取舍、推导 |」（`.claude/singlefs-ai-sop/rules/rules-discipline.md:13`）
- 「认不出的一律拒（偏严）；三种静默漏拒（命令替换里的 case 模式与注释、`coproc`、Z15/Z16）照改成拒」（`records/2026-10-29-一百三十七节那一轮调度记录.md:13`）
- 「cat 不看源文件在不在、重定向目标花括号展开成几个词：照拒，拒绝说明改贴切；函数体外引号里的花括号：放（切词带出引号信息）」（`records/2026-10-29-一百三十七节那一轮调度记录.md:14`）
- 「与带日期的记录写进」（`.claude/agent-common.md:4`）
- 「scratchpad 另立常设的」（`records/2026-10-29-一百三十七节那一轮调度记录.md:99`）
