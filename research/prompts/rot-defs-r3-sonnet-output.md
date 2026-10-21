# rot-defs-r3 辩方腿报告：第二轮判决 R1–R13 落没落、判决那一格够不够得着（K1）

日期 2026-10-19。轮 rot-defs-r3（最后一轮），辩方腿（Sonnet）。被复核：`research/prompts/rot-defs-r2-main-verification.md` 第三节 R1–R13；依据：修补报告 `/home/fy5090/.local/share/singlefs/work/rot-defs-r2-fixes/report.md`、`research/prompts/rot-defs-r3-snapshot/defs-vs-head.diff`（我重算 sha256 为 af25ca3174868acb8bfdd70d511f03d022aff7cc9ec450fe909a367024d933de，与正文一致）。没读禁读清单里的文件。

## 零、复跑命令、sha256、版本说明

模型目录在草稿目录里，没进仓：`/home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model`（探针、用例、倒回改前钩子的规格、快照版两份钩子、期望输出）。复跑（工作目录随意，`--work` 给新目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/rot-defs-r3-snapshot --work <新目录> --model /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model
```

我跑的两次，末尾对照行原样（`same=yes` 五行是复出，`today_vs_snapshot` 只是提示今天的钩子与快照版差几例）：

```
name=compare file=summary-a-snapshot.txt same=yes
name=compare file=summary-v-snapshot.txt same=yes
name=compare file=summary-h-snapshot.txt same=yes
name=compare file=summary-r1-snapshot.txt same=yes
name=compare file=summary-a-pre.txt same=yes
name=today_vs_snapshot file=summary-a-today.txt differing_cases=2
name=today_vs_snapshot file=summary-v-today.txt differing_cases=0
name=today_vs_snapshot file=summary-h-today.txt differing_cases=0
name=today_vs_snapshot file=summary-r1-today.txt differing_cases=0
```

模型目录每个文件的 sha256（`out/*-today*` 是当时今天那一版的输出，工作区被别的工具员继续改时会变，不当复跑比对项）：

```
0d14539ff5c6dc881d0c901a9e494dc5c7eb7b63a39149fbc56c61e43dcf4a42  cases/a.tsv
e3a458012db464bd55b7a4f4d09973901510e2ffc85b77315b47c077b27f98d5  cases/h.tsv
7e8322e06e5caf6e29c4408ca2de38bc7bac763450aa4dc8bd5d7d5522e5e88e  cases/r1.tsv
77c96e7ac692f92f231059506527ed7426ff2d47cdc700bccba207133a181312  cases/v.tsv
3bf1e8a21da2c4dd9abbf3eebacd83ad51f18f830d084c015d1d43bcd78264e2  hooks-snapshot/heavy-test-guard.sh
08efe8b7a1fb0a1d6245c25c6ce384235b462edd3644f81f45178cb28a4152e1  hooks-snapshot/lib_heavy_tests.py
bbd3b6b34e0d4b301eebd970862f4c8aa1aacfc6cc1a59743ba5f5c44218199c  out/probe-a-pre.out
4f6b08e1db7601942546e155f99133c444f85399839c2ac32b25e1dfa877c310  out/probe-a-snapshot.out
20224c3fbb5c35a68f50eaf6cd70d80ed4b6559b4abe69ab68da198cc7fd1825  out/probe-a-today.out
7032caccf9a209f34f659cdb74fdbb392f5d2e249f70fe91c31d1b54a64206c3  out/summary-a-pre.txt
5e5670df56cb6ebc4df497f1a955ac12989d002b352f6021a6c2fe9289b160f4  out/summary-a-snapshot.txt
1a480a338739bc898fd6a2551c0d04320f363fa0b68d809308ffebce5e39fb63  out/summary-a-today.txt
6cc1b3fcf258a558a60d975de74251fb3b4eadcb485e0103c2a58c914b3fe38c  out/summary-h-snapshot.txt
db5460cf3fae7462699e86fb38d33b7900c46604fed1b9d9153964572bafc28b  out/summary-r1-snapshot.txt
8c43a1b7f26b7b5b37d9f48cadb5eb910e50232b7c851a65c0d3b312d47f0b80  out/summary-v-snapshot.txt
5eb36a8018519e48840ebe93642005ef037179a090c049337050f17d63b4085b  specs/spec2.json
71d4059b827897b88148cd751906f7ab6b271f744203627b21d6f6326fa03393  specs/spec.json
3e40a0a5719d3f3e25cdbbcc4af56b0b45b387c190251d7ff83c93ca38e88a54  tools/driver-declared.sh
2071775ad692ce61c5880c6ea6854659ab457e351088464a24a552e1baa6d954  tools/driver-undeclared.sh
b9d8fb167e6999c37818e081ba3b6896c2b6c25e4ce338f03a6a37f0a77727c5  tools/make-pre-hooks.py
aeda4775c861b847ed8c7cc77183cab750207048ce4530694e9bf177d6df26e0  tools/probe.py
3ed61355803b4ea8ca51af898b6660498193346922a5dc31923d6923b6cda409  tools/rerun.sh
e753e6c1f1e8be77234aeba480af6ac42d7398d5ca96dc0f764b0084562cf99f  tools/summarize.py
```

三个版本要分清：
- **快照版**：`lib_heavy_tests.py` sha256 08efe8b7…、`heavy-test-guard.sh` 3bf1e8a2…，与 `defs-sha256.txt` 对上，就是修补报告第二节换上的那两份。我探针的主结论用它（`model/hooks-snapshot/`）。
- **改前版**：把修补工具员 `spec.json`、`spec2.json` 的「新串」倒着换回「旧串」（`tools/make-pre-hooks.py`），得 lib 30f7e232…、guard 37426dc2…。它是工具员动手前的样子，不等于 r2 快照里记的 lib 7cae21b8…（判决第一节自己写了「快照之后又被改过」）。
- **今天版**：开工后别的工具员又换了两份钩子与 `doc-decisions.sh`（`sha256sum -c` 对不上，见第八节附的命令输出）。差在：今天的 `cargo run --bin <登记装置> -- --pinned-cases` 也拒了（用例 A8、A9），其余用例同。

探针做法：把命令当 PreToolUse(Bash) 的输入喂 `bash-command-detector.sh`、`heavy-test-guard.sh`、`run-scope-guard.sh`，不执行命令本身；每个用例打三个退出码，依次是检出闸 / 重型闸 / 范围声明闸闸。钩子按自己所在的树找 `crates/`、`scripts/`、`target/`，所以改前版、快照版都摆在「仓的符号链接树 + 真拷贝的 hooks」里（只拷 hooks 目录的话装置名认不出，改前版的装置用例会成假放行，所以我用带符号链接树的摆法）。另有一个混杂：主工作区合入锁被别的会话占着时，重型闸会拒任何在主工作区起 cargo 的命令，探针把 `SINGLEFS_WORKSPACE_LOCK_FILE` 指到不存在的文件去掉它。

## 一、结论一句

R1–R13 十三条都落在了文件里，落点都在判决那一格点名的地方，钩子两条（R2、R3）改前放行、改后拒都复出了；没有一条落偏到另一件事上。要记的是三类：落了但有一句写偏或一环没接上（R1、R4）、判决那一格没写到而同形的句子还在别处（R5、R9、R2 的两个形态）、R3 的前提只对两个开关值成立。第二问：第二轮判决里对钩子现状的句子，我逐条喂了钩子，对上的占绝大多数，说宽了的两处在第五节。

## 二、判定一览

辩方判定三种：站得住 / 够不着（判决那一格没写到的东西）/ 同样打中别的臂（同形的句子在别处还在）。证据在第三至五节，每条后面写了什么现象会推翻它。

| 条目 | 落没落 | 落的是不是那一格要的 | 辩方判定 |
|---|---|---|---|
| R1 执行员装置单测与编译 | 落：experiment-runner.md 第 31、32、35、39 行 | 要的五样（单测编译固定一步、归主 agent 经 compile-then-swap、主 agent 带前缀跑单测、执行员只执行编好的二进制、bin 名与产物名用英文名）都落了 | 站得住；三处偏：括号「重型闸拒 cargo run」不成立（工具员加的）、「编好的二进制」没人产（判决那一格没写落点）、前缀 `=commit` 与提交之外的规矩冲突（判决只写「带前缀」） |
| R2 装置本体重放收严 | 落：钩子（快照版 lib 与 guard） | 是：登记装置 e165 带 `--pinned-cases` 的直接执行改前放行、改后拒，别的用法与别的装置照放 | 站得住；够不着 `cargo run --bin` 一形（今天已被别的工具员补）与 `cd` 后 `./名`、拷贝改名两形（量过放行，零轮） |
| R3 弄坏开关下的 `--quick` | 落：钩子 | 钩子一侧是：C1、C2、C3、C5 改前放行、改后拒，无开关的 C4 仍放行 | 站得住（钩子一侧）；前提「门禁一侧照整道跑」只对 `GATE_MUTATION_BREAK=quickrunsallrows`、`FORMAL_VERIFICATION_BREAK=quick-core-unselected` 成立 |
| R4 监控行 | 落：main-agent.md 第 33 行 | 选了判决给的「或直接指到那份 skill 的抄法」那一支 | 站得住；指针偏半步（读不到判定行记「作废」在 skill 第 4 步不在第 3 步），N、M、K 没有数法 |
| R5 补丁分两条命令 | 落：main-agent.md 第 51 行 | 是 | 站得住；同形的第 10 条（第 29 行）仍写一条命令，够不着 |
| R6 声明在前包装在后 | 落：implementation-writer.md 第 34 行、agent-common.md 第 58 行 | 是 | 站得住 |
| R7 不写修订号 | 落：main-agent.md 第 91 行 | 是 | 站得住 |
| R8 正推腿冻结副本只读与 sha256 | 落：forward 第 31、37 行，defense 第 32 行 | 是 | 站得住 |
| R9 小节指到段落 | 落：main-agent.md 第 89 行 | 是 | 站得住；同形两处引标题前半截（第 100 行、experiment-designer.md 第 20 行），够不着 |
| R10 数法是重型 | 落：main-agent.md 第 10 行 | 是 | 站得住 |
| R11 path-moves 的 `--write` | 落：path-moves.md 第 33 行 | 是 | 站得住 |
| R12 出路句与拒绝说明 | 落：doc-decisions.sh 三处、guard 与 lib 的拒绝说明 | 是 | 站得住；另 9 处同形今天已被别的工具员补 |
| R13 例子旁指到包装那一条 | 落：agent-common.md 第 56 行、attack.md 第 53 行、experiment-runner.md 第 31 行 | 是 | 站得住 |
| 遗留 1 `cargo run --bin` 没收严 | — | 判决那一格只点「直接执行」 | 判决没写到，不是工具员改窄；今天已补登记装置那一形，别的装置的 `cargo run` 仍放行 |
| 遗留 2 没展开的词偏严 | — | 判决没写 | 判决没写到；工具员沿用 guard 里已有的写法，不算改窄 |
| 遗留 3 R3 子脚本开关偏严 | — | 判决写「各自门禁的弄坏开关……一类」 | 判决没写到子脚本开关；工具员取宽，拒绝说明里的理由对这几个变量写偏 |
| 遗留 4 verification.md 缺一句 | — | 判决没有这一格 | 判决没写到；今天已被别的工具员补，补的那句带着未核的前提 |
| 遗留 5 doc-decisions 另 9 处 | — | 判决点 3 处 | 判决没写到；今天 9 处都已带声明，剩一行是用法说明 |
| 第二问 判决里钩子现状句 | — | 逐条喂钩子 | 对上；说宽两处：R3 的「门禁一侧照整道跑」、监控行的「✓ 0 ✗ 0」 |

## 三、R1–R13 逐条

### R1　执行员的入库装置单测与编译（experiment-runner.md）

**落了什么。**
- `.claude/agents/experiment-runner.md:32`「编译与单测是固定的一步，归主 agent，执行员不对它起 `cargo test`、`cargo run`（checker 档，重型闸拒）」
- `.claude/agents/experiment-runner.md:32`「主 agent 编、换、跑完单测之后续派，执行员只执行编好的二进制（`target/release/e<号>_<英文名>`」
- 第 2 步 ④：`.claude/agents/experiment-runner.md:35`「改完不自己编，停下交回」；`.claude/agents/experiment-runner.md:35`「换上之后由主 agent 跑 `python3 scripts/hosts/compile-then-swap.py --clean --scratch <草稿目录>` 删掉它建的仓副本与编译目录」
- 第 4 步产物名：`.claude/agents/experiment-runner.md:39`「按 `e<号>-<英文名里的下划线换成连字符>.out` 算」。判决括号里的 `e<号>_<英文名>` 紧跟在「bin 名与产物名」后面，字面也可读成产物名用下划线；工具员把产物名写成连字符，与现有产物同形（读法是工具员定的，判决没写清）：

```
$ ls research/results | wc -l; ls research/results | grep -cP '[^\x00-\x7f]'; ls research/results | grep -c '_'; ls research/results | grep -cE '^e[0-9]+'; grep -c 'bin e<号>-<简称>' .claude/agents/experiment-runner.md
177
0
0
177
0
```

（依次：产物 177 份，带非 ASCII 字符 0 份，带下划线 0 份，以 `e<号>` 起头 177 份；定义里 `bin e<号>-<简称>` 还剩 0 处。）

**钩子上量过（用例总表在第八节）。** 执行员 `scripts/experiments` 那一支的单测 A1 放行（0/0/0）；入库装置的单测 A2（带 `-p`）、A3（不带 `-p`）重型闸拒（0/2/0，改前也拒），拒绝说明给出路「在交回报告「要主 agent 跑的命令」一节写明要跑什么」，R1 把这条出路写成了固定一步；执行员执行编好的装置二进制 A4 放行；别的装置的别的用法 B3、B8 放行。主 agent 带 `=commit`、`=user-request` 跑单测都放行（R1-main-unit-test-commit、R1-main-unit-test-user-request 两例都是 0/0/0）。所以 R1 的「执行员只执行编好的二进制」不撞 R2：R2 只拒登记的那一种重放用法。

**三处落偏。**
- ① 括号「（checker 档，重型闸拒）」对 `cargo run` 不成立：用例 A5（`cargo run --release -p singlefs-checker-tier --bin e183_…`）、A6（不带 `-p`）在快照版与今天版都是 0/0/0。钩子文件头自己写着 `.claude/hooks/lib_heavy_tests.py:68`「`cargo run` 起 checker 档包自己的装置二进制也照旧不判（只有 e152 那一个按名字算」。括号是工具员加的（判决 R1 没提 `cargo run`），定义本身仍禁止执行员起，所以后果只是理由写错，不是放行。**判决没写到 + 工具员写偏。**
- ② 「编好的二进制」从哪来没接上。④ 的命令是 `.claude/agents/experiment-runner.md:35`「bash scripts/hosts/capped.sh <线程上限> python3 scripts/hosts/compile-then-swap.py <草稿副本> crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs --scratch <草稿目录>」，没带 `--release`；脚本把编译目录放在草稿目录里：`scripts/hosts/compile-then-swap.py:19`「CARGO_TARGET_DIR 放 <草稿目录>/compile-then-swap/target」，代码是 `scripts/hosts/compile-then-swap.py:264`「target_directory = os.path.join(work, "target")」，`--clean` 又把它整个删掉：`scripts/hosts/compile-then-swap.py:326`「shutil.rmtree(work)」。换上去的只有 `.rs` 源码，主工作区 `target/release/e<号>_<英文名>` 不由这一步产出；而 R1 要执行员执行的正是这个路径，第 4 步又写着 `.claude/agents/experiment-runner.md:39`「入库装置那一支编在仓根的 `target/` 下」，同时 1b 把编译划给主 agent。历史上的做法是带 `--release` 编、直接用草稿目录里编出的二进制：`.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:23`「官方二进制由 `scripts/hosts/compile-then-swap.py --release` 从装置终稿」。钩子上执行员在主工作区 `cargo build --release -p singlefs-checker-tier --bin e183_…`（A7）放行，所以缺的这一环有现成的补法，但定义没写谁补。**判决那一格没写落点（「经 compile-then-swap 编……只执行编好的」默认编出就在 `target/release/`），工具员照抄没核。**这一条读脚本得出，没真跑编译。
- ③ 前缀：R1 写主 agent「带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）」跑单测，这一步发生在跑产物之前，不是提交那一趟。规矩：`.claude/rules/verification.md:13`「默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request`」；共用约束：`.claude/agent-common.md:56`「只在提交代码时跑一次、或用户要求时跑」。重型闸自己对主 agent 不带前缀的拒绝说明写的是：

```
$ grep -F '提交之外要跑先弹窗问用户' /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out/probe-a-snapshot.out | head -1
    → 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户）：
```

钩子不分「哪一趟」（`=commit` 放行），冲突在规则与定义之间。查 `verification.md` 里有没有给入库装置单测的例外：

```
$ grep -c '入库装置' .claude/rules/verification.md
0
```

**判决那一格只写「带前缀」，工具员选了 `commit`。**

**什么现象会推翻。** ①：`cargo run --release -p singlefs-checker-tier --bin e183_…` 经包装被实验执行员起而重型闸退 2（探针重跑 A5）。②：一次 `compile-then-swap.py … --scratch <目录> --release` 之后，主工作区 `target/release/<名>` 的修改时间变新，或定义里别处写了谁来 `cargo build --release`。③：规则里写明新装置交付前的单测算提交那一趟（`verification.md` 或共用约束里有这样一句）。

### R2　直接执行装置本体放行收严（钩子，快照版 lib 与 guard）

**落了什么。** 装置名与选项读登记，不另抄名单：`.claude/hooks/lib_heavy_tests.py:72`「装置名与选项登记在那份脚本的 DEVICE_BINARY、DEVICE_FLAG 两个常量里，这里读那两行，不另抄名单」；登记处是 `scripts/experiments/e165-pin-red-examples.py:247`「DEVICE_BINARY = "e165_replay_bounded_enumeration"」 与 `scripts/experiments/e165-pin-red-examples.py:248`「DEVICE_FLAG = "--pinned-cases"」。装置那一侧只认「选项 + 表」两个参数：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17687`「if option == "--pinned-cases" {」。

**改前放行、改后拒（改前版 = 倒回的两份，改后版 = 快照版；三个数依次是检出闸 / 重型闸 / 范围声明闸）：**

```
$ M=/home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out; join -t$'\t' <(sort $M/summary-a-snapshot.txt) <(sort $M/summary-a-pre.txt) | join -t$'\t' - <(sort $M/summary-a-today.txt) | grep -E '^(A8|A9|B[0-9])' | awk -F'\t' '{printf "%-40s snapshot=%s pre=%s today=%s\n",$1,$2,$3,$4}'
A8-cargo-run-e165-pinned-with-package    snapshot=0/0/0 pre=0/0/0 today=0/2/0
A9-cargo-run-e165-pinned-no-package      snapshot=0/0/0 pre=0/0/0 today=0/2/0
B1-e165-main-pinned                      snapshot=0/2/0 pre=0/0/0 today=0/2/0
B2-e165-deps-pinned                      snapshot=0/2/0 pre=0/0/0 today=0/2/0
B3-e165-main-other-use                   snapshot=0/0/0 pre=0/0/0 today=0/0/0
B4-other-device-pinned-cases             snapshot=0/0/0 pre=0/0/0 today=0/0/0
B5-e165-main-unexpanded-word             snapshot=0/2/0 pre=0/0/0 today=0/2/0
B6-other-device-unexpanded-word          snapshot=0/0/0 pre=0/0/0 today=0/0/0
B7-pin-script-replay                     snapshot=0/2/0 pre=0/2/0 today=0/2/0
B8-e171-device-stream-constants          snapshot=0/0/0 pre=0/0/0 today=0/0/0
```

B1（`target/release/e165_…` 带 `--pinned-cases`）、B2（`deps/` 下同一份）、B5（参数是没展开的变量）改前 0/0/0、改后 0/2/0；B7（起判法脚本 `--replay`）改前改后都拒；B3（e165 的别的用法）、B4（别的装置带同一选项）、B6（别的装置参数没展开）、B8（e171 装置）改后仍放行。「没展开的词偏严」沿用 guard 里已有的写法：`.claude/hooks/heavy-test-guard.sh:64`「参数里的变量只在可能展开成它认的选项名、或判不出落到哪种用法时才算」。

**自证加了该红输入**（快照版；跑的时候磁盘上的两份与快照 sha256 相同，原样）：

```
$ W=/home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet; tail -n 1 $W/out/lib-selftest.out | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 120; tail -n 1 $W/out/guard-selftest.out | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 120; for b in device-main-replay-allowed device-replay-registry-unreadable-allowed; do echo "LIB_HEAVY_TESTS_BREAK=$b: ✗ 行 $(grep -c '✗' $W/out/lib-break-$b.out) 条"; done
  ✓ lib_heavy_tests 自检通过（查了 742 种：cargo 与包装过的命令行 476 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 60 种、bash 起不来时的 POSIX 字符
  ✓ 自检通过（查了 1345 种，其中该拒 255 种）：子 agent 跑 checker 档（按包判）、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表、Verus 那一道与它的判法脚本拒绝，har
LIB_HEAVY_TESTS_BREAK=device-main-replay-allowed: ✗ 行 5 条
LIB_HEAVY_TESTS_BREAK=device-replay-registry-unreadable-allowed: ✗ 行 1 条
```

与修补报告第二节抄的数（742、1345、255、5 红、1 红）一致。

**R1 与 R2 放在一起：** 登记的那一种用法才拒，A4、B3、B8 放行；被拒时拒绝说明写「要跑写进报告「要主 agent 跑的命令」一节交主 agent」（B1 的输出），有出路。

**够不着的形态（判决那一格只写「直接执行装置本体」）：**
- `cargo run --bin e165_replay_bounded_enumeration -- --pinned-cases <表>`（A8 带 `-p`、A9 不带）：快照版放行，**今天版已拒**（别的工具员补的；今天的 lib 文件头：`.claude/hooks/lib_heavy_tests.py:76`「`cargo run --bin <装置> -- <参数>` 起同一个装置同判（cargo_run_device_replay）」）。别的装置的 `cargo run`（A5、A6）仍放行。判决没写到，不是工具员改窄。
- 同一份活换个写法：

```
$ cat /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out/summary-v-snapshot.txt
V1-equals-form	0/0/0
V2-dot-slash	0/2/0
V3-absolute	0/2/0
V4-cd-then-dot-slash	0/0/0
V5-copied-binary	0/0/0
V6-bash-c	0/2/0
V7-debug-profile	0/2/0
V8-profile-first	0/2/0
V9-main-agent-no-prefix	0/2/0
V10-main-agent-commit	0/0/0
```

V4（`cd target/release && ./e165_… --pinned-cases …`）、V5（把装置二进制拷成别的名字再带 `--pinned-cases`）放行；V1（`--pinned-cases=<表>`）放行但装置不认等号形，不构成。**V4、V5 是判决没写到、也没人攻过的新形，按规则记零轮。**

**什么现象会推翻。** 站得住那一半：B1、B2 经包装被子 agent 起而退 0；或 B3、B8 被拒。够不着那一半：V4、V5 在主 agent 对照里被钩子拒（我量的是快照版与今天版，都放行）。

### R3　带 `--quick` 的轻型三条在设了弄坏开关时不认轻型（钩子）

**判决那一格原话：**`research/prompts/rot-defs-r2-main-verification.md:54`「带 `--quick` 的轻型三条，环境里设了各自门禁的弄坏开关（`GATE_MUTATION_BREAK`、`FORMAL_VERIFICATION_BREAK` 一类）时不认轻型、照重型判」

**钩子一侧（量过）：**

```
$ M=/home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out; join -t$'\t' <(sort $M/summary-a-snapshot.txt) <(sort $M/summary-a-pre.txt) | grep -E '^C[0-9]' | awk -F'\t' '{printf "%-40s snapshot=%s pre=%s\n",$1,$2,$3}'
C1-crates-mutation-quick-break           snapshot=0/2/0 pre=0/0/0
C2-formal-quick-break                    snapshot=0/2/0 pre=0/0/0
C3-formal-quick-child-script-break       snapshot=0/2/0 pre=0/0/0
C4-formal-quick-no-break                 snapshot=0/0/0 pre=0/0/0
C5-crash-quick-break                     snapshot=0/2/0 pre=0/0/0
```

C1（`GATE_MUTATION_BREAK=quickrunsallrows`）、C2（`FORMAL_VERIFICATION_BREAK=quick-core-unselected`）、C3（`E165_PIN_RED_EXAMPLES_BREAK=x`）、C5（`CRASH_POINT_REPLAY_BREAK=list-items-builds`）改前 0/0/0、改后 0/2/0；无开关的 C4 仍放行。弄坏开关逐个打开自证红（`LIB_HEAVY_TESTS_BREAK=light-tier-break-variable-ignored`）：

```
$ W=/home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet; echo "light-tier-break-variable-ignored: ✗ 行 $(grep -c '✗' $W/out/lib-break-light-tier-break-variable-ignored.out) 条"
light-tier-break-variable-ignored: ✗ 行 5 条
```

**门禁一侧（我读源码，没真跑；真跑 `--quick` 带开关会被这条钩子拒，崩溃与变异两条轻型也归主 agent）：** 判决写的前提是「弄坏开关下 `--quick` 不再只跑轻型登记的那几项」。只有两个开关值真是这样：
- `.claude/gate.d/checker-tier-crates-mutation-replay.sh:121`「GATE_MUTATION_BREAK=quickrunsallrows  --quick 不读轻型变异登记表，整张表都判」，代码 `.claude/gate.d/checker-tier-crates-mutation-replay.sh:189`「if ((quick)) && ((list_items == 0)) && [[ ",${GATE_MUTATION_BREAK:-}," != *,quickrunsallrows,* ]]; then」；同一个变量的别的值（例 `.claude/gate.d/checker-tier-crates-mutation-replay.sh:108`「GATE_MUTATION_BREAK=keepmutationtmp  拿掉删临时目录那一步」）不改 `--quick` 挑哪几行。
- `.claude/gate.d/checker-tier-formal-verification.sh:196`「if [[ ",${FORMAL_VERIFICATION_BREAK:-}," != *,quick-core-unselected,* ]]; then」。开了它就不把核心三项并进要跑的项，核心三项是 `.claude/gate.d/checker-tier-formal-verification.sh:147`「QUICK_CORE_ITEMS=(toolchain proof table)」。
- 崩溃点重放入口不是这样：`scripts/crash/crash-point-replay.sh:231`「只供自证（自证起的子进程里才认；平常跑时有一个非空就退 2 拒跑」，代码 `scripts/crash/crash-point-replay.sh:272`「[[ -n "$replay_break" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_BREAK)」 加 `scripts/crash/crash-point-replay.sh:305`「elif (( ${#replay_test_variables[@]} )) && (( ! list_items )) && ! replay_break_is honor-test-variables-outside-selftest; then」：设了就退 2，什么都不起，不存在「照整道跑」。
- 子脚本开关（C3 的 `E165_PIN_RED_EXAMPLES_BREAK`）改的是判法不是挑项：`scripts/experiments/e165-pin-red-examples.py:270`「BREAK = {item for item in os.environ.get("E165_PIN_RED_EXAMPLES_BREAK", "").split(",") if item}」，`--quick` 跑的还是核心三项，只是 table 格的判法被弄坏。

钩子把前提写进了拒绝说明和规则：

```
$ awk '/^### C3/{f=1} f&&/heavy-test-guard/{getline; print; exit}' /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out/probe-a-snapshot.out | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 210
    ✗ 重型测试被拒：门禁 checker-tier-formal-verification.sh（带 --quick，但环境里设了它文件头写的弄坏开关 E165_PIN_RED_EXAMPLES_BREAK：弄坏开关下 --quick 不只跑轻型登记的那几项，不认轻型，照整道算；要核弄坏开关下的判红，命令写进报告「要主 agent 跑的命令」一节交主 agent）；它的样本档也起重活（登记表第 160 行：ve
```

`.claude/rules/verification.md:58`「弄坏开关下 `--quick` 不再只跑轻型登记的那几项」。判决自己对这一半的口径是 `research/prompts/rot-defs-r2-main-verification.md:35`「采钩子那一半（量过），门禁一侧按源码推的」 与 `research/prompts/rot-defs-r2-main-verification.md:70`「R2、R3 的改法按钩子一侧量过的形状写」——那是推的，落地时成了钩子拒绝说明与规则里的事实句，对崩溃入口与子脚本开关不成立。

**判定：站得住（钩子一侧）；前提说宽。** 结果是偏严（这几个变量设着 `--quick` 照拒），不是漏。遗留 3 在第四节。**什么现象会推翻。** 主 agent 在副本里跑 `CRASH_POINT_REPLAY_BREAK=x … crash-point-replay.sh --quick` 而它没退 2（说明崩溃入口那一句我读错）；或 `E165_PIN_RED_EXAMPLES_BREAK=x … --quick` 的门禁输出里出现核心三项之外的项。

### R4　监控行的数法（main-agent.md「一轮怎么开、怎么收」第 14 条）

**判决那一格：**`research/prompts/rot-defs-r2-main-verification.md:55`「或直接指到那份 skill 的抄法，不另写一份 grep」——给了两支（改成同一套数法，或指到 skill），工具员选了第二支。

**落了什么。**`.claude/main-agent.md:33`「N、M、K 照 `.claude/skills/crash-test/commit-stages.md` 第 3 步的抄法数」，并且`.claude/main-agent.md:33`「三栏分开报），不另写一份 grep」。自写的 `grep -cE "^  (✓|✗) crash-case:"` 没有了。

**偏半步、一处没数法。**
- skill 第 3 步只管「每个阶段原样抄它的 ✓ 行，或 ✗ 行与紧跟的 →」：`.claude/skills/crash-test/commit-stages.md:29`「退出码 77 记」（后面接的是本次未跑）；记作废的是第 4 步：`.claude/skills/crash-test/commit-stages.md:30`「输出里读不到判定行的阶段记」（后面接的是作废）。第 14 条括号把两步的东西并在「第 3 步的抄法」名下，还把「读不到判定行」和「退出码 77」并记成「本次未跑」「作废」，读起来像一个对应一个。
- 第 3 步是逐阶段抄行，没有「N、M、K 怎么数」的算法；判决给的「同一套」那一支才有「数逐条用例行」。工具员选了「指到 skill」，这一支本身不含数法，N、M、K 仍要主 agent 自己数。落了判决给的字面，没落「数得出来」。**判决那一格的第二支本身不含数法 + 工具员没补。**

**判决的前提「数出 ✓ 0 ✗ 0」只对全绿的快档成立。** 判决原话：`research/prompts/rot-defs-r2-main-verification.md:36`「第 14 条的 grep 在提交那一趟数出 `✓ 0 ✗ 0`」，并且自己写了「真日志同形是推的」。读快档那一段（`check_crash_case_blocks` 起，第 902 行起）：绿的行是 `.claude/gate.d/checker-tier-crash-point-replay.sh:940`「全部块都有键相同的判定、没有红」，行首是四个空格、没有 ✓；红的行是 `.claude/gate.d/checker-tier-crash-point-replay.sh:883`「实跑红（cargo test 退」，行首两个空格加 ✗ 加 `crash-case:…`，原 grep 数得到。拿这两行的形状（构造的样本，不是真日志）喂原来的 grep：

```
$ printf '%s\n' '  ✗ crash-case:alpha：实跑红（cargo test 退 101，上面是日志尾部）' '    crash-case:alpha：全部块都有键相同的判定、没有红（x）' | grep -cE "^  (✓|✗) crash-case:"
1
```

所以红的快档会数出 ✗ ≥ 1。**判决说宽了一点（绿的成立、红的不成立），不是工具员的事。**

**什么现象会推翻。** 一次真实的提交那一趟日志里，原 grep 绿时数出 ✓ ≥ 1（说明快档段也打 ✓ crash-case 行）；或 skill 第 3 步写了 N、M、K 的数法。

### R5　合入后验证分两条命令（main-agent.md 验证段第 1 步）

**落了什么。**`.claude/main-agent.md:51`「打过的与没打过的补丁分两条命令、依次跑」，并写明「脚本先逐份核补丁已打在树上，核不过退 1」「每条命令只给自己那一批的补丁目录」。脚本那一侧与此对得上：`scripts/gate/merge-and-verify.sh:48`「拿锁、进树之后，编之前，对给的每个补丁目录里的 crates.patch 在要验的树上跑 `git apply --check --reverse`」；核不过的出路：`scripts/gate/merge-and-verify.sh:197`「没打过的那几份去掉 --skip-apply 让这里打（打过的与没打过的分两趟）」。判决 §四 说「没真跑混写那一条」，脚本自带的自证有「`--skip-apply` 给的补丁没打在树上」这一格，今天跑：

```
$ nice -n 19 bash scripts/gate/merge-and-verify.sh --selftest > /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/out/mv-selftest-2.out 2>&1; echo "exit=$?"; tail -n 1 /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/out/mv-selftest-2.out
exit=0
    --skip-apply 时补丁反着核得过才往下验、没打在树上的退 1 不编
```

（假 cargo 下的自证，不是真合入；「混写一条命令会退 1、不编」这一半有了真跑的格。）

**同形的句子还在别处，判决那一格没点：**

```
$ grep -noF 'merge-and-verify.sh [--skip-apply]' .claude/main-agent.md; grep -noF '（写阶段已打过的补丁带 `--skip-apply`）：打全部补丁、编一次' .claude/main-agent.md
7:merge-and-verify.sh [--skip-apply]
29:merge-and-verify.sh [--skip-apply]
51:merge-and-verify.sh [--skip-apply]
29:（写阶段已打过的补丁带 `--skip-apply`）：打全部补丁、编一次
```

第 29 行（第 10 条「改、写、验都按批」）仍写成一条命令：`.claude/main-agent.md:29`「（写阶段已打过的补丁带 `--skip-apply`）：打全部补丁、编一次」。按这一条去跑的人会把打过的与没打过的混在一条里。**判决那一格只点了验证段第 1 步，第 10 条同样打中，够不着。**

**什么现象会推翻。** 站得住那一半：脚本自证里「`--skip-apply` 给的补丁没打在树上」那格红。够不着那一半：第 10 条被读作指向验证段第 1 步、不单独成命令（则是我读宽了）。

### R6　范围声明写在包装前面（implementation-writer.md 第 1c 步、agent-common.md 包装那一条）

**落了什么。**`.claude/agents/implementation-writer.md:34`「声明在前、包装在后（起重活时 `capped.sh` 也在声明后面」；`.claude/agent-common.md:58`「不在声明前面再套包装」。脚本打出来的次序本身就是声明在前：`scripts/gate/rerun-failed-tests.py:13`「（`SINGLEFS_RUN_SCOPE='…' bash scripts/hosts/run-with-memory-cap.sh <上限> cargo test …`），范围声明闸把外面的声明带进包装里面那一条」，没改脚本。

**判决前提复出（真跑，原样）：**

```
$ bash scripts/hosts/capped.sh 2 SINGLEFS_RUN_SCOPE=x true 2>&1; echo "exit=$?"; bash scripts/hosts/capped.sh 2 env SINGLEFS_RUN_SCOPE=x true 2>&1; echo "exit=$?"; python3 scripts/gate/rerun-failed-tests.py /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/fake_run.log -p singlefs-harness | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 300
scripts/hosts/capped.sh: line 214: exec: SINGLEFS_RUN_SCOPE=x: not found
exit=127
exit=0
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness --test model_differential 里上一趟红的用例; 范围=1 个测试; 为什么=只重跑 fake_run.log 里红的这 1 条用例，修完看它们转绿，不整份重跑' cargo test -p singlefs-harness --test model_differential -- --include-ignored --exact beta_case
  ✓ 1 条失败的用例，分在 1 个测试目标里
```

钩子上：D1（声明 + capped.sh + 包装 + cargo，R6 写的次序）0/0/0；D3（capped.sh 直接贴在声明前，判决的坏形）也是 0/0/0，三道钩子放行而命令真跑退 127，所以这句规矩靠定义不靠钩子；D2（声明夹进包装里）同样 0/0/0。判决「三道钩子放行、跑起来退 127」（`research/prompts/rot-defs-r2-main-verification.md:38`「三道钩子放行、跑起来退 127」）对上。**站得住。**

**什么现象会推翻。** D3 被范围声明闸或重型闸拒（则「不在声明前面再套包装」可以交给钩子）；或 `capped.sh` 改成认前导的环境赋值。

### R7　不写修订号（main-agent.md 第 91 行）

`.claude/main-agent.md:91`「（以实验页最新一次修订为准）」。定义与共用约束里不再有修订号写法：

```
$ grep -cE '修订 ?[0-9]+|今天是' .claude/main-agent.md .claude/agent-common.md .claude/agents/*.md | grep -v ':0$' | wc -l; ls crates/singlefs-checker-tier/src/bin | grep e171
0
e171_three_oracle_collusion_after_the_oracle_fixes.rs
e171_three_oracle_collusion.rs
```

（命令先打还含修订号写法的文件数，0；再列被引的装置源文件，还在。）**站得住。** 推翻：定义里重新出现「修订 N」或「今天是」。

### R8　正推腿冻结副本只读、写模型的腿给复跑命令与 sha256

**落了什么。**
- 正推第 3 步：`.claude/agents/three-way-forward.md:31`「写模型的，报告开头给复跑命令与每个文件的 sha256」
- 正推写范围：`.claude/agents/three-way-forward.md:37`「冻结副本（主 agent 建的开工快照、`~/.local/share/singlefs/work/<轮>-frozen/` 这类）只读」
- 辩方新增第 4 步：`.claude/agents/three-way-defense.md:32`「4. 写模型的，报告开头给复跑命令与每个文件的 sha256。」；辩方写范围原来就有那一截：`.claude/agents/three-way-defense.md:37`「冻结副本（主 agent 建的开工快照、`~/.local/share/singlefs/work/<轮>-frozen/` 这类）只读」
- 与攻方第 8 条同写法：`.claude/agents/three-way-attack.md:48`「8. 写模型的，报告开头给复跑命令与每个文件的 sha256。」

三份写法逐字同形，辩方的原第 4 步顺延为第 5 步（交回前跑 `cite-check.py`）。这一份报告开头就是照它做的。**站得住。** 推翻：写模型的腿报告开头没有复跑命令与 sha256 而交回闸放行（说明这一句没人用）；不影响判决那一格。

### R9　第 89 行把小节指到段落（main-agent.md）

`.claude/main-agent.md:89`「那一段：逐条核腿报告里的原文引用、产物行与复跑命令」——这一行的引法是「`three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」一节里「核查与判决由主 agent 做」那一段」。被指的段落确在被指的小节里：`.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算`「核查与判决由主 agent 做，不派子 agent 去核子 agent」（它是加粗段，不是标题，所以才写「那一段」）。

**同形的句子还在别处：**

```
$ grep -n '^#.*交岔路' .claude/rules/three-way-inference.md; grep -noF 'three-way-inference.md`「交岔路时写岔路单」' .claude/main-agent.md .claude/agents/experiment-designer.md
138:### 交岔路时写岔路单，派实验时带上它
.claude/main-agent.md:100:three-way-inference.md`「交岔路时写岔路单」
.claude/agents/experiment-designer.md:20:three-way-inference.md`「交岔路时写岔路单」
```

标题是「交岔路时写岔路单，派实验时带上它」，第 100 行与 experiment-designer.md 第 20 行引的是前半截（同一份 designer 的第 16 行引的是整个标题）。判决那一格只点第 89 行。**站得住；同形两处够不着（轻，只影响「标题逐字」的写法）。** 推翻：这两处被读作指到标题的简称、有明文允许前半截。

### R10　数法是重型（main-agent.md 第 10 行）

`.claude/main-agent.md:10`「它起 cargo、是重型，主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` 跑」。脚本里确实起 cargo：`scripts/crash/crash-point-replay.sh:1552`「output="$(cd "$repository_root" && cargo run --release --quiet --offline -p singlefs-checker-tier --bin crash_library_reader -- keys 2>&1)"」。钩子（改前改后相同）：

```
$ grep -E '^E[0-9]' /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out/summary-a-snapshot.txt
E1-list-items-with-keys-no-prefix	0/2/0
E2-list-items-with-keys-commit	0/0/0
E3-list-items-with-keys-subagent	0/2/0
E4-list-items-plain	0/0/0
```

E1（主 agent 不带前缀）、E3（子 agent）拒，E2（带 `=commit`）放行，E4（单独 `--list-items`）放行，拒绝说明开头是给主 agent 照抄的带前缀写法（R1 节贴过它的首句，E1 的输出）。这一步在提交流程里（弹窗请用户跑全量之前），`=commit` 与「提交流程里跑」对得上，不同于 R1 的前缀。**站得住。** 推翻：E2 被拒，或 E4 被拒。

### R11　path-moves.md 第 33 行的 `--write`

`.claude/rules/path-moves.md:33`「--write --check decision-items-sync」；脚本用法：`.claude/gate.d/doc-decisions.sh:39`「--write [--check decision-items-sync]」。钩子：F1（子 agent 写回员）、F3（主 agent）带声明的写法 0/0/0；F2（裸 `--write`）0/0/2，被范围声明闸拒，所以改前的写法本来就走不通，R11 改的是对的。**站得住。** 推翻：F1 被拒，或 `--write --check decision-items-sync` 不重写分项清单。

### R12　出路句与拒绝说明

门禁侧三处（判决点 254、450、474 行）今天的样子，以及全文件剩下几处不带声明的 `--write` 写法：

```
$ awk 'NR==254||NR==450||NR==474 {print NR": "$0}' .claude/gate.d/doc-decisions.sh | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 330; grep -c 'doc-decisions.sh --write' .claude/gate.d/doc-decisions.sh; grep -n 'doc-decisions.sh --write' .claude/gate.d/doc-decisions.sh | grep -v SINGLEFS_RUN_SCOPE | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 60
254:   "权威记录是各决策正文的「### 已定项」/「### 未定项」两节；改完正文跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync 重写分项清单与索引表状态列、进度列"
450:                 print("       改完正文跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync")
474:             print("       改完正文跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync")
14
39:#   bash .claude/gate.d/doc-decisions.sh --write [--check
```

钩子侧拒绝说明补了 `--check-table`：`.claude/hooks/heavy-test-guard.sh:822`「list-items 与红例钉表的追加、--check、--check-table、--backfill、--list-items 不算」；`.claude/hooks/lib_heavy_tests.py:2727`「只有字面写明的 toolchain、fingerprint、tool-versions、list-items 与红例钉表的追加、--check、--check-table、--backfill、--list-items 不算」。**站得住。** 推翻：`doc-decisions.sh` 里还有带 `--write` 却不带声明的出路句（上面最后那条命令只列出第 39 行的用法说明）。

### R13　例子旁指到包装那一条

- `.claude/agent-common.md:56`「这里只写了声明片段，经内存包装的完整拼法照」
- `.claude/agents/three-way-attack.md:53`「例子只写了声明片段，完整拼法（声明在前、包装在后）照」
- `.claude/agents/experiment-runner.md:31`「这里的例子只写了范围声明片段，完整写法（声明在前、包装在后）见」

判决点了三处，我扫了所有定义与规则里「带声明、没带包装」的 `cargo test` 例子，只有这三处：

```
$ grep -noP "SINGLEFS_RUN_SCOPE='[^']*' [^\x60]{0,80}" .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/rules/*.md | grep -v 'run-with-memory-cap\|capped.sh' | grep 'cargo test' | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 170
.claude/agents/experiment-runner.md:31:SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 <bin 名> 的单测; 范围=1 个测试目标; 为什么=…' cargo test --release --bin <bin 名>
.claude/agents/three-way-attack.md:53:SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<crate> 的测试目标 <目标>; 范围=1 个测试目标; 为什么=…' cargo test -p <crate> --test <目标>
.claude/agent-common.md:56:SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<crate> 的测试目标 <自己的目标>; 范围=1 个测试目标; 为什么=…' cargo test -p <crate> --test <自己的目标>
```

钩子上（D4、D5）：只有片段、不包装的 D4 重型闸拒（0/2/0）；拼全的 D5 放行（0/0/0）。**站得住。** 推翻：别的定义里还有带声明片段的 `cargo test` 例子（扫描命令多出一行）。

## 四、修补报告第六节的五条遗留：判决那一格没写到，还是工具员改窄

下表的「判决那一格」都是第二轮判决第三节的原话。「改窄」按你的说法读：工具员做得比那一格要的少。五条里没有一条是改窄，五条都是那一格没写到。

| 遗留 | 那一格写了什么 | 工具员做了什么 | 判 |
|---|---|---|---|
| 1 `cargo run --bin` 没收严 | `research/prompts/rot-defs-r2-main-verification.md:53`「直接执行装置本体放行收严：形式化门禁红例钉表重放那一格用的装置用法」，只点「直接执行」 | 只收直接执行，沿用文件头原口径 | 判决没写到；今天已被别的工具员补上登记装置那一形（A8、A9 由 0/0/0 变 0/2/0），别的装置的 `cargo run`（A5、A6）仍放行 |
| 2 对没展开的词偏严 | 没写 | 装置本体的参数有没展开的词就按可能展开成登记选项算 | 判决没写到；沿用 `.claude/hooks/heavy-test-guard.sh:64`「参数里的变量只在可能展开成它认的选项名、或判不出落到哪种用法时才算」 的已有写法（B5 拒，别的装置 B6 不拒） |
| 3 R3 子脚本开关偏严 | 「各自门禁的弄坏开关……一类」，没点名子脚本开关 | 变量名读该门禁、该入口三份文件头里所有 `*_BREAK`，含子脚本的 | 判决没写到；工具员取宽，且拒绝说明里「--quick 不只跑轻型登记的那几项」对这几个变量写偏（R3 节） |
| 4 verification.md 缺一句 | 这一格没有（R3 只点钩子） | 没碰（该文件归别的工具员） | 判决没写到；今天已补，补的是下面那句，带着未核的前提 |
| 5 doc-decisions 另 9 处 | 点 254、450、474 三处 | 只改三处 | 判决没写到；今天 9 处也带了声明（R12 节最后那条命令只列出第 39 行的用法说明） |

第 3 条的文件头依据：`.claude/gate.d/checker-tier-formal-verification.sh:86`「弄坏开关：格的判法那一份的开关 VERUS_PROOF_CHECK_BREAK=<项> 与 E165_PIN_RED_EXAMPLES_BREAK=<项>」。第 4 条今天的句子：`.claude/rules/verification.md:58`「弄坏开关下 `--quick` 不再只跑轻型登记的那几项」；这个文件相对 HEAD 的改动量：

```
$ git diff --stat -- .claude/rules/verification.md | tail -n 1
```

第 5 条判决那一行：`research/prompts/rot-defs-r2-main-verification.md:45`「`doc-decisions.sh` 出路句第 254、450、474 行」。

**什么现象会推翻。** 第 1 条：判决 R2 那一格被读作含 `cargo run`（则 R2 落窄了）。第 3 条：主 agent 认定判决的「一类」就含子脚本开关（则不是没写到而是写到了）。第 5 条：判决那一行另有「其余同形」之类的话（我读的行里没有）。

## 五、第二轮判决里钩子现状的句子：逐条喂钩子

办法照判决第二节 K1 记第一轮错格的做法：把句子里的命令喂对应的钩子，看退出码。改前版 = 倒回的两份，改后版 = 快照版（第零节）；用例原样在 `model/cases/*.tsv`，结果在第八节附表。

| 判决里的句子 | 喂了什么 | 结果 | 判 |
|---|---|---|---|
| 入库装置单测被重型闸拒 | A2、A3（带与不带 `-p`） | 改前改后都 0/2/0 | 对上 |
| 「这一批 E181、E182 执行员各撞过一次」 | 看门狗检出记录（下面命令） | 各一条 | 对上 |
| 辩方 H1：`--list-items --with-keys` 是重型 | E1、E3、E2、E4 | 无前缀与子 agent 拒，带 `=commit`、单独 `--list-items` 放 | 对上 |
| 攻方「装置本体放行重放」：改前放行 | B1、B2 | 改前 0/0/0 | 对上；「同一份活」是推的，现读：`scripts/experiments/e165-pin-red-examples.py:1173`「command = (["bash", binary] if binary.endswith(".sh") else [binary]) + [DEVICE_FLAG, block_table, *profile_arguments]」，门禁格起的就是 B1 那一形 |
| 攻方「轻型开关」：改前放行 | C1、C2、C3、C5 | 改前 0/0/0 | 钩子一侧对上；**门禁一侧「会照整道跑」说宽**（R3 节：只对两个值成立，崩溃入口退 2） |
| 攻方「声明夹进包装」：三道钩子放行、跑起来退 127 | D2、D3；`capped.sh` 真跑 | 0/0/0；退 127 | 对上 |
| 攻方「监控行」：数出 ✓ 0 ✗ 0 | 构造样本喂原 grep | 绿的 0、红的 ≥ 1 | **说宽**（R4 节） |
| 攻方「混合补丁」：`--skip-apply` 整条命令一个开关 | `merge-and-verify.sh --selftest` | 退 0，含「没打在树上的退 1 不编」一格 | 对上（假 cargo 下） |
| 第一轮错格：`agent-common.md` 范围声明闸那一句 | H1–H10 喂 `run-scope-guard.sh` | whole 对子 agent 退 2、对主 agent 退 0；items 声明没带 `--item` 退 2；单目标入口 target 声明退 0；驱动脚本里的命令没带声明退 2 | 对上，见下 |

```
$ python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/detections.py
共 2 条
e181 被拒 ['重型测试被拒：cargo test 跑到 singlefs-checker-tier 包的测试（checker 档，默认只在提交时跑；按包）（checker 档）：experiment-runner 不跑「check
e182 被拒 ['重型测试被拒：cargo test 跑到 singlefs-checker-tier 包的测试（checker 档，默认只在提交时跑；按包）（checker 档）：experiment-runner 不跑「check
```

表里「第一轮错格」那一行对着 `agent-common.md` 今天的这一句：`.claude/agent-common.md:87`「崩溃放量三份入口同样要声明（`crash-amplification.sh` 不带 `--item` 是 whole），整轮门禁、`cargo test` 整包与整工作区都是 whole；命令位置上执行的驱动脚本读进去逐行判，脚本里每条要声明的命令各自带声明」。用例在范围声明闸那一列的结果：

```
$ cat /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out/summary-h-snapshot.txt
H1-amplification-whole-subagent	0/2/2
H2-amplification-whole-main	0/2/0
H3-amplification-no-item-declared-items	0/2/2
H4-amplification-item-declared-items	0/2/0
H5-amplification-item-sh-target	0/2/0
H6-amplification-two-hosts-target	0/2/0
H7-amplification-undeclared	0/2/2
H8-cargo-test-workspace-whole-subagent	0/2/2
H9-driver-script-undeclared	0/2/2
H10-driver-script-declared	0/2/0
```

（H1、H3、H7、H8、H9 的范围声明闸一列是 2，H2、H4、H5、H6、H10 是 0；重型闸一列的 2 都是它对崩溃放量与整包 `cargo test` 的拒，与这一句无关。）**结论：判决里钩子现状的句子，量得动的都对上；说宽的两处是 R3 的「门禁一侧照整道跑」与监控行的「✓ 0 ✗ 0」，前者判决第四节标了「读源码推的」，后者标了「真日志同形是推的」。**

## 六、这一轮新冒出的形态（第三轮不再为它们开轮，建议写进判决的交用户表，标「零轮」）

上面各节里「判决没写到」且前两轮没人攻过的，汇总如下。都是我量或读到的，没有一条被别的腿攻过。

| 号 | 形态 | 出处 | 性质 | 要验证的办法（推的，没做） |
|---|---|---|---|---|
| Z1 | 入库装置「编好的二进制」没有任何一步产出到 `target/release/`：④ 的命令无 `--release`、编译目录被 `--clean` 删 | R1 节② | 判决那一格没写落点；工具员照抄路径 | 主 agent 在草稿目录跑一次 `compile-then-swap.py … --release`，看产物落点与 `--clean` 之后还剩什么 |
| Z2 | R1 让主 agent 在跑产物之前用 `SINGLEFS_HEAVY_TESTS=commit` 跑装置单测，与「提交之外先弹窗、带 user-request」冲突 | R1 节③ | 判决只写「带前缀」 | 主 agent 定新装置单测算提交那一趟，还是弹窗走 `=user-request`；钩子两者都放行 |
| Z3 | R1 的括号「重型闸拒 `cargo run`」对除 e152 与 e165 登记用法之外的装置不成立 | R1 节① | 工具员加的括号 | 定义里删括号或钩子补判；探针 A5 重跑 |
| Z4 | R2 的两形放行：`cd target/release && ./e165_… --pinned-cases …`、装置二进制拷成别的名字再带选项 | R2 节 V4、V5 | 判决没写到 | 钩子对 `cd` 之后的相对名按换到的目录判、对拷贝按内容或按选项判（要设计，推） |
| Z5 | R3 的前提「弄坏开关下 `--quick` 不只跑轻型登记的那几项」对崩溃入口（退 2）与子脚本开关不成立，却写进了钩子拒绝说明与 `verification.md` 第 58 行 | R3 节 | 前提是推的，落地成事实句 | 主 agent 在副本里对三个入口各带一个非挑项的开关值跑 `--quick`，看挑了哪些项 |
| Z6 | `main-agent.md` 第 10 条（第 29 行）仍把「打过的与没打过的」写在一条 `merge-and-verify.sh` 命令里 | R5 节 | 判决只点验证段第 1 步 | 读一遍即可 |
| Z7 | 第 14 条指到 skill 第 3 步，但「作废」在第 4 步，N、M、K 没有数法 | R4 节 | 判决给的第二支不含数法 | 主 agent 定数法写进 skill，或第 14 条改指第 3、4 步并写三栏怎么数 |
| Z8 | 第 100 行与 `experiment-designer.md` 第 20 行引标题的前半截 | R9 节 | 判决只点第 89 行 | 读一遍即可 |
| Z9 | 旁注，不是 R1–R13 引入：第 10 行把 D13 已定项 9 的条件缩写了一句 | 下面 | 引 kb 条款没整行抄 | 对照整行改 |

Z9 的对照：`.claude/main-agent.md` 里是 `.claude/main-agent.md:10`「单元键对不上、且对不上的键里有判器单元变了」；kb 原文是`.claude/kb/decisions/13-验证路线.md:256`「对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算」。HEAD 里没有这一整句数法，说明它来自 R 之前的一层：

```
$ git show HEAD:.claude/main-agent.md | grep -c 'list-items --with-keys'
0
```

## 七、没做什么

- 没替主 agent 撤销或维持判决，只交复核结果。
- 没真跑 `compile-then-swap.py` 的编译（Z1 是读脚本得出的）；没真跑带弄坏开关的 `--quick`（会被 R3 这条钩子拒，崩溃与变异两条轻型也归主 agent；R3 门禁一侧的结论读源码得出，真伪看第六节 Z5 的办法）。
- K2、K3（前两轮站住的形态、前两轮报的数）归攻方，我没复核；顺带复出的只有：快照版两份钩子的自证数（742 种、1345 种其中该拒 255 种、三个弄坏开关红 5、1、5 条），与修补报告第二节抄的一致。
- 钩子探针只喂 PreToolUse(Bash) 的输入，不执行命令；没喂 Write / Edit 一类的钩子。
- 开工后别的工具员又改了 `lib_heavy_tests.py`、`heavy-test-guard.sh`、`doc-decisions.sh`、`verification.md`：我的钩子结论以快照版为准，今天版只比了一遍（A8、A9 两例不同），今天的文件我没做完整审。
- 模型目录没进仓：派发没给模型目录路径，放在草稿目录 `/home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model`；主 agent 要引复跑数时整个拷走，按第零节命令跑。
- 没读禁读清单里的文件；没做 git 写操作；没动 `.claude/` 下任何文件。
- 交回前删了自己建的副本（草稿目录下的 `post`、`pre`、`rerun1`、`rerun2`、`today`，里面是仓的符号链接树与钩子拷贝）；没有编译目录。

## 八、附：取证命令与用例总表（原样）

取证命令（草稿目录里的 `tools/evid4.sh` 一次跑出，工作目录是仓根；`sha256sum -c` 那三行说明开工后这三份又被别人改过）：

```
== 快照之后哪些被判文件又变了
$ sha256sum -c research/prompts/rot-defs-r3-snapshot/defs-sha256.txt 2>&1 | grep -v ": OK$"
.claude/hooks/lib_heavy_tests.py: FAILED
.claude/hooks/heavy-test-guard.sh: FAILED
.claude/gate.d/doc-decisions.sh: FAILED
sha256sum: WARNING: 3 computed checksums did NOT match

== R6：声明夹进包装的真跑（capped.sh 把声明当命令 exec）
$ bash scripts/hosts/capped.sh 2 SINGLEFS_RUN_SCOPE=x true
scripts/hosts/capped.sh: line 214: exec: SINGLEFS_RUN_SCOPE=x: not found
(exit=127)

$ bash scripts/hosts/capped.sh 2 env SINGLEFS_RUN_SCOPE=x true
(exit=0)

== R6：rerun-failed-tests.py 打出的命令（假日志）
$ python3 scripts/gate/rerun-failed-tests.py /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/fake_run.log -p singlefs-harness
  ✓ 1 条失败的用例，分在 1 个测试目标里
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness --test model_differential 里上一趟红的用例; 范围=1 个测试; 为什么=只重跑 fake_run.log 里红的这 1 条用例，修完看它们转绿，不整份重跑' cargo test -p singlefs-harness --test model_differential -- --include-ignored --exact beta_case
(exit=0)

== R1：产物名与 research/results/ 现有产物同形
$ ls research/results | wc -l
177
$ ls research/results | grep -cP "[^\x00-\x7f]"
0
$ ls research/results | grep -c "_"
0
$ ls research/results | grep -cE "^e[0-9]+"
177

== 遗留 5：doc-decisions.sh 里不带声明的 --write 写法
$ grep -n 'doc-decisions.sh --write' .claude/gate.d/doc-decisions.sh | grep -v SINGLEFS_RUN_SCOPE | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 110
39:#   bash .claude/gate.d/doc-decisions.sh --write [--check decision-items-sync] [项目根]  重写 decisions.md 的分项清单
$ grep -c 'doc-decisions.sh --write' .claude/gate.d/doc-decisions.sh
14

== 遗留 4：verification.md 里弄坏开关那句（今天）
$ grep -nF '弄坏开关下 `--quick` 不再只跑轻型登记的那几项' .claude/rules/verification.md | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 30
58:- 谁跑得了哪一档由 `.claude/hooks/h

== R5：main-agent.md 里还写着 --skip-apply 的几处
$ grep -nF 'skip-apply' .claude/main-agent.md | python3 -I /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/tools/trim.py 12
7:主 agent 只调
29:10. **改、写
51:1. 主 agen

== 快照那一版的两份钩子自证（跑的时候磁盘上的两份与快照 sha256 相同）
  ✓ lib_heavy_tests 自检通过（查了 742 种：cargo 与包装过的命令行 476 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 60 种、bash 起不来时的 POSIX 字符
  ✓ 自检通过（查了 1345 种，其中该拒 255 种）：子 agent 跑 checker 档（按包判）、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表、Verus 那一道与它的判法脚本拒绝，har
LIB_HEAVY_TESTS_BREAK=device-main-replay-allowed: rc 1, ✗ 行 5 条
LIB_HEAVY_TESTS_BREAK=device-replay-registry-unreadable-allowed: rc 1, ✗ 行 1 条
LIB_HEAVY_TESTS_BREAK=light-tier-break-variable-ignored: rc 1, ✗ 行 5 条

== merge-and-verify.sh --selftest（今天跑的）
    两个合入同时起后起的排队并报出前者、前者放锁后才编、锁文件里的持有者已死时报出来并接管、主工作树的锁被占着时 git worktree 加出来的树里照常验、
    测试只挑补丁改到的包（改两个包只跑两个、没改到的逐个报本次未跑）并带上依赖它们的包、改到 Cargo.lock 与带 --all-packages 照旧全跑、只改到 checker 档包时测试本次未跑、
    --skip-apply 时补丁反着核得过才往下验、没打在树上的退 1 不编
```

用例 A 组（R1、R2、R3、R6、R10、R11、R5 的钩子用例；三个数依次是检出闸 / 重型闸 / 范围声明闸；改前版 pre、快照版 snapshot、今天版 today）：

```
$ M=/home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out; join -t$'\t' <(sort $M/summary-a-snapshot.txt) <(sort $M/summary-a-pre.txt) | join -t$'\t' - <(sort $M/summary-a-today.txt) | awk -F'\t' '{printf "%-44s snapshot=%s pre=%s today=%s\n",$1,$2,$3,$4}'
A1-research-unit-test                        snapshot=0/0/0 pre=0/0/0 today=0/0/0
A2-device-unit-test-with-package             snapshot=0/2/0 pre=0/2/0 today=0/2/0
A3-device-unit-test-no-package               snapshot=0/2/0 pre=0/2/0 today=0/2/0
A4-run-built-device-binary                   snapshot=0/0/0 pre=0/0/0 today=0/0/0
A5-cargo-run-device-with-package             snapshot=0/0/0 pre=0/0/0 today=0/0/0
A6-cargo-run-device-no-package               snapshot=0/0/0 pre=0/0/0 today=0/0/0
A7-cargo-build-device-in-root                snapshot=0/0/0 pre=0/0/0 today=0/0/0
A8-cargo-run-e165-pinned-with-package        snapshot=0/0/0 pre=0/0/0 today=0/2/0
A9-cargo-run-e165-pinned-no-package          snapshot=0/0/0 pre=0/0/0 today=0/2/0
B1-e165-main-pinned                          snapshot=0/2/0 pre=0/0/0 today=0/2/0
B2-e165-deps-pinned                          snapshot=0/2/0 pre=0/0/0 today=0/2/0
B3-e165-main-other-use                       snapshot=0/0/0 pre=0/0/0 today=0/0/0
B4-other-device-pinned-cases                 snapshot=0/0/0 pre=0/0/0 today=0/0/0
B5-e165-main-unexpanded-word                 snapshot=0/2/0 pre=0/0/0 today=0/2/0
B6-other-device-unexpanded-word              snapshot=0/0/0 pre=0/0/0 today=0/0/0
B7-pin-script-replay                         snapshot=0/2/0 pre=0/2/0 today=0/2/0
B8-e171-device-stream-constants              snapshot=0/0/0 pre=0/0/0 today=0/0/0
C1-crates-mutation-quick-break               snapshot=0/2/0 pre=0/0/0 today=0/2/0
C2-formal-quick-break                        snapshot=0/2/0 pre=0/0/0 today=0/2/0
C3-formal-quick-child-script-break           snapshot=0/2/0 pre=0/0/0 today=0/2/0
C4-formal-quick-no-break                     snapshot=0/0/0 pre=0/0/0 today=0/0/0
C5-crash-quick-break                         snapshot=0/2/0 pre=0/0/0 today=0/2/0
D1-rerun-declaration-outside                 snapshot=0/0/0 pre=0/0/0 today=0/0/0
D2-rerun-declaration-inside                  snapshot=0/0/0 pre=0/0/0 today=0/0/0
D3-rerun-capped-in-front-of-declaration      snapshot=0/0/0 pre=0/0/0 today=0/0/0
D4-example-fragment-only-unwrapped           snapshot=0/2/0 pre=0/2/0 today=0/2/0
D5-example-fragment-wrapped                  snapshot=0/0/0 pre=0/0/0 today=0/0/0
E1-list-items-with-keys-no-prefix            snapshot=0/2/0 pre=0/2/0 today=0/2/0
E2-list-items-with-keys-commit               snapshot=0/0/0 pre=0/0/0 today=0/0/0
E3-list-items-with-keys-subagent             snapshot=0/2/0 pre=0/2/0 today=0/2/0
E4-list-items-plain                          snapshot=0/0/0 pre=0/0/0 today=0/0/0
F1-doc-decisions-write-with-check            snapshot=0/0/0 pre=0/0/0 today=0/0/0
F2-doc-decisions-write-bare                  snapshot=0/0/2 pre=0/0/2 today=0/0/2
F3-doc-decisions-write-main-with-check       snapshot=0/0/0 pre=0/0/0 today=0/0/0
G1-merge-verify-skip-apply                   snapshot=0/0/0 pre=0/0/0 today=0/0/0
G2-merge-verify-apply                        snapshot=0/0/0 pre=0/0/0 today=0/0/0
```

R1 组（主 agent 跑单测的前缀、`compile-then-swap.py` 谁起都放行）：

```
$ cat /home/fy5090/.local/share/singlefs/work/rot-defs-r3-sonnet/model/out/summary-r1-snapshot.txt
R1-main-unit-test-no-prefix	0/2/0
R1-main-unit-test-commit	0/0/0
R1-main-unit-test-user-request	0/0/0
R1-compile-then-swap-by-main	0/0/0
R1-compile-then-swap-by-runner	0/0/0
```

用例原文在 `model/cases/a.tsv`、`v.tsv`、`h.tsv`、`r1.tsv`；V 组、H 组的结果在第三、五节。

## 九、cite-check

`python3 scripts/agents/cite-check.py research/prompts/rot-defs-r3-sonnet-output.md`，末行原样：

```
  ✓ 核了 75 处引文、5 处裸行号，对上 75 处，没判 5 处（其中裸行号没有引文可比 5 处；逐处列在上面）
```
