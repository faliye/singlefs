# ledger-topdown-governance-r2 攻方腿报告（2026-11-18）

正文 `research/prompts/_ledger-topdown-governance-r2-body.md`，开工快照 `research/prompts/ledger-topdown-governance-r2-snapshot/judged-sha256.txt`：开工时对快照清单跑 `sha256sum -c`，22 份全部 OK（没有一行不是 OK）。这条腿没写模型（只有喂法、探针小树与一份日志样本），所以没有 `rerun.sh`；探针文件都在草稿目录 `/home/fy5090/.local/share/singlefs/work/governance-r2-attack/`，sha256 列在末尾「要主 agent 跑的命令」之前。下面所有「退几」「认不认」凡没写「量的」的都是推的，等主 agent 跑。

## 各格判定一览

| 问 | 结论 | 打中摘要 |
|---|---|---|
| 第 1 问 检出钩子 ⑭ | 推不出（打中） | `cp -b`、`install -b`、`mv --backup` 被当成「不盖目标」放行；`cp -r 草稿/src/.`、`rsync -a 草稿/src/`、`cp -r 草稿/src prune-server/` 落点算成目录放行；`ln -sf`、`tar -x`、硬链接放行；`rm` 之后同名新建（Bash 或 Write）两道闸都放，等于整份换掉服务源码。`mv` 盖上去算「写」、照拒（成立） |
| 第 2 问 写闸两种出路 | 推不出（打中） | 门禁阶段（`.claude/gate.d/broad-mutation.sh` 这类）样本档要临时服务，可按表的写法它们一律不在表里，拒绝说明给「加 --verify」，照做退 5 回滚，与 required 那一支要躲的是同一个失败；表读不到、两种都有两支说明成立 |
| 第 3 问 换上工具新认法 | 推不出（打中） | 库按「文件名」认、不按目录：`from run_directory import …` 解到同目录另一份同名库的调用方被当成被换库的调用方，换 e171 改后那一份 `run_directory.py` 报缺名字、挡换上（误报）；`sys.modules.get("…")` 与 `from sys import modules` 两种取法既不核也不列本次未判（漏）；多出的 4 处逐处判在正文 |
| 第 4 问 规格工具工作根与临时文件 | 推不出（打中，轻） | 在仓副本里起时工作根一律取不到、退 3，拒绝说明叫往「主工作树根的 multi-host.env」写 SHARED_DATA_ROOT，在副本里就是叫往副本里写多机配置，与 minimal-copy 的「多机配置决不拷进副本、经 SINGLEFS_MULTI_HOST_CONFIG 交」相反；派第二台、自证小仓、被杀留垃圾三格成立 |
| 第 5 问 任务服务认记号 | 推不出（打中） | 第一轮判决的改法写「与门禁那一份库同一个认法」，修补只认三种；记号与 `test <名> ... ` 之间隔字、接在别的测试没换行的输出后面、前面有颜色码、后面接制表符四种门禁库认、服务不认；门禁库另把 `CANNOT_RUN_…` 这种更长的词误认成记号 |
| 第 6 问 注释历史判法 | 推不出（打中） | 「以前面那一份为准」「可以前面带 ./」这类「以」「可以」接「前面」的现状句被收窄之后新判红；主工作区两行标记不在三类特殊缘由里，邻行已有不逐字引的写法，该改写、不该豁免 |
| 第 7 问 证红起点 | 推不出（打中，轻） | 判法脚本与重测试目标登记表取主工作区，变异表与行键取副本；副本里新登记的崩溃枚举用例不补 `--release`（登记表读主工作区那一份），行键与验证段门禁算的对不上；实现员定义第 3 步还写证红脚本在副本的 `scripts/` 里 |

四句（每个打中都过一遍，逐问写在各节末尾「四句」一行）：这一轮被判的是治理文件的修补，没有「臂」，「分不分辨臂」一律答「不分辨，打中的是修补本身」；判别它的东西闸当时看不看得到、满足的是条款哪一句、跑前改法在打中的格上还中不中，逐问写。

## 第 1 问 检出钩子 ⑭ 的漏放与出路

结论：推不出。`mv` 草稿盖上去算它判的写，照拒（成立）；别的写法有 9 种放行，另有「先删再建」两步把 ⑭ 与写闸第十一道一起绕过。

证据：

- 认法建在 ⑫ 的 `overwrite_steps` 上，`cp`、`mv`、`install` 带 `-b`、`--backup` 的一律当「已有的目标不被盖」不出写：「-n / --no-clobber / --update=none… / -b / --backup：已有的目标不被整份盖掉」（`.claude/hooks/bash-command-detector.sh:2961`），「elif letter in "nb":」（`.claude/hooks/bash-command-detector.sh:3028`），「elif option in ("--no-clobber", "--backup")」（`.claude/hooks/bash-command-detector.sh:3009`），「if not parsed.keeps_existing_target:」（`.claude/hooks/bash-command-detector.sh:3353`）。GNU 的 `-b` 是先把旧目标改名成 `目标~`、再照样盖写目标。量的（草稿目录里两份普通文件，GNU coreutils 9.4）：`cp -b new.rs dst/protocol.rs` 退 0，之后 `dst/protocol.rs` 是 `new-b`、`dst/protocol.rs~` 是 `old`。
- 写进目录时落点按「目录/源的文件名」算：「names.append(os.path.basename(source.rstrip("/")))」（`.claude/hooks/bash-command-detector.sh:3538`）。源写成 `草稿/src/.` 时文件名是 `.`，源写成 `草稿/src/`（rsync）时文件名是 `src`，落点都成了目录或不存在的路径；而 ⑭ 只认已存在的普通文件：「return listed and os.path.isfile(absolute)」（`.claude/hooks/bash-command-detector.sh:2264`）。量的（同上，草稿目录里）：`cp -r srcdir/. dst/` 之后 `dst/protocol.rs` 变成 `new-c`；`rsync -a srcdir/ dst/`（大小不同）之后变成 `new-rsync-longer`。
- `ln`、`tar`、`unzip`、`git apply`、`patch` 不在 `follow_simple_command` 认的命令里（`rm` 只记成留住旧字节的一步：「elif name == "rm":」（`.claude/hooks/bash-command-detector.sh:3365`），之后「continue   # git add、mv 挪走的源、rm：不是写进去的」（`.claude/hooks/bash-command-detector.sh:2187`））。文件头自己写了判不到的一部分：「经 subprocess 起的写法、mv 挪走与 rm 删掉的源。」（`.claude/hooks/bash-command-detector.sh:303`）、「patch 与 git apply」（`.claude/hooks/bash-command-detector.sh:302`）；`-b`、目录落点、`ln`、`tar`、硬链接文件头没写。
- 硬链接：仓与工作根在同一个 ext4 上（量的：`stat -c %d` 两边都是 66306，`df -T` 同一个 `/dev/nvme0n1p2`）。`ln prune-server/src/protocol.rs <草稿目录>/x.rs` 之后往 `x.rs` 写，落点解符号链接也解不出它是服务源码，⑭、⑫、写闸都只看到草稿目录。
- 先删再建：`rm` 放行（上面第 2187、3365 行）；之后同名的写是新建，⑭ 放行（第 2264 行的 `os.path.isfile`），写闸第十一道同样放行：「except (FileNotFoundError, NotADirectoryError):   # 新建文件」（`.claude/hooks/write-guard.sh:782`）。两步之间服务源码缺一份，正是第十一道要挡的「改到一半编不过」。
- 定义的说法比闸宽：「Bash 里的追加、`cp`、定点替换工具这些写法由检出钩子 ⑭ 执行前拒」（`.claude/agents/tooling-writer.md:31`），没写哪些写法不拦。

主 agent 那一侧：⑭ 与第十一道对主 agent 都不判，是设计（主 agent 打补丁）；主 agent 直接分几次 Edit 服务源码今天没有别的闸，这一点与定义「由主 agent 一次打上」不矛盾，只是没有闸兜，不算打中。拒绝说明的出路（仓副本里改、存补丁、主 agent 打）推的过得了别的闸：第一轮任务 20022 量过副本带 `prune-server`；`scripts/gate/apply-writer-patch.py` 的 `crates.patch` 改到的路径不限、整份改名换上，或主 agent 直接 `git apply`，检出钩子 ⑮ ⑱ 都不管 `git apply`。

最小输入（喂法 `q1/feeds-bash.json` 20 条、`q1/feeds-write.json` 2 条，身份都是 `tooling-writer`、`cwd` 是 `{root}`，Write 那两条不带 agent_id）。该拒而预期放行（喂法写 `exit: 2`，预期报「退 0（应当 2）」）的 13 条：

```
cp -b <草稿目录>/q1/protocol.rs prune-server/src/protocol.rs
install -b -m 644 <草稿目录>/q1/protocol.rs prune-server/src/protocol.rs
mv --backup=numbered <草稿目录>/q1/protocol.rs prune-server/src/protocol.rs
cp -r <草稿目录>/q1/src/. prune-server/src/
rsync -a <草稿目录>/q1/src/ prune-server/src/
cp -r <草稿目录>/q1/src prune-server/
ln -sf <草稿目录>/q1/protocol.rs prune-server/src/protocol.rs
ln prune-server/src/protocol.rs <草稿目录>/q1/protocol-hard-link.rs
printf '// probe\n' >> <草稿目录>/q1/protocol-hard-link.rs
tar -xf <草稿目录>/q1/src.tar -C prune-server
git apply <草稿目录>/q1/change.patch
patch -p1 -i <草稿目录>/q1/change.patch
rm prune-server/src/protocol.rs
```

（喂法里 `<草稿目录>` 写的是绝对路径。）对照 7 条预期都对：`mv 草稿/protocol.rs prune-server/src/protocol.rs`、`cp 草稿/protocol.rs prune-server/src/`、`cp 草稿/Cargo.toml prune-server/`、不带 `-b` 的 `install`、`dd of=`、`truncate -s 0 prune-server/Cargo.lock` 都退 2，`cp` 新建 `prune-server/src/probe_absent_after_removal.rs` 退 0。预期汇总行「喂了 20 条，13 条不对」、整条退 1。写闸那一份：Write 新建 `prune-server/src/probe_absent_after_removal.rs` 退 0、Write 盖已存在的 `protocol.rs` 退 2，「喂了 2 条，0 条不对」、退 0（这两条合上 Bash 那条 `rm` 就是先删再建）。

改法（推的，被攻过零轮）：`-b`、`--backup`、`-S` 不再算「不盖」（只有 `-n`、`--no-clobber`、`--update=none` 算）；写进目录而源是目录（`-r`、`-a`、rsync）时，落点取「目标目录下与源目录同相对路径的每一份已存在文件」，算不出的照拒；`ln`（`-f`、`-s`）的目标与 `tar -C`、`unzip -d` 落在两个服务里的照拒；`rm`、`mv` 挪走两个服务 `src/` 与构建清单下已存在的文件同样拒（出路同一句）；硬链接这一形认不出，文件头「判不到的」补一句。四句：不分辨臂；闸当时看得到命令原文与目标是否已存在，`-b` 与目录落点是认法错，硬链接与先删再建是闸看不到「之后那一步」；满足的是第一轮判决第 2 条「工具员在 Bash 里改两个服务 `src/` 下已存在的文件没有闸拦」；第一轮改法（⑦ 另认追加、`cp`、定点替换）在这 13 格上都不中。

## 第 2 问 写闸拒绝说明的两种出路

结论：推不出。三种情形里「表读不到」「一份规格里两种都有」两支说明成立；「登的是别的判定值」那一支本身成立，但表的收法让一类真要临时服务的脚本落进「加 --verify」那一支，说明写错。

走到的行：

- 表读不到：`temporary_service_required` 交 None，记一处没判全：「要不要临时服务认不出，拒绝说明两种出路都写」（`.claude/hooks/write-guard.sh:735`）；两种出路都写，说明里带「（表这一次读不到）」（`.claude/hooks/write-guard.sh:752`）。成立。
- 登的是别的判定值：只有第二列恰好是 required 才算：「return parts[1] == TEMPORARY_SERVICE_REQUIRED」（`.claude/hooks/write-guard.sh:733`），别的值（own-server、写错的值）都走「return f"再去掉 --dry-run、{SWAP_WITH_VERIFY_REMEDY}。"」（`.claude/hooks/write-guard.sh:755`），即「SWAP_WITH_VERIFY_REMEDY = "加 --verify」（`.claude/hooks/write-guard.sh:739`）。own-server 走这一支是对的；写错的值由表自己的判法（`.claude/hooks/lib_project_code_runs.py` 的 `temporary_service_table_at`）报写坏，这里不另报，可接受。写闸自己另写了一份读表（按空白切、不核理由长短），与 `lib_project_code_runs.py` 那一份对「理由太短的 required 行」判得不一样（写闸认、库不收），推的，没造出会走错的输入。
- 一份规格里两种都有：required 那一份的说明写了「同一份规格里有一份登记成 required，整份规格都不加 --verify」（`.claude/hooks/write-guard.sh:744`）；不在表里那一份的说明只写加 `--verify`，没提这一句。工具员先被拒的是不在表里那一份时读不到这句；照做的后果是 `--verify` 只跑不在表里那一份的自证，过得去，不算失败。成立（偏严，不是错）。

打中：表只收「非门禁阶段」：「selftest-registry.tsv 里非门禁阶段的每一行」（`.claude/hooks/selftest-temporary-service.tsv:14`）。可门禁阶段在第十一道的目录里（`.claude/gate.d/` 是 `SWAP_ONLY_DIRECTORIES` 之一），其中几份的样本档要临时服务：「不在临时服务底下退 2、一份样本都不造、不判」（`.claude/gate.d/broad-mutation.sh:198`），自证登记表写它「样本档包在临时 prune-server 底下起」（`.claude/hooks/selftest-registry.tsv:223`）；`.claude/gate.d/crash-point-replay.sh`、`.claude/gate.d/crates-mutation-replay.sh` 同样在样本根上先调 `selftest_service_problem`（`grep -n selftest_service_problem` 现查，各有一处）。工具员 Edit 这几份，表里查不到，说明给「加 --verify '<被换脚本的 --selftest 命令>' 换上」：门禁阶段没有 `--selftest`，能写的只有样本档那一条，在换上工具里起拿不到临时服务（不带 `--server-binary` 时包装也拿不到编译目录），退非 0，换上工具退 5 回滚。这正是 required 那一支说明里写的失败，第一轮判决第 4 条要修的就是这一种。

最小输入：`q2/edit-gate-stage.json`（tooling-writer 用 Edit 改 `.claude/gate.d/broad-mutation.sh` 第 2 行的一个字，不带 agent_id），预期写闸退 2、说明里是「再去掉 --dry-run、加 --verify '<被换脚本的 --selftest 命令>' 换上。」；对照 `q2/edit-required-script.json`（改 `scripts/mutation/prove-red.sh`，表里登 required），预期退 2、说明里是「不加 --verify 换上」那一段。两条都是推的，喂法见末尾。

改法（推的，被攻过零轮）：第十一道判出路时，门禁阶段（`.claude/gate.d/*.sh`）不查表，另读 `.claude/hooks/selftest-registry.tsv` 那一行的样本档写法里有没有临时服务的包装，有就走「不加 --verify、样本档另交」；或者表的收法不再排除门禁阶段。四句：不分辨臂；闸当时看得到被改的路径，也读得到两张登记表，判别的东西看得到；满足的是第一轮判决第 4 条「出路写了一条照做会失败的命令」；第一轮的改法（按表分两种）在这一格上不中。

## 第 3 问 换上工具新认的两种引法

结论：推不出。新认法会把同名而不同目录的模块算成被换的库（误报、挡换上）；`sys.modules.get(…)` 一类今天在扫描目录里有一处取的就是要核的库，既不核也不列本次未判。

误算进来的同名模块：

- 库只按文件名认，不带目录：「targets[stem] = stem + ".py"」（`scripts/gate/hook-swap.py:1368`）；`from … import` 只看模块名最后一段：「if isinstance(node, ast.ImportFrom) and (node.module or "").rsplit(".", 1)[-1] in self.targets:」（`scripts/gate/hook-swap.py:839`）。这一轮让只用 `from 库 import` 的文件不再提早退出：「只用 from <库> import 名字引库的文件（没有别的载入或绑定）照样核引到的名字」（`scripts/gate/hook-swap.py:88`），同名模块也就跟着被核了。
- 真仓里的一处（量的：我自己写的只读 AST 脚本 `probe/stem_collisions_fast.py`，扫换上工具扫的五个目录，结果 `probe/stem_collisions.out` 末行「duplicate stems 32 total 36」，其中调用方同目录就有那份同名库的 29 处）：「from run_directory import (STREAM_NAMES, device_stream_summaries, device_verdict_line」（`scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py:49`），运行时解到的是同目录那一份，它有「def device_stream_summaries(」（`scripts/experiment-apparatus/e171_three_oracle_collusion/run_directory.py:195`）；`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_directory.py` 里 `grep -n device_stream_summaries` 零命中。换后面这一份（哪怕只改一个注释）时，换上工具会说 `comparator.py` 引的名字缺了，草稿验不过、不换。`comparator.py` 里 `run_directory` 只有这一处 `from` 引（`grep -n run_directory` 现查），换上前它提早退出、不核，所以这是这一轮修补带进来的。同形还有 `system_configuration`、`executor_core`、`collusion_matrix` 等，逐条在 `probe/stem_collisions.out`。
- 逐类排除不了：五个目录里同名 `.py` 有 32 个名字（同一个脚本量的），多数是 `scripts/experiment-apparatus/` 下各实验的同目录模块。

本次未判多出的 4 处（主 agent 日志 `换上工具换上后九份库.log` 与 `底2-九份库.log` 逐行比，现读）：

| 处 | 写法 | 该不该认 |
|---|---|---|
| `reuse_library.py:452` | `return sys.modules[module_name]`，键是形参 | 不认是对的：键运行时才定；它的调用处传写死的名字，跟进去才折得出，可不做 |
| `prune-server-load-simulation.py:66` | `return sys.modules[name]`，在通用载入函数 `load_by_path` 里 | 列出来是噪声：调用处 `load_by_path("singlefs_prune_server_client", …prune_server_client.py)` 已按「名字里带 load」认成载入；函数体里这一处可以不报 |
| `prune_server_client_selftest.py:901` | `sys.modules[Client.__module__]`，`Client` 是从 `prune_server_client` import 的 | 该认：取出的就是要核的库，之后 `client_module.SEGMENT_BYTES`、`client_module.SEGMENT_INDEX_DIGITS`、`client_module.segment_key` 都是真引用，今天没核 |
| `prune_server_client_selftest.py:2024` | `sys.modules.setdefault("singlefs_prune_server_client", sys.modules["prune_server_client"])` | 前半是按写死的键存库，该认成存法；这一行之后本文件没按这个键取名字，认不认结果相同 |

（四处的文件全路径：`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py`、`scripts/prune-server/prune-server-load-simulation.py`、`scripts/prune-server/prune_server_client_selftest.py`。）

`sys.modules.get(…)` 一类现数（量的：`grep -rnE 'sys\.modules\.(get|setdefault|pop)\(|from sys import modules|modules\['` 扫五个目录、去掉 `fixtures/` 与 `sys.modules[`，23 行：写成这三种调用的 6 行，其中 1 行在字符串里的代码里，另 17 行是别的叫 `modules` 的容器）：`scripts/gate/admission.py:4077`（setdefault 存）、`removal_effect.py:133`、`:142`（pop）、`run_replay_result_lines.py:128`（get，键是循环变量）、`crates-mutation-rows.py:1239`（字符串里的代码）、`scripts/crash/crash-flow-coverage.py:898`。最后一处取的就是要核的库：「client = sys.modules.get("singlefs_prune_server_client")」（`scripts/crash/crash-flow-coverage.py:898`），同一份文件按这个名字载入 `prune_server_client.py`：「name = "singlefs_prune_server_client"」（`scripts/crash/crash-flow-coverage.py:1072`），之后「client.reset_process_client()」（`scripts/crash/crash-flow-coverage.py:906`）。这一处引用不核、也不在本次未判里（九份库那一趟的 10 处里没有它），与文件头这一句对不上：「认不出去向的写法逐处列成」（`scripts/gate/hook-swap.py:85`）。`from sys import modules` 之后 `modules[…]` 也一样：只认点名路径是 `sys.modules` 的：「MODULE_REGISTRIES = frozenset({"sys.modules"})」（`scripts/gate/hook-swap.py:685`）。

最小输入（`--check-callers`，只核不换、不写文件）：

- 误报，真仓：`--check-callers scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_directory.py`，预期退 1，✗ 行点名 `scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py` 引 `run_directory.device_stream_summaries`、`run_directory.device_verdict_line`。
- 误报，小树 `q3/stem`：`.claude/scripts/project_preflight.py` 只有 `preflight`，`scripts/other/project_preflight.py` 另有 `sibling_only`，`scripts/other/caller_from_sibling.py` 写 `from project_preflight import sibling_only`。核前一份，预期退 1、点名 `caller_from_sibling.py:1 引了 project_preflight.sibling_only`。
- 漏，小树 `q3/get`：调用方写 `sys.modules.get("project_preflight").shared_preflight_directory()`，库里没有这个名字；预期退 0，「缺 0 处，本次未判 0 处」。
- 漏，小树 `q3/fromsys`：`from sys import modules` 之后 `modules["project_preflight"].shared_preflight_directory()`；预期同上退 0、本次未判 0 处。
- 对照：第一轮的 `governance-r1-fix-swap-tools/q2/root`（`sys.modules["project_preflight"]` 下标取），预期退 1（背景材料写换上后的探针 20053 退 1，这里重跑一次当对照）。

改法（推的，被攻过零轮）：目标库按「目录 + 文件名」认，`import x`、`from x import` 在调用方的 `sys.path` 里能先解到同目录那一份同名文件的（调用方同目录有 `x.py` 而被换的不是它），不算被换库的调用方，列一行「同名不同目录，没核」；`sys.modules.get(…)`、`setdefault`、`pop` 与 `from sys import modules` 照下标取同样认，认不出的进本次未判。四句：不分辨臂；工具当时看得到调用方的路径与同目录的文件，判别的东西看得到；误报满足文件头「把该换的挡住」的反面（草稿验不过），漏满足「认不出去向的写法逐处列成本次未判」那一句的反面；第一轮改法（提早退出把 from 算作绑定）正是误报的来源，对漏那两格不起作用。

## 第 4 问 规格工具的工作根与临时文件

结论：推不出（轻）。派第二台、自证小仓、被杀留垃圾三格成立；「多机配置读不到」里在仓副本起的那一形，退 3 对，出路写错。

三种情形走到的行：

- 多机配置读不到：工作根只在空环境里取：「不看命令前的 SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 与 SINGLEFS_MULTI_HOST_CONFIG」（`scripts/gate/make-swap-spec.py:16`），「return (module.shared_data_path("work", {}),)」（`scripts/gate/make-swap-spec.py:272`）；空环境下配置文件取库所在仓的主工作树根：「return os.path.join(main_worktree_root(REPOSITORY_ROOT), MULTI_HOST_CONFIGURATION_NAME)」（`.claude/hooks/lib_work_root.py:157`）。取不到就退 3，出路是「在多机配置（主工作树根的 multi-host.env）里写 SHARED_DATA_ROOT」（`scripts/gate/make-swap-spec.py:277`）。主工作区里起：配置在，结局照多机配置的工作根判（第一轮修补的探针 20057 退 3「不在工作根下」，现读日志 `后2-九份库.log`）。在仓副本里起（脚本取自副本）：副本是 git init 的新仓，主工作树根就是副本根，而「决不拷的 NEVER_COPIED：仓根的多机配置 multi-host.env」（`scripts/hosts/lib_repository_copy.py:26`）；副本里起命令的约定是「副本里起命令的，多机配置照 lib_repository_copy.py configuration-path <源树> 打的那一份显式交给 SINGLEFS_MULTI_HOST_CONFIG」（`scripts/hosts/minimal-copy.py:35`），可规格工具不看这个变量。结局：照约定交了 SINGLEFS_MULTI_HOST_CONFIG 也退 3「取不到工作根」，出路叫往副本根写多机配置，与决不拷相反。
- 派到第二台：脚本取自共享根上同一路径的仓，主工作树根的 `multi-host.env` 是同一份文件，SHARED_DATA_ROOT 相同，结局与本机相同（推的；第二台上 `realpath` 会不会解出别的路径没量过）。
- 自证的小仓：「root = os.path.join(work, "repository")」（`scripts/gate/make-swap-spec.py:472`），不是 git 仓，主工作树根就是它自己，读它自己那份 `multi-host.env`，两台一样（推的）。

写到一半被杀：临时文件是「descriptor, temporary = tempfile.mkstemp(prefix=os.path.basename(out_path) + ".", suffix=".tmp", dir=os.path.dirname(out_path))」（`scripts/gate/make-swap-spec.py:206`），只在 Python 异常里删：「except BaseException:」（`scripts/gate/make-swap-spec.py:214`）。SIGINT 会删；SIGTERM（默认处理，不抛异常）与 SIGKILL 留下 `<--out 所在目录>/<--out 的文件名>.<8 个随机字符>.tmp`，例 `<草稿目录>/spec.json.k3x9_q2a.tmp`（推的），`--out` 本身没被碰。下一趟取新的随机名，不拒、也不清那一份；完工清理闸只列副本、编译目录与库拷贝，不列它。成立（留垃圾、不碍事）。

最小输入：在主工作区建一份带 `scripts/gate/make-swap-spec.py` 的仓副本，在副本里起它、照约定交 `SINGLEFS_MULTI_HOST_CONFIG`（命令见末尾），预期退 3、说「取不到工作根」并叫写 SHARED_DATA_ROOT；对照：同一条从主工作区起，`--out` 写在草稿目录，预期退 0 写出规格。

改法（推的，被攻过零轮）：取不到工作根时，库所在仓没有 `.git` 目录以外的主工作树、又不在主工作区（副本）的，拒绝说明改成「从主工作区起这条脚本」，不叫往副本里写多机配置；不改「不认命令前的变量」那一条（第一轮第 7 条要的就是它）。四句：不分辨臂；脚本当时看得到自己在哪棵树、那棵树有没有多机配置；满足的是共用约束「所有任务都要多机可用」与「出路本身要过得了别的闸」的反面；第一轮改法在这一格上不中。

## 第 5 问 任务服务认记号行的三种形态

结论：推不出。修补只认三种，不是第一轮判决写的改法：「与门禁那一份库同一个认法」（`research/prompts/ledger-topdown-governance-r1-main-verification.md:100`）。

两边的认法：服务只认这一个记号：「GPU_NOT_RUN_MARK = b"NOT_RUN_NO_GPU_ADAPTER"」（`task-server/task_server.py:897`），位置只认行首「if line.startswith(GPU_NOT_RUN_MARK):」（`task-server/task_server.py:1124`）、行首空格制表符「stripped = line.lstrip(b" \t")」（`task-server/task_server.py:1128`）、紧接 `test <名> ... `「if separator > len(b"test ") and line.startswith(GPU_NOT_RUN_MARK, after):」（`task-server/task_server.py:1134`），记号后面只认空格或行尾「if rest and not rest.startswith(b" "):」（`task-server/task_server.py:1149`）。门禁库：「NOT_RUN_MARKER_FORM = re.compile(r"NOT_RUN_[A-Z][A-Z0-9_]*(?=\s|$)")」（`scripts/gate/lib_libtest_result_lines.py:104`），一行里哪儿都找：「starts = [found.start() for found in NOT_RUN_MARKER_FORM.finditer(line)]」（`scripts/gate/lib_libtest_result_lines.py:203`），`test <名> ... ` 之后的整段也找：「for kind, piece in marker_pieces(started.group("rest")):」（`scripts/gate/lib_libtest_result_lines.py:239`）。

逐字的样本行在 `q5/lines.log`（一段 Running、7 个结果、汇总行 7 passed），各行与预期（都是推的）：

| 行（原样） | 服务认不认 | 门禁库认不认 | 谁对 |
|---|---|---|---|
| `NOT_RUN_NO_GPU_ADAPTER test=form_line_start reason=样本：行首` | 认 | 认 | 一致 |
| `    NOT_RUN_NO_GPU_ADAPTER test=form_indented reason=样本：行首带空白` | 认 | 认 | 一致 |
| `test form_after_result_start ... NOT_RUN_NO_GPU_ADAPTER test=form_after_result_start reason=…` | 认 | 认 | 一致 |
| `进度 42% NOT_RUN_NO_GPU_ADAPTER test=form_after_other_output reason=…` | 不认 | 认 | 门禁库：带 `--nocapture` 几条同时跑时，别的测试 `print!` 没换行，记号就接在它后面 |
| `ESC[33mNOT_RUN_NO_GPU_ADAPTER test=form_colored reason=…`（行首是 0x1b） | 不认 | 认 | 门禁库（今天产品不打颜色码，推的） |
| `test form_words_between ... 卡探测：NOT_RUN_NO_GPU_ADAPTER test=form_words_between reason=…` | 不认 | 认 | 门禁库：单线程时测试先打的字接在结果行起头后面 |
| `NOT_RUN_NO_GPU_ADAPTER<TAB>test=form_tab_after reason=…` | 不认 | 认 | 两可：产品打的是空格 |
| `样本日志摘录：CANNOT_RUN_ANYTHING_HERE 不是记号` | 不认 | 认（误认） | 服务：门禁库的正则前面没有词界，更长的词的后半截也算记号 |

预期服务那一趟「本次未跑」3 条（form_line_start、form_indented、form_after_result_start），门禁库 `recordable` 末行 markers=8。另有两边一起误认的：任务自己的输出里转印别处日志的一行（例：任务里 `grep NOT_RUN_ <上一趟的日志>`，打出 `NOT_RUN_NO_GPU_ADAPTER test=gpu_old reason=…`），两边都认，服务把它报成这一趟本次未跑；这一形分不出两边，不算差。门禁库认任何 `NOT_RUN_<原因>`、服务只认 GPU 那一个，用途不同，不算错。

最小输入：任务 `cat q5/lines.log` 交上去看 `task-server/task status <号>` 的本次未跑那一段；同一份文件喂 `lib_libtest_result_lines.py recordable`。命令见末尾。

改法（推的，被攻过零轮）：服务照第一轮判决改法改成「记号作一个词（前面是行首、空白或非 `[A-Z0-9_]` 的字）出现在一行任何位置都认」，名字取记号自己的 `test=`；门禁库的正则前面加词界（`(?<![A-Z0-9_])`）。四句：不分辨臂；服务读日志时看得到整行，判别的东西看得到；满足的是第一轮判决第 9 条「服务只认行首就漏认」与改法那一句；这一轮的三形认法在「隔字」「接在别的输出后」两格上照漏。

## 第 6 问 注释历史判法收窄后的误红与两行标记

结论：推不出。

判法：「BEFORE_PATTERN = r"(?<!除)(?:以前(?![缀者面])|(?<!所)以前(?=面))"」（`scripts/gate/comment-history-check.py:163`）。「以前」后面接「面」而前面不是「所」的一律判红。这一轮收窄之前这一形一律放过，所以下面两句是这一轮新判红的（都是讲现状）：

- 「两份结果不同时以前面那一份为准」：「以…为准」的「以」接「前面」。
- 「路径可以前面带 ./，归一之后再比」：「可以」接「前面」。同形还有「难以」「足以」「得以」「予以」接「前面」。

另一句不是这一轮带来的、收窄前后都红：「键可以前置一段命名空间」（「可以」接「前置」），记在「没打中的形状」之外，供主 agent 判要不要另记一笔。

最小输入：`q6/root/scripts/probe.sh` 六行注释（上面三句、对照「所以前面已发的写都看得见」不红、「以前这里只认 --bin」红），`--list` 预期列 4 行：第 2、3、5、6 行。

两行标记：`.claude/gate.d/doc-text.sh` 第 659 行（开关说明「「以前」后面紧跟「面」就放过…」，标记理由「history-kept: 这一行引的是判法要判红的样本里的词，说的是开关弄坏哪一处」（`.claude/gate.d/doc-text.sh:659`））与第 978 行（样本说明逐字引两句，标记理由「history-kept: 这一行逐字引的是红样本里要判红的两句原文」（`.claude/gate.d/doc-text.sh:978`））。判：该改写、不该豁免。理由：共用约束写「特殊缘由只三类」（`.claude/agent-common.md:42`），三类都是日期（日期本身是数据、引外部文档写读到的那一天、绕开外部工具缺陷写版本），这两行不是日期；它们也不像别处那几行「自证格名原文引用，按原文匹配」（`scripts/hosts/multi-host-run.sh:234`）有按原文匹配的消费者——开关说明那一行不是格名，样本说明没有判法逐字比它（推的，没找到读它的代码）；同一份文件下一行已经用不逐字的写法说同一类样本：「下一行是真讲过去的句子（词表里的时间词起头）」（`.claude/gate.d/doc-text.sh:979`）。改写例（推的）：第 659 行「时间词后面紧跟「面」就放过，不看前面是不是「所」」；第 978 行「两行时间词接「面临」「面向」的句子（样本 `run.sh` 第 3、4 行）」。

四句：不分辨臂；判法当时看得到前一个字，判别的东西看得到（「以」前面是不是「可」、后面是不是「为准」之类）；满足的是第一轮判决第 10 条改法的反面（收窄过头）；第一轮改法（只认「所」）在这两格上照红。改法（推的，被攻过零轮）：「以前面」前面紧挨「可」「难」「足」「得」「予」的也放过，「以前面…为准/为界/为基准」放过。

## 第 7 问 证红命令从仓根起、--copy 指副本

结论：推不出（轻）。变异表取副本，证的是副本里新写的行，不会证成主工作区那一份表里的行；判法脚本与重测试目标登记表取主工作区。

各样东西取自哪棵树：

- 判法脚本：主工作区。「MAIN_ROOT = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))」（`scripts/mutation/prove-red.sh:271`），行与判法「ROWS = load_module(os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"), "crates_mutation_rows_for_prove_red")」（`scripts/mutation/prove-red.sh:287`），重测试目标的判法「RELEASE_ONLY_TESTS = load_module(os.path.join(SCRIPT_DIRECTORY, "lib_release_only_tests.py"), "lib_release_only_tests_for_prove_red")」（`scripts/mutation/prove-red.sh:295`）。
- 单元闭包：两份。核副本拷全没有照主工作区那棵树算：「还要拷全：按 scripts/hosts/lib_repository_copy.py 的同一份闭包（主工作区那棵树、带测试输入；entries_missing_in_copy）」（`scripts/mutation/prove-red.sh:172`）；行键照副本算：「行键：点名的每一行照 crates-mutation-rows.py 的 KeyContext 在 --copy 那棵树上算」（`scripts/mutation/prove-red.sh:24`）。
- 变异表：副本。「rows, _malformed = ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))」（`scripts/mutation/prove-red.sh:706`）。主工作区与副本的表不同时副本说了算；同名的取副本表里最后一行。范围声明闸读树的次序也是副本在前（`.claude/hooks/lib_run_scope.py` 文件头「命令点明的树」那一条），两边一致。

打中：重测试目标的登记表读主工作区：「登记表读这份脚本所在主工作区那一份」（`scripts/mutation/prove-red.sh:186`）、「.claude/gate.d/stage-inputs.tsv，副本里不一定有」（`scripts/mutation/prove-red.sh:187`）。实现员新写的崩溃枚举用例登记在副本里（共用约束「写段的命令对着自己改动所在的副本（这一次的登记行已写在副本里）」（`.claude/agent-common.md:62`）），证红读不到它：这一行不补 `--release`，基线与变异都在 debug 下跑，行键照没补的参数算，与验证段在快照上（登记已在）算的键对不上、判定复用不上。修补之前从副本起会退 2、跑不起来（第一轮判决第 1 条），这一形是修补之后才走得到的。另一处少改：实现员定义第 3 步仍写「第 3a 步的 `scripts/mutation/prove-red.sh`、…同在这一份 `scripts/` 里」（`.claude/agents/implementation-writer.md:47`），而 3a 已改成「从主工作区的仓根起主工作区那一份证红脚本、`--copy` 指到副本」（`.claude/agents/implementation-writer.md:48`）。

最小输入（推的，要编要跑，没写成命令）：副本里新登记一个崩溃枚举用例目标 `crash_enumeration_copy_only` 并写一条点名它的变异行，主工作区不登，`task-server/task submit --cwd <仓根> -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=1 bash scripts/mutation/prove-red.sh --copy <副本> singlefs-verification <行名>`；预期输出里没有「cargo 参数补上 --release 起」那一行。这一条要真编，代价大，交主 agent 定要不要跑。

改法（推的，被攻过零轮）：重测试目标登记表先读 `--copy` 那一份、没有再退回主工作区（与范围声明闸同一个次序）；第 3 步那一句删掉 prove-red。四句：不分辨臂；脚本当时看得到副本里的登记表；满足的是第一轮判决第 1 条「证红命令照写」那一条修好之后的新一形；第一轮改法（`--cwd` 改仓根）就是它的来源。

## 没打中的形状

- 第 1 问：`mv` 草稿盖上去（照拒）、`cp` 进目录而文件名撞上、`install` 不带 `-b`、`dd of=`、`truncate`、经符号链接写（文件头写了、自证有格）、前面赋过值的变量拼的目标（照代入）、命令替换与通配拼的目标（工具员在写范围表里有行，⑫ 照算不出拒）。主 agent 直接改两个服务 `src/` 没有闸，是设计，不算打中。
- 第 2 问：表读不到、一份规格里两种都有两支说明照代码走得通；表里第二列写错的值不当 required，由表自己的判法报写坏。
- 第 4 问：派第二台、自证小仓、被杀留下的临时文件三格；`--out` 自己是指向别处的链接（改名换掉链接本身，自证有格）。
- 第 5 问：单线程时记号紧接 `test <名> ... ` 之后的那一形（第一轮的漏认）这一轮服务已认。
- 第 7 问：变异表、行键都取副本，范围声明闸读树也是副本在前，不会证成主工作区那一份表里的行。
- 第一轮只按线索记的节号指针：工具员写明没查的另 5 处（`.claude/gate.d/crash-point-replay.sh` 第 265、366 行，`scripts/hosts/run-with-memory-cap.sh` 第 103、112、306 行）现查了节标题：第 265 行指 2026-10-23 那份第一百二十九节、第 366 行指第九十七节，节标题与引的事对得上；第 103 行指 2026-09-16 那份第五十五、五十六节，那份里「五十六、」有两个同号标题（第 1125、1137 行），讲显存 OOM 的那一句在前一个里；第 112 行指第九十二节对得上；第 306 行「第四十节第 32 行」只核了节在，没核第 32 行指哪一行。

取样范围：第 1 问造了 22 条喂法（20 条 Bash、2 条 Write）；第 3 问的同名库扫了换上工具扫的五个目录全部 `.py`；第 5 问 8 种行形；第 6 问 6 句注释。

## 这条腿自己的限度

- 没写模型；除了草稿目录里 `cp`、`rsync` 两条语义演示（`cp -b`、`cp -r 源/.`、`rsync -a 源/` 盖不盖目标，量的）与两个只读 AST 扫描脚本，所有「退几」「认不认」都是照代码推的，等主 agent 跑。
- 第 7 问那一形要真编真跑，没写成可交的命令，只给了形状。
- 第 4 问在副本里起的那一形，要先建一份仓副本，命令写在下面，副本由主 agent 建、用完删。
- 第 3 问的同名库误报，扫描脚本是我写的近似（模块层名字的收法比换上工具的 `module_level_names` 窄）；真仓那一条以换上工具的输出为准。
- 没读工具员五份报告的全文（只 `grep` 过 `check-callers` 那几行），没读 `task-server/task_server_selftest.py`、两份 README、`.claude/gate.d/doc-process-records.sh`、`list_check.py` 与 `.claude/gate.d/break-switch-replay-files.tsv` 的五行。

## 要主 agent 跑的命令

探针文件（sha256，草稿目录 `/home/fy5090/.local/share/singlefs/work/governance-r2-attack/`）：

```
241028abe9e84e216522726ac9f426566d146d159cb738333949cfe38af80d0c  q1/feeds-bash.json
3f026f168256e4d854e130af2a118caeffbdc8e446daf4a89d08492e2e2d9e0e  q1/feeds-write.json
7e2971d6a572c08f0d4c0a37220313e5cc5b752c19588586cece77511a049ea7  q2/edit-gate-stage.json
da36341344357b8d99141e3f8529f64ec476238785ff670d89e3eb2ec438fbdf  q2/edit-required-script.json
eecac1fd3793afe1aafc7b318f46fcb9b3fc4bf3f5cca6990d1063b8a98e905d  q3/stem/scripts/other/caller_from_sibling.py
2507e0fcf6883717e4a17ce4030ff0ae50147c92c0864ce57dd8f47aa9acf6c2  q3/get/scripts/gate/caller_by_get.py
dc05a238b1f0e7edad49c3deea67054cccd744b7e2544a111ed861f63362408c  q3/fromsys/scripts/gate/caller_from_sys_modules.py
9bf70eef93033465d6244e31a47a6ed4d3a5b4ec32c48b9e6a243619f9853c9f  q5/lines.log
709aafc97d2e076c3663d2622112517f4ec6392c0ab4715bb073b38b9133f51e  q6/root/scripts/probe.sh
```

每项：命令；在哪起；看哪几行。

1. `task-server/task submit -- python3 scripts/gate/hook-swap.py --feed-only --hook .claude/hooks/bash-command-detector.sh --feeds /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q1/feeds-bash.json --root /home/fy5090/code/singlefs`
   在仓根起。看 `TASK_SERVER_RESULT` 的 `exit=1`；日志末行「✗ 只喂不换：.claude/hooks/bash-command-detector.sh 喂了 20 条，13 条不对」，不对的 13 行都是「打中」开头、「退 0（应当 2）」；「对照」与「两步里的后一步」7 行都是 ✓。
2. `task-server/task submit -- python3 scripts/gate/hook-swap.py --feed-only --hook .claude/hooks/write-guard.sh --feeds /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q1/feeds-write.json --root /home/fy5090/code/singlefs`
   在仓根起。看 `exit=0`、「喂了 2 条，0 条不对」（新建放行、盖已存在的拒，合上第 1 条里的 `rm` 就是先删再建）。
3. `task-server/task submit -- env CLAUDE_PROJECT_DIR=/home/fy5090/code/singlefs bash -c 'bash .claude/hooks/write-guard.sh < /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q2/edit-gate-stage.json'`
   在仓根起。看 `exit=2`；`task-server/task log <号>` 里说明那一段是「加 --verify '<被换脚本的 --selftest 命令>' 换上」（打中：门禁阶段的样本档要临时服务）。
4. `task-server/task submit -- env CLAUDE_PROJECT_DIR=/home/fy5090/code/singlefs bash -c 'bash .claude/hooks/write-guard.sh < /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q2/edit-required-script.json'`
   在仓根起。对照：看 `exit=2`、说明里是「不加 --verify 换上」那一段。
5. `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_directory.py`
   在仓根起。看 `exit=1`，✗ 行点名 `scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py` 引 `run_directory.device_stream_summaries`（误报：它运行时解到的是同目录那一份）。
6. `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers .claude/scripts/project_preflight.py --root /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q3/stem`
   在仓根起。看 `exit=1`、✗ 行「scripts/other/caller_from_sibling.py:1 引了 project_preflight.sibling_only」。
7. `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers .claude/scripts/project_preflight.py --root /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q3/get`
   在仓根起。看 `exit=0`、汇总行「缺 0 处，本次未判 0 处」（漏：`sys.modules.get` 取出的库引的名字不在库里）。
8. `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers .claude/scripts/project_preflight.py --root /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q3/fromsys`
   在仓根起。看 `exit=0`、「缺 0 处，本次未判 0 处」（漏：`from sys import modules`）。
9. `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers .claude/scripts/project_preflight.py --root /home/fy5090/.local/share/singlefs/work/governance-r1-fix-swap-tools/q2/root`
   在仓根起。对照：看 `exit=1`（下标取的那一形照点名）。
10. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/governance-r2-attack -- cat /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q5/lines.log`
    在草稿目录起。看 `exit=0`，再读下一条。
11. `task-server/task status <第 10 条的号>`
    看「这一趟有 3 条用例本次未跑」，点名 form_line_start、form_indented、form_after_result_start，没有 form_after_other_output、form_colored、form_words_between、form_tab_after。
12. `task-server/task submit -- python3 scripts/gate/lib_libtest_result_lines.py recordable /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q5/lines.log`
    在仓根起。看 `exit=0`、`marker` 开头的行 8 行（多出的 5 行是上一条服务不认的 4 种与 `CANNOT_RUN_ANYTHING_HERE`），末行 `RECORDABLE` 里 `markers=8`。
13. `task-server/task submit -- python3 scripts/gate/comment-history-check.py --list /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q6/root`
    在仓根起。看逐行列出的是 `scripts/probe.sh` 第 2、3、5、6 行，共 4 行；第 4 行（所以前面）不在。
14. `python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/copy --with scripts/gate/make-swap-spec.py=副本里起规格工具看工作根取不取得到`
    主 agent 在仓根直接跑（建副本，不经服务）。看末行写出了副本。
15. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/copy -- env SINGLEFS_MULTI_HOST_CONFIG=/home/fy5090/code/singlefs/multi-host.env python3 scripts/gate/make-swap-spec.py --base /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/base --new /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/new --out /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/spec-from-copy.json scripts/tool.py`
    在副本里起。看 `exit=3`、「✗ 取不到工作根」与出路「在多机配置（主工作树根的 multi-host.env）里写 SHARED_DATA_ROOT」。
16. `task-server/task submit -- python3 scripts/gate/make-swap-spec.py --base /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/base --new /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/new --out /home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/spec-from-main.json scripts/tool.py`
    在仓根起。对照：看 `exit=0`、「✓ 写出」一行。第 15 条跑完由主 agent 删第 14 条建的副本 `/home/fy5090/.local/share/singlefs/work/governance-r2-attack/q4/copy`。

## 只读到没攻到的

- 读了没攻的：`task-server/task_server_lib.py` 异常表那一行与 README 三语的 `--nocapture` 现状句（只确认在）；`.claude/agents/investigator.md:33`（与实现员定义同改，第 7 问的结论同样适用）；`.claude/gate.d/doc-process-records.sh` 三行叫法与 `list_check.py:416` 帮助串（没核改后的叫法对不对）。
- 没读的：工具员五份报告全文、`task-server/task_server_selftest.py`、两份 README、`.claude/gate.d/break-switch-replay-files.tsv` 那五行（第一轮攻过条数，这一轮不攻）。
- 辩方腿的第 8 到 11 问没做；禁读的辩方报告没读。

## 没做什么

- 没交任何任务、没跑项目代码；上面 16 条都等主 agent 跑，没跑出的读数都是推的。
- 不判正推、辩方那几格；不替主 agent 采纳；改法都是推的、被攻过零轮。
- 草稿目录里没有自己建的仓副本与编译目录（`du -sh` 草稿目录在交回前现量）；`semantics/` 演示目录已删；第 14 条的副本由主 agent 建、主 agent 删。
