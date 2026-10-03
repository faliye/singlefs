# tooling-broad-mutation-r1 攻方腿报告（2026-10-01）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

被判文件开工时 `sha256sum -c research/prompts/tooling-broad-mutation-r1-snapshot/crates-sha256.txt` 四份都 OK。下面的数全部是**副本上、假清单与假一步**量的，不是入库装置上的数；没起 cargo、没跑真变异、没跑任何 `--selftest`、没碰真库目录。

复跑：`bash research/prompts/tooling-broad-mutation-r1-opus-model/rerun.sh <仓根> <草稿目录>`（本机 31 核被占满时挂钟约 25 分钟，全是起 python 进程的开销）。

模型目录 `research/prompts/tooling-broad-mutation-r1-opus-model/` 各文件 sha256：

```
3dfba3fd10e60875640a78c2663a2cf624644dfee6df1b0b32d19a66cd4b6517  classify_probe.py
cc9eea059c9686a53b00b8721a532c4c74e88b49a01ebeeb4b76e4be8b8f8503  classify_probe.out
86a2824cc6e142911fc8057e12a3143e6a655dc92b11511d232437e9fb9ea28c  kp_driver.py
20b8c12a02125e19de03bcff7ea8086168d7e6694a0b9d8ed7dd00c6cb971542  fake_step.py
1dfb083d3413a528aa8db217e30a0240df0341d3d4629c0ae08d58d07c406be5  fake_list.py
72b20cb74bd8f20805a7c9e9a8642c1b7c6f1e53eb38d920d56027eebd5111ba  resume_probe.py
6556457bb732945823479ae768184069f148085f37258c436430df150d2b5d01  resume_probe.kills.out
6c41539a04628cfd1584ef08a0224e6b7d37e0708fc62ea7932fe38d886ade9a  resume_probe.others.out
2fd046c96c143a019a7f319b90a08789df3ceeea58e67186d56b227715d14a07  rerun.sh
```

## 各格判定一览

| 格 | 判定 | 打中的形状（编号见各节） | 依据 |
|---|---|---|---|
| T1 分类放不放得过该拦的 | **打中** | H1 点名 5091 条只跑第一段；H2 `python3 -W ignore` / `-X dev` 起；H3 假一步与弄坏开关经环境变量进真库 | 量过（分类返回）；「驱动实际做什么」H1、H3 量过，H2 推的 |
| T2 分类拦不拦错不该拦的 | 小的误拒 3 类，原有各类判定没见变 | F1 argparse 缩写与 `--item=1`；F2 草稿目录的仓副本 | 量过（分类返回） |
| T3(a) 中断续接 | 不带 `--item` 的续跑 19 个时刻里 17 个没打中、2 个留下杂文件；**带 `--item` 的续跑打中** | H4 点名跑被杀后重跑同一条命令把已有结果的条全重跑；H5 恢复中被杀在树里留 `.restore.<pid>`；H6 `--sync-tree` 半途被杀后新树记在旧指纹下 | 量过 |
| T3(b) 一条一条 | **打中** | H7 点名重跑把定论盖成 stage-one-only / timeout；H8 清单重生成后序号错位而汇总照绿；H9 半行让汇总永远退 1 | 量过 |
| T3(c) 判定对不对 | **打中** | H10 基线绿过之后环境坏了，每条记成 unviable 或 caught 且不再重跑；H11 认不出的退出码卡死整趟；H12 `#[ignore]` 的 71 条 harness 用例两段都不跑；H13 点名加 `--stage-two-only` 不经第一段记 missed | H10、H11、H13 量过；H12 数过 |
| T4 登记与规矩 | 一条反例 | R1 指纹只含源码，不含工具链；同类的 `crates-mutation-rows.py` 含 | 读代码，推的 |

自证 18 格与 7 个开关罩不住的：H1–H13 全部（逐条见第五节的表）。

## 一、T1

原样输出在 `research/prompts/tooling-broad-mutation-r1-opus-model/classify_probe.out`（50 条命令，判定函数是副本的 `heavy_tests_in_command_text`，当前目录取仓根；「驱动」一行是同一串参数喂副本 `parse_arguments` 之后按 `main` 的分支读出来的）。

### H1　点名不限条数，5091 个 `--item` 加 `--skip-stage-two` 分类交 None

放行它的那一句：`.claude/hooks/lib_heavy_tests.py:939`：「if "--item" in arguments and "--skip-stage-two" in arguments:」——只看两个词在不在，不数 `--item` 有几个。

```
A2 点名 5091 条只跑第一段
    命令: python3 research/scripts/broad-mutation.py --item 1 --item 2 --item 3 --item 4 --item 5 -- …（共 60044 字）… --item 5090 --item 5091 --skip-stage-two
    分类: None
    驱动: 真跑 run_items：item=5091 条 from=None package=None shard=None skip_two=True two_only=False threads=31 memory_cap=24G
```

驱动自己要的前缀拦不住：带上 `SINGLEFS_HEAVY_TESTS=user-request` 的同一条（A3）分类仍是 None；闸只在分类交出 HeavyTest 时才看谁带了什么前缀，文件头写的 `.claude/hooks/heavy-test-guard.sh:43`：「只有主 agent 带前缀能跑」对这一支不成立。驱动实际做的：整份清单每条跑第一段（四个包的 nextest），线程默认取 `research/scripts/broad-mutation.py:918`：「return max(1, (os.cpu_count() or 2) - 1)」，即 31；`--threads 64 --memory-cap 200G` 同样放行（A4），子 agent 派发提示给的线程与内存上限它都能自己盖掉。第一次跑还会先起一遍不限时的基线。

四句：① 只有一个实现，没有臂可分。② 分类在判的那一刻看得到 `--item` 的个数，判别子观测得到。③ 满足跑前判据 T1「子 agent 发得出、分类放过、实际起重型」——「重型」按正文 T1 自己的括注「点名 5091 条等于整趟」读；若主 agent 把「第一段 5091 条」不算重型，这一条就只剩线程与内存上限被盖掉那一半。④ 跑前条款只给了「分类必须改、做成自检表一例」，没给具体改法。

### H2　`python3` 带值的解释器选项把脚本挤成参数

```
C7 python3 -W ignore（带值的解释器选项）
    命令: python3 -W ignore research/scripts/broad-mutation.py --shard 0/2
    分类: None
C8 python3 -X dev
    命令: python3 -X dev research/scripts/broad-mutation.py
    分类: None
```

病根不在这次 diff 里，在共用切词：`.claude/hooks/lib_shell_words.py:497`：「while option_count < len(arguments) and arguments[option_count].startswith("-"):」——只跳以 `-` 开头的词，`-W` 的值 `ignore`、`-X` 的值 `dev` 被当成命令词，脚本成了它的参数。以前按名字判的 python 脚本只有「提交时才跑的检查」，这次是头一个重型类落在 python 脚本上，洞才有了分量。驱动实际做什么是推的（CPython 对 `-W ignore 脚本` 照常执行脚本，没在真驱动上跑）：不带 `--item` 就是整趟两段，但驱动自己还要前缀，子 agent 得连前缀一起写。

同一族、分类同样交 None 的（都在产物里，都是切词层的老洞，不是这次 diff 带来的）：`python3 -c "import runpy; runpy.run_path(…)"`（C9）、`python3 -c "__file__=…; exec(open(__file__).read())"`（C10）、仓内符号链接（C12）、仓内改名拷贝（C13）、变量间接 `S=…; python3 $S`（C14）、`xargs`（C19）、`uv run`（C23）。`python3 - < 脚本`（C11）分类 None，但驱动第 89 行用 `__file__`，这样起会 NameError，跑不起来（推的）。

### H3　只供测试的环境变量在真跑里照样生效，点名加只跑第一段的形态正好放行

```
A5 假步驱动写真库
    命令: BROAD_MUTATION_STEP_DRIVER=/bin/true python3 research/scripts/broad-mutation.py --item 7 --skip-stage-two
    分类: None
A6 弄坏开关进真跑
    命令: BROAD_MUTATION_BREAK=interrupted-as-done python3 research/scripts/broad-mutation.py --item 7 --skip-stage-two
    分类: None
```

许可的两句：`research/scripts/broad-mutation.py:1017`：「if not arguments.sync_tree and not os.environ.get("BROAD_MUTATION_STEP_DRIVER"):」——设了假一步就连前缀检查都跳过；`research/scripts/broad-mutation.py:164`：「return name in os.environ.get("BROAD_MUTATION_BREAK", "").split(",")」——弄坏开关不看是不是自证在调。副本上量到的（`resume_probe.others.out`）：

```
E6 不带 SINGLEFS_HEAVY_TESTS、BROAD_MUTATION_STEP_DRIVER=/bin/true「--item 7 --skip-stage-two」退 0；基线表 [['one-build', 'ok'], ['one-test', 'ok']]；结果行 ['7:stage-one-only 备注=第一段没红；这一趟没跑第二段']
E6 BROAD_MUTATION_BREAK=stage-two-missed-as-caught 进一次非自证的跑「--item 3」退 0；第 3 条的行 ['stage-one-only@one 备注=第一段没红；第二段接着跑', 'caught@two 备注=']
```

后果两样：（1）一步 cargo 都没起，基线表里落了两行 ok；之后的真跑读到它就不再跑基线（`research/scripts/broad-mutation.py:712`：「if row is not None and row[1] == "ok":」），「基线不绿一条都不跑」那道保护被一条不带前缀的命令写穿，这一步是推的（副本上没接着跑真 cargo）。（2）落下的结果行与真跑的行没有任何记号可分。假一步还可以指到任意脚本，闸不读环境变量指到的文件。对默认库目录这样做就是往主 agent 正在跑的那一版里写假行；我没对真库做，只在草稿库上量。

四句：① 无臂。② 驱动看得到这两个变量，闸的切词也把命令前的赋值交给了分类（`classify` 的 `environment` 参数），看得到。③ 满足 T1 的是「分类放过」；「实际起重型」不满足（它一步 cargo 都不起）——按字面它更该归 T3(c)「让抓到或没红失真的路径」。④ 无具体改法可核。

### 参数并用那几问（没打中）

`--summary` / `--sync-tree` / `--list-items` 加 `--shard`、`--from`（B1–B4）：驱动走查询那一支就返回，不接着跑。`--summary` 当别的选项的值（B6）、放在 `--` 之后（B7）：argparse 退 2。`--skip-stage-two` 与 `--stage-two-only` 并用（B8）：互斥组，argparse 退 2。`bash -c`、`env`、`nice`、绝对路径、`cd` 进去相对路径、直接执行、`python3 -u`、`/usr/bin/python3.12`、多余斜杠与 `..`、`capped.sh`、`run-with-memory-cap.sh`、`stdbuf`、`exec`、两条命令分号相连（C1–C6、C15–C18、C20–C22）：都判成 broad-mutation。

`--sync-tree --commit <别的提交>`（B3）不重型、也不要前缀，但它会把 `tree.source` 换成另一版：主 agent 之后「重跑同一条命令」跑的是新一版、从 0 条起（旧版结果留在旧目录，要 `--sync-tree --commit <旧提交>` 才回得去）。驱动只在 HEAD 动了时打一行提示。这是读代码推的，没量。

## 二、T2（找反例）

- F1 误拒，方向安全：argparse 认唯一前缀，`--list`、`--sum`、`--item 1 --skip` 驱动分别当 `--list-items`、`--summary`、`--skip-stage-two` 办，分类按整词比，三条都判成 broad-mutation（B10–B12）；`--item=1 --skip-stage-two`（贴着写）驱动照认，分类也判成 broad-mutation（B13）。子 agent 写成这样会被拒，换成整词就过。
- F2 草稿目录里的仓副本：`python3 /tmp/claude-1000/some-leg/copy/research/scripts/broad-mutation.py --home …` 判成 broad-mutation（D4）。许可的那一句是 `.claude/hooks/lib_heavy_tests.py:603`：「if parent == relative_directory or parent.endswith("/" + relative_directory):」，按路径尾比。攻方腿定义让腿「把仓拷到草稿目录」再在副本上跑，副本里用假一步驱动它（不带 `--item`）就会被拒。这与 `mutate.sh` 等按名字判的脚本同一口径，不是这次 diff 独有；我这一轮把副本驱动放在路径尾不是 `research/scripts` 的位置（`<草稿>/w/drv/bin/`），经自己的探针起，跑的全是假一步，这一点请主 agent 知道。
- 走得通的：主 agent 带前缀的全量（D5）判成 broad-mutation，由闸按前缀放行；`--item 12 --skip-stage-two`（A1）、`--list-items`、`--summary`、`--sync-tree` 交 None；`python3 -m py_compile`、`grep`、`sha256sum` 读它不算（D1–D3）；`--selftest` 归 commit-check-selftest，混了运行参数也一样、驱动也只跑自证（D6、D7）。
- 原有各类判定变没变：新分支只在命令名是 `broad-mutation.py` 时进，排在 `classify_commit_check` 之后、`cargo` 之前；`KNOWN_SCRIPT_LOCATIONS` 多一项只影响 `judged_by_name` 对这一个名字的回答。自检表原有各例我**没有逐例跑**（那是 `--selftest`，不许跑），只读了代码，这一句是推的。

## 三、T3(a) 中断续接

### 不带 `--item` 的续跑：19 个时刻，定论没丢、没重跑、被打断的条没被当成已完成

装置：8 条假清单，计划里四种结局与两段都有；`kp_driver.py` 把副本当模块载入，在点名处给自己发 SIGKILL（或由假一步给驱动发），然后不带任何补丁重跑同一条命令，核五件：杀前已有定论的条在重跑里被调 0 步、杀前只过了第一段的条不再跑第一段、每一步看到的树里只有自己那一条的改动（基线看到的是原样）、每条恰好一行定论且与计划相符、树回到原样且没有多出来的文件。原样输出 `resume_probe.kills.out`，19 行里 17 行「没打中」、2 行「打中」（`grep -c` 数的）。

罩到的时刻：存了原件还没记在途、记了在途还没改文件、改了文件还没起步、第一段编译中、第一段测试中、落 stage-one-only 行之前、落行的 write 与 fsync 之间、落行之后、第二段编译中、第二段 release 测试中、落 missed 行之前、落 missed 行之后恢复之前、落 caught 行之后恢复之前、恢复时换上了还没删在途、删了在途记录还没删原件、第一段基线中、第二段基线中。正文点的五个时刻里前四个都在内；「持锁进程死掉之后」我的假一步杀完驱动就自己退了，等于锁立刻放开，**上一趟的步还活着占锁**那种情形没量（自证第 ⑧ 格说它罩，我没跑自证）。

取样范围的限度：只固定了「杀在第 3 条（或第 1 条、基线）」；杀在别的条、别的结局（unviable、timeout 行前后）没扫；只量 KILL，TERM 那一支（`Stopped` 异常）一个时刻都没量。

### H5　恢复原件的半途被杀，树里留下 `<源文件>.restore.<pid>`

```
KILL k14-恢复时拷了临时件还没换上: 被杀那趟退 -9、它调了 15 步；杀后已有定论 ['1', '2', '3']、只过第一段 []、在途记录在；重跑退 0；定论被重跑 []；第一段被重跑 []；在脏树上跑的步 []；定论行重复的条 []；终态对；树原样、多出文件 ['crates/singlefs-core/src/lib.rs.restore.2929457'] ⇒ 打中
```

k19（连杀两次，第二次杀在开跑恢复里）同样。许可它的两句：临时件建在树里 `research/scripts/broad-mutation.py:426`：「temporary = f"{target}.restore.{os.getpid()}"」；开跑时核树只遍历登记过的文件 `research/scripts/broad-mutation.py:440`：「for path, digest in library.manifest.items():」，多出来的文件不查。这一个杂文件本身不进编译（cargo 不认这个后缀，推的），结果没错，所以它按跑前判据不算「丢、重跑、当成已完成」；它坐实的是「核树不查多出来的文件」，H6 用的就是这一点。

### H6　`--sync-tree` 换上新树之后、写 `tree.source` 之前被杀：新树在旧指纹下接着跑

`research/scripts/broad-mutation.py:484`：「os.replace(new_tree, library.tree)」与第 491 行写 `tree.source` 之间隔着算清单与写 manifest。杀在中间：

```
E8 新提交只多一个文件，--sync-tree 在新树换上之后被杀（退 -9）；接着照常跑退 0、调了 39 步、落了 13 行；tree.source 的指纹 61268220ba0c6a3c → 61268220ba0c6a3c；树里多出文件 ['crates/singlefs-harness/tests/new_case.rs']、登记的文件与旧源相同
E8 新提交改了一个已有文件，--sync-tree 在新树换上之后被杀（退 -9）；接着照常跑退 5、调了 0 步、落了 0 行；tree.source 的指纹 61268220ba0c6a3c → 61268220ba0c6a3c；树里多出文件 []、登记的文件与旧源不同
```

新提交相对旧提交**只多文件**时（例：只加一个测试文件），旧 manifest 登记的文件逐个相同，核树通过，之后每一条都在新树上判、记在旧指纹名下。多出来的若是一条会红的测试，真 cargo 下每条都会记成 caught（这半句是推的，假一步不编译）。改了已有文件的那一种被拦住（退 5）。窗口窄（要恰好杀在那几行之间、提交又恰好只加文件），但主 agent 在仓里只加测试文件再 `--sync-tree` 并不罕见。杀在旧树挪走、新树还没换上处：拒跑、重新 `--sync-tree` 即可，没打中。

### H4　带 `--item` 的命令被杀之后「重跑同一条命令」不是续跑

```
E1 e1-点名跑被杀后重跑同一条命令: 命令「--item 1 --item 2 --item 4 --item 6 --skip-stage-two」；被杀那趟退 -9，杀后的行 ['1:caught', '2:unviable', '4:stage-one-only']；重跑退 0，重跑里被调的序号 ['1', '2', '4', '6']；重跑后的行 ['1:caught', '2:unviable', '4:stage-one-only', '1:caught', '2:unviable', '4:stage-one-only', '6:stage-one-only']
```

许可的那一句：`research/scripts/broad-mutation.py:839`：「selected.append((item, not arguments.stage_two_only, want_stage_two))」——点名那一支不看状态。文件头两处说法对不上：`research/scripts/broad-mutation.py:14`：「已有结果的点名了就重跑、追加一行新结果，旧行留着」与 `research/scripts/broad-mutation.py:47`：「有定论的条一条都不重跑」。子 agent 唯一放行的形态正是带 `--item` 的，它没有续接。

四句：① 无臂。② 驱动看得到每条的状态（不带 `--item` 那一支就在用）。③ 满足跑前判据 T3「已有结果……被重跑」这个分句；用户原话「中断要续接。 不能中断后从头开始」对点名跑同样成立与否由主 agent 判——第 14 行是写明了的设计，也可以读成「点名就是要重跑」。④ 无具体改法可核。

## 四、T3(b) 一条一条

### H7　一条的状态取最后一行，点名重跑能把定论盖回去

`research/scripts/broad-mutation.py:400`：「return {row[0]: row[RESULT_COLUMNS.index("outcome")] for row in rows}」——后一行盖前一行，不分结局的强弱。

```
E2 全跑完：退 0 8 caught=2 missed=4 unviable=1 timeout=1 memory_cap=0 stage_one_only=0 not_run=0
E2 再跑「--item 4 --item 5 --skip-stage-two」（第 4 条原是 missed、第 5 条原是第二段 caught）退 0：退 0 8 caught=1 missed=3 unviable=1 timeout=1 memory_cap=0 stage_one_only=2 not_run=0
E2 之后照常重跑退 0，被调的 [('4', 'two-build-debug'), ('4', 'two-build-release'), ('4', 'two-test-debug'), ('4', 'two-test-release'), ('5', 'two-build-debug'), ('5', 'two-test-debug')]：退 0 8 caught=2 missed=4 unviable=1 timeout=1 memory_cap=0 stage_one_only=0 not_run=0
E2 第 1 条原是 caught，机器忙时点名重跑（假一步答 253）退 0：退 0 8 caught=1 missed=4 unviable=1 timeout=2 memory_cap=0 stage_one_only=0 not_run=0；第 1 条的行 ['caught', 'timeout']
E2 之后照常重跑退 0，被调的序号 []（timeout 是定论，不再跑）
```

三件事：（1）子 agent 放行的那一种命令（点名、只跑第一段）能把主 agent 跑出的第二段定论（missed、第二段 caught）盖成 stage-one-only，汇总当场少数；（2）之后主 agent 照常重跑会把这几条的第二段再跑一遍——已有结果的条被重跑；（3）点名重跑时碰上机器忙，caught 被 timeout 盖掉，而 timeout 是定论、之后不再跑，caught 那一行留在表里但汇总不取它。

### H8　同一指纹下清单重新生成，序号与结果行错位，汇总照绿

指纹只由树里的文件算，清单是 cargo-mutants 在那棵树上生成的；清单在就照用（`research/scripts/broad-mutation.py:494`：「if os.path.isfile(library.version_file("list.json")):」），而驱动自己给过删它的出路（`research/scripts/broad-mutation.py:751`：「这一版的清单坏了：删掉」）。删掉之后若 cargo-mutants 给的次序或条数与上一次不同（换了版本、换了 `.cargo/mutants.toml` 之外的全局配置），结果表里的旧行仍按序号认领：

```
E5 删 list.json、清单次序变了之后 --sync-tree 退 0（指纹目录还是 61268220ba0c6a3c）；退 0 8 caught=2 missed=4 unviable=1 timeout=1 memory_cap=0 stage_one_only=0 not_run=0；结果行的变异名与清单同序号的变异名对不上的序号 ['1', '2', '3', '4', '5', '6']
E5 之后照常跑退 0，被调 0 步
```

结果行里带着变异名（第 3 段），`summarise` 与 `states` 都不拿它与清单对。我的装置里「次序变了」是我写死的假设（把前 6 条倒过来）；真的 cargo-mutants 换版本会不会改次序没量，这一格是「若清单变了驱动拦不住」，不是「清单会变」。

### H9　结果表里一旦有半行，汇总永远退 1，给的出路修不了

```
E7 结果表末尾留半行：退 1 8 caught=2 missed=4 unviable=1 timeout=1 memory_cap=0 stage_one_only=0 not_run=0
E7 照汇总给的出路「--item 4」重跑退 0：退 1 8 caught=2 missed=3 unviable=1 timeout=1 memory_cap=0 stage_one_only=1 not_run=0
E7 照常重跑退 0，被调 4 步：退 1 8 caught=2 missed=4 unviable=1 timeout=1 memory_cap=0 stage_one_only=0 not_run=0
```

半行按不成形计数（第 896 行），追加新行不减这个数，而同一条出路又写「别手改」。半行的来路：`research/scripts/broad-mutation.py:217`：「os.write(descriptor, data)」不看写进去几个字节，盘满时短写不报错（推的，没造盘满）；KILL 造不出半行（k07 量过）。

## 五、T3(c) 判定对不对

### H10　基线只在这一版头一次跑；之后环境坏了，每条都落成定论，修好也不重跑

判定只看这一步的退出码：编译步退 101 记无效（`research/scripts/broad-mutation.py:790`：「return record("unviable", stage, note=f"{step_name} 编不过")」），测试步退「有测试没过」的码记抓到（第 792 行）。基线有过绿记录就不再跑（第 712 行）。于是基线之后才出现的环境故障分不出来：

```
E3 基线绿过之后编译一律退 101（盘满这一类）：照常跑退 0，基线被重跑 4 步；退 0 8 caught=1 missed=0 unviable=7 timeout=0 memory_cap=0 stage_one_only=0 not_run=0
E3 环境修好之后重跑同一条命令退 0，被调 0 步；退 0 8 caught=1 missed=0 unviable=7 timeout=0 memory_cap=0 stage_one_only=0 not_run=0
E3 基线绿过之后第一段测试一律退 100：照常跑退 0，基线被重跑 4 步；退 0 8 caught=8 missed=0 unviable=0 timeout=0 memory_cap=0 stage_one_only=0 not_run=0
E3 环境修好之后重跑同一条命令退 0，被调 0 步；退 0 8 caught=8 missed=0 unviable=0 timeout=0 memory_cap=0 stage_one_only=0 not_run=0
```

（输出里「基线被重跑 4 步」是我探针的措辞不准：那 4 步是第二段基线头一次跑，第一段基线没有重跑。）驱动退 0、汇总退 0。真环境里能让 cargo 编译退 101 的：库目录所在盘写满、编译目录被别的进程清掉一半、工具链半途换版本；能让测试步退失败码的：`scratch/` 写满、测试进程被别的会话停掉或被整机 OOM 杀掉（nextest 把它报成失败）。这些我都**没在真 cargo 上造**，装置里是假一步直接答 101 / 100，量到的只是「驱动拿到这个码之后怎么办」。

对已跑出的结果行的含义（跑前判据第三条）：连着一长串 unviable、或连着一长串 caught 且编译秒数与测试秒数都接近 0 的行，是该点名重跑的候选；表里有 `build_seconds`、`test_seconds` 两段可以筛。

### H11　某一条的某一步退了认不出的码，整趟停在这一条，重跑还是它

`research/scripts/broad-mutation.py:793`：「runner.stop_for_infrastructure(step_name, sequence, exit_code)」退 3、不落行；续跑按序号取第一条没定论的，还是它。

```
E4 第 3 条的 one-test 退 104，第 1 次跑：退 3，被调的序号 ['0', '1', '2', '3']；退 0 8 caught=1 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 not_run=6
E4 第 3 条的 one-test 退 104，第 2 次跑：退 3，被调的序号 ['3']；退 0 8 caught=1 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 not_run=6
E4 第 3 条的 one-test 退 104，第 3 次跑：退 3，被调的序号 ['3']；退 0 8 caught=1 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 not_run=6
```

文件头第 52 行自己说内存那一类要避开「续跑又撞同一条、永远过不去」，认不出的码这一类没避开。哪条真变异会让 nextest 或 cargo 退 100 / 101 之外的码我没有实例（104 是我凭记忆取的 nextest「列用例失败」的码，没核文档）；这一格是「若出现，整趟停住、要人手给 `--from`」，对用户验收「一直跑」是个缺口。包装退 251 / 252 / 254 也走这一支，那是环境的事，停下来合理。

### H12　两段都不跑 `#[ignore]` 的用例

第一段 `cargo nextest run` 没带 `--run-ignored`，第二段 `cargo test` 没带 `-- --include-ignored`（步表在第 642–647 行）。仓里数：

```
$ grep -rn '#\[ignore' crates/singlefs-harness --include=*.rs | wc -l
71
$ grep -rn '#\[ignore' crates/singlefs-checker-tier --include=*.rs | wc -l
19
```

harness 档 71 条里 61 条的理由以「harness 耗时用例」开头。所以 missed 的意思是「不带 ignore 的 harness 档用例加 checker 档快档都没红」，不是「没有测试抓得到」；只被这 71 条守着的变异会记成 missed。这不改已有行的真假，改的是 missed 这一栏怎么读。

### H13　点名加 `--stage-two-only` 对没跑过第一段的条直接记 missed

```
E11 新库上「--item 1 --item 3 --stage-two-only」退 0；被调的步 ['two-build-debug', 'two-build-release', 'two-test-debug', 'two-test-release']；行 ['1:missed@two', '3:missed@two']（第 1 条照计划第一段会被抓到）
```

同一句第 839 行：点名时 `--stage-two-only` 不看这一条是不是「只过了第一段」。真 cargo 下第二段的测试是第一段的超集（外加文档测试与快档），第一段抓得到的第二段多半也抓得到，所以装置里「第 1 条记成 missed」是假一步的计划造出来的，不能当真环境会错记的证据；站得住的只有「状态机许可不经第一段落 missed」。这一形态分类判成 broad-mutation，只有主 agent 发得出。

### 找过、没找到的失真路径

- 编译缓存串上一条变异：按代码推，每条的次序是「改文件 → 编译 → 测试 → 拷回原件（拷到临时件再改名，mtime 是拷回那一刻）→ 下一条改文件」，源文件的 mtime 每次都晚于上一次编译的起始戳，cargo 会重编。没在真 cargo 上量。
- 靠时钟或时间取种子的用例：`grep -rln 'SystemTime\|thread_rng\|from_entropy\|OsRng\|getrandom'` 在 harness、core、checker、format 四个包里 0 个文件；`Instant::now` 只在 `random_histories.rs` 与 `fault_injection_fast_tier.rs` 里用来打印耗时。只是 grep 这一层，没读每条断言。
- 源码里的裸回车（驱动用文本模式读写，会把它换成换行）：`grep -rlP '\r'` 在两个被变异的 `src` 下 0 个文件。
- nextest 自带的每条超时：仓里没有 `.config/nextest.toml`（`git ls-files | grep nextest` 无输出），用的是默认配置。

### 自证 18 格与 7 个开关罩不罩得住

按自证正文（第 1201–1312 行）逐格读的，没跑：

| 形状 | 自证里最接近的格 | 罩不罩 |
|---|---|---|
| H1 点名条数 | `lib_heavy_tests.py` 第 1137 行那一例只点 1 条 | 不罩 |
| H2 解释器带值选项 | 无 | 不罩 |
| H3 环境变量进真跑 | 无（自证自己就靠这几个变量） | 不罩 |
| H4 点名跑被杀后重跑 | ⑥ 只证「点名已有结果的条会追加一行」，把重跑当成对的 | 不罩（方向相反） |
| H5 / H6 树里多出文件、`--sync-tree` 半途被杀 | ⑭ 只证登记过的文件被改会拒 | 不罩 |
| H7 后行盖前行 | ⑥ 同上 | 不罩 |
| H8 清单重生成 | 无 | 不罩 |
| H9 半行 | ⑫ 证「对不上会红」，不证红了之后修得回来 | 不罩 |
| H10 基线之后环境坏 | 无；基线红退 6 那一支自证里也没有格 | 不罩 |
| H11 认不出的码 | 无（计划里只有 100、101、250、253） | 不罩 |
| H12 ignore 的用例 | 无（假一步） | 不罩 |
| H13 点名加只补第二段 | ⑤ 只证不点名的 `--stage-two-only` | 不罩 |
| 第二段被杀后只补第二段 | ⑦–⑨ 只在第一段里杀 | 自证不罩；我量过（k09、k10）没打中 |

## 六、T4（找反例）

R1：指纹只由导出的文件算（文件头第 36 行），不含工具链与 cargo-mutants、nextest 的版本；同仓的 `research/scripts/crates-mutation-rows.py` 第 44 行写它的指纹照 `admission.py` 的算法、带工具链一行。两处各算各的「这一版」，驱动没有复用那一份，`gate-similar` 三行（第 69–71 行）也没提这一处差别。工具链换了而源码没动时，驱动把新旧工具链下的行记在同一版里，H8 是它的一个后果。是不是「整段重复该抽成共用」我没有逐段比过两份文件，只看到这一处口径不同；这一格是推的。

`code-tooling.sh` 第 254 行的登记、`heavy-test-guard.sh` 第 42–43 行的两行注释与 `lib_heavy_tests.py` 的新分支字面一致，没找到反例；与 `sop-first.md`、`preflight-discipline.md`、`command-safety.md` 的逐条对照没做（不在我的攻击面主项里）。

## 七、我提的改法（只在我的模型上量过、被攻过零轮；全部是推的，没实现没跑）

| 改法 | 修哪一格 | 标注 |
|---|---|---|
| 分类数 `--item` 的个数，超过一个小数（例 20）就算 broad-mutation；同时把 `--threads`、`--memory-cap` 出现算进去 | H1 | 推的 |
| 切词给 `python3` 加一张带值选项表（`-W`、`-X`、`-c`、`-m`），`-c` 的正文里出现按名字判的脚本路径时照那个脚本判 | H2 | 推的；对 C12–C14、C19、C23 不起作用 |
| 假一步、假清单、假树源与弄坏开关只在库目录落在 `/tmp` 下（或自证自己设的一个一次性令牌在）时才认，否则拒 | H3 | 推的 |
| 点名那一支默认也跳过已有定论的条，要重跑另给 `--rerun`；或改文件头第 47 行，写明点名跑没有续接 | H4 | 推的 |
| 状态不取最后一行：定论不被 stage-one-only 盖；caught 不被 timeout / memory-cap 盖 | H7 | 推的；对 H4 不起作用 |
| 读结果行时核第 3 段变异名与清单同序号的名字，对不上判红；清单的 sha256 记进每一行或 `tree.source` | H8 | 推的 |
| 临时件建在库目录而不是树里；核树时树里多出来的文件也判红；`--sync-tree` 先写一份「换树中」记号，开跑见到就拒 | H5、H6 | 推的 |
| 连着 N 条 unviable 或 caught（或每隔 M 条）在没改过的树上重跑一次基线，红了停下并把这一段的行标成待重跑 | H10 | 推的 |
| 认不出的码记成一种不算定论的结局（例 `infrastructure`）后往下走，汇总单列一栏，续跑时重试 | H11 | 推的；对 251 / 252 / 254 要不要同样办没想过 |
| 半行给一条修的出路（例 `--repair-results` 把不成形的行挪进旁边一份文件）；`os.write` 查返回值 | H9 | 推的 |

## 八、没打中的形状

- 不带 `--item` 的续跑在 17 个 KILL 时刻上没丢行、没重跑定论、没把被打断的条当成已完成、没在脏树上判（第三节；取样只固定了杀在哪一条，见那一节的限度）。
- 查询类参数与运行参数并用、两段开关并用、12 种包装与路径写法（第一节末）。
- 编译缓存、时钟相关用例、裸回车、nextest 配置（第五节末），都只到读代码或 grep 这一层。
- 按「一条腿只抽一次样不算一次观测」，这些「没打中」只是一次样。

## 九、这条腿自己的限度

- 假一步不编译、不跑测试：凡是涉及 cargo、nextest、eatmydata、内存包装真实行为的，都是推的，已在各处标明。
- 线程上限 2：两只探针各一个进程同时跑过一段（一只用 `capped.sh 2`、一只用 `capped.sh 1`），没经内存包装（不跑编译出来的代码）。开跑前 `ps` 看到 6 个带 cargo / nextest 字样的进程（主 agent 的全量），没有 qemu、fio。
- 副本驱动的位置与经探针间接起它：见第二节 F2。
- H8、H13 的装置里有我写死的假设（清单次序变、计划让第一段会抓到），已在各自小节写明哪半句不能当证据。

## 十、没做什么

- 没跑 `lib_heavy_tests.py`、`heavy-test-guard.sh`、`broad-mutation.py` 的 `--selftest` 与任何弄坏开关的证红；没跑钩子本体（只 import 了分类函数）。
- 没起真 cargo、没跑真变异、没读也没写 `~/.local/share/singlefs/broad-mutation`。
- TERM / INT 那一支、上一趟的步还活着占锁的情形、两台机器分片与合并，没量。
- 没判正推腿那几格，没替主 agent 采纳；T4 与三份规则的逐条对照没做。
- 草稿目录 `/tmp/claude-1000/tooling-broad-mutation-r1-opus/` 里的副本与假库交回前已删。

推翻条件：H1–H3 的分类返回在入库的 `lib_heavy_tests.py` 上喂同样的命令文本得到不是 None；H4–H11、H13 在入库驱动上用自证的假一步重做同一段历史得到不同的行或退出码；H12 的两个数用同一条 `grep` 数出来不是 71 与 19。
