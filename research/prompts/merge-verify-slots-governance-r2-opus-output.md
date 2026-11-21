# 合入后验证槽位治理第二轮：攻方腿报告（2026-11-21）

被判对象是开工快照点名的三份脚本；仓里此刻这三份与冻结副本 `~/.local/share/singlefs/work/merge-verify-slots-r2-verdict/frozen/` 逐字节相同（sha256 与 `research/prompts/merge-verify-slots-governance-r2-snapshot/judged-sha256.txt` 前三行相同，现核），下面引的行号都是这一版的。模型的数一条都没跑过，全部是「推的，待主 agent 跑」。

## 复跑命令与文件 sha256

复跑（主 agent 交，命令也写在末尾「要主 agent 跑的命令」）：

`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-verdict/frozen /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-opus/rerun-run /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-opus-model`

| 文件（模型目录 `research/prompts/merge-verify-slots-governance-r2-opus-model/` 下） | sha256 |
|---|---|
| `extract_program.py` | e41359e1a44f016796cf8d74da95b8970b28e6164ca961fca20cbbde79cb8e3f |
| `slot_model_lib.sh` | 362cdcf91b590f6f7e59ec561b4e724677b5432a15f0b221f57e81b2fd762b5b |
| `slot_trim_race.py` | 3f3e581271cdd6ec9590ef6be37afaaa859b09431a863e197a55ccc471a906e8 |
| `work_root_gap.sh` | 5a33e352db7eb98b69cfa36f971f654bed61d5a5cc8146dc270c480430a25c7d |
| `incremental_holder.sh` | 90a0bb8c07e5e492d471bbe480e506f976a315581f1deff992a017cebb5c1467 |
| `copy_exclusions.sh` | 77bc3ecc554294c2531f42e083e45619b2954df63e75746f7f23ec0764bc3b1e |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps` 那一段） | a43afeac3860c0a46dc18869527ee73e3978e76932643751b11303c57022bee0 |
| `tools/rerun-from-a-clean-work-directory.sh` | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |

- 四份模型都只读冻结副本里的 `merge-and-verify.sh`（取挑槽那段 python 原文）与 `merge-and-verify-snapshot-lib.sh`（source 进来、函数原样调），仓里只读 `.claude/hooks/lib_work_root.py`、`scripts/hosts/run_build_directory.py` 与 `git ls-files`；工作根、槽那一层、编译根全是模型在 `--work` 下自己建的，不碰真的工作根与编译根，`slot.4u3rewvh` 一概不碰。
- 唯一换掉的冻结函数：`work_root_gap.sh` 里的 `slot_build_root` 换成打替身编译根的一行（真函数在没给 `MERGE_AND_VERIFY_SLOT_BUILD_ROOT` 时读多机配置，读到的就是这一台真的编译根）。
- 样本进程（占锁的、开着文件的、挑槽进程的父进程 `sleep`、挑槽进程本身）都用新会话起，收尾只对各自的组发 TERM（Python 一侧 5 秒后 KILL），再到 `/proc` 里核组里没人，留下的打 `group-left` / `name=model_group_left`。
- 探针的代价（推的）：内存峰值几十 MiB（同一时刻至多十来个小 python 进程）；挂钟约 5 分钟，大头是 `slot_trim_race.py` 的 210 个场景（120 个不弄坏、三个弄坏开关各 30 个，每个约 1–1.5 秒：挑槽进程每半秒核一次父进程、占的时长取 0–0.5 秒）；它先跑头 5 个场景、打 `name=slot_trim_race_estimate`，估出来超过 40 分钟就不跑全量、退 3。
- 这一轮被判的是脚本，不涉崩溃状态枚举：第 3b 步关于 `enumerate_crash_states` 的取样与状态数上限不适用，一个崩溃状态都没枚举。

## 各格判定一览

| 格 | 判定 | 依据（文件:行） | 模型 / 读数 |
|---|---|---|---|
| 1-① 已有 3 槽、上限 1 只占前 1 槽 | 成立（改法拦得住） | `scripts/gate/merge-and-verify.sh:1271`、`:1273`、`:1283` | `slot_trim_race.py` 的 surplus-taken、single-picker-choice、slot-count-over 三格，推的，待主 agent 跑 |
| 1-② `slots=0` 清多出来的槽、不碰留着与占着的 | 成立 | `scripts/gate/merge-and-verify.sh:1248-1261`、`scripts/gate/merge-and-verify-snapshot-lib.sh:406-418` | 同上，kept-removed、external-held-removed、surplus-free-left 三格，推的 |
| 1-③ 清槽与另一趟同时挑槽，会不会删掉别的趟刚占的槽 | 不成立（删不到） | `scripts/gate/merge-and-verify.sh:1268`、`:1297`；全部 `lock_slot` 调用都在 `slots.lock` 之内 | 同上，held-slot-vanished、double-hold 两格；模型自己会不会红由三个弄坏开关臂证，推的 |
| 1-④ 「三样一起给」漏给会删到哪 | 成立（新毛病，低频，只费编译、不错判） | `scripts/gate/merge-and-verify-snapshot-lib.sh:400-402`、`:203`、`:312-313`、`:433` | `work_root_gap.sh` 五条臂，推的 |
| 1-④ 自证里别的格会不会因此去删真编译根下的槽编译目录 | 不成立 | `scripts/gate/merge-and-verify-selftest-lib.sh:27`；没有一格 `env -u` 或 `unset` 这三样 | 读码，命令见下 |
| 1-⑤ 拷进槽前删 `target` 一类会不会删到源树里本来叫 target 的目录 | 不成立（只删槽里那一份；源树里叫这几个名字的目录两种快照本来都不拷） | `scripts/gate/merge-and-verify-snapshot-lib.sh:268`、`:276-277`、`:42` | `copy_exclusions.sh`，推的；仓里跟踪的路径里有这三种目录名的 0 份（量的，命令见下） |
| 1-⑥ 占槽、不占槽两种 cargo 收到的环境 | 成立（对），另记三处代价、一处推不出 | `scripts/gate/merge-and-verify-snapshot-lib.sh:367`、`:371`、`:437`；`scripts/gate/merge-and-verify.sh:2636`、`:2671` | `incremental_holder.sh` 四条臂，推的 |
| 1-⑦ 删 incremental 时有进程开着的出路 | 成立（不删、打一行 !、照往下编）；嵌进去那几行的说法指错对象 | `scripts/gate/merge-and-verify-snapshot-lib.sh:380`、`scripts/hosts/run_build_directory.py:109` | 同上 unset 臂，推的 |
| 2-① 只在本机占槽、`MULTI_HOST_RUN_PEER_SCRATCH` 认法 | 成立（认得对；认错也只换说法、照样不占） | `scripts/gate/merge-and-verify-snapshot-lib.sh:393-396`、`scripts/hosts/multi-host-run.sh:1419` | 读码 |
| 2-② 为旧槽布局、旧放槽命令留的分支 | 不成立（没找到）；槽位汇总读 `kept` 的修改时刻那一支是宽读法，不是认旧版 | `scripts/gate/merge-and-verify-snapshot-lib.sh:344` | 读码、读开工前草稿基线 |
| 「已经量的」逐句核 | 十二句里十句核到、两句只核到窄的一句（见「已经量的逐句核」） | 服务状态行与日志行原文 | 量的 |

唯一的打中是 1-④ 的生产那一半。打中之后的四句（`.claude/rules/evidence-discipline.md:125` 那一条）：

- 分不分辨臂：分辨。现行代码那条臂删掉另一个工作根里四槽中三槽的编译目录，两条改法臂一份都不删（模型臂 gap-take / gap-zero 对 fix-namespace / fix-same-root，推的）。
- 系统当时看不看得到判别它的东西：现行代码看不到。`choose_slot` 手里有工作根的值，但编译根下 `merge-and-verify-slots/<槽名>/` 那一层不记它属于哪一个槽那一层，`sweep_slot_targets` 只能拿「这一个槽那一层里有没有同名的槽」判（`:312`），分不出「槽被删了」与「槽在别的工作根里」。
- 满足的是判据字面的哪一个分句：正文判据格第 1 格第三项「「三样一起给」那一道漏了会怎样」。说的是那一道没盖住的输入（只给 `SINGLEFS_WORK_ROOT`），不是那一道被删掉之后的样子。
- 跑前条款给的改法在打中的格上还中不中：正文没给跑前改法。我提的两条改法见 1-④ 一节，都标「只在我的模型上量过、被攻过零轮」，而且模型也还没跑。

## 第 1 格：第一轮改法的修补

### 1-① 与 1-②：只占前 slots 槽、slots=0 清多出来的槽

成立。挑槽那段 python 先按 `used` 从新到旧排，再切成两份：「    eligible, surplus = (order, []) if broken == "slot-surplus-taken" else (order[:limit], order[limit:])」（`scripts/gate/merge-and-verify.sh:1271`）；同一任务上一次尝试的那一槽只在 `eligible` 里才提前：「    if preferred in eligible and broken != "earlier-attempt-slot-not-preferred":」（`scripts/gate/merge-and-verify.sh:1273`）；新开一槽只看已有槽数：「        if held is None and (len(slots) < limit or broken == "slot-limit-ignored"):」（`scripts/gate/merge-and-verify.sh:1283`）。排在后面的槽一槽都进不了 `eligible`：别的趟占了一槽只会把那一槽的 `used` 拨到最新、在前 slots 槽里换位置，新开只在槽数小于上限时发生、不会把已有的槽挤出前 slots 槽。所以「开场时排在前 slots 槽之后的槽，之后谁都占不到」是这份代码的不变量，模型的 surplus-taken 一格就按它判。

`slots=0` 那一趟：`choose_slot` 在槽那一层里有槽时起挑槽进程的 trim（`scripts/gate/merge-and-verify-snapshot-lib.sh:412-413`），挑槽进程里 `eligible` 是空的，已有的槽全在 `surplus`；`trim_surplus` 先非阻塞拿那一槽的锁、拿不到就跳过（别的趟占着），拿到了放掉、再判留着：「    if is_kept(slot) and broken != "slot-surplus-kept-removed":」（`scripts/gate/merge-and-verify.sh:1258`），两样都过才「        shutil.rmtree(slot)」（`scripts/gate/merge-and-verify.sh:1261`）。`slot.4u3rewvh` 此刻 `kept` 与 `repo/` 都在（`ls -la` 现查，只读），落在 surplus 里也只打一行 `!`、不删。

模型 `slot_trim_race.py` 把这两项做成会红的量：120 个随机场景（0–5 槽，各槽 `used` 的修改时刻互不相同；约三成留着、约三成由样本进程拿着锁；另有 kept 在而 `repo/` 不在、不算留着的那一种；slots 取 0–4；1–4 个挑槽进程错开几十毫秒同时起），判 surplus-taken、surplus-free-left、kept-removed、external-held-removed、slot-count-over，只起一个挑槽进程的场景另判 single-picker-choice（占的必须是前 slots 槽里空着、`used` 最新的那一槽；没有就新开或不占）。预期 `name=slot_trim_race_summary arm=intact … violations=0`（推的，待主 agent 跑）。

### 1-③：清槽与另一趟同时挑槽

不成立：删不到别的趟刚占的槽。理由（读码）：

- 所有占槽的锁都在 `slots.lock` 之内拿：挑槽进程先阻塞拿登记锁「    fcntl.lockf(registry, fcntl.LOCK_EX)」（`scripts/gate/merge-and-verify.sh:1268`），挑、新开、清多出来的槽都做完才「    registry.close()」（`scripts/gate/merge-and-verify.sh:1297`）；`lock_slot` 只在这段里调。
- `trim_surplus` 拿到那一槽的锁之后先 `handle.close()` 放掉、再判留着、再 `rmtree`，中间有一个没锁的窗口；能在这个窗口里拿那一槽锁的只有另一个挑槽进程，而它要先拿 `slots.lock`，拿不到。占着锁的那一趟（锁跟着它的挑槽进程直到收尾）不需要 `slots.lock`，`trim_surplus` 的非阻塞锁拿不到就跳过。
- 刚新开的槽：`mkdtemp` 与 `lock_slot(held)` 之间那一刻没锁，同样在 `slots.lock` 之内。刚新开的槽也不在这一次的 `slots` 清单里，进不了 `surplus`。
- 只在本机占槽之后，第二台的趟在 `choose_slot` 开头就退回一趟一份（`:393-396`），不挑也不清，不会与本机的趟经 NFS 抢 `slots.lock`。

模型的 held-slot-vanished（挑槽进程答了 `SLOT held` 之后、放之前，那一槽目录不在）与 double-hold（同一槽同一时刻两个占着的）两格就是这一项；判法自己会不会红由三条弄坏开关臂证：同一段挑槽 python 把第五个参数换成 `slot-lock-skipped`、`slot-surplus-kept-removed`、`slot-surplus-taken`，各跑 30 个偏向打中的场景，各自要打出 double-hold、kept-removed、surplus-taken 或 surplus-free-left 至少一格，打不出的整趟退 1（`name=slot_trim_race_done … unproven_breaks=none` 才算模型判得出）。

另记一处只费时间的（推的、没量）：挑槽进程拿着 `slots.lock` 逐槽 `rmtree`，挑槽那一侧每一行只等「SLOT_HOLDER_SECONDS=120」（`scripts/gate/merge-and-verify.sh:1098`）秒。清一槽要超过 120 秒时，清的那一趟自己读超时、`release_slot` 把挑槽进程停在 `rmtree` 中间，那一槽删了一半留着（下一趟照 `--checksum` 拷齐，不错判）；同时在等登记锁的别的趟也各自超时、退回一趟一份。本机本地盘上删一份快照多半远不到 120 秒，没量。

### 1-④：「三样一起给」那一道漏给时会删到哪

那一道只在给了自证那两样之一时才判：「  if [[ -n "${MERGE_AND_VERIFY_SLOT_TABLE:-}${MERGE_AND_VERIFY_SLOT_BUILD_ROOT:-}" ]] \」（`scripts/gate/merge-and-verify-snapshot-lib.sh:400`）。两样都没给、只给了 `SINGLEFS_WORK_ROOT` 的不进这一道；`slot_build_root` 在没给 `MERGE_AND_VERIFY_SLOT_BUILD_ROOT` 时读多机配置里这一台真的编译根（`:203-204`），工作根却取 `SINGLEFS_WORK_ROOT` 给的那一个（`scripts/gate/merge-and-verify-snapshot-lib.sh:404`，`.claude/hooks/lib_work_root.py:235`「        return os.path.normpath(os.path.abspath(os.path.expanduser(configured)))」）。之后 `sweep_slot_targets` 拿这个工作根的槽那一层对着这一台真的编译根删：「    [[ ! -e "$1/$name" ]] || continue」（`scripts/gate/merge-and-verify-snapshot-lib.sh:312`），过了就「    if output="$(python3 "$RUN_BUILD_DIRECTORY" remove "$directory/target" 2>&1)" && rmdir -- "$directory" 2> /dev/null; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:313`）。占到槽、没占到、slots 是 0 三条路都走这一步（`:416`、`:427`、`:433`）。

反例输入（生产里，不是自证）：

1. 真的工作根里有 `slot.4u3rewvh`（留着）与 `slot.kyz05duu`（空着），这一台编译根下 `merge-and-verify-slots/` 里各有一份编译目录（调度记录量过槽位这一版两槽编译目录 105G、91G，没核是不是这两槽）。
2. 主 agent 经服务交 `-- env SINGLEFS_WORK_ROOT=<共享根下另一个目录> bash scripts/gate/merge-and-verify.sh …`。服务只拒共享根外的工作根：`task-server/task_server_lib_constants.py:101` 那一行写的是交给命令的 `SINGLEFS_WORK_ROOT` 不在共享根下才拒；`settle_break_switches` 管的变量里没有 `SINGLEFS_WORK_ROOT`（`scripts/gate/merge-and-verify.sh:1046-1047`）。脚本自己在槽那一层落进要验的树里时给的出路就是改工作根：「工作根设到树外（SINGLEFS_WORK_ROOT 不设就是数据根下的 work）」（`scripts/gate/merge-and-verify-snapshot-lib.sh:425`）。
3. 这一趟在新工作根里新开一槽，`sweep_slot_targets` 看见真编译根下 `slot.4u3rewvh`、`slot.kyz05duu` 在新工作根的槽那一层里都不在，两份编译目录都交给 `run_build_directory.py remove`：没有进程开着里面的文件就删（留着的槽的编译目录、正在两步 cargo 之间的占着的槽的编译目录都删），只有此刻正在编、开着文件的那一份留下并打一行 `!`。

后果只是费编译（下一趟那一槽从头编，正在跑的那一趟下一步 cargo 自己重建编译目录），不会把没编的判成编过：编译目录没了只会多编。频率低：服务交的命令里工作根本来就是真路径（`task-server/task_server_lib_constants.py:196`「WORK_ROOT_VARIABLE = "SINGLEFS_WORK_ROOT"       # 工作根：lib_work_root.py 的 work_root() 在服务这一侧现算的真路径」），要调用方特意改。

模型 `work_root_gap.sh`（推的，待主 agent 跑）每条臂一份新现场：原来的工作根 W1 里四槽（空着、留着、样本进程拿着锁、样本进程开着它编译目录里的一份文件），替身编译根 B 下照冻结布局各一份编译目录。预期：

| 臂 | 给的工作根 | 预期 idle / kept / held / busy |
|---|---|---|
| control | W1，slots=3 | 在 / 在 / 在 / 在 |
| gap-take | W2，slots=3，不给另两样 | 删了 / 删了 / 删了 / 在（一行 `!`） |
| gap-zero | W2，slots=0 | 删了 / 删了 / 删了 / 在 |
| fix-namespace | W2；改法：编译根下那一层再按槽那一层真路径的 sha256 头 16 位分一层 | 在 / 在 / 在 / 在 |
| fix-same-root | W2；改法：没给 `MERGE_AND_VERIFY_SLOT_BUILD_ROOT` 时工作根真路径要等于这一台缺省的工作根，不等就不占也不清 | 在 / 在 / 在 / 在 |

两条改法都只在我的模型上量过、被攻过零轮，而且模型还没跑。fix-namespace 会把已有的不分层编译目录留成没人清的，照共用约束「里程碑 17 收尾之前不兼容任何旧东西」（`.claude/agent-common.md:46`）不写迁移，要换上就手删一次旧那一层；fix-same-root 只在模型里拿 W1 当缺省工作根，真改要拿 `lib_work_root.py` 的缺省值（数据根下的 `work`）比，SINGLEFS_DATA_ROOT 也被改过时还要再核。

自证这一半不成立：自证开头把三样一起指进 scratch：「  export MERGE_AND_VERIFY_SLOT_TABLE="$scratch/slot-table-none.conf" MERGE_AND_VERIFY_SLOT_BUILD_ROOT="$scratch/slot-build-none" SINGLEFS_WORK_ROOT="$scratch/work-root-none"」（`scripts/gate/merge-and-verify-selftest-lib.sh:27`）；`grep -nE 'env -u[^|]*(SINGLEFS_WORK_ROOT|MERGE_AND_VERIFY_SLOT)|unset[^#]*(SINGLEFS_WORK_ROOT|MERGE_AND_VERIFY_SLOT)|env -i' scripts/gate/merge-and-verify-selftest-lib.sh` 只命中三行，都是 `-u SINGLEFS_WORK_ROOT_BREAK`（第 1319、1336、1401 行），没有一格去掉这三样。只改 `SINGLEFS_WORK_ROOT` 的两格（第 1209、1302 行）编译根仍是 scratch 那一份。别的格的编译根在 scratch 里，就算工作根取错也删不到真编译根。正文说「这一道没配格与开关」，现查属实：`grep -c '三样不齐' scripts/gate/merge-and-verify-selftest-lib.sh` 是 0，这一道删掉也没有一格会红。

### 1-⑤：拷进槽之前删 target 一类

不成立：删不到源树。`find` 只对槽里那一份快照跑：「    find "$2" -name .git -prune -o -type d \( -name target -o -name 'target-*' -o -name __pycache__ \) -prune \」（`scripts/gate/merge-and-verify-snapshot-lib.sh:268`），`$2` 是 `<槽>/repo`，源树 `$1` 只作 rsync 的源。删的这几种名字，两种快照本来就不从源树拷：一趟一份那一种「--exclude 'target/' --exclude 'target-*/' --exclude '__pycache__/'」（`scripts/gate/merge-and-verify-snapshot-lib.sh:42`），槽那一种同一组排除（`:277`）。所以槽里删掉的只能是上一趟留下的，拷完的槽与一趟一份那一种的快照里少的是同一批。

源树里本来叫这几个名字的目录（例如测试样本里的 `tests/fixtures/target/`）两种快照都不带：这是排除写法本来的样子，不是这一次修补带来的。仓里此刻跟踪的路径没有一份落在这种目录下（量的：`git ls-files | grep -cE '(^|/)(target|target-[^/]*|__pycache__)/'` 输出 `0`）。没跟踪又没被忽略的文件同样适用，这一条我没扫。

模型 `copy_exclusions.sh` 把这些放进源树：`tests/fixtures/target/data.txt`、`src/target-spec/mod.rs`、`tools/__pycache__/m.pyc`、名叫 `target` 的普通文件、名叫 `link-target` 与 `target-link` 的符号链接。槽里另事先放 `repo/target/debug/junk`、`repo/crates/a/tests/fixtures/target/old.txt`、`repo/.git/target/keep`、`repo/tools/__pycache__/old.pyc`。预期（推的）：两种快照都少前三份、都带普通文件与两个符号链接，`name=copy_exclusions_compare same=yes`；槽里 `.git/target/keep` 留着，别的三样删了；`name=copy_exclusions_tracked count=0`。

### 1-⑥：关掉增量之后两种 cargo 收到的环境

成立。`choose_slot` 在入口脚本自己的 shell 里调（「  choose_slot "$logs"」，`scripts/gate/merge-and-verify.sh:2636`，不在 `$( )` 或管道里），`slot_incremental_off` 在它里面（`scripts/gate/merge-and-verify-snapshot-lib.sh:437`），导出那一句「  [[ "${MERGE_AND_VERIFY_BREAK:-}" == "slot-incremental-on" ]] || export CARGO_INCREMENTAL=0」（`scripts/gate/merge-and-verify-snapshot-lib.sh:371`），之后的 fmt、clippy、build、test 与验证包那一步都继承。验证包那一步起的门禁照调用方的 `CARGO_TARGET_DIR` 编（`.claude/gate.d/crash-point-replay.sh:337`），占槽那一趟导出的是这一槽的编译目录（「  CARGO_TARGET_DIR="$SLOT_TARGET"; export CARGO_TARGET_DIR」，`scripts/gate/merge-and-verify.sh:2671`），所以收到 0 的 cargo 编的都是这一槽那一份。不占槽的那一趟不进 `slot_incremental_off`，环境不动。调用方设了非空值的照它（「  if [[ -n "${CARGO_INCREMENTAL:-}" && "${MERGE_AND_VERIFY_BREAK:-}" != "slot-incremental-caller-overridden" ]]; then」，`scripts/gate/merge-and-verify-snapshot-lib.sh:367`）；设成空串的当没设、导出 0（cargo 只认 1 才开，空串本来就是关，结果一样，推的）。服务那一侧没有一处设 `CARGO_INCREMENTAL`（`grep -rn CARGO_INCREMENTAL task-server/*.py` 零命中，量的）。

另记四处，都不是错判：

1. 只费盘（推的）：不占槽的那一趟编译目录收尾整份删，增量开着照写 `incremental/`，那一份从来不会被下一趟用到。第一轮记录量过 `incremental/` 占一槽编译目录的一大块（53G），本机没空槽时退回的每一趟都在 `${TMPDIR}` 下写这一块再删。用户定案只说了槽位那一种，这里只报，不当打中。
2. 只费盘（推的）：调用方自己设 `CARGO_INCREMENTAL=0` 的走「调用方设了」那一支，这一槽编译目录里早先留下的 `incremental/` 不删（`:367-369`）。模型的 zero 臂量这一种。
3. 只费盘（推的）：`trim_slot_target 开跑前`（`:434`）排在 `slot_incremental_off`（`:437`）之前。某一槽的编译目录要是主要因为早先攒下的 `incremental/` 超了 150 GiB，就整份删掉，而不是只删 `incremental/`、留下别的产物。只在关增量之前攒下的那几槽上会出现。
4. 推不出：调度记录写着 `scripts/gate/admission.py` 文件头把生效的 incremental 收进单元键（第 27–29 行；关掉增量还连带把缺省的 codegen-units 从 256 换成 16），所以占槽那一趟与不占槽那一趟、与提交时整轮门禁的同一项键会不同、互不复用。我没核按条复用的请求带不带编译档：`admission_unit_key.py` 的 `profile_lines` 文件串说只给「--release 这一类编译档」出这几行。所以少复用多少、有没有少，这里推不出。量法写在末尾「要主 agent 跑的命令」之后的说明里，归算键库那一边判。

模型 `incremental_holder.sh` 四条臂（推的，待主 agent 跑）：unset 臂预期 `exported=0 child_sees=0 debug=在 triple=删了 release=删了 bang_lines=1 outlet=有`；empty 臂同 unset；one 臂预期 `exported=1`、三份都在、`bang_lines=0`；zero 臂预期 `exported=0`、三份都在。

### 1-⑦：删 incremental 时有进程开着

成立：出路在，照往下编。`run_build_directory.py remove` 先核本机有没有进程开着里面的文件，有就不删、退 1，`slot_incremental_off` 接住打「      echo "  ! 这一槽编译目录里的增量编译缓存 $directory（${size:-?}）没删成，照往下编：${output//$'\n'/；}"」（`scripts/gate/merge-and-verify-snapshot-lib.sh:380`），整个函数照样退 0。照往下编是对的：导出的是 0，这一趟的 cargo 不读也不写那一份 `incremental/`，下一趟占这一槽再删。按协议，这一槽此刻只有本趟在用（锁是排他的），开着里面文件的只能是早先漏下的进程。

说法上有一处指错对象：嵌进那一行的是 `run_build_directory.py` 自己的句子，「这一趟的编译目录 {directory} 还有 {len(holders)} 个进程开着里面的文件，没删（起它的那一趟的子进程没收干净）」（`scripts/hosts/run_build_directory.py:109`），后面还有「修起它的脚本，让它停自己的子进程之后再删」。这里删的是这一槽编译目录下的 `incremental/`，不是「这一趟的编译目录」，开着它的也不一定是本趟漏下的子进程。外面那一句写得对，嵌进去的一句让人去修 merge-and-verify 自己。这只影响读的人，不影响行为，归第 3 格正文那一类，这里只报。

## 第 2 格：多机与不兼容旧东西

### 2-①：只在本机占槽、第二台那一行的认法

成立。第二台那一层给的两样是同一个根：「  build_text="CARGO_TARGET_DIR=$(printf '%q' "$peer_scratch/target") "」（`scripts/hosts/multi-host-run.sh:1419`），同一条命令另带 `MULTI_HOST_RUN_PEER_SCRATCH=$(printf '%q' "$peer_scratch")`（`:1453`）；`--reuse-build-directory` 那一种也是 `builds/<标签>/target` 对 `builds/<标签>`（`:13`）。认法「    if [[ -n "${MULTI_HOST_RUN_PEER_SCRATCH:-}" && "$CARGO_TARGET_DIR/" == "${MULTI_HOST_RUN_PEER_SCRATCH%/}/"* && "${MERGE_AND_VERIFY_BREAK:-}" != "peer-target-called-caller" ]]; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:393`）两边各补一个 `/` 再比前缀，`<s>/target` 认得出，`<s>` 带一个尾 `/` 的也认得出（多于一个尾 `/` 时 `CARGO_TARGET_DIR` 照样带着同样多的 `/`，前缀仍对上，推的）。

两支都在同一处退回一趟一份（`:394` 与 `:396`），都在挑槽、清槽、清编译目录之前退出。所以认错的后果只是那一行说成「调用方给了」，第二台照样不占、不清。本机这一侧：服务只读入口文件头前 60 行找会编的声明（「DECLARED_HEADER_LINES = 60」，`task-server/task_server_lib_constants.py:367`），`scripts/gate/merge-and-verify.sh` 里唯一写着「# task-server-compiles: yes」字样的是第 105 行的说明句，在 60 行以外，又不以声明那几个字起头，服务不会把它当声明、在本机给编译目录。

开工前那一趟整份自证是派到第二台跑的：服务状态行「第 1 次：第二台、…、状态 ended、退 0（succeeded）」，私有路径没抄。它的槽位各格照 `env -u CARGO_TARGET_DIR` 起（`scripts/gate/merge-and-verify-selftest-lib.sh:1336`、`:1401`），所以在第二台上也走占槽那一支，用的是 scratch 里的工作根与编译根，与「生产里第二台不占槽」不冲突。

### 2-②：为旧东西留的分支

不成立：没找到。按共用约束「里程碑 17 收尾之前不兼容任何旧东西」（`.claude/agent-common.md:46`）逐样找过：

- 旧放槽命令：开工前草稿基线（`~/.local/share/singlefs/work/merge-verify-slots-r1-fixes/base/scripts/gate/merge-and-verify-snapshot-lib.sh` 第 70 行）放槽是 `rm -f -- kept .owner .deliverables`，不删 `repo/target`。现行代码没有一处专门认「照旧命令放过的槽」：`refresh_slot_snapshot` 删 `repo/target` 一类，是因为第 2 步在留着的槽里起的 cargo 现在照样编进 `<快照>/target`（`scripts/gate/merge-and-verify.sh:107-108`），新旧都要删。
- 旧槽布局：基线里槽的编译目录只在弄坏开关 `slot-target-on-shared-root` 下才建在 `<槽>/target`，平常已经在这一台编译根下（基线第 324–325 行）。现行代码没有认 `<槽>/target` 的分支；那个开关是演缺陷，不是兼容。
- `kept` 的格式：基线与现行都写两行 `log:`、`created:`（基线第 65 行、现行 `:66`）。槽位汇总读不出 `created` 时「      else started="$(stat -c %Y -- "$slot/kept" 2> /dev/null || echo "$now")"; since="kept 的修改时刻"; fi」（`scripts/gate/merge-and-verify-snapshot-lib.sh:344`）不是为哪一版旧格式留的，是对写坏或手写的 `kept` 的宽读法。它只报、不改行为；要照「认不得的走同一条拒绝」收严，就改成打「kept 认不得」，归第 3 格那一类。
- 槽那一层里没有 `used` 的槽按 0 排在最后（`used_at`），刚 `mkdtemp`、还没写 `used` 的那一刻也会这样，不是旧东西。

## 已经量的逐句核

在哪核的：服务状态行用 `task-server/task status <号>` 逐个读（这一会话交的那一段号，命令原样与结局都在状态行里）；日志行在 `~/.local/state/singlefs/task-server/logs/` 下各自那一份里 `grep -F`。任务号与日志文件名照规矩不写，只写交给服务的命令与那一行的字。

| # | 正文那一句 | 核到没有 | 原文 |
|---|---|---|---|
| 1 | `hook-swap.py` 先 dry-run 再真换都 type=succeeded exit=0 | 核到 | 命令 `python3 scripts/gate/hook-swap.py …/swap-merge-and-verify-slots.json --scratch …/hook-swap-slots --dry-run` 状态行「结局 succeeded、退 0」，日志「--dry-run：停在草稿，主工作区一个字节没写」；不带 `--dry-run` 那一条「结局 succeeded、退 0」，日志「✓ 换上了 4 份（改名换上新 inode、回读一致），主工作区里再验过：4 份文件语法检查、内嵌 python 0 段编译」 |
| 2 | 整份自证 type=succeeded exit=0「✓ 自检通过（查了 172 种）」 | 核到 | 命令 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`「结局 succeeded、退 0」，日志「  ✓ 自检通过（查了 172 种）：测试红时退 1 并列出红的与重跑命令、…」，`✗ 自检` 行数 0；这一趟派在第二台 |
| 3 | 新开关七条与补了点名的六条单跑都 type=command-failed exit=1 | 核到 | `env MERGE_AND_VERIFY_BREAK=<开关> …--selftest` 十三条：slot-surplus-taken、slot-surplus-left、slot-surplus-kept-removed、slot-build-outputs-kept、slot-release-keeps-build-outputs、slot-fallback-unsummarized、peer-target-called-caller、verify-in-main-tree、slot-not-reused、slot-kept-taken、slot-limit-ignored、slot-lock-skipped、slot-target-unbounded，状态行都是「结局 command-failed、退 1」 |
| 4 | 红的格与工具员报告的点名表逐条相同 | 只核到窄的一句 | 新七条红的格与工具员报告 `~/.local/share/singlefs/work/merge-verify-slots-r1-fixes/result.md` 点名表逐条相同（例 slot-surplus-taken 只红「槽位：调低上限之后多出来的槽不占」「槽位：slots 是 0 时清多出来的槽与它的编译目录，留着的槽不动」两格）。补点名的六条，报告表只列新补的格，日志里另红早先已点名的格：slot-not-reused 红七格（「留着的槽不给别的趟…」「同一槽先后两趟…」「源树里改了、删了的文件…」「槽的编译目录里的产物比改过的源码新…」「上一次尝试：撞上限重来…」「调低上限之后…」「拷进槽之前…」），verify-in-main-tree 红 28 格（`grep -c` 量的）。报告表里点名的每一格都在红的格里，红的格里多出来的都在 `scripts/gate/merge-and-verify.sh` 开关表那一条的早先点名里（按第 937–946 行读，减去后来才加的增量两格）。所以「点名表包含在红的格里、多出来的都是开关表的早先点名」核到；字面「逐条相同」只对新七条成立 |
| 5 | 关掉增量换上 type=succeeded exit=0 | 核到 | 增量那份规格真换「结局 succeeded、退 0」，日志「✓ 换上了 3 份（改名换上新 inode、回读一致），…」；删那一行的规格「结局 succeeded、退 0」，日志「✓ 换上了 1 份（改名换上新 inode、回读一致），…」；dry-run 那一条同样「结局 succeeded、退 0」 |
| 6 | 之后整份自证 type=succeeded exit=0「自检通过（查了 183 种）」 | 核到 | 「  ✓ 自检通过（查了 183 种）：测试红时退 1 …」，状态行「结局 succeeded、退 0」 |
| 7 | 增量三条开关只红那一格；`slot-target-on-shared-root` 红那一格与「占槽、放槽…」两格 | 核到 | slot-incremental-on 只一行「✗ 自检：槽位：占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、…：实际 占槽那一趟 cargo 收到的 CARGO_INCREMENTAL 是「unset」，不是每条都是 0；」；slot-incremental-cache-kept 只一行「…实际 这一槽编译目录里的 debug/incremental 还在；没有删了增量编译缓存那一行；」；slot-incremental-caller-overridden 只一行「✗ 自检：槽位：调用方设了 CARGO_INCREMENTAL 的照它、不盖，…实际 cargo 收到的 CARGO_INCREMENTAL 是「0」，不是调用方设的 1；…」；slot-target-on-shared-root 两行，「✗ 自检：槽位：占槽、放槽，槽的编译目录在这一台的编译根下：…」与增量那一格；四条都「结局 command-failed、退 1」 |
| 8 | 换上之后那一趟开跑前「174 条里拒算 174 条」，拒算在别的会话那一批的一处 `bash -c`，已由它改掉 | 核到 | 命令 `bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh`「结局 command-failed、退 1」，日志「  ✗ code-tooling.sh 的 break-switch-replay 格：开跑前算键，174 条里拒算 174 条（单元闭包收不全、键算不出，照跑也一条都入不了库），这一格当场判红、一条都没起」与「  ✗ 拒算：inline-script-not-literal …（174 条，…）」；此刻 `grep -nF 'bash -c "$anchored_check_line"' scripts/gate/merge-and-verify-selftest-lib.sh` 零命中、退 1（改掉了） |
| 9 | 此刻别的会话交的那一趟复跑在跑，罩这份脚本全部开关 | 核到前半句 | 别的会话交的同一条 `bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh` 我读状态时是「running」；「罩全部开关」要等它的结局行，没核 |
| 10 | rules-lint type=succeeded exit=0「命中 0」 | 核到 | 命令 `bash .claude/singlefs-ai-sop/scripts/rules-lint.sh /home/fy5090/code/singlefs`「结局 succeeded、退 0」，日志「  ✓ 规则只写怎么做（语言 zh；扫了 11 份文件 850 行；…词法说明判了 743 行（围栏与表格行不判），命中 0；…）」 |
| 11 | doc-lint type=command-failed exit=1，这一批改过的文件里只命中 `.claude/agent-common.md:47`，那一行 HEAD 里就有 | 核到 | 「结局 command-failed、退 1」，日志「  ✗ .claude/agent-common.md:47  正文不许写"原先 <旧值>"，直接改成现行值」；这一批改过的另九份（规则、技能、`main-agent.md`、`prove-red.sh`、`crash-library-totals.sh`、登记行所在的四份）与合入后验证脚本在日志里各 0 处（`grep -cF <文件名>`，量的）；那份日志只把编号引用的明细截了，按文件的违规行都列全了。那一句此刻在 HEAD 的第 42 行（`git show HEAD:.claude/agent-common.md | grep -nF '「原先」'`），行号与当时不同，内容在 |
| 12 | 门禁五格之后，登记行五处改完，没再单跑那五格 | 只核到窄的一句 | 我读的那一段号里，这一会话交的命令只有 `hook-swap.py`、自证、开关单跑、rules-lint、doc-lint 与开关复跑，没有门禁那五格。整段会话的号我没读全，说「没再单跑」只到这一段为止 |

另记一句与「已经量的」相关的：工具员报告自己写「上面每一处「会红、红在哪一格」都是按代码推的，一条都没跑过」，换上之后的单跑才是第一次量；第 3、4 行核的就是那一次。

## 没打中的形状

- 1-①、1-②、1-③：模型 `slot_trim_race.py` 取的形状写在 1-① 一节，种子 20261120，场景数与各弄坏开关臂的场景数是模型的缺省（120、各 30）。挑槽 python 的第五个参数（弄坏开关）只在三条证红臂里给。没扫：挑槽进程等登记锁超过 120 秒的那一种（要造很大的槽，只推）；第二台经 NFS 拿锁（只在本机占槽之后够不着）；`used` 修改时刻相同的并列。
- 1-⑤：模型只放了 1-⑤ 一节列的那几种名字，没放硬链接、权限不足删不掉的目录、名字带换行的文件。
- 1-⑥：没起真 cargo，cargo 怎么读 `CARGO_INCREMENTAL` 与空串都是推的；`CARGO_BUILD_INCREMENTAL` 与 `[build] incremental` 被导出的 0 盖掉、不照「调用方设了的照它」，这一样我按 cargo 的次序推，没在本机量。
- 2-①：没造第二台，只读 `multi-host-run.sh` 给的两个变量与比前缀那一行。

## 这条腿自己的限度

- 四份模型一条都没跑，判定一览里标「推的」的读数都要等主 agent 复跑。
- 1-④ 的生产那一半要调用方特意给一个别的工作根才出，频率我没量；后果只算到「费编译」，没量从头编一槽要多久。
- `work_root_gap.sh` 换掉了 `slot_build_root`：模型证的是「编译根取到哪一个，就清哪一个」；「没给 `MERGE_AND_VERIFY_SLOT_BUILD_ROOT` 时取到的是这一台真的编译根」靠读 `:203-204` 与 `SLOT_BUILD_ROOT_PROGRAM` 推。
- 第 1-⑥ 节第 4 条（键里的增量）推不出，没量。
- 这一腿只看了槽位一段与快照库，`--unresolved-to-commit-gate` 一族与变异锚点核一族没读。

## 要主 agent 跑的命令

1. 四份模型整份复跑（新 shell 里起；运行目录在草稿目录下、名字以 `-run` 收尾，外层脚本先删它再调 `tools/rerun.sh`）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-verdict/frozen /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-opus/rerun-run /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-opus`
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 是 `type=succeeded exit=0`（退非 0 的件数为 0 才算过：`name=rerun_done failed_models=0`，四行 `name=rerun_model … exit=0`）；之后逐行对报告里的预期（都是推的）：
     - `name=slot_trim_race_summary arm=intact scenarios=120 violations=0`，三行弄坏开关臂 `expected_kinds_hit=` 都不是 `none`，`name=slot_trim_race_done … unproven_breaks=none`；`name=slot_trim_race_estimate` 那一行是挂钟估数；
     - `name=work_root_gap arm=control idle=在 kept=在 held=在 busy=在`，`arm=gap-take` 与 `arm=gap-zero` 是 `idle=删了 kept=删了 held=删了 busy=在`，`arm=fix-namespace`、`arm=fix-same-root` 四样都是 `在`；
     - `name=incremental_holder arm=unset exported=0 child_sees=0 debug=在 triple=删了 release=删了 bang_lines=1 outlet=有`，`arm=empty` 同 unset，`arm=one exported=1 … debug=在 triple=在 release=在 bang_lines=0`，`arm=zero exported=0 … 三样都在`；
     - `name=copy_exclusions_compare same=yes`，两行 `name=copy_exclusions kind=…` 的 `missing=` 都是 `./crates/a/src/target-spec/mod.rs,./crates/a/tests/fixtures/target/data.txt,./tools/__pycache__/m.pyc`、`extra=` 空，`slot_leftover_git_target=在`，`name=copy_exclusions_tracked count=0`；
     - 不该有 `name=model_group_left` 与 `group-left`、`model-error` 字样的行。
   - 跑完之后读到的哪一行与上面不同，那一格的判定照读数改；1-④ 的打中要 gap-take、gap-zero 两臂读数坐实才算量的。
2. 引文核对（交回闸对三方腿跑同一个脚本；我在本会话里直接跑的结果写在下一节，这一条是给判决时复核用的）：
   - 命令：`task-server/task submit -- python3 scripts/agents/cite-check.py research/prompts/merge-verify-slots-governance-r2-opus-output.md`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看哪几行：末行 `TASK_SERVER_RESULT` 是 `type=succeeded exit=0`，日志里「核了 A 处引文、B 处裸行号」那一行没有对不上的。

1-⑥ 第 4 条的量法（不是这一腿的格，只写给算键库那一边）：同一条按条复用的请求，在 `CARGO_INCREMENTAL=0` 与不设两种环境下各算一次单元键，比两把键是不是相同；我没读 `scripts/gate/admission.py` 的命令行，交那一边定怎么起。

## 没做什么

- 不判正推、辩方那几格（第 3、4、5 格），不替主 agent 采纳；第 1-⑦ 节与第 2-② 节里归第 3 格的两处说法只报。
- 四份模型一条都没跑，判决要引的数都没复跑：命令在上一节。没往统一任务服务交任务。
- 副本上没量任何数：这一腿没建仓副本，也没建编译目录。
- 没碰真的工作根与编译根：只在真的工作根上 `ls -la` 看过两槽有哪些文件（只读）。
- 别的会话那一趟开关复跑的结局没等，第 9 句后半句没核。

## 引文核对与草稿清理

- `python3 scripts/agents/cite-check.py research/prompts/merge-verify-slots-governance-r2-opus-output.md` 在本会话里直接跑，退 0，末行：「  ✓ 核了 25 处引文、28 处裸行号，对上 25 处，没判 28 处（其中裸行号没有引文可比 28 处；逐处列在上面）」。这是写完本节之前那一版的读数，本节加的几行里没有新引文。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-opus/` 里删了自己建的中转：拼 `tools/rerun.sh` 的 `build-rerun/`（20K）、语法检查的 `pycache/`（1.1M）与几份读日志时的列表。没建仓副本、编译目录与工作树；留着 `cite-check.txt`（引文核对的全量输出）。复跑的运行目录 `rerun-run/` 由主 agent 交的那一趟建。
