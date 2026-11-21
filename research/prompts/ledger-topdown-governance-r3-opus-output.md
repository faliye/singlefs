# ledger-topdown-governance-r3 攻方腿报告（2026-11-18）

这条腿没往任务服务交任务，下面凡是钩子、脚本的判法结局都是读源码推的，标「推的」；要主 agent 跑的 9 条命令在末尾一节。标「量的」的只有三类：`grep` 与 `git show` 读到的源码行、主 agent 给了号的任务日志原句、我在草稿目录里照抄判据正则算的句子（那一类另注「照抄正则算的」，不等于跑了工具本身）。没写模型目录（这一轮的活是喂法与探针小树，没有计数模型），所以没有 `rerun.sh`；探针文件与 sha256：

| 文件（都在 `/home/fy5090/.local/share/singlefs/work/governance-r3-attack/` 下） | sha256 |
|---|---|
| `q1/feeds-detector.json`（23 条喂法） | bb3a7559ab977a840d310f502f46eceaa27c52d559f2743f053a27c7d32a2f32 |
| `q3/copy/multi-host.env`（仓副本根上手写的多机配置） | 1f2deed173f0aac12ab60f7f896ab45b6e5a08542ba8626dd33adfed90990c3e |
| `q4/qemu-comment-spec.json`（只改一行注释的脚本规格） | c6a32e109d9859478cab5289d91b3eda68c26768e5b3e399df36c81d58c8d87c |
| `q5/lines.log`（记号分歧样本日志） | b4cd4f559e2d4d1e4ec465f8fb082a71f21ad223e5af34319b16991be52ff760 |
| `q6/root/scripts/probe.sh`（注释历史探针） | 0eb02608a9521a862b422c9c2154f046029baf7ec8e155991ce81e7e32bdcb2b |
| `q4/tree/`（14 份 .py，换上工具认库的探针小树） | 逐份见草稿目录，`sha256sum $(find q4/tree -name '*.py')` 现算 |

## 各格判定一览

| 问 | 回 | 打中的是什么 | 证据的性质 | 命令 |
|---|---|---|---|---|
| 1 | 不成立（打中） | 15 种写法过得了 ⑭ 而真能删掉、挪走或在原 inode 上改服务源码；3 种只读或只写草稿的命令被「按目录判」误拒；跨台硬链接推不出 | 源码推的 | 1、2 |
| 2 | 成立 | 名单、文件头开关表、代码里读的名字三处都是 111 个，逐个对上；「里的项都认得」逐项判 | 量的（grep 源码） | 无 |
| 3 | 不成立（打中） | 照规矩建的副本带 `.git`，「没有 `.git`」认不出它；副本根手写一份多机配置就写得出规格，工作根随手写的值走，第一轮堵的口子换个摆法又开了 | 源码推的，20309 量的 | 3、4 |
| 4 | 不成立（打中） | e171 那 14 处本次未判里 7 处是运行时真解到被换那一份的引用；qemu 阶段的 heredoc 读法错，连带那一道没法经换上工具换；探针小树里 6 种写法既不核也不列 | 日志量的、源码推的 | 5、6 |
| 5 | 不成立（打中） | 一行两处记号服务只认一处；`NOT_RUN_` 一族别的原因服务不认；另三处认法各差几种形态，变异那一处会把本次未跑记成没红 | 源码推的 | 7、8 |
| 6 | 不成立（打中） | 「以 前X 为准」一类讲现状的 10 句判红；判法词表外的 11 句讲过去不判；`lag-growth-unjudged` 多判错那一格在换上前就有 | 照抄正则算的、`git show HEAD` 量的 | 9 |
| 7 | 不成立（打中） | 第 89 行「新建的文件…不判」写错；两行各少写一处 | 量的（逐句对源码） | 无 |

打中之后的四句（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）在这一题上多半落空：这一轮没有「臂」，判据是「修补做没做到判决那一条说的事」。逐问写在各节末尾一句：分不分辨臂一律答「不涉及臂」；系统当时看得到判别它的东西没有、满足的是哪一句、改法在打中的格上还中不中，各节分别写。

## 第 1 问：检出钩子第十四项

### 1a 过得了 ⑭、真能改主工作区服务源码的写法（推的，命令 1 量）

⑭ 认哪些写法，取决于 `overwrite_steps` 交出的步。重定向只认四个运算符：「OVERWRITING_REDIRECTS = {">", ">|", "&>", ">&"}」（`.claude/hooks/bash-command-detector.sh:3090`），`<>` 不在里面；`follow_simple_command` 只为 tee、sed / perl、rsync、cp / mv / install、ln、tar、patch、dd、truncate、rm、git、shell 几种命令名出步（`.claude/hooks/bash-command-detector.sh:3628` 到 `:3707`）。启动器里面那条命令的落点是占位词时，服务那一支交进算不出、不拒：「if has_unknown_piece and any(placeholder in word for placeholder in launcher_input_words()):」（`.claude/hooks/bash-command-detector.sh:2278`），而 `service_source_refusal` 把算不出那一份丢掉：「refused, _ = bash_scope_verdict(command, repository_root, start_directory or repository_root, [], service_view=ServiceSourceView(repository_root))」（`.claude/hooks/bash-command-detector.sh:2443`）；⑫ 那一支 `rm` 只是留住旧字节的一步、不是写（`.claude/hooks/bash-command-detector.sh:2236`），所以启动器里的 rm 两支都不拦。python 那一半认的是 open、write_text、shutil.copy*，加上换 inode 的写的目标：「path = python_literal_path(call_argument(node, 1, "dst"), assignments)」（`.claude/hooks/bash-command-detector.sh:4342`），`os.rename` 的源、`os.remove`、`os.open` 都不在里面。

喂法 `q1/feeds-detector.json` 里 15 条打中候选（`exit` 写 2，推的结局是退 0）：

| 写法 | 对服务源码做了什么 | 在工具员写明判不出的单里没有 |
|---|---|---|
| `unlink prune-server/src/protocol.rs` | 删掉，之后同名新建是整份换上（与 `rm` 同一件事，`rm` 拦了） | 不在单里 |
| `find prune-server/src -name protocol.rs -delete` | 删掉 | 不在单里 |
| `find … -exec rm {} +` | 删掉（启动器里的 rm，见上） | 不在单里 |
| `rsync -a --delete <空目录>/ prune-server/src/` | 删光 src/（整目录拷只比源里有的文件，`--delete` 删的是源里没有的） | 不在单里 |
| `rsync -a --remove-source-files prune-server/src/protocol.rs <草稿>/` | 拷出去再删源，等于 `mv` 挪走（shell 的 `mv` 挪走拦了） | 不在单里 |
| `gzip prune-server/src/protocol.rs` | 原文件换成 `.gz` | 不在单里 |
| `printf 'x' 1<> prune-server/src/protocol.rs` | 在原 inode 上从头改写、不截断：字面意义的「改到一半」 | 不在单里 |
| `shred -n 1 …`、`sort -o 同一文件 同一文件`、`gawk -i inplace …` | 原 inode 上整份改写或就地重写 | 不在单里 |
| `unzip -o <包> -d prune-server/src` | 逐份盖掉 | 在单里 |
| python `os.remove`、`os.rename(服务源码, 草稿)`、`os.open` 加 `os.write`、`shutil.copytree(…, dirs_exist_ok=True)` | 删、挪走、原 inode 上改、逐份盖 | `os.rename` 的源与 `os.open` 不在单里，另两种在 |

对照 5 条：`rm` 照拒、`tar -x` 解进草稿放行、`patch --dry-run` 放行、`git apply --check` 放行；工具员写明判不出的「源带变量的整目录拷」我推的是挡得住：落点算成 `prune-server/src/*`，`destination_words` 对源里算不出的段写 `*`（`.claude/hooks/bash-command-detector.sh:3860`），工具员在写范围表里有行，⑫ 照算不出拒。这一条要是喂出来退 0，就是我推错了。

`git checkout HEAD -- prune-server/src/<没改过的文件>` 也能整份换掉服务源码（⑱ 只拦路径上有没暂存改动的），我没写进喂法：结局取决于喂的那一刻暂存区与那份文件的状态，同一条喂法两次结局可能不同。

四句：系统当时看得到判别它的东西（命令文本里写着命令名与路径），满足的是判决第 1 条「第一轮修补要挡的…换这几种写法照样做得到」（`research/prompts/ledger-topdown-governance-r2-main-verification.md:53`）那一句；判决「三、改法」那一条（逐种加写法）在这 15 格上照样中——见第八节的撤回。

### 1b 「按往哪个目录里写判」的误拒（推的，命令 1 量）

工具员照定义走的那条路——`minimal-copy.py` 建副本、在副本里用 Edit 改、`git -C <副本> diff > <草稿>/x.patch`、命令写进报告——没有一步碰得到 ⑭（建副本、读、在仓外改都不判），这一半没打中。误拒落在另外三种常见命令上（工具员的 Bash 每次从仓根起，起点就是仓根）：

1. `tar -xOf <草稿>/包.tar 某文件`：只解到标准输出、一个字节不写盘。`tar_extraction_directory` 只看有没有 `-x`，不认 `-O` / `--to-stdout` / `--to-command`：「return directory if extracting else None」（`.claude/hooks/bash-command-detector.sh:3302`）；起点是仓根，`touches` 认仓根包着服务源码：「或它里面有已有的服务源码（仓根、prune-server/ 这类上级）」（`.claude/hooks/bash-command-detector.sh:2386`），拒。
2. `patch <草稿>/new/write-guard.sh <草稿>/fix.patch`（与 `patch -i <补丁> <文件>`）：补丁只改点名的那一份草稿文件，`patch_target` 不认位置参数里的原文件，目录照「.」算：「就地打补丁的目标是 -d / --directory 给的目录」（`.claude/hooks/bash-command-detector.sh:3305`），又是仓根，拒。工具员在草稿目录里给新版钩子打补丁正是这一形。

四句：系统看得到判别它的东西（`-O`、位置参数都在命令文本里），满足的是正文第 1 问「这条路上工具员自己有没有哪一步被新判法拒」——定义写的那条路上没有，常用的别的两形有；改法修的是 `tar_extraction_directory` 与 `patch_target` 两处，碰得到打中的格（推的）。

### 1c 按 inode 认硬链接，跨台判得对不对：推不出（命令 2 量）

同一台上判得对：本机仓根、工作根、`/tmp` 都在同一个 ext4 上（`stat -c %d` 三处都是 66306，量的），建得出硬链接，`(st_dev, st_ino)` 认得出：「return status.st_nlink > 1 and (status.st_dev, status.st_ino) in self.inodes()」（`.claude/hooks/bash-command-detector.sh:2371`）。跨文件系统建不出硬链接（EXDEV），没有对象可判。

推不出的是第二台：多机配置的共享根是三段（量的：只数了段数，没抄值），仓与工作根在第二台上是两次 NFS 挂载。本机建在两处之间的硬链接，第二台上看是同一个 fileid，但两次挂载的 `st_dev` 同不同取决于第二台的 NFS 客户端共不共用超级块，读源码判不了；`st_nlink` 在属性缓存的窗口里也可能是旧值。钩子跑在会话那一台；但喂法经任务服务会派到第二台：硬链接那一份喂法 20324 就是第二台跑的（量的：「第 1 次：第二台」），那一份的「仓根」与硬链接都在工作根里、同一次挂载，没碰到这一形。命令 2 读第二台上两处的设备号：两个数不同，就是「同一份喂法派到哪台结局不同」，违反共用约束「所有任务都要多机可用」。

## 第 2 问：检出钩子认得的开关名单

成立（量的）。三处各抽出来比（命令与中间文件在草稿目录 `code-names.txt`、`list-names.txt`、`header-long.txt`、`header-short.txt`）：`KNOWN_DETECTOR_BREAK_ITEMS`（`.claude/hooks/bash-command-detector.sh:1384` 到 `:1416`）111 个、不重复；文件头开关表带 `BASH_COMMAND_DETECTOR_BREAK=` 前缀的 91 个，加短写法 20 个，并起来 111 个；两份 `comm` 两头都空。代码里以字面量调 `detector_break_is_set("…")` 的 107 个，名单比它多的 4 个都另有读法：两个经常量 `GUARD_OFF_ITEM`、`GUARD_OFF_WITH_OTHERS_ITEM`（`.claude/hooks/bash-command-detector.sh:1427`、`:1428`），两个在 bash 那一层按 `,${BASH_COMMAND_DETECTOR_BREAK:-},` 认（`.claude/hooks/bash-command-detector.sh:1105`、`:8370`）。新加的六项都在名单里（`:1415`、`:1416`）。

「里的项都认得」那一格对逗号连着的几项逐项判：「.split(",") if item and item not in KNOWN_DETECTOR_BREAK_ITEMS]」（`.claude/hooks/bash-command-detector.sh:7346`），与读开关的 `detector_break_is_set` 同一种切法（`:1419`），带空格的项照认不得报，两处口径一样。

对不上的一处（对不上就当场说）：正文写「用短写法登记的 19 个」，我数出来是 20 个，与复跑登记行自己写的数相同：「文件头里另有二十行不带 BASH_COMMAND_DETECTOR_BREAK= 前缀的短写法开关，这一格读不到、没复跑」（`.claude/gate.d/break-switch-replay-files.tsv:75`）。这也说明 20351「复跑了 117 条开关…逐格相同 117 条」不含这 20 条，117 是 91 加三个共用库变量的 26 条，不是名单的 111 条。

## 第 3 问：规格工具认仓副本的判据

### 3a 今天在副本里起实际发生什么（20309 量的，命令 3、4 量剩下的）

判据是「仓根下没有 `.git`」：「if mode != "repository-copy-accepted" and not os.path.lexists(os.path.join(repository_root, ".git")):」（`scripts/gate/make-swap-spec.py:281`）；照规矩建的副本一律带 `.git`：「副本是 git 工作树：拷完副本根已是一棵 git 工作树的顶的不动，不是的（不在 git 里，或落在别的工作树里面）在副本根 git init」（`scripts/hosts/minimal-copy.py:28`）。我这一轮建的副本 `q3/copy` 打出「在副本根 git init 了（没有提交、索引是空的）」（量的，`q3/minimal-copy.txt` 末行）。自证那一格的小仓故意不带 `.git`：「root = sandbox_repository(os.path.join(work, f"copy-{number}"), configuration, git=False)」（`scripts/gate/make-swap-spec.py:602`），所以 20282 逐格相同而真副本照旧走不到新加的那一支。

过了这一关之后，工作根库照「这份库所在仓的主工作树根」找多机配置：「return os.path.join(main_worktree_root(REPOSITORY_ROOT), MULTI_HOST_CONFIGURATION_NAME)」（`.claude/hooks/lib_work_root.py:157`），`.git` 是目录就把这一层当主工作树根（`.claude/hooks/lib_work_root.py:134`）。副本里就是副本根，于是：

- 副本根上没有多机配置：退 3，说明是通用那一支（20309 的原句「多机配置 …/q4-copy/multi-host.env 不在」）。那一支的出路叫人「在多机配置（主工作树根的 multi-host.env）里写 SHARED_DATA_ROOT」（`scripts/gate/make-swap-spec.py:303`），在副本里照做，写的就是副本根上那一份——出路本身把人领到下一条。
- 副本根上有一份手写的多机配置：工作根取它的值，写得出规格（推的，命令 3）。`q3/copy/multi-host.env` 只两行，`NFS_SHARED_ROOTS` 与 `SHARED_DATA_ROOT` 都指草稿目录 `q3/` 下，共享根那一道核的是同一份配置里的两个值，自洽就过。命令 3 预期退 0、写出 `q3/fake-data/work/spec-in-copy.json`；命令 4 把 `--out` 换到真工作根里、手写工作根之外的 `q3/outside-forged/`，预期退 3、说明里「认的根」是 `…/q3/fake-data/work`——这一条证明管 `--out` 能写到哪的是副本里手写的那份，不是真工作根。

为什么这就是第一轮堵的口子：工作根一项取 realpath（「roots = [os.path.realpath(root) for root in work_roots]」，`scripts/gate/make-swap-spec.py:204`），手写的数据根下放一个指到仓里某一层的 `work` 符号链接，`--out` 就能落进仓里，`os.replace` 整份换掉那里的文件（推的，没跑：不在探针里往主工作区写）。而它是谁都能直接跑的文本工具：「"scripts/gate/make-swap-spec.py": "any",」（`.claude/hooks/lib_project_code_runs.py:1107`），副本里逐字节相同的那一份同样认（`.claude/hooks/lib_project_code_runs.py:1206`）；⑫ 只认五个定点写脚本的目标（`.claude/hooks/bash-command-detector.sh:1961`），不看它的 `--out`。写闸与 ⑫ 都看不见这一笔写。

### 3b 换成什么判据（都是推的、被攻过零轮）

| 判据 | 主工作区 | git worktree | minimal-copy 的副本 | 反例 |
|---|---|---|---|---|
| A：仓根的 HEAD 解得到一个提交（读 `.git/HEAD` 与 refs / packed-refs；worktree 照 gitdir、commondir 读） | 过 | 过（分离的 HEAD 是哈希，也过） | 拒（git init、没有提交） | 副本里 `git commit` 一次就被当成主工作区，手写配置照认（共用约束禁 git 写操作，没有钩子拦）；副本是用 `git worktree add` 建的那一种，本来就读主工作区的配置，过了也无害 |
| B：不认副本，改看 `--out` 落在哪：⑫ 把 `make-swap-spec.py` 的 `--out` 当定点写脚本的目标（`KNOWN_WRITE_SCRIPTS` 按文件名认，`.claude/hooks/bash-command-detector.sh:2047`，副本里那一份同样认），规格工具自己再加「`--out` 的 realpath 往上任何一层有 `.git` 就拒」 | 不受影响（草稿目录往上没有 `.git`：家目录、本地数据父目录、`share`、数据根四层量过都没有） | 不受影响 | 不必认出它 | 主 agent 没有写范围，⑫ 不判主 agent；主 agent 在副本里照手写配置起，仓外不在 git 里的文件（家目录下的点文件）挡不住 |
| C：`minimal-copy.py` 在副本里留一个记号，规格工具认记号 | 过 | 过 | 拒 | 记号删得掉；之前建的副本没有记号 |

我交出去的是 B（第八节写为什么撤回判决第 4 条那个方向）。A、C 都还要回答「副本里的东西谁都改得了」，B 不依赖副本长什么样。

四句：系统做决定那一刻看得到判别它的东西（副本根有 `.git` 但没有提交、`--out` 的 realpath），今天的判据看的那一样（有没有 `.git`）两种情形相同，是「判别子观测不到」那一形；满足的是判决第 4 条改法列里「判不过就改成「副本里起一律拒」」那一支写了而没生效；判决给的两个改法在打中的格上都还中（认 `SINGLEFS_MULTI_HOST_CONFIG` 那一支判决自己写了会重开口子；「一律拒」那一支被判据挡住）。

## 第 4 问：换上工具按调用方真解到的那一份认库

### 4a 三种写法各落在哪（探针小树 `q4/tree`，推的，命令 5 量）

被核的库是 `scripts/pkg/fake_target.py`（只有 `present_name`），另两份同名的 `scripts/pkg/sub/fake_target.py`、`scripts/decoydir/fake_target.py` 都有 `missing_name`；每个调用方都引 `missing_name`，运行时都解到被核的那一份。

| 调用方 | 写法 | 运行时解到 | 推的结局 |
|---|---|---|---|
| `pkg/caller_plain.py` | 同目录 `import fake_target` | 被核那一份 | 核，点名缺（对照） |
| `pkg/caller_dot.py` | `from .fake_target import missing_name` | 被核那一份 | 核，点名缺（对照） |
| `envpath/caller_environment_path.py` | `sys.path.insert(0, os.environ[…])` | 看环境 | 本次未判（对照，自证有这一格） |
| `viaimport/caller_through_imported_path.py` | 先 `import path_setter`，那一份往 `sys.path` 最前放 `scripts/pkg` | 被核那一份 | 本次未判（「都没有」那一句） |
| `decoydir/caller_shadowed.py` | 同上，但自己目录里有一份同名的 | 被核那一份（`path_setter` 插在自己目录前面） | 漏：照「自己所在的目录」解到同目录那一份，不是被核的库，不核也不报 |
| `pkg/sub/caller_relative.py` | `from .. import fake_target` | 被核那一份（上一层包） | 漏：同上，解到自己目录那一份 |
| `caller_dotted.py` | `import scripts.pkg.fake_target` 再 `scripts.pkg.fake_target.missing_name` | 被核那一份 | 漏：带点的 import 不带 as 不绑（`scripts/gate/hook-swap.py:1198` 要 `alias.asname or "." not in alias.name`） |
| `dynamic/caller_constructed_name.py` | `importlib.import_module("fake_" + "target")` | 被核那一份 | 漏：文本里没有库名，整份不读（「if not is_library and not any(stem in text for stem in stems):」，`scripts/gate/hook-swap.py:1618`） |
| `dynamic/caller_source_file_loader.py` | `SourceFileLoader("fake_target", 路径).load_module()` | 被核那一份 | 漏：`SourceFileLoader` 不是小写起头的 load，`load_module()` 的参数里没有库名（`LOADER_CALLEE`，`scripts/gate/hook-swap.py:710`） |
| `dynamic/caller_run_path.py` | `runpy.run_path(路径)["missing_name"]` | 被核那一份 | 漏：`run_path` 不算载入 |

所以三种写法的答案：`sys.path` 运行时拼出来的，本文件自己拼、折不出的列本次未判，**别的模块替它拼的**认不出——自己目录里没有同名的列本次未判，有就漏；经 `importlib` 按路径装的，名字与路径写死的核、路径折不出的列本次未判，**库名拼出来的、`SourceFileLoader`、`runpy` 漏**；包里相对导入，一层的核，**两层、带点的绝对写法漏**。漏的那几种在真仓里我只查了相对导入（0 处，量的：`grep` 五个扫描目录）与 `.claude/singlefs-ai-sop/scripts` 的同名文件（0 份，量的）；另几种在真仓里有没有，没查全。

### 4b 本次未判 11 处与 e171 那 14 处，该不该认

20271 那 14 处里 7 处是 e201 的调用方（量的，日志原句，理由两种：「认不出（它往 sys.path 放的目录有折不出的）」3 处、「认不出（它所在的目录与它往 sys.path 放的目录里都没有这个名字的文件）」4 处）。它们运行时解到的正是被换的 e171 修后版那一份：`whole_table_matrix.py` 先「import e171_research_side  # noqa: F401  （把 E171 研究一侧放进 import 路径）」（`scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/whole_table_matrix.py:13`），那一份「sys.path.insert(1, E171_DIRECTORY)」（`scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/e171_research_side.py:28`），再「from run_directory import load_run_directory」（`scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/whole_table_matrix.py:17`）。它们引的 `load_run_directory`、`TOUCH_STREAM_NAMES`、`StreamRun` 这些名字换上时一个都没核，20271 的「缺 0 处」不含它们。该认：仓里已有一份照这个认的判法，「import 进来的模块加的 sys.path（PythonModule.search_directories）」（`scripts/gate/lib_function_units.py:237`），换上工具没照它做——这正是 4a 那一形。

20269 的 11 处里另有两类该认：

- `.claude/gate.d/qemu-device-streams.sh` 那一处「python 解析不了（unexpected indent，第 1 行）」是读法的错，不是写法认不出。那一段起头是「python3 - "$VERDICT_LIBRARY_MODULE" "$unit_key_batch_file" <<'PY' \」（`.claude/gate.d/qemu-device-streams.sh:513`），下一行是续行的 `|| fail …`，heredoc 正文从续行结束之后才起；`embedded_python_segments` 从起头那一行的下一行就收正文（「body, cursor = [], index + 1」，`scripts/gate/hook-swap.py:459`）。于是那一段里 `import lib_verdict_library as library` 之后引的名字从来不核。同一个读法也喂给换上前的语法检查（「compile("\n" * start + body, f"{label}（内嵌 python 第 {number} 段）", "exec")」，`scripts/gate/hook-swap.py:486`）：任何一份含这一道的脚本规格都会在草稿里被判「编译不过」、一个字节不写（推的，命令 6 量）。写闸第十一道又不许工具员用 Write / Edit 直接改它——这一道门禁阶段今天没有能走的换法。
- `scripts/prune-server/prune_server_client_selftest.py:901` 那一处：「client_module = sys.modules[Client.__module__]」（`scripts/prune-server/prune_server_client_selftest.py:901`），`Client` 是 `from prune_server_client import (…)` 进来的（`:96`），键折得出就是 `prune_server_client`；取出来之后引的 `SEGMENT_BYTES`、`SEGMENT_INDEX_DIGITS`、`version_registry_key`、`segment_key` 都是被换库的真名字，换上客户端库时缺了也只报本次未判。

别的几处（`handback-guard.sh:2041`、`project-code-guard.sh:1769` 的元组存取，`run_replay_result_lines.py:128` 按循环变量取，`prune-server-load-simulation.py:66` 的通用载入函数里按形参取）认了收益小，不认我也站得住。从 10 涨到 11：我没拿到换上前那一趟的逐处清单，不判哪一处是新的（推不出）。

四句：系统看得到判别它的东西（`e171_research_side` 的插入写在被 import 的那一份里、在扫描目录里），满足的是文件头「认不出是哪一份的…列成「本次未判」，不当成查过」那一句的字面、却不满足判决第 3 条「按调用方真解得到的那一份」；判决给的改法（同目录优先、`sys.path` 插入的目录其次）碰不到「别的模块替它插」这一格。

## 第 5 问：记号行认法

### 5a 两边结局不同、其中一边错的日志（推的，命令 7、8 量）

`q5/lines.log` 只有两行记号：

```
NOT_RUN_NO_GPU_ADAPTER test=gpu_case_alpha reason=样本：一行两处记号的头一处 NOT_RUN_NO_GPU_ADAPTER test=gpu_case_beta reason=样本：同一行的第二处
NOT_RUN_OUT_OF_GPU_MEMORY test=gpu_case_gamma reason=样本：同一族的另一个原因
```

- 头一行：库逐处认，「starts = [found.start() for found in marker_form.finditer(visible)]」（`scripts/gate/lib_libtest_result_lines.py:233`），认两处、两条都不记绿；服务「行里任一处有 GPU_NOT_RUN_MARK 整词的是（一行只算一次、取头一处）」（`task-server/task_server.py:1126`），提醒里只点名 `gpu_case_alpha`、说 1 条。服务那一边错：`gpu_case_beta` 同样没跑，提醒少报一条。这一形要两条用例的输出挤在一行，crates 用 `println!` 整行打，今天少见；`--nocapture` 下几条同时跑时出得来（推的）。
- 第二行：库认（一族），服务不认（只认「GPU_NOT_RUN_MARK = b"NOT_RUN_NO_GPU_ADAPTER"」，`task-server/task_server.py:897`）。crates 那一侧写的约定是一族：「门禁认的 GPU 用例那一族记号」（`crates/singlefs-verification/src/crash_state_enumeration.rs:235`），今天 crates 实际只打 `NOT_RUN_NO_GPU_ADAPTER`（量的：`grep -rn` crates/ 下的字符串），所以这一行今天是潜在的；哪天加一个原因，服务那一边漏报。

命令 7 推的读数「RECORDABLE binaries=1 green=0 withheld=3 markers=3」；命令 8 推的提醒「这一趟有 1 条用例本次未跑（日志里 1 行带 NOT_RUN_NO_GPU_ADAPTER 记号）：gpu_case_alpha」。

### 5b 另三处认法与这两处差在哪（读源码，量的是源码行，后果是推的）

| 位置 | 写法 | 与库相比漏认的行 | 多认的行 | 差了会记错哪一条 |
|---|---|---|---|---|
| 变异：`scripts/mutation/crates-mutation-rows.py:1399` | 「NOT_RUN_LINE = re.compile(r"^\s*NOT_RUN_NO_GPU_ADAPTER test=(\S+)(?: reason=(.*))?$", re.M)」 | 行首之外的（`进度 42% NOT_RUN_NO_GPU_ADAPTER test=x …`）、带颜色码的、记号后接制表符的、一族的别的原因 | 无 | 点名的测试 ok 又打了这样一行时，`not_run_reason` 交 None（`:3582`），这一条记成「没红」写进判定库，盖掉行键原有的算数结论；文件头自己写的是「宁可记本次未跑，不记没红」（`scripts/mutation/crates-mutation-rows.py:3534`）。它固定带 `--show-output`，记号多在本段行首，所以多半认得出；漏的是用例自己没换行就打记号的那一形 |
| 入口：`scripts/crash/crash-point-replay.sh:834` | 先剥掉「test 名 ... 」起头（`:831`），再「not_run = sum(1 for line in printed if line.startswith("NOT_RUN_NO_GPU_ADAPTER"))」 | 带颜色码的、接在别的输出后面的、结果行起头与记号之间隔着字的、一族的别的原因 | `NOT_RUN_NO_GPU_ADAPTER_X …`、`NOT_RUN_NO_GPU_ADAPTER:` 这类更长的词（没有右边界） | 漏认时测试函数这一段不落「本次未跑」，照通过报（`functions_state` 那一支，`:876`） |
| 门禁 `crash-point-replay`：`.claude/gate.d/crash-point-replay.sh:3706` 与 `:3753` | 「while (match(rest, /NOT_RUN_[A-Z][A-Z0-9_]* test=/) > 0) {」与「while ((at = index(rest, "NOT_RUN_NO_GPU_ADAPTER ")) > 0) {」 | 记号后接制表符的、行尾光一个记号没有 `test=` 的 | `CANNOT_RUN_X test=…`（没有左边界；第二个函数另认 `XNOT_RUN_NO_GPU_ADAPTER …`） | 同一道阶段里记库走库的认法、报本次未跑走这两个 awk：制表符那一形库不记绿、awk 也不报本次未跑，那一条两头都没有；`CANNOT_RUN_X test=` 那一形库照记绿、awk 另报一条本次未跑 |
| 研究复跑门禁：`.claude/gate.d/research-build-and-replay.sh:972`（正文没点名，顺手查的） | 「if grep -Eq 'NOT_RUN_[A-Z_]+ test=' "$output_file"; then」 | 名字里带数字的（`NOT_RUN_GPU2 test=`）、接制表符的 | `CANNOT_RUN_X test=` | 这一格见到就整格判红，多认的那一形误红 |
| 广谱变异：`scripts/mutation/broad-mutation.py:2701` | 「if GPU_NO_ADAPTER_MARK in text and not break_is("gpu-not-run-as-missed"):」 | 一族的别的原因 | 整份日志里任一处子串（更长的词、说明文字里提到这个名字的都算） | 多认时整个 GPU 那一步记本次未跑、不落行 |

四句：系统看得到判别它的东西（都在日志行里）；满足的是判决第 5 条「两边要对到同一个认法」——这一轮只对了库与服务两处，而且没对到同一个（一族对一个、逐处对头一处）；判决改法写「两边各配同一份样本日志」，这份样本日志（库里的 `NOT_RUN_MARKER_SAMPLE_LOG`）没有一行两处记号、没有一族别的原因，碰不到 5a 两格。

## 第 6 问：注释历史新判据

判据：「BEFORE_PATTERN = BEFORE_BOUND + r"以前(?![缀者置]|面[那这的带])"」（`scripts/gate/comment-history-check.py:176`），`BEFORE_BOUND` 是「(?<![除乘所可])」（`:175`）。下面的命中与不中是我把这两行照抄进草稿里的 python 算的，不是跑工具；命令 9 拿工具本身跑 `q6/root/scripts/probe.sh`。

误红 10 句（讲现状，判红；`probe.sh` 第 13 到 22 行）：两份不同时以前一份为准；目录名以前 16 位十六进制为名；键以前两段拼成；以前导零补齐到 6 位；以前台进程组为准；以前提条件不成立为由拒；难以前一项推出后一项；借以前一份的结果做基准；取值以前端给的为准；以前半段为键、后半段为值。形态是「以 + 前X」作介词短语（前一份、前 N 位、前导、前台、前提、前端、前半段），或「以」属于前一个词而那个字不在除乘所可里（难以、借以）。真仓里这几形今天 0 行（量的：`grep -P` 扫射程目录，只命中判法脚本自己的词表与自证）。自证要它判红的「以前一次读两遍，现在只读一遍」（`scripts/gate/comment-history-check.py:947`）与「两份不同时以前一份为准」逐字只差「一次」「一份」，前后各一个字的判法分不开这两句——这是判据看的那几个字分不出两种情形，不是哪个例外漏写。

漏判 11 句（讲过去，不判；第 2 到 12 行）：先前、当初、从前、早先、单字「曾」、旧版、上一版、不带「了」的「改成」、英文的 Previously、used to，加一句「许可以前只给主 agent 跑」（与已知的「认可以前」同一形，换了个词）。前 10 句在词表之外（词表在 `scripts/gate/comment-history-check.py:38`），英文注释整类没有词。对照第 23 行「以前这里只认 --bin」判红。

`lag-growth-unjudged` 多判错 `comment-history-lag-green`：原来就有，不是这一轮的修补带进来的（量的，源码依据）。开关下判法交的说明「弄坏开关 lag-growth-unjudged：没与 gate 基准比」（`scripts/gate/comment-history-check.py:631`）会嵌进成功句，而绿样本要的是「与 gate 基准 HEAD 那一版比没涨（基准 2 行，此刻 1 行）」（`.claude/gate.d/fixtures/doc-text.sh/comment-history-lag-green/expect:3`），开关一开这一行就对不上；文件头开关表只点名红样本（`.claude/gate.d/doc-text.sh:668`）。换上前那一版一样：

```
$ git show HEAD:scripts/gate/comment-history-check.py | grep -n '没与 gate 基准比'
599:        return [], "弄坏开关 lag-growth-unjudged：没与 gate 基准比", None, ""
$ git show HEAD:.claude/gate.d/doc-text.sh | grep -n 'lag-growth-unjudged' -A1   （只摘这一段）
658:#   COMMENT_HISTORY_CHECK_BREAK=lag-growth-unjudged  comment-history 的滞后表不与 gate 基准那一版比（滞后表涨了也不红）
659-#     comment-history-lag-growth-red
$ git log --format='%h %ad' --date=short -S'没与 gate 基准比' -- scripts/gate/comment-history-check.py | tail -3
d4bcd2e20 2026-11-07
```

绿样本的 expect 这一轮没动（`git diff --stat HEAD` 不列它），HEAD 那一份第 3 行与今天逐字相同。

四句：误红那 10 句里判据看的「以前」前后各一字，两种情形逐字相同（判别子观测不到）；漏判那 11 句满足的是共用约束「不写改动经过」那一句，不是判法写的形态——判法从来只拦词表，打中的是判法的射程；判决改法（正面认时间词：前面是句首、标点或空白，或后面接是、的、只、曾、这）对误红那一类碰得到一部分：前面是别的字的六句（时以前一份、名以前 16 位、键以前两段、难以前、借以前、值以前端）放过，句首起的四句（以前导零、以前台、以前提、以前半段）照判红（推的，按判决那一句的字面算）；对漏判那一类碰不到。

## 第 7 问：共用约束第 89、101 行与两份钩子逐项对

| 行 | 哪一处 | 共用约束写的 | 钩子写的 | 判 |
|---|---|---|---|---|
| 89 | 新建的文件 | 「新建的文件、草稿目录与仓副本里的同名文件、主 agent 与别的 agent 类型不判」（`.claude/agent-common.md:89`） | 解包与打补丁「只新建文件的也拒」（`.claude/hooks/bash-command-detector.sh:315`）；`ln` / `cp -l` 从服务源码建出的新文件也拒（`:302`）；草稿目录里与服务源码是同一个文件的硬链接照认（「落在仓外的草稿目录里的硬链接也认」，`:303`） | 写错：新建、草稿目录里的同名文件都有拒的情形 |
| 89 | 认不出类型的子 agent | 没写 | 「认不出 agent 类型的子 agent 调用（带 agent_id 而没有 agent_type；⑫ 同样不判）」（`.claude/hooks/bash-command-detector.sh:312`） | 少写 |
| 89 | 判不到的 | 「写法认得与 ⑫ 一样多，另认…」，读起来是一张全单 | 「判不到的（这一项不罩它们）」那一串（`.claude/hooks/bash-command-detector.sh:313` 到 `:316`），落点算不出的这一道不判、靠 ⑫ | 少写；第 1 问那 15 种不在两边任何一张单上 |
| 101 | Bash 里改脚本 | 「Bash 里往后两类已存在的文件写」（`.claude/agent-common.md:101`）由 ⑭ 拒，第一类（脚本）在 Bash 里怎样没说 | 「Bash 里用定点替换工具分几次改同一份脚本，这道闸看不见。」（`.claude/hooks/write-guard.sh:102`） | 少写：工具员用 `replace-once.py` 分几次改主工作区的脚本，两道都不拦（⑫ 照写范围放行，⑦ 不拦换 inode 的写），正是第十一道要防的中间态 |
| 101 | 别的 | 目录、后缀、执行位、`target/` 与 `fixtures/` 例外、三种出路 | 文件头第十一道（`.claude/hooks/write-guard.sh:77` 到 `:101`） | 逐项对上，没有多写 |

四句：不涉及判据分辨，打中的是说明文字与钩子行为对不上；改法是照钩子改那两行，碰得到。

## 第八节：撤回与改法

撤回三处先前给的方向（撤回的理由都是这一轮现查到的，不是印象）：

1. **判决第 1 条的改法（第二轮攻方腿提的，逐种把写法加进 ⑭）不该再当主修法。** 第二轮补了 13 种，这一轮照同一个办法又找到 15 种（1a）。`follow_simple_command` 按命令名出步，Bash 里能改文件的命令没有闭集，这条路每一轮都还会漏。逐种补可以照做，但它挡的只是第一轮写的那件事的一部分。
2. **判决第 4 条两个方案都在「认出副本」上做文章，我撤回这个方向。** 认多机配置那一支判决自己写了会重开口子；「副本里一律拒」那一支要认出副本，而副本是谁都改得了的一棵树（3b 表里 A、C 两行的反例）。要守的是「`--out` 落不进仓里」，按落点判（3b 的 B）不用认副本。
3. **判决第 5 条「两边各改到这一条、各配同一份样本日志」我判写法本身留了口子。** 两处各写一份正则，这一轮就对成了一族对一个、逐处对头一处；另有四处各写一份（5b）。改成一处定义、几处导入：库里的 `NOT_RUN_MARKER_FORM` 与逐处切分是唯一一份，服务按路径导入它（服务的自证已经这样导入样本日志），shell 与 awk 那几处改调库的子命令；认一族还是认一个，在那一处定一次。

改法逐格（全部只在我的推理上、没在模型上量过、被攻过零轮；「推的，待主 agent 跑」指要跑的命令在下一节）：

| 改法 | 修哪几格 | 标 |
|---|---|---|
| ⑭ 另认 unlink、find -delete 与启动器里的 rm（落点取 find 的起点目录按 holding 判）、rsync 的 --delete 与 --remove-source-files、不带 -c / -k 的 gzip 一类、`<>` 重定向、shred、sort -o、gawk -i inplace，python 的 os.remove / unlink / shutil.rmtree / os.rename 的源 / 带写标志的 os.open | 1a 的 15 格里这几种（unzip、copytree 不在内） | 推的，待主 agent 跑（命令 1 是改前的读数） |
| 不在命令层拦：服务只从登记过内容哈希的快照编，主工作区里改到一半的源码编不进服务 | 1a 全部 15 格 | 推的；服务现在从哪里编、换编法要动多少，没核 |
| `tar_extraction_directory` 认 -O、--to-stdout、--to-command（不写盘，交 None）；`patch_target` 有位置参数的原文件时照那一份文件判 | 1b 3 格 | 推的，待主 agent 跑（命令 1） |
| ⑫ 把 make-swap-spec.py 的 --out 当定点写脚本的目标；规格工具加「--out 的 realpath 往上有 .git 就拒」 | 3a 两格 | 推的，待主 agent 跑（命令 3、4 是改前的读数） |
| 换上工具跟模块级 import 进来的仓内模块往 sys.path 加的目录（照 `scripts/gate/lib_function_units.py` 的 search_directories） | 4b 那 7 处、4a 的 caller_shadowed 与 caller_through_imported_path | 推的，待主 agent 跑（命令 5） |
| `embedded_python_segments` 起头那一行以 `\` 收尾时，正文从续行结束的下一行起 | qemu 那一处本次未判与那一道换不了 | 推的，待主 agent 跑（命令 6） |
| `X.__module__` 里的 X 是 from 目标库 import 进来的，键折成那一份库 | `scripts/prune-server/prune_server_client_selftest.py:901` | 推的 |
| 文本里提到库名、却一处载入与绑定都没认出的调用方，列本次未判 | 4a 的 SourceFileLoader、runpy、带点 import 三格（拼库名那一格文本里没有库名，碰不到） | 推的 |
| 服务逐处认、记号认法一处定义几处导入（撤回第 3 条） | 5a 两格、5b 五处 | 推的，待主 agent 跑（命令 7、8） |
| 注释历史照判决改法正面认时间词，另补先前、当初、从前、早先、旧版、上一版与几个英文词 | 第 6 问误红 6 句、漏判 10 句（「许可以前」那一句仍漏） | 推的，补词在真仓上带出多少新红没量 |
| doc-text 文件头开关表给 lag-growth-unjudged 补点名 comment-history-lag-green，或让开关下的成功句不嵌那一句 | 第 6 问那一处 | 推的 |
| 共用约束第 89、101 行照第 7 问那张表改 | 第 7 问 4 格 | 推的 |

## 要主 agent 跑的命令

都是只读或只写我的草稿目录；第 6 条带 `--dry-run`，停在草稿里验，主工作区一个字节不写。

1. `task-server/task submit -- python3 scripts/gate/hook-swap.py --feed-only --hook .claude/hooks/bash-command-detector.sh --feeds /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q1/feeds-detector.json --root /home/fy5090/code/singlefs`
   - 在仓根起。看 `task-server/task log <号>` 的「喂了 23 条，N 条不对」：推的 N 是 18，其中 15 条「退 0（应当 2）」是 1a，3 条「退 2（应当 0）」是 1b；5 条对照该对。「源带命令替换的整目录拷」那一条要是报退 0，是我推错了（⑫ 没挡住，那就是第 16 种）。
2. `task-server/task submit -- stat -c '%d %i %n' /home/fy5090/code/singlefs/prune-server/src/lib.rs /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q1/src/protocol.rs`
   - 在仓根起。先看 `task-server/task status <号>` 的「第 1 次：」落在哪一台：落在本机的这一次不算数；落在第二台时，两行开头的设备号不同就是 1c 打中，相同就是 1c 不成立。
3. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/copy -- python3 scripts/gate/make-swap-spec.py --base /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/base --new /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/new --out /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/fake-data/work/spec-in-copy.json scripts/tool.py`
   - 在副本 `/home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/copy` 起（`.deliverables` 写成交接）。`TASK_SERVER_RESULT` 的 `exit=0`、日志有「✓ 写出」就是 3a 打中；`exit=3` 而说明点名「仓副本」是我推错了。
4. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/copy -- python3 scripts/gate/make-swap-spec.py --base /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/base --new /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/new --out /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/outside-forged/spec.json scripts/tool.py`
   - 在同一个副本起。`exit=3`、说明「不在工作根下（认的根：/home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/fake-data/work）」就是打中：管 `--out` 的是副本里手写的那份。
5. `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers scripts/pkg/fake_target.py --root /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q4/tree`
   - 在仓根起。推的是退 1：缺名字的点名 `scripts/pkg/caller_plain.py` 与 `scripts/pkg/caller_dot.py`；本次未判列 `caller_through_imported_path.py` 与 `caller_environment_path.py`；`caller_shadowed.py`、`caller_relative.py`、`caller_dotted.py`、`caller_constructed_name.py`、`caller_source_file_loader.py`、`caller_run_path.py` 六份在缺与本次未判两处都不出现，出现一份就少一处打中。
6. `task-server/task submit -- python3 scripts/gate/hook-swap.py /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q4/qemu-comment-spec.json --scratch /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q4/swap-scratch --dry-run`
   - 在仓根起。推的是退非 0、日志有「.claude/gate.d/qemu-device-streams.sh：内嵌 python 第 … 段（第 513 行起）编译不过」；退 0 是我推错了。规格是手写的一处（只改一行注释，旧串在那份文件里恰好一处，量的 `grep -c`），不是工具员的正式规格。
7. `task-server/task submit -- python3 scripts/gate/lib_libtest_result_lines.py recordable /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q5/lines.log`
   - 在仓根起。看 `RECORDABLE` 那一行：推的是 `markers=3`、`withheld=3`。
8. `task-server/task submit -- cat /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q5/lines.log`
   - 在仓根起。看服务推的结局说明（形同第二轮任务 20186 那一句）：「这一趟有 1 条用例本次未跑…：gpu_case_alpha」就是 5a 打中（该是 3 条或至少 2 条）。
9. `task-server/task submit -- python3 scripts/gate/comment-history-check.py --list /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q6/root`
   - 在仓根起。推的命中行是 `scripts/probe.sh` 第 13 到 23 行共 11 行，第 2 到 12 行一行都不在；第 13 到 22 行里每命中一行是一句误红，第 2 到 12 行里每不中一行是一句漏判。

## 没打中的形状

- 第 1 问：工具员照定义走的那条路（建副本、在副本里 Edit、`git -C <副本> diff` 出补丁、命令交主 agent）每一步都不碰 ⑭；`git apply --check`、`patch --dry-run`、`tar -x -C <草稿>` 放行；源带命令替换的整目录拷我推的是被 ⑫ 挡下；同一台上建硬链接、往硬链接写都认得出。取样范围：`follow_simple_command` 的命令名分支逐个对过，Bash 里别的改文件的命令我按「常见的 coreutils、压缩、编辑器、python 标准库」想了一遍，不是穷举；第二轮那 13 种读源码都进了认法，没有逐条重喂。
- 第 2 问：三处名单、逗号切分、带空格的项，都对上。
- 第 3 问：主工作区与 git worktree 里起，读源码判照常取主工作区那一份多机配置（worktree 的 `.git` 文件照 gitdir、commondir 认到主工作区）。
- 第 4 问：`.claude/singlefs-ai-sop/scripts` 与五个扫描目录同名的 .py 0 份、真仓里的相对导入 0 处（都是量的）；元组存取、循环变量键、通用载入函数里的形参键那几处本次未判，判它们认不认我不打中。
- 第 5 问：库与服务在颜色码、制表符、左右词界上两份正则同形（读源码逐字对过，`scripts/gate/lib_libtest_result_lines.py:115` 与 `task-server/task_server.py:898`），这几种行两边结局相同。
- 第 6 问：真仓里「以 + 前X」一类讲现状的句子今天 0 行，872 行命中里不会有这一类新红（量的 `grep -P`，只扫了射程目录、没跑判法）。
- 第 7 问：第 101 行的目录、后缀、执行位、例外与出路逐项对上。

## 这条腿自己的限度

- 一条命令都没交，钩子与脚本的结局全是读源码推的；会推错的地方在各命令下写了「推错了是什么样」。
- 读到的范围（只有我这条腿读到的话，以这里为准）：`.claude/hooks/bash-command-detector.sh` 的文件头 ⑦ ⑪ ⑫ ⑭ 与开关表，代码 1380–1430、2160–2460、3086–3360、3586–3870、4230–4346 几段；`.claude/hooks/write-guard.sh` 文件头第十一道；`scripts/gate/hook-swap.py` 的文件头、450–500、686–715、1104–1213、1460–1716；`scripts/gate/make-swap-spec.py` 的文件头、196–305、496 与 602、700–760；`.claude/hooks/lib_work_root.py` 128–237；`task-server/task_server.py` 896–905、1125–1150；`scripts/gate/lib_libtest_result_lines.py` 40–50、110–130、226–240；`scripts/gate/comment-history-check.py` 的文件头、165–186、600–650；`.claude/gate.d/doc-text.sh` 660–670；`.claude/agent-common.md` 第 89、101 行；`.claude/agents/tooling-writer.md` 第 3 步；复跑登记表第 75、91 行。另读了正文没点名的 `scripts/hosts/minimal-copy.py`、`.claude/hooks/lib_project_code_runs.py`、另三处认记号的脚本与 e201 的两份文件。
- 没读：`task-server/task_server_lib.py`、`task-server/task_server_selftest.py`、`task-server/README.md`、`.claude/gate.d/doc-process-records.sh`、`.claude/agents/implementation-writer.md`（攻方腿七问没问到它们），五份工具员报告（`governance-r2-fix-*/result.md`）也没读，「工具员写明判不出的」取自正文与钩子文件头。
- 第二台的 NFS 行为（1c）、服务现在从哪里编（第八节第二行改法的前提）没核。
- 20269 的 11 处里哪一处是这一轮新出的，我没拿到换上前那一趟的逐处清单，推不出。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数；没往统一任务服务交任务、判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没写模型目录与 `rerun.sh`：这一轮没有计数模型，只有喂法与探针小树；没调崩溃枚举（不涉及 `crates/`）。

## 清理

- 建了一份仓副本 `/home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/copy`（`du -sh` 105M），命令 3、4 要在里面跑，在草稿目录的 `.deliverables` 写成交接。
- 没删 /home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/copy：交接给主 agent 跑命令 3、4，跑完由主 agent 删。
- 没建编译目录。草稿目录里另有喂法、探针小树与中间文件（`q1/`、`q4/`、`q5/`、`q6/`、名单对照的几份 .txt、两份任务日志拷贝），都是普通文件。
