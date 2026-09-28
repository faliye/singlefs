# defs-gatebatch-m2-r2 云端攻方（Opus）报告：K1、K3、K4

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 K1 K2 K3 K4 K5 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 D1 D2 D3 -->

写于 2026-09-27。被判的是开工快照 `research/prompts/defs-gatebatch-m2-r2-snapshot/sha256sums.txt` 那 11 个文件；开工时 `sha256sum -c` 11 个全 OK。派发没给禁读清单，按「无」办。

## 复跑

```
cd <仓根> && bash research/prompts/defs-gatebatch-m2-r2-opus-model/rerun.sh <输出目录> [草稿目录]
```

- `rerun.sh` 依次跑四个探针（`probe_k1_hook.py`、`probe_k1_manifest.py`、`probe_k3_judge.py`、`probe_k4_cost.py`）、仓里 `admission.py --selftest`、`verify_semantics.sh`（经 `run-with-memory-cap.sh 4G`，在真工具上核攻击格的前提），再 `rsync` 一份仓副本、用 `make_fix_copy.py` 打上我提的改法 P1–P10，在副本上跑三份自证与两个 K1 探针。探针退出码：0 全成立、1 有攻击格打中、2 对照格破了。
- 探针只把命令当 JSON 喂给闸（不执行），或在草稿目录的小仓上跑被判的 `admission.py` 子命令；被判的重型命令一条都没执行。
- 入库的输出在 `research/prompts/defs-gatebatch-m2-r2-opus-model/outputs/`（这一次 `rerun.sh` 的原样），每个文件的 sha256 在同目录 `SHA256SUMS`（34 行，`SHA256SUMS` 自己的 sha256 是 `3f6b8fa9afa546a6b2b2bad43276c2c9d38cdef2ce30b60aa21956363a598753`）。脚本本身：

| 文件 | sha256 |
|---|---|
| `make_fix_copy.py` | `b697ed3864c8e68bf90c7334030c303735397332c601f463f7beaa2be1624a6e` |
| `probe_common.py` | `6029bac85ad8383e3ff6c13103a290fcf0b0a23678ebba7af6e08436597b9e43` |
| `probe_k1_hook.py` | `f2658f6b96a2c35af17d12e17f5341c5ed703afe2c06a6bda1f842f45756f452` |
| `probe_k1_manifest.py` | `767aa88bd23f7c328d9ab40d65944453d51105aaf3517ca6c234b6d1c1ea3883` |
| `probe_k3_judge.py` | `a50db88a4a421278182485e1819f7bf3f6df91cfb09574e71396de24a3c73c14` |
| `probe_k4_cost.py` | `f2acccefae82187b973161730dc064254a437425572ef79db9095cf107840752` |
| `rerun.sh` | `984533c5fa0d943cd6f0b4d578244e8c63d9b257b36ea231c5430ae9012c7d75` |
| `verify_semantics.sh` | `fadeeb2f2fa000a8bdfec77f0445535c2fa0e952e2a35e68ce509710195375ba` |

各探针的原样汇总行（`outputs/*.log`）：

```
SUMMARY K1 闸与属性解析: cells=44 attack_cells_broken=18 control_cells_broken=0          （k1-hook.log，仓里被判的那一份）
SUMMARY K1 排除规则与 runner 指纹: cells=12 attack_cells_broken=8 control_cells_broken=0   （k1-manifest.log，同上）
SUMMARY K3 两条登记行的判法: cells=12 attack_cells_broken=0 control_cells_broken=0        （k3-judge.log；另有 GAPS 5）
SUMMARY K4 窄的判法摘要: cells=8 attack_cells_broken=0 control_cells_broken=0             （k4-cost.log）
SUMMARY K1 闸与属性解析: cells=44 attack_cells_broken=2 control_cells_broken=0            （fix-k1-hook.log，改法副本）
SUMMARY K1 排除规则与 runner 指纹: cells=12 attack_cells_broken=2 control_cells_broken=0   （fix-k1-manifest.log，改法副本）
```

## 各格判定一览

「新引入」指这一格在 Y 改动之前是对的（或那一类文件原来在指纹里），是改法带进来的；「没修全」指改法要堵的那一类里还剩的形状。

| 格 | 判定 | 形状（量过的格号） | 新引入 / 没修全 |
|---|---|---|---|
| K1-Y1 `--extra-file` 两处是不是同一份 | **没打中** | 快档与 `--full` 都经 `write_crash_case_manifest` 同一个函数算；`crash-case-record` 不重算，记的是开跑那一次、且与跑完那一次相等才写 | — |
| K1-Y2 属性解析 | **打中（量过）** | V3 同名函数 cfg 二选一（debug 标 ignore、release 不标）、V4 子模块里同名且标 ignore 的写在前面：闸放行、`crash-cases` 退 0；V5 宏生成的用例：闸放行（自查退 2）；V7 `# [ignore]` 误拒、自查误判红 | 没修全（V3–V5）；误拒（V7） |
| K1-Y3 剥包装 | **打中（量过）** | 短选项合写 `strace -fo`、`flock -xw`、`systemd-run -qu`、`/usr/bin/time -ao`（L2、L4、L6、L7；L13 不套内存包装时两道闸都漏）；`systemd-run -E` / `--setenv=` 设 runner（R2、R3）；`--config` 带引号的 `"runner"` 键（R7）。`flock … bash -c`、`env -i`、`systemd-run --scope` 三个派发点名的形状都拒对了 | 没修全 |
| K1-Y4 `--list` 不算 | **打中（量过），新引入** | `-- --include-ignored --skip --list`、测试二进制带同样参数、`--logfile --list`：`--list` 是前一个选项的值，libtest 照样全跑，闸按「只列」放行（T2–T4） | **新引入**：Y4 之前带 `--include-ignored` 就拒 |
| K1-Y5 排除规则的「拼出来的名字」 | **打中（量过）** | M1 `::core::include_str!(::core::concat!(…))`、M2 `include_str!(env!(…))`（路径在 `.cargo/config.toml` 的 `[env]`）：别的测试目标的文件被减掉，改它指纹不变；M3（`build.rs` 只写 `mod scan;`）属文件头已声明认不出的那一类 | 没修全（M1、M2）；已声明（M3） |
| K1-Y6 `mutations.tsv` 与 `src/bin` 不进 | **打中（量过），新引入** | M4 `include_str!(concat!(…))` 拼出 `crates/mutations.tsv`、M5 `include!(concat!(…))` 拼出 `src/bin/tool.rs`：Y5 的「整包不减」管不到这两种，改它们指纹不变 | **新引入**：Y6 之前这两类都在指纹里 |
| K1-Y7 runner 按内容进 | **打中（量过）** | G1 `runner = "bash tools/r.sh"`、G2 `runner = ["bash", "tools/r.sh"]`：只哈希了 bash；G3 runner 脚本 `source` 的文件改了 | 没修全（G1、G2 看得见；G3 判别子观测不到） |
| K3 floor-raise（第 36 行） | **没打中** | 只跑了部分切片（读回 0 + 这一趟 256 ≠ 512）判红（F2），靠的是 `threads=` 那一格的片数守恒；缺 `exhaustive=` 在实质上不弱于两条流：用例断言 `tally.states` 等于闭式，与两条流 `exhaustive=` 的算法相同 | — |
| K3 c561（第 37 行） | **问句属实（量过）；不算新打中** | 门禁只要 1 passed 加一行 `C561_SIGMA_FULL`：状态数、判缺席数、线程数一概不看（S2–S4 判绿）；标记里 `worker_threads=` 那一格记的是配置值，不是起了几个线程 | 规则第 89 行最后一句写明「门禁看不出」，属已声明；标记那一格的字面会误导 |
| K4 整份 admission.py 进指纹的代价 | **代价是真的，今天为零；有更窄的办法（量过）** | 今天 common-dir 里崩溃枚举用例的全绿标记 0 格，这一次什么都没废；以后：判法闭包 365 / 3194 行（11%），这一批 21 个 hunk 0 个落在闭包里，可 Y1 下每一个都让四条全重跑；窄的「判法摘要」对 4 种非判法改动不变、对 4 种判法改动都变 | — |

## K1：修补本身

### Y1 `--extra-file`：两处是同一份，没打中

- 快档与 `--full` 都调同一个 `write_crash_case_manifest`，`--extra-file` 只写在一处：`.claude/gate.d/54-layer0-replay.sh:173` `      --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" \`、`:174` `      --extra-file "<判它的准入模块：admission.py>" "$layer0_admission_module" --toolchain --build-environment)"; then`。`crash-case-record` 不重算指纹，记的是开跑那一次的（跑完那一次与它相等才走到这一步）；`crash-case-marker-check` 拿的是快档刚算的。
- 两处读的 54 号与准入模块都是「这一份 54 号所在的那棵树」里的：`gate.sh --staged` 在临时 worktree 里按 `$ROOT/.claude/gate.d` 起阶段（`.claude/singlefs-ai-sop/scripts/gate.sh:505` 取 `GATE_D`），`--full` 照定义用 worktree 里那一份，两边都是暂存内容。名字那一格取 `basename "$0"`，门禁用 `bash <阶段>` 起（不 `source`），`$0` 两边相同。
- 仓里自证原样（`outputs/selftest-admission.log:151`、`:165`、`:166`）：`  ✓ 54 号快档：两条流跑不标 ignored 的用例，三条用例的标记都作数，判绿`；只改 54 号、只改准入模块各加一行注释，快档照样核标记、三条都判红。
- 试过没打中的：主工作区不带 `--staged` 跑快档（工作区里准入模块有没暂存的改动）⇒ 指纹与 worktree 里 `--full` 写的不同，只会假红；这一形第一轮 J3-c 已记，不重复。

### Y2 `#[ignore]` 属性解析：打中（量过）

判法只看源码里第一处 `fn <名>(`：`research/scripts/admission.py:1270` `    found = re.search(r"\bfn\s+" + re.escape(function) + r"\s*\(", code)`；认的属性写法 `research/scripts/admission.py:153` `IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\[\s*ignore\s*(?:\]|=)")`；闸判不出时放行 `.claude/hooks/lib_heavy_tests.py:185` `    return module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function) is False`（它上面 `:181` 的说明：`    标了、找不到那个目标或那个函数、导入不了 admission.py 都交 False（判不出的不在这里拒，提交时 54 号的 crash-cases 自查判红）。"""`）。

小仓（包 `pkg`、目标 `own_case`、登记 `test=pkg:own_case:the_case`），闸喂的是子 agent 经内存包装的 `cargo test --release -p pkg --test own_case`（不带 `--ignored`）。原样（`outputs/k1-hook.log`）：

```
33:ATTACK	BROKEN	V3 debug 下一份标 ignore、release 下一份不标（同名、cfg 二选一）：闸 ⇒ 应当拒	闸退 0
34:ATTACK	BROKEN	V3 debug 下一份标 ignore、release 下一份不标（同名、cfg 二选一）：crash-cases 自查 ⇒ 应当退 2	自查退 0：crash-case:own	pkg	own_case	the_case
35:ATTACK	BROKEN	V4 子模块里一份标 ignore 的同名函数写在前面，顶层那一份（--exact the_case 跑的）不标：闸 ⇒ 应当拒	闸退 0
36:ATTACK	BROKEN	V4 子模块里一份标 ignore 的同名函数写在前面，顶层那一份（--exact the_case 跑的）不标：crash-cases 自查 ⇒ 应当退 2	自查退 0：crash-case:own	pkg	own_case	the_case
37:ATTACK	BROKEN	V5 宏生成的用例（没有字面的 fn the_case(），不标 ignore：闸 ⇒ 应当拒	闸退 0
41:OVER	BROKEN	V7 # [ignore]（# 与 [ 之间有空格，rustc 认）：闸 ⇒ 应当放行	闸退 2
42:OVER	BROKEN	V7 # [ignore]（# 与 [ 之间有空格，rustc 认）：crash-cases 自查 ⇒ 应当退 0	自查退 2：…
```

前提在真 rustc 上成立（`outputs/semantics.log`，`rustc --test -C debug-assertions=off -O` 编的 release 版、不带 `--ignored`）：第 8–9 行 `THE_CASE_RELEASE_UNIGNORED_RAN`、`test result: ok. 2 passed; 0 failed; 3 ignored; …`（V3 的 release 那一份不带 `--ignored` 照跑；3 个 ignored 里含 `# [ignore]` 那一条，V7 rustc 认）；第 19–20 行 `TOP_LEVEL_UNIGNORED_RAN`、`… 1 filtered out`（V4：`--exact the_case` 跑的是顶层不标 ignore 的那一份，子模块那一份叫 `slow::the_case`）。

- V3、V4：闸与 `crash-cases` 自查都拿「第一处」的属性判，第一处标了就放行，而 `--release`（V3）或 `--exact`（V4）跑到的是没标的那一处。54 号 `--full` 本身带 `--include-ignored`，不受影响；受影响的是 Y2 要挡的「不带 `--ignored` 的 cargo test 照样跑到全量」。形状是构造的：今天四条真用例在各自的测试文件里 `grep -cE "\bfn\s+<用例函数>\s*\("` 都是 `1`（四条逐个现跑），打了 P4 的副本模块在真仓上 `crash-cases` 退 0。
- V5：自查退 2（判红），闸放行。闸在「判不出」时按没事处理，理由是提交时 54 号判红；可在这之前那一条已经跑完了。
- V6（`#[cfg_attr(…, ignore)]`）两边都按没标算，照设计（`research/scripts/admission.py:152` `# 用例函数要标的属性：#[ignore] 或 #[ignore = "…"]（cfg_attr 里的条件 ignore 不算：条件不成立时不带 --ignored 照样跑它）`），是对照格，判对了。
- V2（`#[ignore = "有 ] 方括号与 \" 引号"]` 加 `#[rustfmt::skip]` 夹在中间）、V8（文档注释、块注释夹在中间）判对了。

四句：
- 分不分辨臂：这一轮没有并列的臂，被判的是 Y2 一种写法；V3–V5 打的是 Y2 的判法本身，不牵连别的改法。
- 看不看得到：看得到。V3、V4 两处 `fn the_case(` 都在同一份源码里，判法只要看全部定义；V5 找不到定义这件事本身就看得到。
- 分句：K1 那一行「每一处修补有没有引入新的假绿（该红不红）或新的假红」。V3–V5 属「该红不红」，但不是新引入的（Y2 之前闸与自查都不查 ignore）；V7 属「新的假红」（新引入，危害小，rustfmt 会把 `# [` 改成 `#[`）。
- 跑前条款的改法：第一轮判决给的改法就是 Y2 本身（`research/prompts/defs-gatebatch-m2-r1-main-verification.md:31`），V3–V5 上照样中。我的 P4、P4b、P5 见第六节。

### Y3 剥包装与 runner / 别名：打中（量过）

剥包装的循环见到不以 `-` 开头的词就当成被起的那条命令：`.claude/hooks/lib_heavy_tests.py:586` `        if not word.startswith("-") or word == "-":`；一个选项词吃不吃下一个词只按整词查表：`:592` `        position += 2 if option in options_with_value and not has_attached_value else 1`。于是短选项合写时，要值的那个字母藏在合写词里，它的值被当成命令。`systemd-run` 的 `-E` / `--setenv` 在表里（`:544` `    "systemd-run": {"-u", "--unit", "-p", "--property", "--description", "--slice", "-E", "--setenv", "-M", "--machine", "-H", "--host",`），只被当成「吃一个值」跳过，设的变量不带进里面那条命令。`--config` 里的 runner 按正则认：`:262` `RUNNER_CONFIGURATION = re.compile(r"\.runner\s*=")`。

子 agent 身份（`implementation-writer`）喂 JSON，除 L5、L6、L13、L14 外都套一层 `run-with-memory-cap.sh 8G`；目标都是 c561（`record_checker_judges_absence_by_the_persisted_set`），带 `-- --ignored` 或设 runner。`outputs/k1-hook.log` 原样（每行截到判定，完整命令在同一行后面）：

```
2:ATTACK	BROKEN	L2 strace -fo <文件>（短选项合写，strace 常见写法） ⇒ 应当拒	退 0
4:ATTACK	BROKEN	L4 flock -xw 10 <锁>（短选项合写） ⇒ 应当拒	退 0
6:ATTACK	BROKEN	L6 systemd-run --user --scope -qu k1（短选项合写） ⇒ 应当拒	退 0
7:ATTACK	BROKEN	L7 /usr/bin/time -ao <文件>（短选项合写） ⇒ 应当拒	退 0
13:ATTACK	BROKEN	L13 不套内存包装的 strace -fo：重型与「没经内存包装」两道都漏 ⇒ 应当拒	退 0
16:ATTACK	BROKEN	R2 systemd-run -E 设 runner、不带 --ignored ⇒ 应当拒	退 0
17:ATTACK	BROKEN	R3 systemd-run --setenv= 设 runner、不带 --ignored ⇒ 应当拒	退 0
21:ATTACK	BROKEN	R7 --config target.<三元组>."runner"=…（带引号的键） ⇒ 应当拒	退 0
```

对照都判对了：L1 `strace -f -o`、L3 `flock -x -w 10`、L5 `systemd-run … -q -u k1`、L8 `flock <锁> bash -c '…'`、L9 `flock <锁> -c '…'`、L10 `env -i PATH=…`、L11 `nice`/`timeout`/`strace -f` 套三层、R1 命令前缀设 runner、R6 不带引号的 runner 键、R8 `export` 设 runner，全部退 2。派发点名的三个形状（`flock … bash -c '…'`、`env -i`、`systemd-run --scope`）都没被剥成非重型。

前提在真工具上成立（`outputs/semantics.log`）：第 21–31 行 `strace -fo s.txt true`、`flock -xw 1 lockf true`、`/usr/bin/time -ao t.txt true`、`systemd-run --user --scope -qu <名> true` 都 `rc=0`；第 33 行 `K1_PROBE_VAR=seen`（`systemd-run --scope -E` 把变量带给里面那条命令）；第 35–37 行不带引号与带引号的 `runner` 键 cargo 都用上了（`RUNNER_USED target/debug/semanticsprobe`）。

不算打中的两格：
- R5（`--config 'alias."xt"=…'`）闸放行，但 cargo 1.98 根本不认 `--config` 里定的别名：`outputs/semantics.log:40-41` `== cargo --config alias.k1zz="version" k1zz`、`error: no such command: \`k1zz\``，而 `CARGO_ALIAS_K1ZZ=version` 认（第 43 行 `cargo 1.98.0 (797e8a9bc 2026-08-05)`）。这条路在本机走不到。顺带：第一轮 J4-a 的 B3（`--config` 别名）按这个结果在本机同样走不到，交辩方（K5）核，我不下判。
- L12（`env -S '<整条命令>'`）闸放行、`env -S` 真能起命令（第 28 行 `ENV_S_RAN`），但 `env` 的剥法在 `lib_shell_words.py`，这一批没改它（与改法前的备份逐字节相同，`cmp` 现核），属第一轮「闸的绕法」那一面，不算这一批的修补。

四句：
- 分不分辨臂：不涉及臂。
- 看不看得到：都看得到，写法全在命令文本里；短选项合写只要逐个字母查同一张表，`-E` 只要把 `NAME=VALUE` 带进里面那条命令的环境，带引号的键只要按 TOML 读 `--config` 的值。
- 分句：「该红不红」。都属「没修全」：Y3 之前这几个程序整个不剥，同样放行。L13 另多漏一道：剥不开时 `runs_compiled_code` 也看不到 cargo，「没经内存包装」那一道同样放行（L14 对照：剥得开的 `strace -f -o` 跑非重型目标，照样因没经内存包装被拒）。
- 跑前条款的改法：Y3 本身在这几格上照样中。我的 P1、P2、P10 见第六节。

### Y4 带 `--list` 不算：打中（量过），新引入

判法只看 `--list` 这个词在不在：`.claude/hooks/lib_heavy_tests.py:422` `    if LIST_ONLY_TEST_ARGUMENT in libtest_arguments:`、`:607` `        if LIST_ONLY_TEST_ARGUMENT in arguments:`。libtest 里 `--skip`、`--logfile`、`--test-threads`、`--format`、`--color` 都带一个值，`--list` 可以是那个值。闸的文件头说的是 `.claude/hooks/heavy-test-guard.sh:24` `# cargo mutants 与 --config / CARGO_ALIAS_ 定的别名；libtest 参数带 --list 的只列用例、一条都不跑，哪一类都不算）：`。

```
24:ATTACK	BROKEN	T2 -- --include-ignored --skip --list（--list 是 --skip 的值，照样全跑） ⇒ 应当拒	退 0
25:ATTACK	BROKEN	T3 直接执行测试二进制 --include-ignored --skip --list ⇒ 应当拒	退 0
26:ATTACK	BROKEN	T4 -- --include-ignored --exact <全量用例> --logfile --list ⇒ 应当拒	退 0
```

真 libtest 上（`outputs/semantics.log:10-17`）：`--include-ignored --skip --list` 跑了全部 5 条，含两条 ignored（`FULL_IGNORED_RAN`、`SPACED_RAN`），`test result: ok. 5 passed; 0 failed; 0 ignored; …`；`--include-ignored --exact full_ignored --logfile --list` 跑了那一条 ignored（`FULL_IGNORED_RAN`、`… 1 passed …`），日志写进名叫 `--list` 的文件。对照 T1（`-- --ignored --list`）放行，判对了。

四句：
- 分不分辨臂：不涉及臂。
- 看不看得到：看得到，libtest 各选项带不带值是固定的一张表。
- 分句：「引入新的假绿」。**这是新引入的**：Y4 之前闸见到 `--include-ignored` 就拒（第一轮 R1 那一格的误拒正是它），Y4 为了放行只列的命令，连带放行了真跑全量的这三种。写法不常见，但 `--skip <过滤>` 本身是常用参数，只要过滤串恰好是 `--list` 就触发；闸是执行前唯一的一道，看门狗在进程那一层用的也是同一个 `classify`（`lib_heavy_tests.py` 文件头第 3 行），同样放行（推的，没喂看门狗）。
- 跑前条款的改法：Y4 本身在这三格上照样中；P3 见第六节。

### Y5、Y6 排除规则：打中（量过），Y6 那两格是新引入的

「拼出来的名字」只认一种字面：`research/scripts/admission.py:158` `COMPILE_TIME_CONCATENATED_INCLUDE = re.compile(r"\binclude(?:_str|_bytes)?!\s*[(\[{]\s*concat!")`，而且只管测试文件的减法（`:1381` `    concatenating = [name for name, text in code_texts.items() if name.endswith(".rs") and COMPILE_TIME_CONCATENATED_INCLUDE.search(text)]`、`:1386` `        if any(name.startswith(package_prefix(package_directory)) for name in concatenating):`）；Y6 新加的两类减法各有自己的「点名」判法、不看它：`:1404` `    left_out = {name for name in CRASH_CASE_FILES_NOT_READ`、`:1405` `                if name in listed_set and not any(os.path.basename(name) in text for text in code_texts.values())}`；`:1415` `            mention = "bin/" + name[len(bin_prefix):]`。构建脚本那一条只读构建脚本那一份文件：`:1379` `    if any(tests_word.search(code_texts.get(name, "")) for name in build_script_names(root, package_directories, listed_set)):`。

小仓（包 `pkg`，用例 `own_case`，另有测试目标 `other_target`、`src/bin/tool.rs`、`crates/mutations.tsv`），跑被判的 `crash-case-manifest … --toolchain --build-environment`，改一份用例其实读得到的文件，看指纹。`outputs/k1-manifest.log` 原样（截到指纹）：

```
3:ATTACK	BROKEN	M1 ::core::include_str!(::core::concat!(…))（带路径的宏名，编得过、读同一份） ⇒ 改 crates/pkg/tests/other_target.rs 指纹应当变	改前 730d1b23600cc67d… 改后 730d1b23600cc67d…
4:ATTACK	BROKEN	M2 include_str!(env!("K1_FIXTURE"))，路径在仓根 .cargo/config.toml 的 [env] 里（relative = true） ⇒ 改 crates/pkg/tests/other_target.rs 指纹应当变	改前 07253dee10dc690a… 改后 07253dee10…
6:ATTACK	BROKEN	M3 gen 的 build.rs 只写 mod scan;，按目录读 pkg 的 tests/ 的那几行在 scan.rs 里 ⇒ 改 crates/pkg/tests/other_target.rs 指纹应当变	改前 d5ad98ece8532053… 改后 d5ad98ece8532053…
7:ATTACK	BROKEN	M4 include_str!(concat!(…)) 拼出 crates/mutations.tsv ⇒ 改 crates/mutations.tsv 指纹应当变	改前 f4bcff450e202357… 改后 f4bcff450e202357…（文件数 / 减去数 改前 5/2 改后 5/2）
8:ATTACK	BROKEN	M5 include!(concat!(…)) 拼出 src/bin/tool.rs 里的代码 ⇒ 改 crates/pkg/src/bin/tool.rs 指纹应当变	改前 43f35d7b9292b91e… 改后 43f35d7b9292b91e…
```

对照判对了：C1（用例不读 `other_target`，改它指纹不变）、C2（不带路径的 `include_str!(concat!(…))`，整包不减，改它指纹变）、C3（别的包的 `build.rs` 自己写 `read_dir("../pkg/tests")`，改它指纹变）。

M1、M2 的前提在真 cargo 上成立：`outputs/semantics.log:44-46` `ENV_LEN=62 QUALIFIED_LEN=62 SAME=true`、`test result: ok. 1 passed; …`——`include_str!(env!("K1_FIXTURE"))`（`.cargo/config.toml` 的 `[env] K1_FIXTURE = { value = "tests/other_target.rs", relative = true }`）与 `::core::include_str!(::core::concat!(…))` 读到的都是 `tests/other_target.rs` 的原文。

派发问的「`include_str!(env!(…))` 这类怎么算」：**按「用例读不到」减掉**。路径不在任何 `.rs` 里，而在仓根 `.cargo/config.toml`；那份配置按内容进指纹（构建环境一行），可排除法不读它找点名，改 `other_target.rs` 本身指纹不变（M2）。`option_env!`、构建期从环境里取路径的同一类。

- M1、M2：Y5 要按宽处理的正是「include 套拼出来的名字」，换个写法就漏。文件头把「编译期拼出来的名字（include!(concat!(…)) 这一类）指到别的包的测试文件」列为认不出（`research/scripts/admission.py:97`），M1、M2 指的是**本包**的测试文件，文件头说按宽处理的正是这一种（`:98` `被减掉而它其实被读了。认得出形状、认不出读的是哪一份的两种按宽处理：包里有 include! / include_str! / include_bytes! 套 concat! 的，`），所以算没修全，不算已声明。
- M3：文件头第 96–97 行已声明「构建脚本（build.rs 与 package.build 指的那一份）之外的构建期代码按目录读 tests/」认不出；`build.rs` 经 `mod scan;` 引进来的 `scan.rs` 可以读成属于这一句。算已声明，只补一个具体形状，不计打中。
- M4、M5：**新引入**。Y6 之前 `crates/mutations.tsv` 与 `src/bin/*.rs` 在每条用例的输入里；Y6 让它们在「没有代码按文件名点名」时减掉，而 Y5 那条「拼出来的名字就不减」只挡测试文件，挡不到这两类。文件头「认不出的」一段（第 96–99 行）只写了测试文件，没写这两类。形状是构造的：今天真仓 `crash-case-manifest` 四条都是 `61 77`（`outputs/real-repo-manifests.log` 前四行）；`grep -rn mutations crates --include=*.rs | wc -l` 现跑是 `38`，而四条的清单里 `crates/mutations.tsv` 照样被减掉，说明这 38 处都不在注释之外按文件名点名它（没逐处看，推的）。

四句：
- 分不分辨臂：不涉及臂。
- 看不看得到：M1 看得到（宏名前允许 `::core::`、`std::` 这类路径）；M2 看得到一半：`env!` 套在 include 里看得到，但路径的值在 `.cargo/config.toml`，要么把「include 套 `env!`」同样按宽处理，要么把配置文件也纳入找点名；M4、M5 看得到（同一个正则用到 ② ③ 两类上）。
- 分句：「该红不红」（该重跑的用例被判成可复用）。M4、M5 是「新的假绿」；M1、M2 是没修全。
- 跑前条款的改法：Y5、Y6 本身在这几格上照样中；P6、P7 见第六节。

### Y7 runner / wrapper 按内容进指纹：打中（量过）

只哈希值的第一个词：`research/scripts/admission.py:501` `    program = words[0] if words else ""`；环境变量那一处 `:481` `            lines.append((f"<构建环境：环境变量 {variable} 指的程序>", program_contents(environment[variable].split(), root, environment)))`；配置文件那一处 `:539` `        words = runner.split() if isinstance(runner, str) else [str(word) for word in runner] if isinstance(runner, list) else []`。cargo 的 runner 按空白切、首词是程序、其余是参数，所以 `runner = "bash tools/r.sh"` 真正决定行为的脚本在参数里。

```
10:ATTACK	BROKEN	G1 runner = bash <仓>/tools/r.sh（前面带解释器，cargo 按空白切），改 r.sh ⇒ 改 tools/r.sh 指纹应当变	改前 43d61d5de7dc5a05… 改后 43d61d5de7dc5a05…
11:ATTACK	BROKEN	G2 .cargo/config.toml 里 runner = ["bash", "tools/r.sh"]，改 r.sh ⇒ 改 tools/r.sh 指纹应当变	改前 f62727a21db0898f… 改后 f62727a21db0898f…
12:ATTACK	BROKEN	G3 runner = <仓>/tools/r.sh，它 source 的 r-lib.sh 改了 ⇒ 改 tools/r-lib.sh 指纹应当变	改前 bbe7581bb94425fd… 改后 bbe7581bb94425fd…
```

对照 C4（`runner = <仓>/tools/r.sh` 直接指脚本，改它指纹变）判对了。前提：`outputs/semantics.log:38-39` 环境变量 `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="bash <脚本>"` 时 cargo 照样经它起（`RUNNER_USED target/debug/semanticsprobe`）。

派发问的「runner 指向的脚本再读别的文件时指纹跟不跟」：**不跟**（G3）。这一格属「判别子观测不到」：脚本 `source` 什么、读什么，不跑它就不知道；只能按宽处理（例：runner 指到仓内脚本时整条用例直接不许复用），或写进文件头「认不出的」。G1、G2 看得到（把参数里指到的现存文件一并按内容哈希）。今天本机 `~/.cargo/config*` 不在、仓根 `.cargo/` 下只有 `mutants.toml`（`ls` 现核），这三格今天走不到。

四句：分不分辨臂——不涉及；看不看得到——G1、G2 看得到，G3 看不到；分句——「该红不红」（换了 runner 行为而复用旧标记），都属没修全（Y7 之前连首词都不哈希）；跑前条款的改法——Y7 本身在三格上照样中，P9 修 G1、G2，G3 不修。

## K3：`stage-inputs.tsv` 第 36、37 行两条用例的判法

这一节只攻「这两条登记在今天的判法下会不会放过该红的日志」；两条登记行逐字段的事实表归本地攻方，这里不重复。探针拿真登记表里这两行（`admission.crash_case_of_key` 读出来的 `count_lines` / `exhaustive_lines` / `thread_lines`），把合成日志喂 `judge_crash_case_log`（54 号 `--full` 调的 `crash-case-judge` 就是它），判绿的再拿要记的行拼一格标记过 `crash_case_marker_problems`（快档核的那一步）。`outputs/k3-judge.log` 原样：

```
1	# floor-raise 登记：count_lines=['LAYER0_PARALLEL_FINISHED'] exhaustive=[] threads=['LAYER0_PARALLEL_FINISHED']
2	# c561 登记：count_lines=['C561_SIGMA_FULL'] exhaustive=[] threads=[]
4	CONTROL	holds	F2 只跑了一半的片（resumed 0 + fresh 256 ≠ 512），用例却报过了 ⇒ 判红	--full 判：红：LAYER0_PARALLEL_FINISHED 那一行读回的片 0 + 这一趟跑的片 256 ≠ 总片数 512：…
5	CONTROL	holds	F3 跑了 512 片却只起 1 个线程（没显式设） ⇒ 判红	--full 判：红：本机 32 核、线程数没显式设成 1，这一趟跑了 512 片却只起了 1 个工作线程（多半是线程数没传进去）：…
8	GAP	BROKEN	F6 片数守恒、线程对，但 states=64（展开被改窄、用例断言跟着改） ⇒ 判红	--full 判：绿；快档核标记：作数
9	GAP	BROKEN	F7 片数守恒，但 resumed=256（NoProgressFile 下不该有读回的片） ⇒ 判红	--full 判：绿；快档核标记：作数
11	GAP	BROKEN	S2 计数行 states=4（只评了 4 个状态、用例断言跟着改） ⇒ 判红	--full 判：绿；快档核标记：作数
12	GAP	BROKEN	S3 计数行报 5 个判缺席（用例断言被改成不判它） ⇒ 判红	--full 判：绿；快档核标记：作数
13	GAP	BROKEN	S4 单线程跑完（日志里没有线程的任何痕迹），本机 32 核、线程数没显式设 ⇒ 判红	--full 判：绿；快档核标记：作数
```

（第 3、6、7、10、14 行是 F1、F4、F5、S1、S5 五个对照，全部 `holds`。GAP 格是「门禁这一层判不出、只靠用例自己的断言挡」，不计入打中数。）

### floor-raise（第 36 行，没有 `exhaustive=`）：部分切片判不绿，没打中

- **只跑了部分切片就退出的日志不会判绿**（F2）。挡住它的不是 `exhaustive=`，而是登记了 `threads=` 之后 `judge_worker_threads` 顺带核的片数守恒：`research/scripts/admission.py:1476` `    if resumed_slices + freshly_run_slices != slices:`。用例不留进度文件（`crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs:140` `        &Layer0Resume::NoProgressFile,`），`SINGLEFS_LAYER0_SHARD` 这一类续跑与分片变量它不读，54 号没清掉 `SINGLEFS_LAYER0_SHARD` 这件事在它身上没有后果。被打断的一趟没有 `test result: ok. 1 passed`，照样判红（F5 同理）。
- 缺 `exhaustive=` **在实质上不比两条流弱**：两条流的 `exhaustive=` 是用例自己算的 `tally.states == closed_form`（`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:750` `        tally.states == closed_form,`）；floor-raise 用例断言同一件事，闭式也取自同一个函数（同文件 `:112` `    let states = singlefs_harness::crash::closed_form_state_count(&segments_of_the_raise);`、`:142` `    assert_eq!(tally.states, states, "每一段都展开，状态数与闭式相同");`）。两边都靠用例代码，用例代码都在指纹里。差别只在标记里少一个人读得到的 `exhaustive=true` 见证，以及「断言被删而打印留着」这一种改动在两条流上门禁看得到、在它身上看不到（F6，推的改动形状）。
- F7（`resumed_slices` 非 0）今天到不了：`NoProgressFile` 下读回的片恒为 0，是枚举器打的数，用例改不了它。
- 单线程那一面：多于 1 核、没显式设 1、跑了至少两片却只起 1 个线程判红（F3）；不带进度文件时片数至少 64（`crates/singlefs-harness/src/crash.rs:1529` `const LAYER0_MINIMUM_SLICE_COUNT: u64 = 64;`），「只剩 1 片」那条豁免在它身上走不到。挂载那一遍自己起线程，不在判里，登记行注释已写明。

### c561（第 37 行，只有 `count-line=`）：问句属实，缺口已声明，标记里一格字面会误导

- 门禁对它只要「恰好一行 `test result: ok. 1 passed; …`」加「恰好一行 `C561_SIGMA_FULL `」。计数行里的 `states=`、`record_claimed_state_missing_unit=` 一个都不读（S2、S3 判绿），靠的全是用例自己的断言（`crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs:588` `    assert_eq!(state_count, 262_144, "σ 有 18 个写");`、`:590` `        reported_missing, 0,`）。
- **单线程跑完也判绿**（S4）：它按 `SINGLEFS_LAYER0_THREADS` 自己起线程（同文件 `:554` `    let worker_threads = u64::try_from(Layer0Parallelism::from_environment().worker_threads.get())`），不打任何线程行。规则那一段（`.claude/rules/implementation-workflow.md:89`）先写 54 号「这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红」，末句是「别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。」c561 是登记给 54 号的崩溃枚举用例，读者按前一句会以为它也被判了，实际它落在末句里。线程数不进它的判，这一层看不见的是「c561 自己的代码把线程写死成 1」：54 号没把线程数传进去、`Layer0Parallelism::from_environment` 坏了，都会改到 54 号或 `crates/singlefs-harness/src/crash.rs`，另外三条用例的指纹跟着变、跟着重跑、跟着被判线程（推的）；只改 c561 那份测试文件时，只重跑 c561，没人判它的线程。
- **标记里一格的字面会误导**：判绿之后 `worker_threads=` 那一格记的是 `--threads-text`（`research/scripts/admission.py:1911` `                   ("judged_root", values["--judged-root"]), ("worker_threads", values["--threads-text"])]`），对 c561 那一句只有 `SINGLEFS_LAYER0_THREADS=<值>（没设，取本机核数），本机 <n> 核`（`.claude/gate.d/54-layer0-replay.sh:384` 拼的）：是配置值，不是起了几个线程；读标记的人（崩溃验证员原样抄计数行）会当成「用了 n 个线程」（推的读法）。

四句：
- 分不分辨臂：不涉及臂。
- 看不看得到：c561 的线程数，54 号在它的日志里看不到（「判别子观测不到」）；要门禁判，得先让用例打一行（背景第二节已写要改 `crates/` 另派）。
- 分句：K3 那一行两个问句都属实——没有 `exhaustive=` 的那一条，部分切片的日志判不绿；c561 没有线程判定，单线程跑完判绿。
- 跑前条款的改法：Y8 只补登记字段、不改用例，对 c561 这两格不起作用（它打不出可登记的行）；这一点 Y8 自己已写明交主 agent。

## K4：Y1、Y6 让四条指纹全变的代价

### 这一次：什么都没废（量过）

`find "$(git rev-parse --path-format=absolute --git-common-dir)" -maxdepth 1 -name 'singlefs-crash-case-green*' | grep -c .` 现跑是 `0`：common-dir 里一格崩溃枚举用例的全绿标记都还没有。背景第二节「四条指纹都变了，全绿标记一个都不再作数」在今天没有实际损失，下一趟 `--full` 本来就要四条全跑。另外这一次就算换成更窄的办法，四条也照样全变：Y1 给清单加了新的一行（准入模块），Y6 把每条的输入从 69 个文件减到 61 个（真仓现跑四条都是 `61 77`，`outputs/real-repo-manifests.log`），清单本身变了。

### 以后：每改一次 admission.py 或 54 号，四条全重跑（量过在合成仓上，频率是推的）

- 仓里自证原样（`outputs/selftest-admission.log:166`）：`  ✓ 54 号快档（不带 SINGLEFS_GATE_FULL=1）：只改准入模块（加一行注释） ⇒ 照样核标记，判红并点名 crash-case:stream-a、crash-case:stream-b、crash-case:case-c，不退 77`；第 165 行同样一句，改的是 54 号。一行注释就让所有用例重跑。
- admission.py 里真正决定「日志判不判绿、标记作不作数、登记行怎么读」的，从 13 个入口（`judge_crash_case_log`、`judge_worker_threads`、`crash_case_marker_problems`、`read_crash_case_marker`、`check_crash_case_marker`、`write_crash_case_marker`、`crash_case_marker_path`、`parse_crash_case`、`crash_case_of_key`、`crash_cases_of` 与三个 `command_crash_case_*`）按 ast 顺着模块级名字求闭包：`outputs/k4-cost.log:1` `MEASURE admission.py 全文 3194 行；闭包 37 个定义、365 行（11%）；自证从第 1944 行起到文件尾共 1251 行`。其余 89% 是实验准入、门禁复用、门禁前提、构建环境、排除法、自证。
- 这一批 Y1–Y8 对 admission.py 的改动：`outputs/k4-cost.log:33` `MEASURE 附录二 admission.py 的 21 个 hunk：落在判法闭包里 0 个，闭包外 21 个`；改法前后判法闭包逐字节相同（第 11 行 `…判法摘要 0f18d61a0a890957…，今天 0f18d61a0a890957…，相同…`）。也就是说这一批 21 处改动没有一处改判法，按 Y1 却每一处都足以让四条全重跑。
- 54 号 403 行，决定「跑的是什么、怎么判」的只有起用例的那几行（`.claude/gate.d/54-layer0-replay.sh:205` 起的 `run_crash_case`、`:210` `    cargo test --release -p "$1" --test "$2" -- --include-ignored --exact "$3" --nocapture 2>&1 \`、线程数那一段与调 `crash-case-judge` / `crash-case-record` 的参数），其余是快档、出路句与说明。54 号的提交历史：`git log --format='%ad' --date=format:'%m-%d' -- .claude/gate.d/54-layer0-replay.sh` 现跑是 `09-24 09-22 09-20 09-19 09-18 09-16 09-14 09-14`，11 天里 8 次（admission.py 是这一批新建的、没进 git，没有历史可数）。照这个频率外推，54 号平均不到两天改一次，第二条流全量约 2.3–2.8 天（第一轮判决与背景里的数，都是推的），它的标记会一直追不上（推的）。

### 代价对不对

J1-b 要堵的是「判法改了，旧判法写下的标记照样作数」。整份文件进指纹堵住了它，但把「判法以外的任何字节」也算成判法：代价与堵的东西不成比例。用户原话「改了只跑改了的部分」（`records/2026-09-24-里程碑二收尾调度.md:190`）在这一处是反的：改的是准入模块的实验那一半，重跑的是崩溃枚举。我的判断：**方向对（判法必须进指纹），范围太宽**。

### 更窄、仍堵得住 J1-b 的三种办法

| 办法 | 做法 | 堵不堵 J1-b | 量过 / 推的 |
|---|---|---|---|
| D2 判法摘要 | admission.py 自己按 ast 从上面 13 个入口求模块级闭包，把闭包里每个定义的源码按名字排好算 sha256，以一行 `<判法：…>` 进清单，替掉「整份 admission.py」那一行；54 号那一侧，把起用例的命令与环境改由 admission.py 交出（例：一个 `crash-case-command` 子命令），54 号整份不再进指纹 | 堵：判法与它调的小函数、常量、登记行的读法，改一个字节摘要就变 | **量过（只在我的模型上，被攻过零轮）**：`outputs/k4-cost.log` 第 3–10 行，改实验准入、门禁复用说明、自证、排除规则四种，摘要不变；改 `judge_worker_threads` 的片数门槛、`fields_of_line`、`PASSED_ONE_TEST_FORM`、`parse_crash_case` 四种，摘要都变。54 号那一侧的改法是推的，没实现 |
| D1 拆文件 | 判法那 365 行搬进单独一份（例 `research/scripts/crash_case_judge.py`），只有它按 `--extra-file` 进指纹 | 堵一半：以后有人把判法写回 admission.py 的别处，就是第一轮 J1-b 原样再来一次；要另加一道自证核「admission.py 里不许有判日志的代码」 | 推的 |
| D3 每次重判存档的证据 | 标记里存判法读的全部行（`test result`、各计数行、全部 `LAYER0_PARALLEL_FINISHED` 行）与核数、线程数设没设；快档与 `--full` 复用前拿**今天的**判法把存档再判一遍，判法不进指纹 | 堵，且更强：旧标记被新判法重判，判法收严的旧标记自动作废，判法放宽的不必重跑 | 推的。代价：判法要守「读不到的行一律判红」，否则新规则读的行旧标记里没有、却被当成通过；今天的 `crash_case_marker_problems` 已经重判了 test result、计数行与 `exhaustive=`，没重判线程与片数守恒 |

D2 的限度（推的）：闭包靠静态引用，`getattr`、字符串拼出来的函数名看不见（今天闭包里没有这种写法，没逐行核）；子命令分派表（`main` 里那张）不在闭包里，有人把 `crash-case-judge` 指到别的函数时摘要不变，要把分派表里这几项一并算进去；Python 自身升级不进摘要。排除规则与构建环境那几段不进摘要是对的：它们改了，只要结果变，清单本身就变（`files_crash_cases_do_not_read` 改一句、结果不变时指纹不变，这时输入确实相同）。

## 我提的改法（只在我的模型上量过、被攻过零轮）

补丁由 `make_fix_copy.py` 打在仓副本上（原样 diff 在 `outputs/admission.py.patch` 98 行、`outputs/lib_heavy_tests.py.patch` 102 行）。副本上的数不算入库装置上的数。副本上三份自证原样：`outputs/fix-selftest-admission.log` 末行 `  ✓ admission.py 自证通过：167 格都对（…）`、`fix-selftest-lib.log` 末行 `  ✓ lib_heavy_tests 自检通过（查了 109 种：…）`、`fix-selftest-guard.log` 末行 `  ✓ 自检通过（查了 628 种，其中该拒 112 种）…`，与仓里三份的格数相同、全过（这几个补丁没补自证格，格数不变）。副本的准入模块在真仓上 `crash-cases` 退 0，四条 `crash-case-manifest`（不带 `--extra-file`）与仓里那一份逐字节同指纹、同 `61 77`（`outputs/real-repo-manifests.log` 前四行对后四行）。

| 改法 | 做什么 | 修哪几格 | 量过 / 推的 |
|---|---|---|---|
| P1 | `command_under_launcher` 认短选项合写：逐个字母查同一张表，头一个要值的字母之后剩下的是值，没剩下就吃下一个词 | L2、L4、L6、L7、L13 | 量过：`outputs/fix-k1-hook.log` 里五格都 `holds` |
| P2 | `systemd-run -E NAME=VALUE` / `--setenv=NAME=VALUE` 带进里面那条命令（交回的词前面补 `NAME=VALUE`，再切一遍时当环境变量） | R2、R3 | 量过：同上两格 `holds` |
| P3 | libtest 带值的选项（`--logfile`、`--skip`、`--test-threads`、`--format`、`--color`、`-Z`、`--shuffle-seed`）跳过它们的值再找 `--list`，cargo 与直接执行测试二进制两处都换 | T2、T3、T4 | 量过：三格 `holds`，对照 T1（`--ignored --list`）照样放行 |
| P4 | 属性按**每一处**同名的 `fn <名>(` 判：有一处没标 `#[ignore]` 就算没标（自查判红、闸按重型） | V3、V4 | 量过：四格 `holds`。代价（推的）：同一目标里别的模块有同名、合法不标 ignore 的快用例时误判红 |
| P4b | `#` 与 `[` 之间许空白 | V7 | 量过：两格 `holds` |
| P5 | 闸在「判不出标没标」（找不到函数、找不到目标、导入不了 admission.py）时按没标算 | V5（闸那一格） | 量过：`holds`。代价（推的）：admission.py 坏了时，点名登记目标的每条 `cargo test` 都被拒 |
| P6 | 「拼出来的名字」的正则认 `::core::` / `std::` 这类路径前缀，并把 `env!`、`option_env!` 与 `concat!` 同样对待 | M1、M2 | 量过：两格 `holds` |
| P7 | 任何一份 `.rs` 里有 P6 那种 include，`mutations.tsv` 与 `src/bin` 那两类也一份都不减 | M4、M5 | 量过：两格 `holds` |
| P8 | 构建脚本那一条连它 `mod` 进来的文件一起找整词 `tests` | M3 | 推的，没实现（M3 属已声明） |
| P9 | runner / wrapper 的值里，首词之后指到现存文件的参数也按内容进指纹 | G1、G2 | 量过：两格 `holds` |
| P10 | `--config` 的值按 TOML 读，定了 `target.<任何>.runner` 就当可能带 `--ignored`（正则那一道留着） | R7 | 量过：`holds` |
| D2 | 见 K4：判法摘要替掉整份 admission.py；54 号起用例的命令改由 admission.py 交出 | K4 的代价 | 摘要那一半量过（`outputs/k4-cost.log`），54 号那一半推的 |
| — | L12（`env -S`）、R5（`--config` 别名，cargo 不认）、G3（runner 脚本 `source` 的文件） | 不修 | L12 不在这一批；R5 走不到；G3 判别子观测不到，建议写进 admission.py 文件头「认不出的」 |

副本上改法之后剩下的原样：`outputs/fix-k1-hook.log` 汇总 `SUMMARY K1 闸与属性解析: cells=44 attack_cells_broken=2 control_cells_broken=0`（剩 L12、R5）；`outputs/fix-k1-manifest.log` 汇总 `SUMMARY K1 排除规则与 runner 指纹: cells=12 attack_cells_broken=2 control_cells_broken=0`（剩 M3、G3）。

## 推翻条件

- Y2 的 V3、V4 被推翻，如果：闸与 `crash-cases` 自查之外另有一道在执行前按编译结果（而不是源码文字）判用例有没有 `#[ignore]`；或者 libtest 在 `--release` 下不跑 cfg 选出来的那一份（`outputs/semantics.log:8` 已量到它跑）。
- Y3 的 L2–L7 被推翻，如果：项目 settings 里另有一道 PreToolUse 钩子拒这些写法（我只喂了 `heavy-test-guard.sh`），或看门狗在进程那一层剥得开合写的短选项（看门狗用的是同一个 `command_under_launcher`，推的，没喂）。
- Y4 的 T2–T4 被推翻，如果：真的 cargo 在把 `--` 之后的参数交给 libtest 之前改写了它们（`outputs/semantics.log` 量的是直接执行测试二进制，cargo 那一层是推的）。
- Y6 的 M4、M5「新引入」被推翻，如果：Y6 之前 `mutations.tsv` 或 `src/bin` 就已经被减掉（按附录二与改法前的备份，之前它们在清单里）。
- K3 的「部分切片判不绿」被推翻，如果：floor-raise 用例改成读续跑变量（这时读回的片能让片数守恒成立，而这一格还要看续跑的判法）。
- K4 的「代价与堵的东西不成比例」被推翻，如果：admission.py 在判法之外的部分几乎不再改（今天无历史可数），或第二条流全量实测远短于 2.3 天。

## 没打中的形状（试过什么、范围多大）

- Y1：快档、`--full`、`gate.sh --staged`、主工作区直接跑四种起法下 `--extra-file` 的名字与内容；只有主工作区带没暂存的改动时指纹不同，方向是假红（第一轮 J3-c 已记）。
- Y2：`#[ignore = "…"]` 里带 `]` 与转义引号、`#[rustfmt::skip]` 夹在中间、`pub(crate)`、文档注释与块注释夹在中间、`#[cfg_attr(…, ignore)]`（按设计判重型），全部判对（V2、V6、V8）。字符串里写 `fn <名>(`：头部截在字符串中间，`end_of_literal` 吞到末尾，属性串为空，判成没标，方向是假红（读代码推的，没量）。
- Y3：`flock <锁> bash -c '…'`、`flock <锁> -c '…'`、`env -i PATH=…`、`systemd-run --user --scope -q -u`、`nice` / `timeout` / `strace -f -o` 三层嵌套、`export` 设 runner，全部拒对（L1、L3、L5、L8–L11、R1、R6、R8）。`prlimit --nofile 1024 cmd`：util-linux 的资源选项值是可选的、要贴着写，`1024` 本来就被 prlimit 当成命令，剥错也只错在不存在的程序上（读 `prlimit --help` 推的，没量）。
- Y4：`--skip=--list`（一个词）照拒；`--format --list`、`--test-threads --list`、`--color --list` 在 libtest 上是参数错、一条都不跑（推的，没量）。
- K3：`SINGLEFS_LAYER0_SHARD=0/2` 从调用方环境漏进 54 号 `--full`（54 号不清它，`grep -c SHARD` 在 54 号里是 `0`）：两条流按分片跑一片就 `return`、不打计数行，判红，是假红不是假绿；floor-raise 与 c561 不读它。`merge/<n>` 那一种读账本交整条流的计数，账本文件头核输入指纹，是设计里的双机路径，不算打中（读代码推的）。
- 取样范围：`k1-hook` 44 格（26 格命令喂闸；Y2 九种写法各判闸与自查共 18 格），排除规则与 runner 12 格小仓，K3 12 格合成日志，K4 8 种改动；全是构造的，不是随机取样。

## 这条腿自己的限度

- 闸只喂了 JSON（`heavy-test-guard.sh`），没喂看门狗（`research/scripts/agent-watch.py`）；看门狗在进程那一层能不能认出 L2–L7、R2、R3、T2–T4 起的子进程是推的。cargo 把 `--` 之后的参数原样交给测试二进制这一句也是推的（libtest 的语义是在 `rustc --test` 直接编的小二进制上量的）。
- V3–V5、M1–M5、G1–G3 的形状全是构造的；今天真仓里没有这些写法（四条真用例各一处定义、真仓清单前后不变、本机没有 cargo 配置文件里的 runner）。它们说的是「改法留下的口子」，不是「今天已经漏了」。
- K4 的频率只有 54 号的 8 次提交可数，admission.py 没进 git；「标记一直追不上」是外推。第二条流 2.3–2.8 天是第一轮判决与背景里的推算，我没重量。
- D2 的判法摘要只在我的探针里算过，没接进 54 号、没跑过 54 号的场景自证；54 号那一侧（起用例的命令改由 admission.py 交出）没实现。
- 被判的文件在我干活的时候没变（开工快照 11 个全 OK）；`crates/` 有别的会话在改：floor-raise 那份测试文件的行号在我读的前后挪了 5 行，K3 引的行号是写报告那一刻现查的。
- 三次小编译（`rustc --test` 两份小测试、`cargo run` / `cargo test` 两个一文件的小 crate）头一遍没经 `capped.sh 6` 起，是我漏了派发的线程上限；之后写进 `verify_semantics.sh` 的那一遍（`outputs/semantics.log` 就是它的输出）都经 `capped.sh 6`，跑编出来的东西都经 `run-with-memory-cap.sh`。

## 没做什么

- 没跑任何重型测试：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的测试目标、全量 `cargo test` 一条都没起；54 号只在 admission.py 自证里以假 cargo 跑（仓里自证，没改它）。
- 没判 K2、K5（归辩方），没替主 agent 采纳；第一轮 B3 在本机走不到这一句只报事实，交 K5 核。
- 没改被判的任何文件；P1–P10、D2 只在草稿目录的仓副本上改过、量过，副本上的数不算入库装置上的数。
- 没跑登记给我的门禁阶段：三方攻方腿的定义没写要跑，`stage-owners.tsv` 里也不归这一类。
- 草稿目录 `/tmp/claude-1000/defs-gatebatch-m2-r2-opus/` 里的仓副本（`fixcopy/`）、小 crate 与编译产物（`semantics/`、`runnercrate/`、`includecrate/`、`libtest/`）与探针建的小仓，交回之前删掉；`rerun.sh` 会重建。
