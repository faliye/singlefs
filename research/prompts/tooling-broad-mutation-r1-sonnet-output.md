# tooling-broad-mutation-r1：正推腿报告（T1–T4）

轮名 tooling-broad-mutation-r1；立场正推：核代码做的是不是题面与用户验收说的。日期 2026-09-30。

## 0. 开工核对与方法

`sha256sum -c research/prompts/tooling-broad-mutation-r1-snapshot/crates-sha256.txt` 原样输出：

```
.claude/hooks/lib_heavy_tests.py: OK
.claude/hooks/heavy-test-guard.sh: OK
.claude/gate.d/code-tooling.sh: OK
research/scripts/broad-mutation.py: OK
```

方法：读四份文件全文与背景正文、`lib_shell_words.py` 的切词、`run-with-memory-cap.sh` 的起法；用三类不起 cargo、不跑 `--selftest` 的手段现验：

- 把 `lib_heavy_tests.py` 当模块导入，逐条喂命令文本看 `heavy_tests_in_command_text` 的返回（`research/prompts/tooling-broad-mutation-r1-sonnet-model/classify_probe.py`，输出 `classify_probe.out`）。
- 走真实入口，给 `heavy-test-guard.sh` 从标准输入喂 JSON、读退出码（`hook_probe.sh`，输出 `hook_probe.out`；检出记录落草稿目录 `/tmp/claude-1000/tooling-broad-mutation-r1-sonnet/`）。
- 用同构的小 mock 复现进程与信号的行为（`fdprobe.py`、`termprobe.py`）；对驱动本体只跑了不起任何一步的读路径：`--summary`、参数互斥的用法错、不带前缀的拒跑。

现场事实（影响读法）：本机此刻有一趟真跑在进行（`ps` 看到 `python3 research/scripts/broad-mutation.py --threads 31 --memory-cap 24G --skip-stage-two --shard 0/2`），所以我读到的 `--summary` 与主 agent 背景里的数不同（我读到 caught=11、unviable=12、not_run=5068）。我没有碰它，只读 `/proc`、结果表与日志。

## 1. T1　分类放不放得过该拦的

### 1.1 分类逻辑本身

分类只在命令词的名字等于 `broad-mutation.py` 且落在 `research/scripts` 时触发：`.claude/hooks/lib_heavy_tests.py:894`：「name, arguments = os.path.basename(words[0]), words[1:]」；`.claude/hooks/lib_heavy_tests.py:936`：「if name == "broad-mutation.py" and located_in(words[0], directory, "research/scripts") and not break_is_set("broad-mutation-ignored"):」。之后三步：`.claude/hooks/lib_heavy_tests.py:937`：「if BROAD_MUTATION_QUERY_ARGUMENTS & set(arguments):」（参数里任何位置有五个查询词之一就放）；`.claude/hooks/lib_heavy_tests.py:939`：「if "--item" in arguments and "--skip-stage-two" in arguments:」（两个词都出现就放）；其余交回 kind broad-mutation。子 agent 的放行表里没有这一类：`.claude/hooks/heavy-test-guard.sh:160`：「"crash-verifier": {"checker-tier-cargo"」起的那张 AGENT_KINDS 里只有 crash-verifier 与 gate-triage 两行，都不含 broad-mutation，所以「分类交回重型」对子 agent 一律拒；「分类交回不算」则闸不再核前缀、不再核是谁。

### 1.2 逐条命令（分类返回来自 classify_probe.out / hook_probe.out；驱动行为来自读代码，注明的是实跑）

| # | 命令原文 | 分类 | 驱动实际会做什么 |
|---|---|---|---|
| a | `python3 research/scripts/broad-mutation.py --summary --shard 0/2` | 不算 | 只汇总，一步不跑。`research/scripts/broad-mutation.py:1010`：「if arguments.list_items or arguments.summary:」进去后 `research/scripts/broad-mutation.py:1016`：「return summarise(library)」。实跑 `--summary --item 1 --shard 0/2` 退 0，只打 SUMMARY 行 |
| b | `... --sync-tree --item 1`（无 skip） | 不算 | 导出树、生成清单后 `research/scripts/broad-mutation.py:1023`：「sync_tree(library, arguments.commit)」接 `return 0`，不进 run_items，`--item` 被忽略 |
| c | `... --list-items --from 1` | 不算 | 只列项，同 a 的分支 |
| d | `... --skip-stage-two --stage-two-only` | 不算（有 `--item` 才走到这格，无 `--item` 算重型） | argparse 互斥：`research/scripts/broad-mutation.py:933`：「stage = parser.add_mutually_exclusive_group()」；实跑退 2、一步不跑（输出 `argument --stage-two-only: not allowed with argument --skip-stage-two`） |
| e | `... --item 1 --item 2 --item 3 --skip-stage-two`，给到 5091 次 | 不算（hook_probe 第 9 行，子 agent，退 0） | 逐条收进选中表，`research/scripts/broad-mutation.py:833`：「for wanted in arguments.item:」，没有条数上限；每条走第一段。点名 5091 条就是整趟第一段 |
| f | `... --item=1 --skip-stage-two`（等号写法） | 算重型（classify_probe：`['broad-mutation']`） | argparse 认 `--item=1`，实际只跑一条：这是误拦，方向偏保守 |
| g | `bash -c 'python3 …'`、`env X=1 python3 …`、`nice -n 19 python3 …`、`timeout 10 python3 …`、`./research/scripts/broad-mutation.py`、`cd research/scripts && python3 broad-mutation.py`、`sh -c './…'`、`/usr/bin/time -v python3 …`、`systemd-run --user --scope python3 …`、`python3.12 …`、`python3 -B -O …` | 都算重型 | 同整趟；这些包装切词都剥得掉 |
| h | `python3 -W ignore research/scripts/broad-mutation.py`（整趟） | 不算（hook_probe 第 12 行，子 agent 自带前缀，退 0） | 整趟。原因在共用切词：`.claude/hooks/lib_shell_words.py:497`：「while option_count < len(arguments) and arguments[option_count].startswith("-"):」把 `-W` 的值 `ignore` 当成脚本词，命令词取成 `ignore`；`-X dev` 同 |
| i | `cd research/scripts && python3 -m broad-mutation`（整趟） | 不算（hook_probe 第 13 行，退 0） | `-m` 之后的模块名被当成脚本词，名字是 `broad-mutation`、少了 `.py`，对不上 `research/scripts/broad-mutation.py` 的名字判。用同构的 `stand-in.py` 实测：`python3 -m stand-in --item 1 --skip-stage-two` 里 `__name__ == "__main__"`、`__file__` 有值，驱动的 `if __name__ == "__main__":` 会进 |
| j | `python3 -c 'import runpy; runpy.run_path("research/scripts/broad-mutation.py", run_name="__main__")'` | 不算 | 同 i，`run_name="__main__"` 实测进主块；`-c` 之后的代码串被当脚本词 |
| k | 先 `ln -s <绝对路径>/research/scripts/broad-mutation.py /tmp/x/bm.py`，再 `python3 /tmp/x/bm.py` | 不算（名字是 `bm.py`） | 驱动 `research/scripts/broad-mutation.py:89`：「SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))」把链接解回仓里，整趟能跑；改名拷贝到仓外跑不起来（推，没实跑：仓根按拷贝位置算，`project_preflight` 导不进，退 1），拷进 `research/scripts/` 改名则跑得起来（推）、名字对不上 |
| l | `python3 - < research/scripts/broad-mutation.py` | 不算 | 跑不起来：stdin 脚本的 `__file__` 是 `<stdin>`，`preflight(__file__)` 读不到（实测 FileNotFoundError），一步没跑 |
| m | `bash research/scripts/multi-host-run.sh -- env SINGLEFS_HEAVY_TESTS=user-request python3 research/scripts/broad-mutation.py` | 不算（hook_probe 第 15 行，退 0） | 整趟。同一口子对别的重型也开着：第 16 行 `… -- bash .claude/gate.d/checker-tier-layer0-replay.sh --full .` 同样退 0。`grep -n 'multi-host' .claude/hooks/*.py .claude/hooks/*.sh` 命中 0 行。老问题，不是这次改动引入 |
| n | `echo | xargs python3 …`、`find . -exec python3 … \;`、`eval 'python3 …'` | 不算 | 整趟；共用切词不认这三种包装，同样是老口子 |

`--stage-two-only` 与 `--skip-stage-two` 并用：驱动在解析阶段退 2（d 行实跑），不会同时执行。

### 1.3 子 agent 自己带前缀就能过驱动那一道

驱动自己的闸只认环境变量：`research/scripts/broad-mutation.py:949-954` 的 `check_real_run_is_allowed` 判 `SINGLEFS_HEAVY_TESTS` 是 commit 或 user-request。分类放过的命令，钩子不核前缀；子 agent 在命令前自己写 `SINGLEFS_HEAVY_TESTS=user-request` 就过（e、h、i 三行的 hook_probe 都是这样退 0 的）。`.claude/hooks/heavy-test-guard.sh:43`：「的每条只起 harness 档四个包的测试、不算；只有主 agent 带前缀能跑」——后半句对「分类放过」的那几类不成立：子 agent 带前缀同样跑得了。

## 2. T2　分类拦不拦错不该拦的

### 2.1 主 agent 的三种用法都走得通（一致）

`hook_probe.out` 原样（主 agent，输入里没有 agent_type）：

```
agent=- exit=0 cmd=python3 research/scripts/broad-mutation.py --list-items
agent=- exit=0 cmd=python3 research/scripts/broad-mutation.py --summary
agent=- exit=2 cmd=python3 research/scripts/broad-mutation.py
agent=- exit=0 cmd=SINGLEFS_HEAVY_TESTS=user-request python3 research/scripts/broad-mutation.py
agent=- exit=0 cmd=SINGLEFS_HEAVY_TESTS=user-request nice -n 19 python3 research/scripts/broad-mutation.py --item 12 --skip-stage-two
agent=- exit=2 cmd=python3 research/scripts/broad-mutation.py --selftest
```

带前缀的全量、`--item … --skip-stage-two` 的小样、`--list-items` 与 `--summary` 各自走得通；不带前缀的整趟与 `--selftest` 被拒（`--selftest` 归提交时才跑的检查，钩子表里就是这样，见 `.claude/hooks/lib_heavy_tests.py:1141` 的自检行）。

「小样只起 harness 档」这半句在驱动一边成立：`research/scripts/broad-mutation.py:825`：「want_stage_two = not arguments.skip_stage_two」；点名分支 `research/scripts/broad-mutation.py:839`：「selected.append((item, not arguments.stage_two_only, want_stage_two))」；第一段没红且不要第二段时直接交回，`research/scripts/broad-mutation.py:803`：「if not with_stage_two:」；开跑只补第一段的基线，`research/scripts/broad-mutation.py:969`：「runner.ensure_baseline("one")」。第一段的包表是 `research/scripts/broad-mutation.py:636`：「harness = package_arguments(HARNESS_TIER_PACKAGES)」，四个包（harness、checker、core、format），不含 checker 档包。

### 2.2 分类改动没有改变已有类的判定（一致，逐例自检表对照我复核不了）

- 改动面：`git diff HEAD --numstat` 原样：`1 0 .claude/gate.d/code-tooling.sh`、`2 0 .claude/hooks/heavy-test-guard.sh`、`27 1 .claude/hooks/lib_heavy_tests.py`。那删掉的一行是弄坏开关清单末尾的标点改成顿号，其余全是新增。
- 新分支只在名字等于 `broad-mutation.py` 时才触发（`.claude/hooks/lib_heavy_tests.py:936`），放在 `mutation-shard-run.sh` 与 e152 之间；新增的表项是 `.claude/hooks/lib_heavy_tests.py:141`：「"broad-mutation.py": "research/scripts/broad-mutation.py",」与 KIND_CATEGORY 一项、查询词一个常量。
- 新旧并排的差分：从 `git show HEAD:.claude/hooks/lib_heavy_tests.py` 取旧版，与工作区新版各自导入，喂同一批命令，`diff_probe.out` 原样：

```
语料条数 133 新旧不同 0
```

语料是 code-tooling.sh 里 SELFTEST_RUNNERS 全部条目、每个门禁阶段的三种参数（无参、`--list-items`、`--check x`）、cargo 各形（test、nextest、mutants、`--workspace`、checker 档包、run e152）、mutate.sh 整表与 `--selftest`、两个分片驱动、gate-staged、check.sh、e152-run、vm-bench、herd7、qemu、以及几条只提到 `broad-mutation.py` 文件名而不是执行它的命令（`cat`、`grep`、`git add`、`python3 -m py_compile`、`bash -n`、`wc -l`、`sha256sum`）。
- 新增自检例数：`git diff HEAD -- .claude/hooks/lib_heavy_tests.py | grep -c '^+            ("'` 输出 9，与背景说的 9 例相符。
- 复核不了：把自检表原有各例逐例跑一遍需要 `python3 .claude/hooks/lib_heavy_tests.py --selftest`，本轮不让跑。上面的差分是它的替代，不是它本身。
- 什么现象会推翻：有一条命令新旧返回不同、且它的名字不是 `broad-mutation.py`。

### 2.3 拦错（误拦）与漏拦的对照

- 误拦：`--item=1 --skip-stage-two` 判重型（`classify_probe.out`），因为分类只认空格写法的 `--item`（`.claude/hooks/lib_heavy_tests.py:939`），argparse 认等号写法。偏保守，代价是子 agent 被拒一次。
- 漏拦：见 1.2 的 e、h、i、j、k、m、n。
- 自检表对新增表项的覆盖：`KNOWN_SCRIPT_LOCATIONS` 加了 broad-mutation.py（`.claude/hooks/lib_heavy_tests.py:141`），但按名字判的那张自检表 `.claude/hooks/lib_heavy_tests.py:1296`：「("仓内位置上的 mutation-shard-run.sh 按名字判"」那一组里没有对应的 broad-mutation.py 一行；别的三个分片、阶段脚本各有一行。这一项表没有一格钉着它，删了它自检不会红（我没删了试，是读表的判断）。

## 3. T3　三条验收站不站得住

### 3.1 （a）中断续接：逐个时刻

驱动的写顺序（`research/scripts/broad-mutation.py:740-760` 的 apply_mutation）：先存原件 `research/scripts/broad-mutation.py:752`：「shutil.copyfile(target, library.inflight_original)」，再原子写在途记录 `research/scripts/broad-mutation.py:755`：「write_atomically(library.inflight_record, json.dumps(」，最后就地改源文件 `research/scripts/broad-mutation.py:757`：「with open(target, "w", encoding="utf-8") as handle:」。判完后 `research/scripts/broad-mutation.py:810` 的 finally 里恢复原件。开跑时 `research/scripts/broad-mutation.py:1036`：「checked = verify_tree_is_pristine(library)」核整树一次。

| 时刻 | 会发生什么（SIGKILL 为准；TERM 见 3.2） | 丢已有结果 | 重跑已有结果 | 把被打断的条当完成 | 坏树上判下一条 |
|---|---|---|---|---|---|
| ① 结果行落盘之前（步已判完、record 还没调） | 没有行；树里留着改过的文件与在途记录；下一趟先恢复原件（`research/scripts/broad-mutation.py:414-434`），该条没有定论、从第一段重跑 | 否 | 否 | 否 | 否 |
| ② `os.write` 与 `fsync` 之间（`research/scripts/broad-mutation.py:217`：「os.write(descriptor, data)」、`research/scripts/broad-mutation.py:218`：「os.fsync(descriptor)」） | SIGKILL 不丢页缓存，行已在，下一趟认它为定论；一行不足一页，一次写完，撕不出半行。掉电才会丢或撕半行（见下） | 否（SIGKILL） | 否 | 否 | 否 |
| ③ 恢复原件之前（行已落、finally 没跑到） | 行在、在途记录在、树脏；下一趟恢复，定论保留 | 否 | 否 | 否 | 否 |
| ④ 恢复中间（`os.replace` 与删记录之间，`research/scripts/broad-mutation.py:428`：「os.replace(temporary, target)」） | 记录还在就再恢复一遍（拷同一份原件，幂等）；记录删了、`inflight.original` 还在时下一趟只删它 | 否 | 否 | 否 | 否 |
| ⑤ 第二段编译中 | 第一段的 `stage-one-only` 行已在，下一趟只补第二段（`research/scripts/broad-mutation.py:857`：「elif state == STAGE_ONE_ONLY and want_stage_two:」）；被杀驱动起的那一步还在跑，见 3.2 | 否 | 否（第一段不重跑） | 否 | 否 |
| ⑥ 持锁进程死掉之后 | 锁是 `flock`，进程一死就放；但每一步是驱动的子进程，见下一节：真链上持锁的是内存包装的 bash，不是 cargo | 否 | 否 | 否 | 见下 |
| ⑦ apply_mutation 内部被打断 | 记录先于改文件写好，任何一点被杀都能恢复；只有「存了原件、记录还没写」那一窗树没动、下一趟删掉孤儿原件（`research/scripts/broad-mutation.py:418`） | 否 | 否 | 否 | 否 |

即：SIGKILL 的七个时刻，我没找到丢已有结果、重跑有定论的条、或把被打断的条记成完成的路径。什么现象会推翻这一条：任一时刻杀后，下一趟 `--summary` 里该条的最后一行是杀之前没有的结局，或 `calls` 里出现已有定论的序号（自证 ⑦、⑨ 已在假步上核过两种：前两条行已落盘、被杀的第 3 条只重跑不记成完成）。

掉电与 SIGKILL 不同，三处只在掉电时成立：

- 恢复后不 fsync 树里那份文件就删记录（`research/scripts/broad-mutation.py:428` 之后到 `:431`）。掉电后盘上的文件可能是空的或旧的，下一趟 `verify_tree_is_pristine` 读盘上的内容会退 5，不会悄悄在坏树上判。
- 半行：`append_row` 遇到没换行结尾的上一行会补换行（`research/scripts/broad-mutation.py:202`：「def append_row(path, header_columns, fields):」），那半行留在表里；`research/scripts/broad-mutation.py:895`：「if malformed:」让 `--summary` 此后一直退 1，而出路文字说「坏掉的条用 --item <序号> 重跑」——重跑追加的新行盖不掉那条不成形的行，计数不会归零。要修只能手改结果表，而汇总退 1 时打的出路文字说「别手改」。没有自证格罩它（⑫ 只喂了一行认不出的结局，不是段数不对的行）。
- 这一条只有掉电、磁盘满或写一半失败才会出现，SIGKILL 出不来（同上，一行一次写完）。

### 3.2 （a）续：真链上谁持锁、被 TERM 停的驱动、被 KILL 的驱动

持锁：驱动开一个可继承的锁描述符并交给每一步：`research/scripts/broad-mutation.py:684`：「pass_fds=(self.library.lock_descriptor,)」。文件头说 `research/scripts/broad-mutation.py:49`：「被 KILL 的驱动留下的 cargo / 测试进程还活着时它们占着库目录的锁（锁的描述符传给了每一步）」。真链上不是这样：每一步先经内存包装，包装里起 systemd-run 的那一句是 `research/scripts/run-with-memory-cap.sh:812`：「bash -c "$SCOPE_HOLDER" run-with-memory-cap "$started_marker" "$peak_file" "$CGROUP_ROOT" "$@" 2>&3 3>&-; } 3>&2 2>/dev/null」——花括号组外层把 fd 3 改指 stderr、里面又关 fd 3，驱动的锁描述符恰好是 3。

实测一（同构 mock，`fdprobe.py`，输出 `fdprobe.out` 原样）：

```
driver lock fd number = 3
holders while running: [('2926555', 'python3', '3'), ('2926556', 'bash', '10')]
```

外层 bash 靠花括号组保存的副本（fd 10）持锁；里面的 `sleep`（对应 systemd-run 那一支）不持。

实测二（此刻在跑的真趟，只读 `/proc/<pid>/fd`，命令输出原样）：

```
pid=2837903 comm=python3 lock_fds=1
pid=2912006 comm=bash lock_fds=1
pid=2912050 comm=bash lock_fds=0
pid=2912051 comm=cargo-nextest lock_fds=0
pid=2916474 comm=checker_known_b lock_fds=0
holder pid=2837903 python3
holder pid=2912006 bash
```

2837903 是驱动，2912006 是内存包装的外层 bash，2912050 是 scope 里的外壳，2912051 是 cargo-nextest，2916474 是一个测试二进制。持锁的只有驱动与包装外层 bash。

后果：
1. 只 SIGKILL 驱动时，包装外层 bash 还在等 scope，锁在它手里；这一步要跑到自己结束（上限是它的单步超时，第二段的 cargo test 可以是几小时）。下一趟等 `--lock-wait`（默认 120 秒）后退 4。不丢结果，但「重跑同一条命令就续」变成「先等孤儿那一步结束或手工停它」。出路文字已写在 `research/scripts/broad-mutation.py:290`（proc.py stop），一致。
2. 只有包装外层 bash 也死、scope 里的 cargo 还活着（例如单独杀包装）时，锁空了，下一趟会在孤儿 cargo 还在编译或跑测试时开跑并先恢复原件。cargo 自己有 target 目录锁会让两边编译排队；孤儿产物是否被后一趟当成新的，取决于 cargo 的 mtime 新鲜度判断（推，没在真 cargo 上验）。自证 ⑧ 用假步、且假步是驱动的直接子进程，罩不住这一条。

TERM / INT：信号处理函数抛 `Stopped`（`research/scripts/broad-mutation.py:1002`：「raise Stopped()」）。步里的处理 `research/scripts/broad-mutation.py:688`：「child.terminate()   # 只点自己起的这一个进程」只给包装外层 bash 发 TERM，等 10 秒：`research/scripts/broad-mutation.py:690`：「child.wait(timeout=CHILD_STOP_GRACE_SECONDS)」。包装设的 TERM 处理是 `research/scripts/run-with-memory-cap.sh:800`：「trap 'rm -f -- "$started_marker"」起的 `exit 143`，但 bash 在前台等子进程时把 trap 推迟到子进程结束才执行。同构 mock（`termprobe.py`，输出 `termprobe.out` 原样）：

```
wrapper still alive 3s after TERM (trap deferred)
total wait 5.0s rc=143
trap-ran
```

所以 10 秒宽限过后驱动照样抛 `Stopped`，finally 恢复原件，退 143（消息 `research/scripts/broad-mutation.py:1341`：「被 TERM / INT 停了：在途那一条没落行，原件已拷回」属实），但这一步还在跑，下一趟同 1。文件头 `research/scripts/broad-mutation.py:47`：「中断续接：任何时刻被 TERM / KILL，重跑同一条命令」在这一点上要加一句「等孤儿那一步结束」。我只复现了同构 mock，没起真包装、没测终端 Ctrl-C（那是整个前台进程组一起收 SIGINT，另一种形状）。

apply_mutation 在 try 之外：`research/scripts/broad-mutation.py:796` 调 apply_mutation，`research/scripts/broad-mutation.py:797` 才 `try:`。信号处理函数在任意两条字节码之间抛 `Stopped`，落在 apply_mutation 里已改文件之后，finally 不会跑：树留脏，在途记录在，下一趟会恢复；但 `research/scripts/broad-mutation.py:1341` 那句「原件已拷回」在这一窗里不属实。只影响消息，不影响数据。

什么现象会推翻：真链上 `ls -l /proc/<cargo 或测试进程>/fd` 里有 `broad-mutation/lock`（那就是说持锁的不止包装，我上面的读法错）。

### 3.3 （b）一条一条：键、清单序号、追加取哪一行

- 键与版本：结果表放 `versions/<指纹>/results.tsv`，行里再带指纹，读时 `research/scripts/broad-mutation.py:394`：「rows = [row for row in rows if row[fingerprint_column] == self.fingerprint]」。指纹变了（`--sync-tree` 到别的提交）是另一个目录，序号从 1 重编，`--item <序号>` 按 `research/scripts/broad-mutation.py:830`：「by_sequence = {str(item["sequence"]): item for item in library.items}」在当前版本的清单里解。旧版结果不与新版混。同一提交再 `--sync-tree` 指纹不变、清单照用（`research/scripts/broad-mutation.py:494-497`），序号不变。一致。
- `--sync-tree` 换的是全局指针：`hook_probe.out` 第 14 行，子 agent 跑 `python3 research/scripts/broad-mutation.py --sync-tree --commit HEAD` 退 0（分类当查询）。它拿的是驱动自己的库锁，只在没有驱动在跑时能拿到；拿到后 `tree.source` 指向新版，之后主 agent 「重跑同一条命令」续的就是新版，不是它跑了几天的那一版；`research/scripts/broad-mutation.py:989` 的 `notice_if_head_moved` 提示的是 HEAD 与树不同，换了之后不再提示。回得去：`--sync-tree --commit <旧提交>` 指纹相同回到旧版（文件头说了，`research/scripts/broad-mutation.py:37` 那一段库目录说明）。不丢结果，但续接可以被别的会话悄悄切走。
- `--item` 重跑取哪一行：`research/scripts/broad-mutation.py:400`：「return {row[0]: row[RESULT_COLUMNS.index("outcome")] for row in rows}」——文件次序，后写的覆盖先写的，`--summary` 取最后一行。
- 降级：`research/scripts/broad-mutation.py:14`：「已有结果的点名了就重跑、追加一行新结果，旧行留着」。对一条已经有第二段定论的条（missed、第二段抓到、第二段超时）发 `--item N --skip-stage-two`：第一段没红时 `research/scripts/broad-mutation.py:802`：「row = record(STAGE_ONE_ONLY, "one", note=」追加一行 stage-one-only，最后一行变成它，`--summary` 里 missed 少一条、只过了第一段多一条。下一趟默认选条会因 `research/scripts/broad-mutation.py:857` 把它当「还差第二段」再补一遍，自愈，但代价是重跑整个第二段；而 1.2 的 e 行说明这种命令分类放过、子 agent 发得出。自证 ⑥ 只重跑了第一段就抓到的第 1 条，罩不住这条。
- 同一条两行结局不一致（例如第一次抓到、重跑因负载记超时）没有任何报警，`--summary` 只显示最后一行。这是缺一个提示，不是错读。

什么现象会推翻降级这一条：在一条已有 missed 的条上跑 `--item N --skip-stage-two`（第一段绿），`--summary` 里它仍是 missed。

### 3.4 （c）判定对不对

A. 「抓到」「无效」「超时」会不会来自环境

1. 基线一版只量一次、永久复用。`research/scripts/broad-mutation.py:712`：「if row is not None and row[1] == "ok":」——这一版的基线只要有一行 ok，之后每趟、每台机器、任何负载与工具状态下都不再重验；单步超时 `research/scripts/broad-mutation.py:663`：「return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))」也取那一次的秒数。环境事后变坏（缺一个测试要用的外部状态、共用目录被占）时，测试步退 100 或 101 都记 caught。
2. 「无效」只看退出码。`research/scripts/broad-mutation.py:789`：「if kind == "build" and exit_code == CARGO_FAILED_EXIT:」——构建步退 101 就记 unviable，不读日志。cargo 因编译以外的原因退 101（磁盘满、`--locked` 与 Cargo.lock 不符、链接器被外部杀）同样记 unviable。此刻本机 `df -h` 报 3.6T 用了 1.7T，不是现实风险；现场读过的 1、2、12 三条 unviable 日志（`~/.local/share/singlefs/broad-mutation/versions/<指纹>/logs/`）都是真的 E0277（`Default` 没实现），没见到环境型。没有检测手段，也没有自证格。
3. 超时与内存撞顶是终态。`research/scripts/broad-mutation.py:115`：「FINAL_OUTCOMES = ("caught", "missed", "unviable", "timeout", "memory-cap")」；默认选条只选没有结果的与 stage-one-only（`research/scripts/broad-mutation.py:855`：「if state == NOT_RUN and not arguments.stage_two_only:」与 :857）。负载造成的超时不会被自动重选，也没有「只重跑超时的」这样的选法，只能逐个 `--item`。文件头自己引了 `.claude/kb/tooling.md` 讲过这类误记。
4. 第一段与 C489 原口径不同。第一段是 nextest（每个测试一个进程）加 eatmydata：`research/scripts/broad-mutation.py:643`：「"one-test", "one", "test", sync_free」；C489 的账按 `cargo test --all` 算。第一段抓到的条不再过第二段，所以「抓到」在第一段的含义是「nextest 加 eatmydata 下抓到」。基线同口径，所以不是环境假阳性，但和 `cargo test` 下抓不抓得到不是同一个问题。
5. 编译缓存串上一条变异：共用 `target/`，靠 cargo 按文件 mtime 判新鲜；每条是「改文件（新 mtime）、编译、恢复（更新的 mtime）」，代码次序上没有一条的产物能被下一条当成新的。我没在真 cargo 上验（本轮不起 cargo），只读到现场日志 `1.one-build.log` 首行是 `Compiling singlefs-core v0.1.0 (…/broad-mutation/tree/crates/singlefs-core)`，说明每条都重编了被改的包。推，不是证明。

B. 「没红」会不会来自测试根本没跑到

1. `#[ignore]` 不跑，且两段都不带 `--ignored`。`grep -rn '#\[ignore' crates/<包>` 按包计数原样输出：`singlefs-checker-tier 19`、`singlefs-harness 71`。其中 60 条的原因写着 `harness 耗时用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装`。只被这些用例抓得到的变异会记 missed（`research/scripts/broad-mutation.py:808`：「record("caught" if break_is("stage-two-missed-as-caught") else "missed", "two")」）。missed 的准确含义是「非 ignore 的用例没抓到」；按 `.claude/rules/mutation-sampling.md` 分类没红那一桶之前要先带上这一点。
2. 第二段的 checker 档是 `research/scripts/broad-mutation.py:640`：「checker_tier = ["--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]」，不含它的文档测试（推：cargo 的 `--tests` 不含文档测试）；四个包那一支是整包 `cargo test`，含文档测试。
3. 过滤词：两段都没有；fail-fast 只出现在「有测试红」那一支，没红的那一支把全部跑完。一致。
4. 不核「跑了几个测试」：基线与每条都只看退出码，某一步实际跑了 0 个测试也退 0 时会记 missed，且基线也是绿的。没有自证格。
5. 未认出的退出码不落行、整趟退 3：`research/scripts/broad-mutation.py:793`：「runner.stop_for_infrastructure(step_name, sequence, exit_code)」。这是活性问题不是失真：同一条命令重跑会卡在同一条上，出路是 `--from <下一条>`。
6. 整树只在开跑核一次（`research/scripts/broad-mutation.py:1036`），每条只核要改的那一个文件（`research/scripts/broad-mutation.py:745` 的 refuse）。若某个测试在树里写了别的文件，后面几天的每一条都在坏树上判。我 grep 了 `CARGO_MANIFEST_DIR` 出现的 10 个文件，没查到往那里写的（推），所以这是没有机制、不是发生过。

C. 只供测试的环境变量能往真库写假行

`research/scripts/broad-mutation.py:1017`：「if not arguments.sync_tree and not os.environ.get("BROAD_MUTATION_STEP_DRIVER"):」——这个变量一设就跳过前缀检查，且 `research/scripts/broad-mutation.py:674`：「argv = [self.fake_driver, step_name, str(sequence), self.library.tree]」换掉每一步。默认库目录不受影响地被写：假步全退 0，就记 baseline ok 与 missed 行。文件头说「只供测试」，但没有把它和易失库目录绑在一起（自证靠 `--home` 指到临时目录）。触发需要变量泄漏或有意设置；分类放过的 `--item … --skip-stage-two` 加这个变量，子 agent 不带前缀、不起 cargo 就能写。我没有实跑（会写真库），是读代码的判断。

### 3.5 自证 18 格与 7 个开关罩不罩得住

18 格：`grep -c 'selftest.check(' research/scripts/broad-mutation.py` 输出 18。七个开关各对得上一处：rerun-done（`research/scripts/broad-mutation.py:824` 对应 ⑨）、interrupted-as-done（对应 ⑨ 的第 3 条）、no-row-flush（对应 ⑦ 杀之前前两条行已落）、stage-two-missed-as-caught（对应 ⑩）、summary-mismatch-silent（对应 ⑫）、no-restore（对应 ⑨、⑭）、accept-unknown-item（对应 ③）。这七对我是读代码对应的，没跑开关（本轮不跑 `--selftest`）；主 agent 背景里说逐个退 1。

罩不住的（本节 3.1–3.4 各条对应）：

| 缺的格 | 对应 | 为什么假步罩不住 |
|---|---|---|
| 给驱动发 TERM 或 INT | 3.2 | 所有格都只 KILL（`research/scripts/broad-mutation.py:1254`：`driver.kill()`），`Stopped` 那一支没格 |
| 在第二段里杀，再原命令续 | 3.1 ⑤ | ⑦ 杀在第一段的挂住步；⑤ 的 `--stage-two-only` 是主动选的，不是被杀之后的续 |
| 真链上的持锁者（包装外层 bash） | 3.2 | ⑧ 的假步是驱动的直接子进程，走 `pass_fds`，不经内存包装的 fd 3 处理 |
| 在途记录在、原件存不下来或对不上（退 5 的另一条路） | 3.1 | ⑭ 只喂「树脏、无记录」 |
| 基线红（退 6）、基线复用之外的基线路径 | 3.4 A1 | 计划表里没有让基线失败的步 |
| 内存包装自己的退出码 251、252、254（退 3、不落行） | 3.4 B5 | 计划只有 250 与 253 |
| `--item` 重跑把定论降级 | 3.3 | ⑥ 只重跑第 1 条 |
| 段数不对的行、半行 | 3.1 | ⑫ 只喂认不出的结局 |
| `--sync-tree` 被打断、换版本 | 3.3 | 三份库各自独立同步，没有换指纹的格 |
| 多个 `--item`、给全清单 | 1.2 e | ④ 只点名两条 |
| 环境变量 `BROAD_MUTATION_STEP_DRIVER` 落在真库目录 | 3.4 C | 自证自己设了它 |
| 环境型的 caught、unviable、timeout、`#[ignore]`、零测试、缓存 | 3.4 A、B | 假步不产生这些，必须真 cargo |

什么现象会推翻这张表：任一「缺的格」其实在 18 格里有断言（点到格号与行）。

## 4. T4　登记与规矩

### 4.1 `code-tooling.sh` 的登记（一致）

`.claude/gate.d/code-tooling.sh:254`：「"python3 research/scripts/broad-mutation.py --selftest"」，写法与表里别的行同形（解释器、脚本、参数）。自证每次起 `tempfile.mkdtemp` 与 `/dev/shm/broad-mutation-selftest-<pid>`，各给各的目录，符合表头说的「几份自证互相抢同一份状态的，并行下才会红」那一条；没在这一格放进 `SELFTEST_RUNNERS_STARTED_ALONE`，与它的临时目录独立一致。它在批里并行跑时的耗时与稳定性（⑦ 里有 60 秒等挂住步的上限）我没量，复核不了。

### 4.2 `lib_heavy_tests.py` 自检表新增各例

9 例（2.2 的计数）：整趟、`nice` 加绝对路径加分片、点名但两段都跑、点名只跑第一段、`--list-items`、`--summary`、`--sync-tree`、`--selftest`、仓外同名。开关 `broad-mutation-ignored` 下前三格判红；第四格起本来就是 None，开关下不变（`.claude/hooks/lib_heavy_tests.py:1137`：「("广谱变异点名、只跑第一段：只起 harness 档的测试"」）。缺的格：`--help` 与 `-h`、`--item` 给多次、`-m`、`-W`/`-X` 带值的解释器选项（1.2 的 e、h、i）、按名字判那一组的 broad-mutation.py 一行（2.3）。`heavy-test-guard.sh` 自己的 `--selftest`、看门狗 `agent-watch.py` 的自检里都没有为这一类新增格（`git diff HEAD --numstat` 里 `heavy-test-guard.sh` 只有 2 行新增、都是注释）。

### 4.3 `heavy-test-guard.sh` 文件头，与规则文字对不对得上

- 文件头 `.claude/hooks/heavy-test-guard.sh:43`：「只有主 agent 带前缀能跑」——对「分类交回重型」的那几类成立（子 agent 表里没有这一类），对「分类放过」的 `--item … --skip-stage-two` 与 1.2 的 e、h、i 不成立，1.3 已列。
- 重型的定义有三处各列一张清单，这次只改了钩子这一处：`.claude/agent-common.md:52`：「重型测试就是 checker 档那一批」起的那一条、`.claude/rules/verification.md:105`：「E152 装置算重型」结尾的那张表、钩子拒绝时打出的 `.claude/hooks/heavy-test-guard.sh:560`：「POLICY = ("→ 规矩：重型测试就是 checker 档那一批」，三处都没有「广谱变异」。子 agent 被拒时读到的出路文字里没有点名这一类，只在拒绝那一句的 detail 里有。这是钩子与规则文字的分叉，不是钩子内部的错。
- `.claude/agents/mutation-triage.md:19`：「不接没有表的广谱变异活动」写着走 experiment-designer 登记；驱动入库后这一句要不要改，不归本格判。

### 4.4 `sop-first.md`「加门禁或钩子之前，先找已有的」

- 钩子这一头：把新判据追加进已有的 `lib_heavy_tests.py` 与 `heavy-test-guard.sh`，没有新建钩子，符合 `.claude/singlefs-ai-sop/rules/sop-first.md@加门禁或钩子之前，先找已有的`：「有管同一件事的，把新判据追加进它」。一致。
- 驱动这一头：文件头有三行 `gate-similar:`，点了 checker-tier-crates-mutation-replay.sh、mutate.sh、crash-amplification.sh（`research/scripts/broad-mutation.py:69-71`）。没点：`research/scripts/mutation-shard-run.sh`（`--shard k/n` 「两台机器各跑一半」与它同形）、`research/scripts/layer0-shard-run.sh`，也没点被它 import 的 `research/scripts/crates-mutation-rows.py`。同一规则第 4 条：`.claude/singlefs-ai-sop/rules/sop-first.md@加门禁或钩子之前，先找已有的`：「与字面上很像的已有门禁或钩子，每个都要点名」。驱动本身是研究脚本、不是门禁或钩子，这一条是否适用由主 agent 判；点没点全归 review，`gate-overlap.py` 我没跑（本轮不跑 lint）。

### 4.5 `preflight-discipline.md`

- 头两行齐：`research/scripts/broad-mutation.py:2`：「# admission: always 每次调都判此刻库目录里那棵树与它这一版的结果表」；`research/scripts/broad-mutation.py:3`：「# run-condition: command git python3」。开头顺序照规矩：`sys.dont_write_bytecode = True`（88 行）在 import `project_preflight`（92 行）之前，`preflight(__file__)` 是主块第一句（`research/scripts/broad-mutation.py:1334`）。
- `research/scripts` 不在 `.claude/preflight-dirs` 里（该文件只登记 `research/e7-index-bench/src/bin` 与 `crates/singlefs-checker-tier/src/bin`），所以这份头是自愿写的，规则的门禁不判它。
- 声明与实际要的工具对不上：真跑要 cargo、cargo-nextest、eatmydata、systemd-run，`--sync-tree` 还要 cargo-mutants 与 tar，脚本自己在 `research/scripts/broad-mutation.py:955` 之后判。规则原文 `.claude/singlefs-ai-sop/rules/preflight-discipline.md@写法`：「none 只写给确实没有环境要求的脚本；要工具、设备、权限的，写成 command 或 check，不留到脚本里自己判。」这里写的是 `command git python3`，不是 none，但没写全；写全会让 `--selftest`、`--summary` 在没装 cargo 的机器上也被拒。取舍留给主 agent，我只报对不上。

### 4.6 `command-safety.md`

- 终止进程只点名一个：驱动只对自己起的 `child` 发 TERM（`research/scripts/broad-mutation.py:688`），不用 `pkill -f`、`killall`、不带参数的 `wait`；出路文字里让人用 `proc.py stop <pid>`。一致。
- 回读确认：`apply_mutation` 写完回读 sha256（`research/scripts/broad-mutation.py:759` 附近），`restore_original` 拷回后核 sha256。一致。
- 结果抓取的完整性闸：`--summary` 七栏之和与清单条数比、认不出的结局与序号判红（`research/scripts/broad-mutation.py:897`：「if problems and not break_is("summary-mismatch-silent"):」）。一致；半行的死角见 3.1。
- 删目录的守卫：`.claude/singlefs-ai-sop/rules/command-safety.md@退不回去的操作，动手前先想一遍`：「变量路径加 ${VAR:?} 守卫」。`--home` 可给任意目录，`research/scripts/broad-mutation.py:261`：「def resolve_home(given):」只拒易失目录，不要求它是空的或带过本驱动的标记；驱动会删其中的 `scratch`、`tree.new`、`tree.old`（`remove_tree`，`research/scripts/broad-mutation.py:336`）并整棵换掉 `tree`。默认目录是专用的，风险小；给了一个存着别的东西的目录才会碰到。
- 就地写：`apply_mutation` 用 `open(target, "w")` 就地改（第 757 行），仓里 `research/scripts/lib_atomic_replace.py:3`：「为什么不就地写」讲的是别的进程按偏移在读那份文件。这里改的是驱动自己导出的树、又在库锁里，没有别的读者，风险不成立；只是与仓约定写法不同，报一下。

### 4.7 与已有脚本有没有整段重复

- 退出码 250–254 的常量在四处各写一份：`research/scripts/broad-mutation.py:123`：「MEMORY_CAP_HIT_EXIT = 250」、`research/scripts/crates-mutation-rows.py:111`：「MEMORY_CAP_HIT_EXIT = 250」、`research/scripts/mutate.sh:56`：「MEMORY_CAP_HIT_EXIT=250」、包装自己 `research/scripts/run-with-memory-cap.sh:117`：「MEMORY_CAP_HIT_EXIT=250」。
- 退出码到结局的映射两处写：`research/scripts/crates-mutation-rows.py:619`：「def wrapper_outcome(returncode, memory_max, timeout_seconds):」与驱动 `run_stage` 里 785–793 行的内联判断加 `stop_for_infrastructure`（696–701 行）：250 是内存撞顶、253 是超时、251/252/254 是包装自己的结局。两份 python，可以抽成一个小模块，两边 import；驱动已经用 importlib 载入了 `crates-mutation-rows.py`（`export_paths` 那里），所以最小的改法是它直接调那边的 `wrapper_outcome`。
- 原子写：`research/scripts/broad-mutation.py:187`：「def write_atomically(path, data):」、`research/scripts/crates-mutation-rows.py:441`：「def write_atomically(path, text):」，外加共用库 `lib_atomic_replace.py`，三种写法：驱动的这份 fsync 文件与目录，`crates-mutation-rows.py` 那份不 fsync。
- 起内存包装的整套环境变量（`RUN_WITH_MEMORY_CAP_KEY`、`RUN_WITH_MEMORY_CAP_TIME_LIMIT`）与 `crates-mutation-rows.py:736` 的 `run_under_wrapper` 同一套，各自拼一份。
- 已经共用的：导出范围（驱动 import `copied_paths`，没有抄拷贝范围），符合「抽成共用的库」。两台机器的结果合并导入：`crates-mutation-rows.py` 有 `import` 子命令核指纹，驱动还没做（背景说明），做的时候放同一处比再写一份好（推荐，不是判）。
- 「按条记录」的形态不同（驱动是一个 TSV 追加、`crates-mutation-rows.py` 是一条一个文件），不算重复。

## 5. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 查询参数与运行参数并用 | 一致 | `--summary`、`--list-items`、`--sync-tree`、`-h` 与 `--shard`、`--item`、`--from` 并用，驱动都只做查询（实跑 `--summary --item 1 --shard 0/2` 退 0） |
| T1 `--stage-two-only` 加 `--skip-stage-two` | 一致 | argparse 互斥，实跑退 2，一步不跑 |
| T1 包装（bash -c、env、nice、timeout、./、cd 后、sh -c、/usr/bin/time、systemd-run、python3.12） | 一致 | 都判重型 |
| T1 `--item` 给多次或给全清单加 `--skip-stage-two` | 冲突 | 分类放过（hook_probe 第 9 行退 0），驱动无条数上限，点名 5091 条等于整趟第一段；子 agent 自带前缀即过驱动自己的闸 |
| T1 `python3 -W ignore`、`-X`、`-m broad-mutation`、`-c runpy`、符号链接、改名拷贝 | 冲突 | 整趟被放过（前四类实测，符号链接实测分类，拷贝是推）；病根在共用切词与按名字判，`-m` 是这一类新引入的名字缺口 |
| T1 `multi-host-run.sh -- …`、xargs、find -exec、eval | 冲突（老口子） | 同一个口子对 54 号、gate 同样开着，不是这次引入 |
| T1 `--sync-tree` 对子 agent 放行 | 冲突（设计层） | 它改全局版本指针，续接可被别的会话切走；不丢结果，可用 `--sync-tree --commit <旧提交>` 回去 |
| T1 `--item=1 --skip-stage-two` | 误拦 | 等号写法判重型，偏保守 |
| T2 主 agent 三种用法 | 一致 | 带前缀整趟、点名小样、`--list-items`、`--summary` 都退 0；`--selftest` 与不带前缀整趟被拒（设计） |
| T2 已有类判定 | 一致（逐例自检表复核不了） | 新旧并排差分 133 条命令 0 条不同；`--selftest` 不让跑 |
| T2 自检表对新表项的覆盖 | 缺格 | `KNOWN_SCRIPT_LOCATIONS` 加了 broad-mutation.py，按名字判那组自检没有对应一行 |
| T3(a) SIGKILL 七个时刻 | 一致 | 没找到丢结果、重跑有定论的条、把被打断的条记完成、或在坏树上判下一条的路径（掉电另有两处，见 3.1） |
| T3(a) 真链上持锁者 | 冲突（文件头说法与实测不符，后果是等） | 持锁的是内存包装外层 bash，不是 cargo 与测试；被 KILL 或 TERM 后要等孤儿那一步结束才能续 |
| T3(a) TERM 后 | 冲突 | `terminate()` 只发给包装，bash 推迟 trap，10 秒后驱动照退 143，那一步继续跑；「重跑同一条命令」实际会先退 4 |
| T3(a) apply_mutation 在 try 之外 | 小缺口 | TERM 落在改文件之后那一窗时消息「原件已拷回」不属实，数据不受影响 |
| T3(a) 半行（掉电、磁盘满） | 缺口 | `--summary` 此后一直退 1，出路文字说重跑，重跑盖不掉半行；无自证格 |
| T3(b) 键与版本 | 一致 | 结果按指纹分目录、行带指纹，序号只在当前版本内有意义 |
| T3(b) `--item` 重跑后的汇总 | 冲突（降级） | 取最后一行；对已有第二段定论的条发 `--item N --skip-stage-two` 会把它降回 stage-one-only，下一趟自愈但重跑第二段；自证 ⑥ 罩不住 |
| T3(c) 环境造成的抓到、无效、超时 | 缺口 | 基线一版只量一次；unviable 只看退出码 101；timeout 与 memory-cap 是终态且无重选口；现场三条 unviable 日志读过都是真编译错 |
| T3(c) 测试没跑到造成的没红 | 缺口 | `#[ignore]` 计 harness 71、checker-tier 19，两段都不跑；不核测试个数；missed 的含义是「非 ignore 用例没抓到」 |
| T3(c) 只供测试的环境变量 | 缺口 | `BROAD_MUTATION_STEP_DRIVER` 跳过前缀检查、可往真库写假行，需变量泄漏或有意设置 |
| T3 自证 18 格与 7 开关 | 部分罩得住 | 开关一一对得上；3.5 表里 12 项罩不住 |
| T4 code-tooling.sh 登记 | 一致 | 一行，写法同形；批内并行的耗时我没量 |
| T4 自检表新增 9 例 | 一致（有缺格） | 缺 `--help`、多 `--item`、`-m`、`-W` 与按名字判一行 |
| T4 文件头与规则文字 | 冲突（小） | 文件头「只有主 agent 带前缀能跑」对放过的那几类不成立；agent-common、verification、POLICY 三处重型清单没加这一类 |
| T4 sop-first | 一致（钩子头）；驱动头是否要点名分片驱动与 crates-mutation-rows 由主 agent 判 | 钩子是追加进已有的；驱动头的 gate-similar 少点了 mutation-shard-run.sh 与 crates-mutation-rows.py |
| T4 preflight-discipline | 一致（有取舍） | 头两行与开头顺序照规矩；command 只写了 git python3，真跑要的工具由脚本自己判 |
| T4 command-safety | 一致（有小缺口） | 不用 pkill、wait、有回读与完整性闸；`--home` 无「目录里有别的东西就拒」守卫 |
| T4 重复 | 冲突 | 退出码常量四份、退出码映射两份 python、原子写三种写法、包装环境变量套法两份；导出范围已共用 |

## 6. 没做什么

- 没跑三份 `--selftest`（`lib_heavy_tests.py`、`heavy-test-guard.sh`、`broad-mutation.py`），没跑 `gate.sh`、任何 lint（`gate-overlap.py`、`doc-lint`、`preflight-lint` 等），没起真 cargo；对驱动本体只跑了不起任何一步、不写库的路径（`--summary`、参数互斥的用法错、不带前缀的拒跑）。
- 逐例对照自检表原有各例：复核不了，用新旧并排差分代替。
- 没在真 cargo 上验编译缓存串上一条（3.4 A5 是推）、没测真内存包装被 TERM 时的行为（3.2 用同构 mock）、没测终端 Ctrl-C。
- 没验：子 agent 自带前缀跑 `--item … --skip-stage-two` 的真实运行（分类与钩子退出码实测，驱动一头是读代码）；`BROAD_MUTATION_STEP_DRIVER` 写真库（会污染，没跑）；改名拷贝与 `python3 -m` 之外的别的起法。
- 不判别的腿的格没做：没替主 agent 采纳、没出判决、没改任何被判文件（四份都没动）；没读禁读清单里的文件。
- 现场一趟真跑在进行（3.2 的 `/proc` 读数来自它），我没碰它；那些 pid 之后会变，`/proc` 那段输出是当时的原样。

## 7. 产物与草稿

- 模型目录 `research/prompts/tooling-broad-mutation-r1-sonnet-model/`：`classify_probe.py` 与 `classify_probe.out`、`diff_probe.py` 与 `diff_probe.out`、`hook_probe.sh` 与 `hook_probe.out`、`fdprobe.py` 与 `fdprobe.out`、`termprobe.sh`、`termprobe.py` 与 `termprobe.out`。`fdprobe.py` 与 `diff_probe.py` 会在当前目录或草稿目录里留一个 `lockfile`、一份 `old/` 副本。
- 草稿目录 `/tmp/claude-1000/tooling-broad-mutation-r1-sonnet/`：检出记录 `det.jsonl`、`stand-in.py` 与 `standin.py`（打印 `__name__` 与 `__file__` 的替身脚本）、`elsewhere/bm.py`（指向驱动的符号链接，只用来喂分类，没执行）、`old/`（HEAD 版 `lib_heavy_tests.py` 与当前 `lib_shell_words.py` 的副本）。没有编译目录、仓副本或工作树。
- 「evidence-in-repo」：3.2 那段 `/proc` 读数当时没落文件，只在本报告里原样贴；它对应的是此刻在跑的一趟，之后看不到了。
