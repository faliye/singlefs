# test-script-align-r3 攻方腿（Opus）：V1

日期：2026-10-01。被攻的格：V1（前两轮站住、已落地的形态）。前几轮判决：`research/prompts/test-script-align-r1-main-verification.md`、`research/prompts/test-script-align-r2-main-verification.md`。不重复 r1 的 T2/T3、r2 的 U1/U2：r2 已采「命令词带通配（`*?[`）照 bash 展开判」（U1-1），这一轮找那一套展开认不出、bash 却展开成重型的新写法，与崩溃放量全量入口的放行面。

## 复跑

```
bash research/prompts/test-script-align-r3-opus-model/rerun.sh /tmp/claude-1000/<新草稿目录>
```

只喂本仓 `.claude/hooks/heavy-test-guard.sh` 的 JSON、看退出码（`probe.py`，被判命令一条都不起）；hook-swap 场景在草稿目录自建假仓假钩子（`swap-scenarios.py`）；主工作区一个字节没写。模型目录 `research/prompts/test-script-align-r3-opus-model/`，`SHA256SUMS` 原样：

```
729af68807b3616d4b29dfa06ad17e6b60529b94cc7d56e762e9d6031fc83fd0  probe.py
20ce3716c2b2ce1901d8bdc602c3bc0dc62cf55c729b0da4b760d3f8c590629f  swap-scenarios.py
d472636aae4e1979f34cc2bc360dfbf7854008874324d99ae54569a2b4b296e4  rerun.sh
85b023b2f3f7673155c78cd8ccadded43a426653e6751eea4e5fe42b056beeda  expansion-check.txt
ef8279563fc2b3e0e0e365030e7230de5b40a43b0d6385a359f3adb6a2dc5466  probes.out
7072ac75acde713770bf6df8c93d8f9c03ff2ffd5a1b704e0537d01e44c0e02e  swap-scenarios.out
```

## 各格判定一览

| 编号 | 攻击面 | 被攻的 | 判定 | 证据形态 |
|---|---|---|---|---|
| A1 | 钩子放行面 | 命令词里的 POSIX 字符类 `[[:alpha:]]` / 等价类 `[[=h=]]`：bash 默认展开成真重型，钩子的 `command_word_expansions` 用 python glob 认不出（零命中）、照字面放行 | **打中（真重型放行）**：层 0、QEMU、herd7、crates 变异整表、vm-bench(QEMU) 一律退 0 | 钩子 JSON 量过；bash 展开量过；真跑那一步是推的 |
| A2 | 钩子放行面 | 崩溃放量全量入口 `crash-amplification.sh`（D13 已定项 9 的全量两样之一）不在按名字判的重型表里，正文里 cargo 藏在 bash 数组里，钩子读脚本也看不见 | **打中（真重型放行）**：`crash-amplification.sh`、`crash-amplification-two-hosts.sh` 退 0；同一道的 `--selftest`（登记 heavy）与直接 cargo 照拒 | 钩子 JSON 量过；真跑 checker 档那一步按代码推 |
| A3 | hook-swap.py | 先验后换、换上一半、回滚冲突的落地状态 | **没打中**：四种落地（退 0/4/5/6）都有定义、不静默声称成功，回滚不成时写出逐处 Edit 指路 | 草稿假钩子量过 |
| A4 | D13/main-agent 里程碑收尾 | 照字面走一遍，找做不了的一步 | **打中（做不了一步）+ 确认旧账**：崩溃放量全量没有门禁阶段、没有崩溃验证员那一道、没有派发表里的 runner；标记与路径清单欠账为 C635/C20（旧账，确认） | 读定义与 kb 量过 |

## A1 命令词里的 POSIX 字符类 / 等价类：bash 展开成真重型，钩子照字面放行

r2 的 U1-1 让钩子对命令词里的 `*?[` 先照 bash 展开、按展开出来的那几份判。展开那一步落在 `command_word_expansions`（`.claude/hooks/lib_heavy_tests.py:1742`），它用的是 **python 的 `glob.glob`**（`.claude/hooks/lib_heavy_tests.py:1754` `return sorted(glob.glob(pattern, root_dir=directory))`）。python glob 不认 POSIX 字符类 `[[:alpha:]]`、等价类 `[[=h=]]`、排序类 `[[.x.]]`——它把 `[[:alpha:]]` 当成「一个 `[`、一个 `:`、`a`/`l`/`p`/`h` 之一，再跟一个字面 `]`」，对不含 `]` 的真文件名零命中。bash 默认（无 `shopt`）却把它展开成真文件。

模型 `expansion-check.txt` 原样（两侧对照，每一行 bash 命中真脚本、python glob 零命中）：

```
bash «.claude/gate.d/checker-tier-layer0-replay.s[[:alpha:]]» -> .claude/gate.d/checker-tier-layer0-replay.sh
python glob «.claude/gate.d/checker-tier-layer0-replay.s[[:alpha:]]» -> [] 零命中
bash «.claude/gate.d/checker-tier-qemu-device-streams.s[[:lower:]]» -> .claude/gate.d/checker-tier-qemu-device-streams.sh
python glob «.claude/gate.d/checker-tier-qemu-device-streams.s[[:lower:]]» -> [] 零命中
bash «research/scripts/mutate.s[[:alpha:]]» -> research/scripts/mutate.sh
python glob «research/scripts/mutate.s[[:alpha:]]» -> [] 零命中
```

两头都失手，命令退 0：`command_word_expansions` 零命中 → `pattern_expansion_use` 交 None → 落到 `classify` 末尾按 basename 判（`.claude/hooks/lib_heavy_tests.py:1944` `name, arguments = os.path.basename(words[0]), words[1:]`），basename `checker-tier-layer0-replay.s[[:alpha:]]` 过不了 `GATE_STAGE_PATH`（`.claude/hooks/lib_heavy_tests.py:817`，正则结尾要 `([^/]+\.sh)$`）。`probes.out` 原样（退 0 = 放行）：

```
exit=0	agent=implementation-writer	command=bash .claude/gate.d/checker-tier-layer0-replay.s[[:alpha:]] /home/fy5090/code/singlefs	stderr1=
exit=0	agent=implementation-writer	command=bash .claude/gate.d/checker-tier-layer0-replay.s[[=h=]] /home/fy5090/code/singlefs	stderr1=
exit=0	agent=implementation-writer	command=bash .claude/gate.d/checker-tier-qemu-device-streams.s[[:lower:]] /home/fy5090/code/singlefs	stderr1=
exit=0	agent=implementation-writer	command=bash .claude/gate.d/checker-tier-lkmm.s[[:alpha:]] /home/fy5090/code/singlefs	stderr1=
exit=0	agent=implementation-writer	command=bash research/scripts/mutate.s[[:alpha:]] crates/mutations.tsv	stderr1=
exit=0	agent=implementation-writer	command=bash research/scripts/vm-bench.s[[:alpha:]] --selftest	stderr1=
```

非 POSIX 类对照（同一 agent、同一份钩子，照拒）：plain `.sh` 退 2、r2 已采的 `.s?` 退 2（`probes.out` A1 控制两行）。

**这不在文件头「罩不住的」里**：命令词带通配那一段（`.claude/hooks/lib_heavy_tests.py` 文件头，`SELFTEST_PATTERN_CHARACTERS = set("*?[")`、`GLOB_CHARACTERS = set("*?[")`，分别在第 303、489 行）罩不住的只列了「bash 按区域设置排序头一份不同」「超 PATTERN_EXPANSION_LIMIT」「展开出的脚本不读正文」「cd 到变量路径的相对通配」，没有 POSIX 字符类；花括号那一段罩不住的也没有。它是 r2 收紧（命令词带通配照展开判）之后新开的口子。

四问：
- 分不分辨臂：不分辨。层 0、QEMU、herd7、crates 变异、vm-bench 一起放行，病根在 `command_word_expansions` 共用的 python glob。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」，是「打中不分辨臂」那一型：把共用前提另立一笔账先修，不拿它判某一类。
- 被判的系统当时看不看得到判别它的东西：看得到。命令词里出现 `[[:`、`[[=`、`[[.`，钩子可以认出这是 python glob 认不全的记号、保守拒（判不出展开成哪一份），或改用 bash 自己展开（`compgen -G` / subprocess）。它现在选择照字面放行。
- 满足的是判据字面的哪一个分句：V1「钩子里还有哪种 bash 会展开成重型、钩子却放行的写法（文件头『罩不住的』以外）」。
- 跑前条款给的每个改法在打中的格上还中不中：这一格是零轮形态，r2 没给过改法。我提的改法（推的、只在我的模型上量过、被攻过零轮、没实现）：`command_word_expansions` 遇到命令词里带 `[[:`、`[[=`、`[[.`（python glob 认不全的记号）时，要么用 bash 自己展开，要么保守拒（当作判不出跑的是哪一份，与自证那一侧对通配词的保守一致）。要留意 python glob 还有别的与 bash 不一致处（`**` globstar 非递归、扩展通配 extglob），改法要把「认不全就拒」做成兜底，不逐个补。

## A2 崩溃放量全量入口不在按名字判的重型表里：子 agent 能跑真·崩溃放量全量

D13 已定项 9 把「全量」定成两样（`.claude/kb/decisions/13-验证路线.md:185`：「`「全量」指两样：54 号 `--full`（门禁 checker-tier-layer0-replay，已定项 15 的全量）与崩溃放量全量（`research/scripts/crash-amplification.sh` 不带参数）。`」）。这一样是真重型：`crash-amplification.sh` 不带参数跑全部分项，每项经 `crash-amplification-item.sh` → `crash-amplification-lib.sh`，后者起的是 checker 档 cargo（`research/scripts/crash-amplification-lib.sh:82` `cargo_test_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" -- --nocapture)`，第 100 行把这个数组交给 cargo 真跑）。无多机配置时单机就跑（run-condition 文件不在就 holds）。

可钩子对它一无所知：`crash-amplification.sh` 不在按名字判的重型表（`KNOWN_SCRIPT_LOCATIONS`，`.claude/hooks/lib_heavy_tests.py:319`，里面有 `mutate.sh`、`vm-bench.sh`、`gate.sh` 等，没有 `crash-amplification*`），`classify` 也没有它的分支。钩子于是把它当普通脚本读正文，而正文里的 cargo 命令是从 bash 数组 `${cargo_test_arguments[@]}` 拼出来的、分项驱动又经变量 `$driver` 起——都是文件头「看不见的（变量里拼出来的命令）」那一类，于是整条放行。`probes.out` 原样：

```
exit=0	agent=implementation-writer	command=bash research/scripts/crash-amplification.sh	stderr1=
exit=0	agent=implementation-writer	command=bash research/scripts/crash-amplification-two-hosts.sh	stderr1=
```

对照（同一 agent、同一份钩子）把这一处的倒错照出来：**同一道的 `--selftest` 登记成 heavy、照拒；真跑放行**：

```
exit=2	…	command=bash research/scripts/crash-amplification-two-hosts.sh --selftest	stderr1=✗ 提交时才跑的检查被拒：research/scripts/crash-amplification-two-hosts.sh --selftest（登记表第 40 行记它起重活：自证在本机起两份真的 checker 档用例…
exit=2	…	command=cargo test --release -p singlefs-checker-tier --test crash_amplification_fixed_script_through_rollback	stderr1=✗ 重型测试被拒：cargo test 跑到 singlefs-checker-tier 包的测试（checker 档…
exit=0	…	command=bash research/scripts/crash-amplification.sh --selftest	stderr1=
```

登记表把三份各记了轻重（`.claude/hooks/selftest-registry.tsv:38` `research/scripts/crash-amplification.sh	light	…`、第 39 行 item.sh light、第 40 行 `research/scripts/crash-amplification-two-hosts.sh	heavy	# 自证在本机起两份真的 checker 档用例（cargo test，经 crash_amplification_run_on_this_host）再导入对账`），但登记表只管 `--selftest`。真跑（不带参数）没有一处罩着：直接 cargo 被拒、假自证被拒，而把同一批 cargo 经 `crash-amplification.sh` 包一层就放行。可达的历史：任何子 agent 想跑真·崩溃放量全量，写 `bash research/scripts/crash-amplification.sh`（不要前缀、不要参数）即可。

四问：
- 分不分辨臂：分辨。只打中崩溃放量这一组入口（`crash-amplification.sh`、`crash-amplification-item.sh`、`crash-amplification-two-hosts.sh`）；别的重型各有名字那一格或登记行罩着。
- 看不看得到：看得到。`mutate.sh` 已经按名字判成「crates 变异整表」；`crash-amplification.sh` 同样是仓内固定位置的重型入口，可以照 `mutate.sh` 加进 `KNOWN_SCRIPT_LOCATIONS` 并在 `classify` 里给一支（不带 `--selftest`/`--list-items`/`--summary` 的算重型，`--selftest` 落自证登记）。
- 字面分句：V1「拦重型测试的钩子现在的放行面」「哪一步做不了、做错或被绕过」。
- 改法还中不中：零轮形态。我提的改法（推的、只在我的模型上量过、被攻过零轮、没实现）：把 `crash-amplification.sh`、`crash-amplification-item.sh` 加进按名字判的重型表（类名可归 checker 档或另立「崩溃放量」），`--selftest` 照登记表（已是 light），`--list-items`/`--summary`/`--help` 放行，别的放行条件照提交时才跑的检查；`crash-amplification-two-hosts.sh` 真跑同样按名字算重型（它的 `--selftest` 已登记 heavy）。

## A3 hook-swap.py 的落地状态：没打中（先验后换、换上一半、回滚冲突都落到定义好的状态）

在草稿目录自建假仓（`.claude/hooks/` 下一份假钩子 `fake-guard.sh` 与假共用模块 `fake_lib.py`）、用 `swap-scenarios.py` 导入 `research/scripts/hook-swap.py` 的 `swap()` 真跑四种落地，主工作区一个字节没写。`swap-scenarios.out` 结论：

- **S1 正常换上（单文件）→ 退 0**：草稿里验过，改名换上新 inode、回读一致，主工作区里再验过。
- **S2 换上之后别的会话改了新串那行（回滚找不回）→ 退 6**：换上之后喂法验不过 → 触发回滚 → 反向定点替换在当前内容里找不到新串（`research/scripts/hook-swap.py:354` 「反向定点替换对不上…换上之后有人改了这几处，没回滚」），`rollback` 交 `EXIT_ROLLBACK_FAILED`（第 78 行退出码表 `=…6`），逐处写出「要改回的几处（Edit：旧串写『现在』，新串写『改回』）」（第 334 行），反向规格落在草稿目录。钩子停在别的会话改成的样子，不静默声称成功。
- **S3 第二份有多个硬链接（换上一半）→ 退 4**：换法先把每一份都预演一遍（`research/scripts/hook-swap.py:435` `rehearse(…)`，第 322 行停在改名之前），第二份硬链接在预演阶段就被拒、一个字节没写，第一份保持原文（模型里核到「第一份 fake_lib.py 还是原文： True」）。rehearse-全部-再换的次序让「可预检的失败」不会落成半换。
- **S4 换上之后别的会话另加一行、钩子仍好 → 退 0**：加的那行与新判法不冲突，换上之后验过，别的会话加的一行留着。（「换上之后验不过、反向找得回 → 退 5、别的会话加的一行留着」那一支由 `hook-swap.py --selftest` 的第 5 种覆盖，主 agent 已跑过 9 种退 0。）

结论：hook-swap 没把主工作区落成「半换又声称成功」的坏态。最坏是退 6（回滚不成），它写出逐处 Edit 指路、反向规格留盘，给人收尾——这正是 r2 U4「改钩子的流程会拦死会话」要的那份兜底。没打中。

## A4 照 D13 已定项 9 与 main-agent「禁止」第一条走一遍里程碑收尾：一步做不了 + 旧账确认

`.claude/main-agent.md:10`「先弹窗请用户跑一趟全量（54 号 `--full` 与崩溃放量全量两样）」（「禁止」第一条，另见该行「里程碑收尾、动了崩溃点重放或判器的那一批…提交之前」）。逐步走：

1. **判「动了崩溃点重放或判器」**：路径清单只写在 D13 已定项 9 正文里，不是机器可读的文件（C635 的「前置」栏自认：「路径清单今天只写在 D13（验证路线） 已定项 9 正文里，门禁要读得先登记成一份文件」）。主 agent 只能手判。确认旧账，不新。
2. **跑 54 号 `--full`**：有去处——崩溃验证员定义第 1b/2 步在 HEAD+暂存区的 worktree 里跑，逐条有全绿标记。能做。
3. **跑崩溃放量全量**（`crash-amplification.sh` 不带参数）：**做不了对等的一步**。它没有门禁阶段（`.claude/gate.d/` 下没有 `crash-amplification*`），崩溃验证员定义只跑那五道门禁、不含它，派发表里也没有哪个 agent 负责跑它。主 agent 没法像派崩溃验证员跑 54 `--full` 那样派人跑崩溃放量全量；而钩子又不认它是重型（A2），没有 `SINGLEFS_HEAVY_TESTS` 纪律、没有复用标记、没有 worktree 隔离。这一样全量落在自动化/门禁机器之外。
4. **判绿写标记**：54 `--full` 有逐条全绿标记；崩溃放量全量只留 `runs/entry-<UTC>-<pid>/`、没有门禁读的标记（C635「怎么拦」栏）。「上一次全量」只能由主 agent 手写进里程碑记录一行。确认 C635、C20，不新。

A4 里真·新的是第 3 步：崩溃放量全量没有 runner / 门禁阶段 / 崩溃验证员那一道，与 A2（钩子不认它是重型）同根。这归「做得了但做不对等、且无人守」。C635/C20 已记「标记」与「上次跑过」那一半，没记「崩溃放量全量没有 runner、钩子也不拦」这一半。

## 没打中的形状

- **hook-swap.py**（A3）：先验后换、换上一半、回滚冲突的四种落地都有定义、不静默声称成功；没找到让它把坏钩子换上去又报成功的输入。只用草稿假钩子跑了导入 `swap()` 的四种，没穷举所有并发时序。
- **登记表按真实路径认 / `--item` / 样本档**：r1 T2-3、r2 U1-2、U2 已攻并采，这一轮不重复。喂了几条确认 plain `.sh`、`.s?`、直接 cargo、登记 heavy 的 `--selftest` 照拒（`probes.out` 控制行），没找到这几处的新绕法。
- **命令词带通配的别的不一致处**：`**` globstar（python glob 非递归）、extglob 对 54/门禁阶段都只隔一层或默认关着，没造出比 A1 更短的绕法；A1 的「认不全就拒」兜底已覆盖这一类，没逐个演。

## 这条腿自己的限度

- 跑过的全是喂钩子 JSON、看退出码。A1、A2 的最后一步（bash 展开通配后真跑起重型、`crash-amplification.sh` 正文里真起 checker 档 cargo）是按 bash 展开（`expansion-check.txt` 量过）与代码（`crash-amplification-lib.sh:82/100`）推的，我没有在带 git 的真树上真起那些重型目标——那就是跑重型测试。
- A1、A2 两个改法全是推的、只在我的模型上想过、被攻过零轮、没实现；写回时要标「被攻过零轮」，并各记一笔自己的检查账（A1 的改法会不会误拒合法的 POSIX 类用法、bash 与 python glob 在别的记号上还差多少，我没扫全；A2 的改法要不要连 `crash-amplification-lib.sh` 一起认、`--selftest` 之外还有没有只读子命令，没扫全）。
- A4 第 3 步是读定义与 kb 推的（崩溃放量全量没有 runner / 门禁阶段），没真走一遍里程碑收尾流程。

## 没做什么

- 没判 V2、V3（正推那两格）；没替主 agent 采纳；坏树上量出的放行只是副本观测，主 agent 要坐实请在本仓路径上只读地复喂同一批 JSON（`rerun.sh`）。
- 没跑任何重型测试：没真起 54 号、QEMU、herd7、crates 变异整表、崩溃放量全量、重型自证。
- 没改主工作区任何文件；写进去的只有本报告与模型目录。
- 草稿目录 `/tmp/claude-1000/test-script-align-r3-opus/` 里留着 `run/`（探针输出、检出记录、swap 假仓）与辅助脚本，没有编译目录、没有仓副本；交回前删掉，见交回正文。
