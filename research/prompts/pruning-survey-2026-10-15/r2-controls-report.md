# 第二轮复查·路子 controls：每条机制新键的阳性与阴性对照样本（底稿第三节第 8 问）

日期 2026-10-15。主工作区 HEAD c1e9b324，只读：没改仓里任何文件、没起 cargo、门禁、测试二进制，没动判定库。
样本的行号全部由 `tools/samples.py check` 在主工作区逐条现查（每条改动核「那一行含不含原文」）；今天的键对这些样本怎么反应，在草稿目录的仓副本上量（`tools/keys.py`、`tools/samples.py run`、`tools/replay_probe.py`），缓存只写副本自己的 `.git`。
标注：「量的」指到本报告第 1 节贴的原样输出那一行或 `文件:行`；「推的」写明推的是哪一半。

## 0 结论

- 量的：27 条机制（L1–L27）每条都给齐了「必须作废」与「必须不作废」两类各至少 3 个样本（第 3 节；awk 按节数表行，27 节里阳、阴两类不足 3 行的 `short=0`，最少的是 3 行）。探针样本目录 74 个（S 类 31、N 类 43，共 80 处改动）在副本上逐个跑过，另有研究复跑 5 个（R01–R05）、HEAD 演示 3 步；84 处带行号的锚点在主工作区逐条核过，`mismatches=0`（第 1.2 节）。第 3 节另用到没进探针的样本（环境类，与只给行号、没在副本上施加的），表里逐个标了。必须作废类六样（判器函数体、下游函数、常量含 V5 / V11、类型定义、wgsl 内核函数、依赖版本）与必须不作废类十二样（.rs / wgsl / .py / .sh / .toml / .tsv 注释、同文件加没人调的函数、测试模块加单测、改名、登记表别的行、lock 无关包、限额、HEAD 移动）的覆盖表在第 2.3 节。
- 量的：拿这批样本打今天的键，N 类 43 个里今天至少换一把键的 39 个，一把不换的只有 N01、N02、N25、N30 四个（第 1.3 节逐行，awk 数的；第 4 节归类）；必须作废的样本里今天不作废的（漏判）量到 3 个：GPU 派活回读归位（S12：`gpu` 不变）、release profile 的 `overflow-checks`（S16：三把结论版本都不变）、提交进 HEAD 之后的改动（`change-touches-crates.sh` 报「没碰」）；另有研究复跑的 GPU 开关一个，是 `replay.sh:55` 文件头自认、没在副本上造（推的）。这些就是新键自证里今天会红的格。
- 量的：探针自己的两个对照都过了：什么都不改（Z00）两次 `changed=0`；V5（S04）一改 12 把全换。
- 量的、新撞见：① 登记了整个 `scripts/` 的两道（QEMU、研究复跑）把 `scripts/` 下 git 列得出的任何文件原样收进整道键，连 `__pycache__/*.pyc` 也算——探针自己踩到的：副本起初没拷仓根 `.gitignore`（主仓 `.gitignore:26` 忽略 `__pycache__/`），`.pyc` 被 `git add -A` 收进去，之后 `admission.py` 一改一还原、`.pyc` 重写，两道整道键就换了，什么都没改的 Z00 也报 `changed=2`（第 1.4 节）；② Cargo.lock 里 vstd 抬版本不动 verus-proof 那一格的键，这一格本不该动（Verus 验证用发布包自带的 vstd，`.claude/gate.d/verus-pins.tsv:10-17` 的同源说明），该动的是 verus-toolchain-pinned 那一格。
- 推的：L13 里「CPU 判器改了不作废 GPU 结论」这一条，今天的 GPU 版本会跟着换（S01 量的），新键能不能做到不换，要看 GPU 判器入口的调用图是否真不经 `crash.rs` 的判器函数；这一条给成样本，但它的期望写成条件式（第 3 节 L13）。
- 每道门禁的自证做法（弄坏开关名、红在哪一格）在第 5 节，共 12 处。

## 1 核法与原样输出

### 1.1 探针与副本

- 副本：`rsync -a --exclude target --exclude .git` 拷 `crates`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`scripts`、`litmus`、`.claude/gate.d`、`.claude/scripts`、`.claude/singlefs-ai-sop/scripts`、`.claude/heavy-thread-default`、`.claude/kb/layout/01-first-txn.md` 与仓根 `.gitignore`；`research/results/` 只放一份桩文件 `stub-product.out`。在副本里 `git init` 并 `git add -A`、提交过几次（副本自己的 `.git`，主仓的 git 一样没碰），因为 `admission.py` 列文件走 `git ls-files`。
- `tools/keys.py`（草稿目录）在副本上算今天的 18 把键：crates 侧三种结论版本（`gpu`；`cpu_mr` / `obs_mr` = 设点文件取 `tests/crash_enumeration_multi_record_publish_stream.rs` 的 CPU 与观察者；`cpu_ts` = 设点文件取 `tests/crash_enumeration_tree_split_streams.rs` 的 CPU），用的是第一轮 crates 路的移植算法 `closure_port.py`（那一路与编译表 137/137 逐行相等）；门禁 7 道 / 格的 `admission.py stage-fingerprint`；崩溃枚举用例判法摘要 `crash_case_judging_digest_text`；crates 变异判法版本 `judge_version`；广谱变异两条变异的 `mutation_key`（`crates/mutations.tsv:678` 那条在 `check_root_slot` 里、`:439` 那条是 fn 外常量，只索引 `singlefs-checker` 与 `singlefs-format` 两份 crate 的 src）；形式化 `judge_version` 与 `shared_components` 里的核 crate、`verus-pins.tsv` 两项。
- `tools/samples.py`：样本目录（每条 = 文件、现在的行号、那一行必须含的原文、改成什么），`check` 在主工作区只读核，`run` 在副本上施加、算键、还原。
- 基线（`baseline.json`，原样）：

```
{"cpu_mr": "4991f2465c05", "cpu_ts": "fe35be42de6d", "crash_case_judging": "29c7a10a5c84", "crates_mutation_judge_version": "c6d72979bd38", "gpu": "d17732ad4260", "obs_mr": "79eaf967aca5", "prune_row439_const": "709d7458c70e", "prune_row678_check_root_slot": "dfe5d99c6b5f", "stage:checker-tier-crash-point-replay": "23d40aee1f65", "stage:checker-tier-crates-mutation-replay": "d9b2021f7864", "stage:checker-tier-formal-verification:verus-proof": "8eedfcb924f0", "stage:checker-tier-lkmm": "1096492bc435", "stage:checker-tier-qemu-device-streams": "62fe65897d9a", "stage:checker-tier-research-build-and-replay": "9a344cbc7b4a", "stage:harness-model-differential-and-scenarios": "518aa24229e3", "verus_crate": "63cdc36856dc", "verus_judge_version": "3eecb6661fb0", "verus_pins": "9cc578807d1d"}
```

`gpu` 前 12 位与第一轮 crates 路 `scenarios.out` 的 `gpu=d17732ad426081b7` 同、`cpu_ts` 与它的 `cpu_of_tree_split=fe35be42de6dccdd` 同、`crash_case_judging` 与第一轮 gates 路的 `original 29c7a10a5c84` 同（三条独立来源的数对上，量的）。

### 1.2 行号现查（主工作区只读）

```
$ python3 -I tools/samples.py check | grep -c "	ok	"
84
$ python3 -I tools/samples.py check | tail -1
mismatches=0
```

（N45 起初插在 `:321` 文档注释之后，会把 `check_root_slot` 的文档注释挂到新函数上，样本本身不干净；改到 `:320` 那一行空行之后重跑，`N45 crates/singlefs-checker/src/lib.rs:320 ok`。）没写进 `samples.py` 的样本（环境类、第 3 节里标「行号现查：…」的）各自的行号在第 2 节表里写了现查命令的结果。

### 1.3 每个样本打今天的键：换了哪几把（`tools/samples.py run`，`runs.out` 原样 76 行）

每行：样本号、换掉的键的把数、换掉的是哪几把（键名见 1.1 节；比的是 1.1 节那份基线）。Z00 在开头与末尾各跑一次。

```
Z00	changed=0	
S01	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S02	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S03	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S04	changed=12	cpu_mr cpu_ts gpu obs_mr prune_row439_const prune_row678_check_root_slot stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S05	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S06	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S07	changed=12	cpu_mr cpu_ts gpu obs_mr prune_row439_const prune_row678_check_root_slot stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S08	changed=12	cpu_mr cpu_ts gpu obs_mr prune_row439_const prune_row678_check_root_slot stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S09	changed=7	gpu stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S10	changed=7	gpu stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S11	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S12	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S13	changed=9	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S14	changed=9	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S15	changed=9	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S16	changed=5	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S17	changed=8	cpu_mr obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S18	changed=8	cpu_mr obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S19	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S20	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S21	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S22	changed=7	gpu stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S23	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S25	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
S27	changed=2	stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm
S28	changed=2	stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm
S29	changed=4	crash_case_judging stage:checker-tier-crash-point-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay
S30	changed=4	crates_mutation_judge_version stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay
S32	changed=12	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-formal-verification:verus-proof stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios verus_crate
S33	changed=1	stage:checker-tier-crash-point-replay
S34	changed=11	cpu_mr cpu_ts gpu obs_mr prune_row678_check_root_slot stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N01	changed=0	
N02	changed=0	
N03	changed=7	gpu stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N04	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N05	changed=7	gpu stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N06	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N07	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N08	changed=9	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N09	changed=9	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N10	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N11	changed=7	cpu_ts stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N12	changed=9	cpu_mr cpu_ts obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N13	changed=8	cpu_mr obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N14	changed=7	cpu_ts stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N15	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N16	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N17	changed=7	gpu stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N18	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N19	changed=7	cpu_ts stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N20	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N21	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N22	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N23	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N24	changed=1	crash_case_judging
N25	changed=0	
N26	changed=1	crates_mutation_judge_version
N27	changed=8	crash_case_judging stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-formal-verification:verus-proof stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N28	changed=2	stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay
N29	changed=1	stage:checker-tier-qemu-device-streams
N30	changed=0	
N31	changed=2	stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm
N32	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N33	changed=4	crates_mutation_judge_version stage:checker-tier-crates-mutation-replay stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay
N34	changed=6	stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N35	changed=1	crates_mutation_judge_version
N37	changed=12	cpu_mr cpu_ts gpu obs_mr prune_row439_const prune_row678_check_root_slot stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N38	changed=1	verus_crate
N39	changed=1	verus_crate
N40	changed=2	stage:checker-tier-formal-verification:verus-proof verus_pins
N41	changed=1	verus_judge_version
N42	changed=8	crates_mutation_judge_version stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-formal-verification:verus-proof stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N43	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
N45	changed=10	cpu_mr cpu_ts gpu obs_mr stage:checker-tier-crash-point-replay stage:checker-tier-crates-mutation-replay stage:checker-tier-lkmm stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay stage:harness-model-differential-and-scenarios
Z00	changed=0	
```

### 1.4 探针踩到的污染，与它说明的事（量的）

第一趟跑（输出留在 `runs-contaminated.out`、基线 `baseline-contaminated.json`）副本没拷仓根 `.gitignore`，`git add -A` 把 `scripts/gate/__pycache__/admission.cpython-312.pyc` 这类文件收成了被跟踪的文件；探针在 S29、N24、N27 改过 `admission.py` 再还原，`.pyc` 随源码的 mtime 重写。之后的样本里，QEMU 与研究复跑两道的整道键平白多换。原样（`runs-contaminated.out`）：

```
1:Z00	changed=0	
32:N01	changed=2	stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay
33:N02	changed=2	stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay
76:Z00	changed=2	stage:checker-tier-qemu-device-streams stage:checker-tier-research-build-and-replay
```

处置：拷入仓根 `.gitignore`、删掉副本里全部 `__pycache__`、重新提交副本，全部样本带 `PYTHONDONTWRITEBYTECODE=1`、`python3 -B` 重跑（1.3 节是重跑那一份，首尾两个 Z00 都是 `changed=0`）。旧的那份只作这一节的证据，结论一律不引它。
它说明的事：登记了整个 `scripts/` 的两道（`stage-inputs.tsv:37`、`:48`）对 `scripts/` 下 git 列得出的任何文件都敏感，而 git 列不列只看忽略规则——新 clone、别的工作树、`.gitignore` 改一行，键就可能变。这是 L19、L25 的又一种 ③⑤ 形态（推的：主仓今天有 `.gitignore:26` 挡着，没在主仓造过）。

### 1.5 研究复跑按行键（`tools/replay_probe.py`，`admission.py replay-row-inputs <副本> E23 E140`，`replay-runs-clean.out` 原样）

```
R00 E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192=fb062c6a22c2 E23|e23_journal_geom|=f5e963857c7e
R01 E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192=fb062c6a22c2 E23|e23_journal_geom|=c8d31df93090
R02 E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192=fb062c6a22c2 E23|e23_journal_geom|=f5e963857c7e
R03 E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192=fb062c6a22c2 E23|e23_journal_geom|=b50f6a3eb355
R04 E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192=5b9198c0ad99 E23|e23_journal_geom|=a5e26333b31d
R05 E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192=5ccbefabe752 E23|e23_journal_geom|=09913560228f
R00 E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192=fb062c6a22c2 E23|e23_journal_geom|=f5e963857c7e
```

R01–R05 的改动见第 2 节。旁证（量的）：`grep -c "Lru\|aes_gcm" scripts/experiments/e23_journal_geom.rs` 输出 `0`，`grep -c "aes_gcm" scripts/experiments/e140_header_alignment.rs` 输出 `3`；E23 从共用库只 `use e7_index_bench::Emitter;`（`e23_journal_geom.rs:28`）。

### 1.6 HEAD 移动（副本里提交，`scripts/gate/change-touches-crates.sh <副本> crates/`；退 0 = 碰了、退 1 = 没碰）

```
base HEAD=46cff5a
step=edit_uncommitted rc=0
HEAD=44d061f
step=edit_committed rc=1
HEAD=341de1c
step=empty_commit rc=1
== ctc-1
碰了：1 个路径落在 crates/ 底下，第一个是 crates/singlefs-checker-tier/src/crash.rs
== ctc-2
没碰：与基 HEAD 之间一个改动都没有（干净工作树；要重新验 HEAD 用 SINGLEFS_GATE_FULL=1）
== ctc-3
没碰：与基 HEAD 之间一个改动都没有（干净工作树；要重新验 HEAD 用 SINGLEFS_GATE_FULL=1）
```

步骤：在 base 上施加 S01 不提交（rc=0 碰了）；把 S01 提交（HEAD 带着改动前移，rc=1 没碰——它判不出「上一次判过的是哪份内容」，只看 HEAD）；还原 S01 并提交，再空提交一次（HEAD 前移、内容与 base 相同，rc=1 没碰，这一格判对了）。另量了整道键不随空提交变：同一副本上空提交前后 `admission.py stage-fingerprint <副本> checker-tier-qemu-device-streams.sh` 两次都打 `7e80cc955d772fb775577d78d430726f0270c12bf7ebf8a854bc7e28b790cb22 1115`（那一次在污染还没清的副本上，只用来比前后，不与 1.1 节基线比）。

## 2 样本目录

### 2.1 进了探针的样本（`tools/samples.py`、`tools/replay_probe.py`；「文件:行」是主工作区现在的行号，1.2 节核过）

前缀 `CT` = `crates/singlefs-checker-tier/`，`CK` = `crates/singlefs-checker/src/lib.rs`，`MR` = `CT/tests/crash_enumeration_multi_record_publish_stream.rs`，`TS` = `CT/tests/crash_enumeration_tree_split_streams.rs`，`LIT` = `litmus/new-pool-file-creation-root-implies-units.litmus`，`RP` = `crates/singlefs-replay-prefix/src/lib.rs`。

| 号 | 文件:行 | 改成什么 | 类别 |
|---|---|---|---|
| S01 | `CT/src/crash.rs:1198`（`judge_crash_image_against_the_version_table` 体内） | `JournalPolicy::Ignore` → `JournalPolicy::Consult` | 判器函数体改一个字 |
| S02 | `CT/src/crash.rs:255`（`rollback_floor_written_by`） | `.expect("130")` → `.expect("131")` | 同 crate 下游函数 |
| S03 | `CK:814`（`check_journal_record` 体内） | `crc32_castagnoli_bitwise` → `crc32_castagnoli_table`（结果相同、词不同） | 跨 crate 下游函数 |
| S04 | `CK:712` | `JOURNAL_PAYLOAD_CHECKSUM_OFFSET` 95 → 91（E164 V5 那一行，`crates/mutations.tsv:439`） | 常量（文件里有非测试 fn） |
| S05 | `crates/singlefs-format/src/lib.rs:86` | `DATA_POINTER_BYTES` 88 → 89（E164 V11，`crates/mutations.tsv:315`） | 常量（文件里没有非测试 fn） |
| S06 | `CT/src/crash.rs:251` | `36 + NODE_POINTER_BYTES + 8` → `+ 9`（常量引常量，跨 crate） | 常量 |
| S07 | `CK:97-98`（`enum Verdict`） | `Valid,` 与 `BadMagic,` 对调 | 类型定义 |
| S08 | `CK:746-747`（`struct JournalRecordView`） | `instance` 与 `counter` 两个字段对调 | 类型定义字段 |
| S09 | `CT/src/crash_judge_recovery.wgsl:299`（`classify_oracle`） | `<` → `<=` | wgsl 内核函数 |
| S10 | `CT/src/crash_judge_common.wgsl:164` | `MAX_DATA_POINTERS` 256u → 255u | wgsl 常量 |
| S11 | `crates/singlefs-checker/src/image.rs:113-114`（`IMPLEMENTED_INVARIANTS`） | 头两项对调 | 常量数组（GPU 结论位序也靠它，`crash_judge_gpu.rs:84`、`:136`、`:189`） |
| S12 | `CT/src/crash_judge_dispatch.rs:955`（`judge_one_dispatch` 回读归位） | `entries[state * …` → `entries[(state) * …` | GPU 派活回读 |
| S13 | `Cargo.lock:453`（naga） | 30.0.1 → 30.0.2 | 依赖版本（只有 GPU 判器用） |
| S14 | `Cargo.lock:361`（libc） | 0.2.189 → 0.2.190 | 依赖版本（core 文件后端 `block_device.rs`） |
| S15 | `Cargo.lock:1009`（vstd） | `0.0.0-2026-09-20-0158` → `0.0.0-2026-09-27-0158` | 依赖版本（编译用；验证用 Verus 自带的） |
| S16 | `Cargo.toml:20`（`[profile.release]`） | `overflow-checks = true` → `false` | 构建环境 |
| S17 | `MR:232`（观察者 `observe_the_publish_boundary`，`:222` 起） | `…, 1);` → `…, 2);` | 观察者函数体 |
| S18 | `MR:157` | `content_needing(3, 2)` → `content_needing(3, 4)`（写 C 的内容） | 流的写内容 |
| S19 | `CT/src/crash_facts.rs:1160`（`verify_state_from_facts`） | 方法调用改成全限定写法 | 核对函数体 |
| S20 | `CK:420`（`check_unit`） | `unit.len() < 42` → `< 43` | 跨 crate 下游函数（核对与 CPU 都调） |
| S21 | `CT/src/crash_facts.rs:382`（`extract_flow_facts`） | 调用加 `self::` | 事实抽取（录入） |
| S22 | `CT/src/crash_judge_gpu.rs:702`（`pack_tables`） | `tables.len() * 2` → `+ 0` | 判表打包（录入） |
| S23 | `CT/src/crash.rs:2409`（`quick_tier_expansion`） | 展开方式换 `EveryProperSubset` | 段展开 |
| S25 | `crates/singlefs-core/src/recovery.rs:1287`（`choose_root`，litmus 锚点 `LIT:13`） | 加一对括号 | 被锚定的函数 |
| S27 | `LIT:11` | `singlefs-expect: Never` → `Sometimes`（这一行在 `(* *)` 里） | litmus 期望 |
| S28 | `LIT:22` | `WRITE_ONCE(*journal, 1)` → `2` | litmus 体 |
| S29 | `scripts/gate/admission.py:2234`（`judge_worker_threads`） | `count_line` → `count_line.strip()` | 判日志的函数 |
| S30 | `scripts/mutation/crates-mutation-rows.py:1269`（`verdict_of_run`） | `output_tail(output)` → `output_tail(output[:])` | 判结局的函数 |
| S32 | `RP:3985`（`choose_root_slot`） | `= 0;` → `= 0usize;` | 被验函数体 |
| S33 | `.claude/gate.d/checker-tier-crash-point-replay.sh:495`（`report_crash_case_findings`） | `grep -c` → `grep -c --` | 阶段里判发现的 shell 函数 |
| S34 | `CK:326`（`check_root_slot`） | `.expect("457")` → `"458"` | 被变异函数的别一行 |
| N01 | `CT/src/crash.rs:1211` 之后 | 判器函数体里加一行 `//` 注释 | .rs 注释 |
| N02 | `CT/src/crash.rs:249` | 文档注释改几个字 | .rs 文档注释 |
| N03 | `CT/src/crash_judge_recovery.wgsl:12` 之后（`recover` 体内） | 加一行 `//` 注释 | wgsl 注释 |
| N04 | `CT/src/crash.rs:261` 之后 | 加一个 `#[allow(dead_code)]` 没人调的函数 | 同文件加函数 |
| N05 | `CT/src/crash_judge_recovery.wgsl` 末尾 | 加一个没人调的 wgsl 函数 | 同文件加函数（wgsl） |
| N06 | `CT/src/crash.rs:2786`（`mod tests {`）之后 | 加一条空单测 | 测试模块加单测 |
| N07 | `crates/singlefs-core/Cargo.toml:9` | 注释里加几个字 | .toml 注释 |
| N08 | `Cargo.lock:548`（objc2-metal，只在 macOS 目标进依赖图） | 0.3.2 → 0.3.3 | lock 无关包 |
| N09 | `Cargo.lock:431`（lz4-sys，只有 RocksDB 存储用） | 1.11.1 → 1.11.2 | lock 无关包 |
| N10 | 目录 `crates/singlefs-format` → `crates/singlefs-on-disk-format`，连带 `Cargo.toml:6` 与四份 `Cargo.toml` 的 `path =`（core `:11`、checker `:9`、checker-tier `:11`、harness `:11`） | 内容不变 | 目录改名 |
| N11 | `TS` → `CT/tests/crash_enumeration_tree_split_flows.rs`，连带 `TS:46` 的 `THIS_FILE` | 内容不变 | 文件改名 |
| N12 | `CT/tests/common_crash_points/mod.rs:78` | 崩溃点名 `"sequential_write"` → `"sequential_overwrite"` | 崩溃点改名 |
| N13 | `MR:312` | 观察者名 `"publish_boundary"` → `"boundary_of_the_publish"` | 观察者改名 |
| N14 | `TS:504` | 项名 `"tree_split_{stream:?}"` → `"tree_split_flow_{stream:?}"` | 项改名 |
| N15 | `CT/src/bin/e156_allocation_basis_counts.rs:703` | 给定状态项名加后缀 | 项改名 |
| N16 | `CT/src/crash_judge_dispatch.rs:31` | `LONGEST_DISPATCH_SECONDS` 0.5 → 0.4（只管快慢） | 只改快慢 |
| N17 | `CT/src/crash_judge_gpu.rs:504` | `GPU_JUDGE_REVISION` 1 → 2，代码不动 | 只抬手写号 |
| N18 | `CT/src/crash_amplification.rs:14060` | `OBSERVER_CONCLUSION_COMPOSITION_VERSION` 2 → 3，代码不动 | 只抬手写号 |
| N19 | `TS:479`（别的测试 `the_accounting_streams_read_the_tree_height_from_the_root_node_header`，`:477` 起） | `2` → `2 + 0` | 同设点文件里别的测试 |
| N20 | `crates/singlefs-core/src/transaction.rs` 末尾 | 加一个没人调的函数 | 同文件加函数 |
| N21 | `CT/src/crash_facts.rs:1160` 之后 | 核对函数体里加一行注释 | .rs 注释（被 `include_str!` 读的文件） |
| N22 | `.claude/gate.d/stage-inputs.tsv:50`（E142 那一行） | 多登记一条路径 | 登记表别的行 |
| N23 | `.claude/gate.d/stage-inputs.tsv:34` | 注释行加几个字 | .tsv 注释 |
| N24 | `scripts/gate/admission.py:2234` 之后 | 判法函数体里加一行 `#` 注释 | .py 注释 |
| N25 | `.claude/gate.d/checker-tier-crash-point-replay.sh:433` 之后 | 阶段函数体里加一行 `#` 注释 | .sh 注释 |
| N26 | `scripts/hosts/run-with-memory-cap.sh:416` 之后 | 内存包装函数体里加一行注释 | .sh 注释 |
| N27 | `scripts/gate/admission.py:338` | `INPUT_KEY_FORM_VERSION` "2" → "3"，代码不动 | 只抬手写号 |
| N28 | `research/results/` 下一份产物（副本里是桩） | 末尾加一行 | 无关产物 |
| N29 | `.claude/kb/layout/01-first-txn.md` 末尾 | 加一句 | 文档 |
| N30 | `scripts/kb/insert-row.py:1` 之后 | 加一行注释 | .py 注释（无关脚本） |
| N31 | `LIT:4`（`(* *)` 里的说明文字） | 加几个字 | litmus 注释 |
| N32 | `CT/src/bin/e170_o3_spec_executor_export.rs` 末尾 | 加一个没人调的函数 | 无关 crate 文件 |
| N33 | `scripts/mutation/crates-mutation-rows.py:306` | 每条限时 1800 → 3600 | 限时 |
| N34 | `crates/mutations.tsv:439` 变异名那一列 | 变异名改字 | 变异改名 |
| N35 | `scripts/mutation/crates-mutation-rows.py:1269` 之后 | 判结局函数体里加一行注释 | .py 注释 |
| N37 | `CK:18` 之后 | 加一条没人用的 `use` | 同文件加项 |
| N38 | `RP:3984` 之后（`choose_root_slot` 体内） | 加一行注释 | .rs 注释（Verus 核） |
| N39 | `RP:1196` 之后（`:1197` 是 `requires`） | 加一行空行 | 行号挪位 |
| N40 | `.claude/gate.d/verus-pins.tsv:2` | 注释行加几个字 | .tsv 注释 |
| N41 | `scripts/gate/verus-proof-check.py:788` 之后（`judge_version`） | 加一行注释 | .py 注释 |
| N42 | `.cargo/config.toml:2` | 注释行加几个字 | .toml 注释（构建配置） |
| N43 | `CT/src/crash_facts.rs` 末尾 | 加一个没人调的函数 | 同文件加函数（被 `include_str!` 读的文件） |
| N45 | `CK:320` 之后 | 加一个没人调的函数 | 同文件加函数 |
| R01 | `scripts/experiments/e23_journal_geom.rs:104`（`simulate_journal_run`，`:102` 起） | `= 0;` → `= 0u64;` | 装置函数体 |
| R02 | `scripts/experiments/e23_journal_geom.rs:103` 之后 | 加一行注释 | .rs 注释 |
| R03 | `scripts/experiments/replay.sh:747`（E23 那一行） | 产物文件名改字 | 产物改名 |
| R04 | `scripts/experiments/lib.rs:343`（`Lru::new`，E23 不用） | 断言消息改字 | 共用库里别的函数 |
| R05 | `scripts/experiments/Cargo.lock:28`（aes-gcm，E140 用、E23 不用） | 0.11.1 → 0.11.2 | 依赖版本 / lock 无关包 |
| H1–H3 | 副本 git：S01 不提交 / S01 提交 / 还原后空提交 | 见 1.6 节 | HEAD 移动 |

### 2.2 没进探针的样本（行号这一轮 `sed -n <行>p` 现取；环境类没有行号）

| 号 | 文件:行 | 改成什么 | 类别 |
|---|---|---|---|
| T01 | `MR:193`（版本表里 A 那一版的期望内容） | `file_content()` → 别的内容 | 判器另读的输入 |
| T02 | `MR:389`（全量用例 `full_enumeration_of_the_multi_record_publish_stream_is_exhaustive_and_clean`，`:387` 起） | 函数体改一个字 | 用例测试函数体 |
| T03 | `CT/tests/crash_injection_campaign.rs:417` | `seed_count: 2` → `3` | 给定状态多一个种子 |
| T04 | `CT/src/crash_amplification.rs:14066`（`observer_versions_to_write_under` 起头那一行；改它体内任一处） | 函数体改一个字 | 观察者拼块函数 |
| T05 | `CT/src/crash_amplification.rs:12095`（`block_key_of_the_table` 里的输入串） | 拼法改一处 | 算键函数自己 |
| T06 | `CT/src/crash_identity.rs:604` | `"singlefs conclusion version 1"` → `2`，代码不动 | 只抬手写号 |
| T07 | `CT/src/crash_library.rs:866` 起（`facts_through_the_pairing_library`） | 函数体改一个字 | 对拍比较 |
| T08 | `CT/src/crash_verify_gpu.rs:363` 起（`gpu_verifier_digest` 与它收的着色器） | 着色器里一个 fn 体改一个字 / 只加注释 | GPU 核对 |
| T09 | `CK:58`（`crc32_castagnoli_table` 体内） | 改一个字 | 跨 crate 下游函数 |
| T10 | `CT/src/on_device_modes.rs:290`（`publish_the_fourth_version`）、`:328`（`raise_the_rollback_floor_to_its_ceiling` 体内）、`CT/src/bin/new_pool_file_creation_on_device.rs:396`（`swallow_the_next_barrier_on`） | 各改一个字 | 虚机档执行到的函数 |
| T11 | `litmus/commit-publish.litmus:13` | 不改（`singlefs-models: none`，用来定 S25 的作废面） | litmus 锚点 |
| T12 | `RP:107`（spec fn `is_last_record_of_its_publish`）、`RP:3103`（`record_ends_its_publish` 的 ensures）、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:15`（替换文） | 各改一个字 | Verus 单元 / 必红世界行 |
| T13 | `.claude/gate.d/e165-pinned-red-examples.tsv:26` 红例账那一列（`…segment2-red-examples.tsv:2`）、`CT/src/bin/e165_replay_bounded_enumeration.rs:1662`（`record_is_completely_on_disk`，`:1661` 起） | 账路径改名 / 装置函数体改一个字 | 红例钉表 |
| T14 | `scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv:1`（M1 的名与替换文）、`:2`（M2 整行）、`scripts/experiments/e142_new_pool_file_creation_dry_run.rs:3490`（M4 原文所在的 `read_unit_via_locations`，`:3484` 起）的别一行、`:477`（`castagnoli_crc32` 体内） | 改名 / 替换文改字 / 别行改 / 函数体改字 | 实验变异表 |
| T15 | `scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:88`（库目录带轮名）、`:89`（schema 手写串）、`:546`（`row {row.number}`）；同目录 `bad-implementations-expansion-164.tsv:1`；`CT/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:2200`（`first_batch_points`，`:2199` 起） | 轮名改 / schema 抬号 / 表上方插一行 / 替换文改字 / 函数体改字 | E171 修订 11 |
| T16 | `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:142`（辅助函数 `build_pool`，`:141` 起） | 函数体改一个字 | 抓到它的测试调到的辅助函数 |
| T17 | `crates/singlefs-core/src/admission.rs:966`（`macro_rules! enum_listing_every_member`）同文件里一个与宏无关的函数 | 改一个字 | 宏定义文件里的别处 |
| T18 | `scripts/experiments/lib.rs:73`（`Emitter::emit` 体内，`:72` 起） | 改一个字 | 共用库里被用到的函数 |
| T19 | `.claude/kb/decisions/08-核心索引结构.md`（登记在 `stage-inputs.tsv:50`） | 改一句 | kb 决策文档 |
| T20 | `scripts/crash/crash-point-replay-shard-run.sh:77`（`run_merged_log` 体内，`:76` 起） | 改一个字 | 双机驱动里用例走到的函数 |
| T21 | `scripts/kb/insert-row.py:82`（`anchor_hints`） | 函数体改一个字 | 研究脚本自证走到的函数 |
| E01 | 工具链：`rustc -V` 现为 `rustc 1.98.0 (88d9e12ae 2026-08-18)`（这一轮现跑） | 换一版 | 工具链 |
| E02 | 构建环境：`RUSTFLAGS` 加 `-C target-cpu=native` | — | 构建环境 |
| E03 | GPU 驱动版本换一版 | — | 驱动（块属性） |
| E04 | 判同一批块的卡由 RTX 5090 换成 RTX 5060 Ti（本机两张都在，资源钩子这一轮打的） | — | 卡型（块属性） |
| E05 | `qemu-system-x86_64` 换一版；E06：来宾内核镜像或 initramfs 换内容（`scripts/devices/vm-kernel.sh` 取 `/boot` 或 `SINGLEFS_KERNEL`） | — | 虚机工具与镜像 |
| E07 | herd7 换一版；E08：内核树 `tools/memory-model` 的 `.cat` / `.bell` 改一处 | — | LKMM 工具与模型 |
| E09 | `.claude/gate.d/verus-pins.tsv:18`（release 行）换一版 Verus | — | Verus 版本 |
| E10 | 每条变异内存上限 `DEFAULT_MEMORY_MAX_PER_MUTATION`（`crates-mutation-rows.py:301`）4G → 8G，或 `GATE_MUTATION_MEMORY_MAX`；E11：`GATE_VERUS_TIMEOUT_SECONDS` 600 → 1200（`verus-proof-check.py:37`）；E12：`.claude/heavy-thread-default` 改数 | — | 限额与线程 |
| E13 | 什么都不改，同一条命令连跑两趟 | — | 重跑 |
| E14 | 同一条流先经崩溃点重放判、再经崩溃放量判 | — | 两个入口 |
| E15 | 库子目录名：`scripts/crash/crash-amplification-lib.sh:113`、`CT/src/crash_library.rs:131` 改名 | — | 库改名 |
| E16 | 用例改名：`stage-inputs.tsv:57` 的 `crash-case:multi-record-publish-stream` 与 `.claude/gate.d/crash-point-replay-items.tsv:17` 一起改 | — | 用例改名 |
| E17 | QEMU 先 `--item direct` 跑一趟，再整道跑 | — | 点名跑 |
| E18 | 库里只有某块的 GPU 结论，这一趟要的是 CPU 结论 | — | 判器种类 |
| E19 | 在判器调到的函数里加 `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/…"))` | — | 闭包算法认不出的读法 |
| E20 | 往 `CT/tests/` 加一份新测试文件（与正在跑的项无关） | — | 新文件 |
| E21 | feature-gated-tests 格的特性集合多一个特性 | — | 特性组合 |
| E22 | 研究复跑切 `ENABLE_GPU`（`scripts/experiments/replay.sh:55` 自认不进键） | — | GPU 开关 |

### 2.3 覆盖表（派发提示点名的每一类由哪些样本担）

| 类 | 样本 |
|---|---|
| 必须作废·判器 / 判法函数体改一个字 | S01、S09、S29、S30、S32、S33、R01、T02、T07、T21 |
| 必须作废·它调到的下游函数 | S02、S03、S20、T09、T16、T18、T20 |
| 必须作废·引用的常量（E164 V11 形态） | S04（V5，文件里有非测试 fn）、S05（V11，文件里没有非测试 fn）、S06（常量引常量）、S10（wgsl 常量）、S11（常量数组） |
| 必须作废·类型定义字段 | S07（枚举变体次序）、S08（结构体字段次序） |
| 必须作废·wgsl 内核函数 | S09、S10、T08 |
| 必须作废·依赖版本抬 | S13、S14、R05（对 E140）、E05、E07、E09；S15 只对 verus-toolchain-pinned 那一格 |
| 必须不作废·只加注释 | .rs：N01、N02、N21、N38、R02；wgsl：N03；.py：N24、N30、N35、N41；.sh：N25、N26；.toml：N07、N42；.tsv：N23、N40；litmus：N31 |
| 必须不作废·同文件加一个没人调的函数 | N04、N05（wgsl）、N20、N43、N45；另 N37（没人用的 `use`）、N32（无关 crate 文件） |
| 必须不作废·测试模块加一条单测 | N06 |
| 必须不作废·改名 | 目录 N10；文件 N11；崩溃点 N12；观察者 N13；项 N14、N15；用例 E16；变异 N34；产物 R03；库 E15；账路径 T13；轮名与行号 T15 |
| 必须不作废·`stage-inputs.tsv` 别的行 | N22（另 N23 是它的注释行） |
| 必须不作废·`Cargo.lock` 无关包升级 | N08、N09；研究侧 R05（对 E23） |
| 必须不作废·内存上限或限时 | N33、E10、E11；线程数 E12（条件见 L18）；只改快慢 N16 |
| 必须不作废·提交 / HEAD 移动而内容不变 | H3（1.6 节）；对照 H2（内容带着 HEAD 一起变，必须作废） |

## 3 逐机制样本

每条机制先写新键下「一条判定」是什么，再列样本。「作废面」写新键下该作废的那几条判定（不是整库），「今天」写拿这个样本打今天的键的结果：量的指 1.3–1.6 节对应那一行。共用的缩写：CPU 结论 / GPU 结论 / 观察者结论 = 崩溃判定库里按块存的三种结论；核对行 = 第 ② 段核对表；录入产物 = 事实表、判表、计划与清单。

### L1 判器闭包按文件收

一条判定 = 一块的一种结论；键 = 那一种判器从入口 fn 起执行到的单元指纹集合 + 工具链 + 依赖版本。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S01 | CPU 判器本体变 | 全部 CPU 结论；GPU、观察者结论不动 | 量的：`cpu_mr cpu_ts gpu obs_mr` 四把全换（GPU、观察者多作废） |
| 阳 | S03 | 下游 `check_journal_record` 变 | 执行单元里有它的 CPU 结论与观察者结论里池级 checker 那一把（链：`crash.rs:1211` `check_pool_image` → `walk.rs:4162`、`:7678`，链是推的）；GPU 结论不动（GPU 内核不调 Rust 这一份，推的） | 量的：四把全换 |
| 阳 | S04（V5） | 下游函数引的 fn 外常量变 | 同 S03（常量单元经 `CK:813` 被引）；E164 量过点名测试由绿转红（`E7RESULT name=q4_flip variant=V5 base_exit=0 variant_exit=101 green_to_red=true`，`.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md:64`） | 量的：四把全换 |
| 阳 | S05（V11） | 没有非测试 fn 的文件里的常量 | 执行单元里有 `position_addressed.rs:270` `extent_upper_leaf_entry_view`（读它在 `:278`）的 CPU 结论；GPU 不动（`grep -rn DATA_POINTER_BYTES crates/singlefs-checker-tier/src` 无命中，量的；后果推的） | 量的：四把全换 |
| 阳 | S07 | 类型定义变 | 执行单元引 `Verdict` 的 CPU 结论与核对行 | 量的：四把全换 |
| 阳 | S13 | GPU 用的 naga 抬版本 | 全部 GPU 结论与 GPU 核对行；CPU、观察者不动 | 量的：四把全换（CPU、观察者多作废） |
| 阳 | E01 | 工具链 | 全部结论 | 推的（版本消息带 `rustc=`、`cargo=`、`target=`：`crash_identity.rs:647-648`） |
| 阴 | N01、N02 | 只加注释 | 无 | 量的：`changed=0`（对） |
| 阴 | N04 | 判器文件加没人调的函数 | 无 | 量的：四把全换 |
| 阴 | N06 | 测试模块加单测 | 无（`#[cfg(test)]` 项不进单元） | 量的：四把全换 |
| 阴 | N20、N45 | 别的 crate 文件加没人调的函数 | 无 | 量的：都是四把全换 |
| 阴 | N07、N08、N09 | 清单注释、lock 无关包 | 无 | 量的：都是四把全换 |

### L2 wgsl 与 `include_str!` 文件按原文

一条判定 = 一块的 GPU 结论（或 GPU 核对行）；内核单元 = wgsl 的 `fn` 去注释；入口是 Rust 串里拼出来的 `@compute fn main`（`crash_judge_gpu.rs:565`），所以切分要对「前置常量 + 内核文件 + 入口串」拼好的整份源码做。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S09 | 恢复内核里的 `classify_oracle` 变 | 调用图里有它的那几种内核判的 GPU 结论；CPU 不动 | 量的：只换 `gpu`（对） |
| 阳 | S10 | wgsl 常量变 | 调用图里有引它的函数（`crash_judge_common.wgsl:2811`、`:2927`，`crash_judge_checker.wgsl:4157`）的内核判的 GPU 结论 | 量的：只换 `gpu`（对） |
| 阳 | S11 | Rust 常量数组定 GPU 结论位序 | GPU 与 CPU 结论都作废 | 量的：四把全换（对） |
| 阳 | S22 | 判表打包 `pack_tables` 变 | 录入产物里的 GPU 判表，与用它判的 GPU 结论 | 量的：只换 `gpu` |
| 阳 | T08 | GPU 核对着色器的 fn 体 | GPU 核对行 | 推的（今天整份着色器原文进键：`crash_verify_gpu.rs:361-367`） |
| 阴 | N03 | wgsl 只加注释 | 无 | 量的：`gpu` 换 |
| 阴 | N05 | wgsl 加没人调的函数 | 无（不在任何入口调用图里） | 量的：`gpu` 换 |
| 阴 | N21 | 被 `include_str!` 读的 `crash_facts.rs` 只加注释 | 无 | 量的：6 道整道键换；核对键拿它整份原文（`crash_amplification.rs:3937-3939`，这一把探针没算，推的） |
| 阴 | N43 | 同一份加没人调的函数 | 无 | 量的：四把全换 |
| 阴 | T08 的注释变体 | 着色器只加注释 | 无 | 推的（同上原文进键） |

### L3 路径与名字进键

一条判定同 L1；块键只由内容定（段写表与起步镜像摘要、段展开、判器另读输入、段内序号），路径与名字只进索引表。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S18 | 写 C 的内容变 | C 所在段与其后各段的块（段写表或起步镜像变了）；暖机、A、B 段的块不动（推的：起步镜像只依赖之前的写） | 量的：`cpu_mr obs_mr` 两把结论版本换（版本本不该随写内容变；块键随写表变是对的，第一轮 C11） |
| 阳 | S23 | 段展开方式变 | 用快档展开的那些段的块（块键里有段展开） | 量的：四把结论版本全换（展开是排计划的函数，不是判器，版本不该动） |
| 阳 | T01 | 判器另读的期望版本变 | 这条流里 judge_inputs 含它的那几段的 CPU 结论 | 推的 |
| 阴 | N10 | crate 目录改名 | 无 | 量的：四把全换 |
| 阴 | N11 | 设点文件改名（连 `THIS_FILE`） | 无 | 量的：`cpu_ts` 换；节点名带设点文件路径（`crash_identity.rs:71-78`，代码；块键换是推的） |
| 阴 | N12 | 崩溃点改名 | 无 | 量的：`cpu_mr cpu_ts obs_mr` 换；节点名带崩溃点名（同上） |
| 阴 | N13 | 观察者改名 | 无 | 量的：`cpu_mr obs_mr` 换；观察者种类串带名（`crash_identity.rs:576`） |
| 阴 | N14、N15 | 项改名 | 无 | 量的：N14 `cpu_ts` 换；N15 给定状态块键带 `item=`（`crash_amplification.rs:12095`，代码；后果推的） |

### L4 `Cargo.lock` 与各 `Cargo.toml` 整份原文

一条判定同 L1；依赖版本 = 判器单元真引到的外部包（及传递依赖）从 lock 解析出的名、版本、来源、checksum；清单只取生效的特性、edition、profile。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S13 | naga 抬版本 | 全部 GPU 结论与 GPU 核对行 | 量的：CPU、观察者也换（多） |
| 阳 | S14 | libc 抬版本 | 执行单元落到 libc 的判定：core 文件后端（`crates/singlefs-core/Cargo.toml:9` 写的用途）走到的测试与虚机档；崩溃判定库的结论不动（判器跑在内存池上，推的） | 量的：四把结论版本全换（多） |
| 阳 | S16 | release profile 关掉溢出检查 | 全部编出来跑的判定（结论、门禁测试、变异、复跑） | 量的：四把结论版本**都不换**（漏判：根 `Cargo.toml` 的 profile 不进结论版本，`judging_closure.rs:993` 只收各 crate 清单与 lock）；整道键 5 道换 |
| 阳 | E02 | RUSTFLAGS 变 | 同 S16 | 推的 |
| 阴 | N07 | crate 清单注释 | 无 | 量的：四把全换 |
| 阴 | N08 | 只在 macOS 进依赖图的包 | 无（linux 目标解析出的闭包里没有它） | 量的：四把全换 |
| 阴 | N09 | 只有 RocksDB 存储用的包 | 无（判器单元不引 rocksdb） | 量的：四把全换 |
| 阴 | N42 | `.cargo/config.toml` 注释 | 无 | 量的：结论版本不换（对）；8 把门禁键换 |

### L5 设点文件 / 观察者文件整份闭包

一条判定 = 一项的观察者结论或 CPU 结论；入口是这一项真执行的测试 fn 与观察者 fn 单元，不是整份文件。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S17 | 观察者函数体变 | 多记录流那几项的 `publish_boundary` 观察者结论；CPU、GPU 结论不动 | 量的：`cpu_mr obs_mr` 换（CPU 多） |
| 阳 | T01 | 期望版本（判器读的输入）变 | 那一项的 CPU 结论 | 推的 |
| 阳 | S04 | 池级 checker 用的常量变 | 每个观察者结论里池级 checker 那一把（第一轮 C18） | 量的：`obs_mr` 换（对） |
| 阴 | N19 | 同设点文件里别的测试改 | 无 | 量的：`cpu_ts` 换 |
| 阴 | 第一轮的设点文件样本 | 设点文件末尾加没人调的函数（第一轮 crates 路 `scenarios.out` 的 `unused_function_appended_to_the_setting_file`） | 无 | 量的（第一轮）：CPU、观察者两把换 |
| 阴 | N13、N12 | 观察者改名、共用设点辅助文件里改崩溃点名 | 无 | 量的：见 L3 |

### L6 块按整点 / 整条流切，给定状态表一张表一块

一条判定 = 一段（或一态）的结论；块键 = 段写表与起步镜像摘要 + 段展开 + 判器另读输入 + 段内序号。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S18 | 中段一次写变 | C 段与其后各段；之前各段不动 | 推的（今天崩溃枚举用例整条流一点：`crash_state_enumeration.rs:745-754`，整条流换键） |
| 阳 | S23 | 展开方式变 | 快档展开的段 | 推的 |
| 阳 | 夹具 F1（自造一张三态的给定状态表，只改中间那一态的持久集合） | 一态的输入变 | 只那一态（或它那一段）的块 | 推的（今天一张表一块，计划串带 `states=`：`crash_amplification.rs:12136`） |
| 阴 | T03 | 多一个种子 | 已有种子那几张表的块全部沿用，只判新种子的表 | 推的：今天 `seeds_and_scale=` 进块键（`crash_injection_campaign.rs:81-91` 拼、`crash_amplification.rs:12095` 收），全换 |
| 阴 | E14 | 同一条流两个入口各判一次 | 第二个入口全部沿用 | 量的（底稿一之二：两边块键、输入指纹、计划哈希交集都是 0） |
| 阴 | N14 | 项改名 | 无 | 量的：见 L3 |

### L7 手写版本号

一条判定同 L1；「写法」由算键与判结局那几个函数单元的指纹代替，手写号删掉。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S12 | 派活里读回、按序号归位的代码变 | 全部 GPU 结论 | 量的：`gpu` **不换**（漏判，第一轮 C20） |
| 阳 | T04 | 观察者拼块函数变 | 全部观察者结论 | 推的：拼块代码在停止文件里，第一轮量过往 `crash_amplification.rs` 加函数三把都不动 |
| 阳 | S21 | 事实抽取函数变 | 全部事实表（录入产物）与按它核的核对行 | 量的：四把结论版本全换（GPU 也换，多）；事实表键今天另靠手写号 `crash_facts.rs:35` |
| 阳 | T05 | 给定状态算键函数变 | 全部给定状态块 | 推的：今天靠手写串 `given-states-input-v3` |
| 阴 | N17 | 只抬 `GPU_JUDGE_REVISION` | 无 | 量的：`gpu` 换 |
| 阴 | N18、T06 | 只抬观察者拼块号、结论版本头 | 无 | 量的 N18：结论版本不换、6 道整道键换；T06 推的（`crash_identity.rs:604` 进版本消息） |
| 阴 | N16 | 只改派活快慢的常量 | 无 | 量的：`gpu` 不换（对；今天对是因为整份派活文件都不进键，改后要 S12 换而 N16 不换） |

### L8 核对复用缺一格、核对键不含跨 crate 调到的函数

一条判定 = 一块的核对行；键 = `verify_state_from_facts`（`crash_facts.rs:1159`）起执行到的单元指纹（跨 crate）+ 存着的事实表摘要。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S19 | 核对函数体变 | 全部 CPU 核对行 | 推的：核对键拿 `crash_facts.rs` 整份原文（`crash_amplification.rs:3937-3939`），会换（对） |
| 阳 | S20 | 核对调到的 `singlefs_checker::check_unit`（`crash_facts.rs:20` 引进来）变 | 全部 CPU 核对行，加执行单元里有它的 CPU 结论 | 推的：核对键不含 `singlefs-checker`，不换（漏判，第一轮 C24）；结论版本量的四把全换 |
| 阳 | T09 | `crc32_castagnoli_table` 变 | 同 S20 | 推的，同 S20 |
| 阳 | S21 | 抽取变了、事实表跟着变 | 存着的事实表摘要与这一趟读回的对不上的核对行 | 推的：今天只问（块键，核对代码摘要）在不在（`crash_amplification.rs:4047`），照沿用（第一轮 C26） |
| 阴 | N21 | 核对文件只加注释 | 无 | 推的：核对键换（原文）；整道键量的换 |
| 阴 | N43 | 同文件加没人调的函数 | 无 | 同上 |
| 阴 | `crash_facts.rs:1259`（`mod tests {`）之后加一条单测 | 测试模块加单测 | 无 | 推的：整份原文换 |

### L9 编译时整张摘要表与盘上整表比

一条判定 = 这一趟要写的那几项各自的结论；只比这一趟要用的那几把，表的行按登记的项与入口 fn 单元列。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S01 | 判器变 | 这一趟各项的 CPU 结论不沿用旧的，现判、写回 | 推的 |
| 阳 | S17 | 一个观察者变 | 只那一项的那个观察者结论 | 推的 |
| 阳 | E01 | 工具链变 | 全部 | 推的 |
| 阴 | N32 | 无关装置文件加函数 | 这一趟各项照常写库 | 推的：今天整表比（`crash_identity.rs:771`），任何一行不等整趟退 78 不写；整道键量的换 |
| 阴 | N19 | 别的项的设点文件里别的测试改 | 同上 | 推的，同上 |
| 阴 | E20 | 加一份新测试文件 | 同上 | 推的：表的行按目录列（`judging_closure.rs:1181-1262`，第一轮 C7） |

### L10 流水线单测的另一套判法版本、起跑清库、对拍结论不入库

一条判定 = 一条流水线单测、一次对拍的结论；键 = 两边判器单元指纹 + 输入摘要；空库起跑只在一次性副本里建库。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S01 | CPU 判器变 | 用 CPU 判器的流水线单测判定、对拍里 CPU 那一侧的结论 | 推的（`crash_amplification.rs:1644-1646`：那一套「判法版本」） |
| 阳 | S09 | GPU 内核变 | 对拍结论（GPU 那一侧） | 推的：今天对拍结论不存、每趟重判（第一轮 C33） |
| 阳 | T07 | 对拍比较函数变 | 全部对拍结论 | 推的，同上 |
| 阴 | E13 | 原样重跑 | 流水线单测与对拍全部沿用 | 推的：今天起跑清库（`crash_library.rs:186`）、对拍不存，全部重判 |
| 阴 | `CT/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1304` 之前加一行注释 | 测试文件注释 | 无 | 推的 |
| 阴 | N09 | lock 无关包 | 无 | 量的：四把结论版本全换 |

### L11 录入段每趟无条件跑

一条判定 = 一项的录入产物；键 = 录入函数单元指纹 + 流输入摘要，命中就跳过录入。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S18 | 那一项的流变 | 那一项的录入产物 | 推的 |
| 阳 | S21 | 抽取函数变 | 全部项的事实表 | 推的 |
| 阳 | S22 | 打包函数变 | 全部项的 GPU 判表 | 推的 |
| 阴 | E13 | 原样重跑 | 录入整段跳过 | 推的：今天 `scripts/crash/crash-amplification-item.sh:764` 无条件先跑录入 |
| 阴 | S09 | 只改 GPU 内核 | 录入产物全部沿用（只有 GPU 结论重判） | 推的 |
| 阴 | `scripts/crash/crash-amplification-item.sh:763` 注释改字 | .sh 注释 | 无 | 推的 |

### L12 判定库分六份、目录名带类名与存法名、名字当清单键

一条判定同 L1；库只有一份，类只当索引。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S01 | 判器变 | 那一份库里的 CPU 结论作废一次（同一批状态不按入口各作废、各重判一次） | 量的（底稿一之二：同一批状态在两份库里各判） |
| 阳 | S18 | 流变 | 那条流的那几段，一次 | 推的 |
| 阳 | E01 | 工具链变 | 全部，一次 | 推的 |
| 阴 | E15 | 库子目录改名 | 无（一条结论都不丢） | 量的（底稿一之二：孤立库 620M、271M 没人读） |
| 阴 | E14 | 两个入口判同一条流 | 第二个入口全部沿用 | 量的（同 L6） |
| 阴 | N14、E16 | 项、用例改名 | 无（清单、分片进度按内容摘要找得到） | 推的：今天清单键是流名（`verdict_store.rs:917-919`、`crash_library.rs:703`） |

### L13 GPU 与 CPU 不互借、驱动卡型不进键、judge_inputs、`include_str!(concat!…)`

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | E18 | 只有 GPU 结论、要 CPU 结论 | 这一块现判 CPU（不借） | 推的：今天种类进键（`verdict_store.rs:629-631`），对 |
| 阳 | T01 | 判器另读的输入变 | 只 judge_inputs 含它的那几段 | 推的：今天每点的 judge_inputs 链住之前全部写（`crash_amplification.rs:577`、`:718-719`），之后所有点换 |
| 阳 | E19 | 闭包算法认不出的读法 | 算键拒算、整趟判红（不许静默漏） | 推的：今天只认字面量串（`judging_closure.rs:498-500`），静默不收 |
| 阴 | E03、E04 | 驱动版本、卡型换 | 无（块属性，`verdict_store.rs:562-564`） | 推的：今天对 |
| 阴 | S09 对 CPU | GPU 内核变 | CPU 结论不作废 | 量的：`cpu_mr cpu_ts obs_mr` 不换（对） |
| 阴（条件） | S01 对 GPU | CPU 判器变 | GPU 判器入口的执行单元里没有 `crash.rs` 判器那几个函数时，GPU 结论不作废；有就作废（这一条的期望由新算法打出的 GPU 单元清单定，样本只核两者一致） | 量的：`gpu` 换 |

### L14 门禁整道键（整张登记表、整份阶段脚本、登记行原文、手写档位、目录前缀列文件、文件级摘要）

一条判定 = 门禁下面的一条测试函数、一条用例的一项判定、一份自证；键 = 它执行到的单元 + 工具链 + 依赖版本；登记表只答「从哪些入口起算」，整张表不进任何键。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S29 | 判日志的函数变 | 全部崩溃枚举用例的「日志判绿」那一项（块级结论不动） | 量的：`crash_case_judging` 换（对）；另 crash-point-replay、QEMU、研究复跑三道整道键换（后两道多） |
| 阳 | S33 | 阶段里判发现的 shell 函数变 | 全部用例的「发现日志」那一项 | 量的：只换 crash-point-replay 整道键（对，但粒度是整道） |
| 阳 | S04 | V5 | harness 测试 `new_pool_file_creation_publish` 的 `root_slots_system_configurations_and_journal_ring_hold_the_published_state`（`crates/mutations.tsv:439` 的 cargo 参数与测试名两列）与执行单元里有 `check_journal_record` 的别的测试 | 量的：6 道整道键全换（多） |
| 阳 | 夹具 F2：`scripts/gate/lib_source_digest.py` 里切单元的函数改一个字 | 算键函数自己变 | 用它算的全部键（写法就是这几个函数的单元指纹） | 推的 |
| 阴 | N22 | 登记表别的行 | 无 | 量的：6 道整道键换 |
| 阴 | N23 | 登记表注释 | 无 | 量的：6 道换 |
| 阴 | N25 | 阶段函数只加注释 | 无 | 量的：`changed=0`（对） |
| 阴 | N24 | 判法函数只加注释 | 无 | 量的：`crash_case_judging` 换 |
| 阴 | N27 | 只抬 `INPUT_KEY_FORM_VERSION` | 无 | 量的：8 把换 |
| 阴 | N32 | 无关装置文件加函数 | 无（没有一条门禁测试执行它） | 量的：6 道换 |
| 阴 | N30 | 无关脚本只加注释 | 无 | 量的：`changed=0`（对） |

### L15 树对比与按 HEAD 取改动

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | H2 | 改动带着 HEAD 一起前移 | 执行单元里有 S01 那个函数的判定（CPU 结论、走到它的测试） | 量的：`change-touches-crates.sh` 报「没碰」rc=1（漏判；整道能不能跳过另看 `refs/sop/staged-green` 那棵树，推的） |
| 阳 | H1 | 工作区改动没提交 | 同 H2 | 量的：rc=0（对） |
| 阳 | E01（树与 staged-green 那棵相同） | 只换工具链 | 全部编出来跑的判定 | 推的：`admission.py:1608` 没登记 `environment=` 的道比完路径就放行 |
| 阴 | H3 | 空提交，HEAD 前移、内容不变 | 无 | 量的：rc=1（对）；整道键空提交前后相同（1.6 节） |
| 阴 | 把 `refs/sop/staged-green` 改指一棵单元全同的树 | 只动 ref | 无（新键不读这个 ref，底稿第二节第 9 条 ④） | 推的 |
| 阴 | `git commit --amend` 只改提交说明 | 只动 HEAD | 无 | 推的（同 H3） |

### L16 构建环境与工具链按原文取

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | E01 | 工具链 | 全部编出来跑的判定 | 推的 |
| 阳 | E02 | RUSTFLAGS | 同上 | 推的（`admission.py:669-717` 收变量值，门禁这一侧对；结论版本那一侧不收，见 S16） |
| 阳 | S16 | profile | 同上 | 量的：结论版本不换（漏判）；整道 5 道换 |
| 阴 | N42 | `.cargo/config.toml` 注释 | 无 | 量的：8 把换 |
| 阴 | N26 | 内存包装脚本函数里只加注释 | 无 | 量的：`crates_mutation_judge_version` 换 |
| 阴 | `.cargo/config.toml:7` 两个 `-I` 互换次序 | 只影响 librocksdb-sys 的 bindgen | 执行单元不落到 librocksdb-sys 的判定都不作废 | 推的 |

### L17 崩溃枚举用例全绿标记

用例级标记整个删；一条判定 = 用例里每块的结论（归 L1–L12）加这条用例的「日志判绿」一项。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | T02 | 用例测试函数体变 | 只这一条用例的日志项（块是否重判看块键） | 推的：今天整个 `crates/` 进用例指纹（`admission.py:2158-2175`），全部用例换 |
| 阳 | S29 | 判日志的函数变 | 全部用例的日志项 | 量的：`crash_case_judging` 换 |
| 阳 | T20 | 双机驱动里用例走到的函数变 | 只登记了 `shard=across-machines` 的用例 | 推的：今天四份驱动整份进（`admission.py:326`） |
| 阴 | N24 | 判法函数只加注释 | 无 | 量的：`crash_case_judging` 换 |
| 阴 | E16 | 用例改名 | 无 | 推的：今天标记文件名带用例名（`admission.py:2213-2218`） |
| 阴 | N32 | 用例闭包外的 crate 文件 | 无 | 量的：crash-point-replay 整道键换（用例指纹同口径，推的） |

### L18 快档、模型对拍、research-unit-tests、research-script-selftests 没有库

一条判定 = 一条测试函数（或一份自证）；键 = 它执行到的单元 + 特性组合 + 工具链 + 依赖版本；只跑没命中的那几条。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S04 | V5 | L14 里点名的那条 harness 测试（E164 量过由绿转红）与执行单元里有 `check_journal_record` 的别的测试 | 推的：快档没有库，每趟全跑（`checker-tier-crash-point-replay.sh:433-441`） |
| 阳 | S05 | V11 | `pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for`（`crates/mutations.tsv:315` 必须红的测试名那一列；E164 量过 `E7RESULT name=q4_flip variant=V11 base_exit=0 variant_exit=101 green_to_red=true`，`164-…md:65`）与执行单元里有引它的函数的测试 | 同上 |
| 阳 | E21 | 特性组合变 | feature-gated-tests 格里带那个特性编的测试 | 推的 |
| 阳 | S14 | libc | 走到 core 文件后端的测试 | 推的 |
| 阳 | T21 | 研究脚本自证走到的函数变 | 只那一份脚本的自证 | 推的：今天每趟全跑（`code-tooling.sh:311`） |
| 阴 | N06 | 加一条单测 | 别的测试都不作废（只判新加那条） | 推的 |
| 阴 | N19 | 改测试 U | 测试 T 不作废 | 推的 |
| 阴 | E10 | 内存上限（例 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`） | 无 | 推的 |
| 阴（条件） | E12 | 线程数 | 断言不读线程数的测试不作废；日志判法核线程数的那几项（`admission.py:2225` `judge_worker_threads`）照常要现判 | 推的 |

### L19 QEMU 六档一个标记

一条判定 = 一档；键 = 宿主与来宾两侧执行到的单元 + QEMU 版本 + 内核镜像与 initramfs 内容哈希 + 工具链。档的步序看 `checker-tier-qemu-device-streams.sh:185-193`（各档不许出现的行）与 `:199-204`（各档的步序）。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | T10 `on_device_modes.rs:290` | 发布 fourth_transaction 的函数变 | 只 raise-rollback-floor 一档（只有它走到 fourth_transaction） | 推的：今天六档一把 |
| 阳 | `on_device_modes.rs:264`（`publish_the_third_version`）改一个字 | 发布 third_transaction 的函数变 | second-instance、raise-rollback-floor 两档 | 推的 |
| 阳 | T10 `new_pool_file_creation_on_device.rs:396` | 吞屏障的注入变 | 只 skip-new-pool-file-creation-barrier 一档 | 推的 |
| 阳 | E05、E06 | QEMU 版本、来宾内核或 initramfs 变 | 六档全部 | 推的：今天不进键（`stage-inputs.tsv:37` 条件那一列只有前提），漏判 |
| 阴 | N28 | 无关产物 | 无 | 量的：QEMU 整道键换 |
| 阴 | N29 | 布局文档 | 无 | 量的：QEMU 整道键换 |
| 阴 | N30 | 无关脚本注释 | 无 | 量的：`changed=0`（对：`.py` 按词法）；`.tsv` 一类按原文的无关文件改了照换（推的） |
| 阴 | E17 | 先 `--item direct` 再整道跑 | 整道那一趟 direct 沿用 | 推的：今天点名跑不写标记（`checker-tier-qemu-device-streams.sh:453-454`） |

### L20 LKMM 八份 litmus 一个标记

一条判定 = 一份 litmus 的 herd7 结论对它的 `singlefs-expect`；键 = litmus 去掉说明文字之后的内容（`singlefs-expect:`、`singlefs-models:` 两种行虽在 `(* *)` 里也要留）+ 锚点指到的函数单元指纹 + herd7 版本 + `.cat` / `.bell` 内容。锚点今天已经写到函数（`LIT:12` `crates/singlefs-core/src/transaction.rs::publish_first_file`、`LIT:13` `…recovery.rs::choose_root`，量的）；四份 `-nofence` 与 `commit-publish` 两份没有指到代码的锚点（`grep -c "singlefs-models: crates"` 各为 0，量的）。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S27 | 期望行变（在注释里） | 只这一份 litmus | 量的：LKMM 整道键换（对，但八份一把） |
| 阳 | S28 | litmus 体变 | 只这一份 | 量的：同上 |
| 阳 | S25 | 锚定的 `choose_root` 变 | 只锚定它的 `new-pool-file-creation-root-implies-units`；T11 `commit-publish`（`singlefs-models: none`）等七份不动 | 量的：LKMM 整道键换（八份一起，多） |
| 阳 | E07 | herd7 换版 | 八份全部 | 推的：今天经 `environment=` 进（`stage-inputs.tsv:38`），对 |
| 阳 | E08 | `tools/memory-model` 的模型文件变 | 八份全部 | 推的：今天不进键（`checker-tier-lkmm.sh:22` 自认），漏判 |
| 阴 | N31 | 说明文字变 | 无 | 量的：LKMM 整道键换 |
| 阴 | N32 | 没被锚定的 crate 文件变 | 无 | 量的：LKMM 整道键换 |
| 阴 | `litmus/commit-publish.litmus` 改名、内容不变 | 改名 | 无 | 推的：今天清单行带路径 |

### L21 crates 变异

一条判定 = `crates/mutations.tsv` 的一行；键 = 被改坏的单元指纹 + 原文 + 替换文 + 点名测试执行到的单元集合指纹 + 工具链 + 依赖版本；判法单元 = 判「抓没抓到」的那几个函数。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S34 | 被变异函数 `check_root_slot` 的别一行变 | 原文落在它里面的 `crates/mutations.tsv:678`、`:679` 两行，加点名测试执行到它的行 | 量的：`prune_row678` 换、`prune_row439` 不换（都对）；判法版本不换（对） |
| 阳 | S04 | 行 439 原文所在的常量变 | 行 439（原文对不上，锚点腐化判红）与点名测试执行到 `check_journal_record` 的行 | 量的：`prune_row678` 也换（多：`JOURNAL_PAYLOAD_CHECKSUM_OFFSET` 在这份文件里只出现在 `CK:712`、`:813`，`check_root_slot` 不引它） |
| 阳 | S30 | 判结局的函数变 | 全部行 | 量的：`crates_mutation_judge_version` 换（对） |
| 阳 | E01 | 工具链 | 全部行 | 推的：行键带工具链（`crates-mutation-rows.py:641`），对 |
| 阴 | N33 | 限时 | 无（限时只管超时、撞顶、没红三档的记录） | 量的：判法版本换；行键带限时（`:641`），底稿一之二量过 163 行全换 |
| 阴 | E10 | 内存上限 | 无 | 推的：行键带 `memory_max`（`:641`） |
| 阴 | N34 | 变异改名 | 无 | 推的：行键是整行六段（`:641`）；整道键量的换 |
| 阴 | N35、N26 | 判结局函数注释、内存包装注释 | 无 | 量的：判法版本都换 |
| 阴 | N45 | 被变异文件加没人调的函数 | 无 | 推的：今天遇到新加函数就现跑（`crates-mutation-rows.py:958`）；`prune_row678` 量的不换 |

### L22 广谱变异

一条判定 = 一条 cargo-mutants 变异的结论（抓到 / 没红 / 无效 / 升档）；键 = 被变异 fn 单元指纹 + 相对位置 + 替换文 + 抓它的测试执行到的单元集合指纹 + 依赖版本。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S34 | 被变异函数别一行变 | 落在 `check_root_slot` 里的变异 | 量的：`prune_row678` 换（对） |
| 阳 | T16 | 抓到它的测试调到的辅助函数变 | 被那条测试抓到的变异的「抓到」结论 | 推的：今天只比测试函数自己（`mutation-prune.py:715`），照沿用，漏判 |
| 阳 | `crates/singlefs-core/src/admission.rs:966` 起那个宏的定义体改一个字 | 宏变 | 真调这个宏的函数里的变异 | 推的：今天按定义宏的整份文件（`mutation-prune.py:519`），对但宽 |
| 阳 | S04 | fn 外常量变 | 引它的函数（`check_journal_record`）里的变异 | 量的：`prune_row678` 也换（多） |
| 阴 | N37 | 加一条没人用的 `use` | 无 | 量的：`prune_row678`、`prune_row439` 都换 |
| 阴 | N45 | 加一个没人调的函数 | 无 | 量的：都不换（对） |
| 阴 | T17 | 宏定义文件里与宏无关的函数变 | 无 | 推的：今天调这个宏的文件全部变异的依赖指纹换 |
| 阴 | `scripts/mutation/broad-mutation.py:469` 只改 `JUDGE_RULES_VERSION` | 只抬手写号 | 无 | 量的（底稿一之二：两版只差 `rules` 一行，304 行一行没进跨版本结论表） |
| 阴 | N07 | crate 清单注释 | 无 | 推的：全局键收清单原文（`broad-mutation.py:2877`） |
| 阴 | H3 | 空提交 | 无 | 推的：版本树按提交导出（`broad-mutation.py:1442`），内容不变指纹不变（对）；工作区里没提交的改动它看不见，属 H2 那一类漏判 |

### L23 实验变异表 `mutate.sh`

一条判定 = 一张表的一行（平常跑：抓到 / 没红 / 无效 / 超时；只编不跑：编得过没有）；键 = 原文 + 替换文 + 包住原文的 fn 单元与它依赖的单元 + cargo 参数与特性 + 工具链 + 依赖版本。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | T14 `tables/e142_new_pool_file_creation_dry_run.tsv:1` 替换文 333 → 334 | 替换文变 | 只 M1 | 推的：今天没有库（`mutate.sh:2` `admission: always`），整张表现跑；只编不跑那一格按表一项（`checker-tier-research-build-and-replay.sh:424`） |
| 阳 | T14 `e142_new_pool_file_creation_dry_run.rs:3484` 起 `read_unit_via_locations` 的别一行 | 包住 M4 原文的函数变 | M4，加测试执行到它的行 | 推的 |
| 阳 | T14 `e142…rs:477`（`castagnoli_crc32`） | 下游函数变 | 平常跑：测试执行到它的行；只编不跑：只有原文落在它里面的行 | 推的 |
| 阳 | E01 | 工具链 | 全部行 | 推的 |
| 阴 | T14 `:1` 只改 M1 的名 | 改名 | 无 | 推的：今天整表一项 |
| 阴 | T14 `:2` 改 M2 整行 | 别的行变 | M1 不作废 | 推的：今天整表一项，全表现编 |
| 阴 | `scripts/mutation/mutate.sh` 函数里只加注释 | .sh 注释 | 无 | 推的：今天只编不跑那一格的判法版本是整份阶段脚本与判定库原文（`checker-tier-research-build-and-replay.sh:504`、`:521`） |

### L24 形式化（Verus 四格与红例钉表）

一条判定 = 一项 Verus 判定（按被验函数拆的证明项、一处 `requires` 的空真项、一行必红世界）或一行钉表重放；Verus 项的键 = 这一项涉及的 Verus 单元（被验 fn、它 requires / ensures 用到的 spec fn、proof fn）去注释指纹 + Verus 与它自带 vstd 的版本。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | S32 | 被验函数 `choose_root_slot` 体变 | 它的证明项、它体内的空真项、原文落在它里面的必红世界行；用它 ensures 的推论（`corollary_recovery_replays_from_the_newest_self_verified_root`，`RP:2982`，推的） | 量的：`verus_crate` 换、verus-proof 格键换（整 crate 一项，全部作废，多） |
| 阳 | T12 `RP:107` | spec fn 变 | 用到它的全部证明项（含 `record_ends_its_publish` 的 ensures `RP:3103`）与必红世界行 14、15 | 推的 |
| 阳 | T12 `verus-must-red-worlds.tsv:15` | 必红世界一行的替换文变 | 只这一行 | 推的：今天这一行原文进它自己的键（`verus-proof-check.py:1201-1202`），对 |
| 阳 | E09 | Verus 换版 | 全部 Verus 项 | 推的 |
| 阳 | S15 对 verus-toolchain-pinned 格 | 编译用的 vstd 抬了 | 只这一格（同源核对） | 推的：这一格登记了 `Cargo.lock`（`stage-inputs.tsv:39`），对 |
| 阳 | T13 `e165_replay_bounded_enumeration.rs:1662` | 钉表装置函数变 | 判到的状态走到它的那些钉表行 | 推的：今天装置闭包按文件原文（`e165-pin-red-examples.py:1180`），360 行全部 |
| 阴 | N38 | 被验函数里只加注释 | 无 | 量的：`verus_crate` 换（格键不换） |
| 阴 | N39 | 空行让 `requires` 下挪一行 | 无 | 量的：`verus_crate` 换；空真项名带行号（`verus-proof-check.py:1026`） |
| 阴 | N40 | pins 注释 | 无 | 量的：`verus_pins` 与格键换 |
| 阴 | N41 | 判法函数注释 | 无 | 量的：`verus_judge_version` 换 |
| 阴 | E11 | 限时 | 无（超时不入库） | 推的：限时进共用输入（`verus-proof-check.py:811`） |
| 阴 | S15 对证明项 | Cargo.lock 的 vstd 抬了，验证用的是 Verus 自带那一份 | 证明、空真、必红三格的项都不作废 | 量的：`verus_crate` 与 verus-proof 格键都不换（对） |
| 阴 | T13 红例账路径改名 | 改名 | 无 | 推的：今天行的 22 段原样进键（`e165-pin-red-examples.py:1204`） |

### L25 研究复跑按行

一条判定 = 复跑表一行；键 = 装置入口起执行到的单元 + 这一行参数 + GPU 开关与特性集 + 期望产物内容 + 工具链 + 依赖版本；实验号、文件名不进。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | R01 | E23 装置函数体变 | 只 E23 那一行 | 量的：E23 换、E140 不换（对） |
| 阳 | R05 对 E140 | E140 用的 aes-gcm 抬版本 | 只用到它的行（E140） | 量的：E140 换（对） |
| 阳 | T18 | 共用库里被用到的 `Emitter::emit` 变 | 用它的行（E23 在内） | 推的 |
| 阳 | E22 | 切 GPU 开关 | 带 GPU 判段的行 | 推的：今天不进键（`replay.sh:55` 自认），漏判 |
| 阴 | R02 | 装置函数里只加注释 | 无 | 量的：都不换（对） |
| 阴 | R03 | 产物改名 | 无 | 量的：E23 换 |
| 阴 | R04 | 共用库里两行都不用的函数变 | 无 | 量的：E23、E140 都换（`grep -c Lru` 在两份装置里都是 0） |
| 阴 | R05 对 E23 | 无关包抬版本 | 无 | 量的：E23 换 |
| 阴 | N28 | 无关产物变 | 无 | 量的：研究复跑整道键换 |

### L26 实验准入（`admission.py experiment`）

一条判定 = 「这个实验的产物还算数、拒绝重跑」；键 = 装置入口执行到的单元 + 跑前登记里影响结论的参数 + 工具链 + 依赖版本。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | T14 `e142…rs:477` | E142 装置函数变 | E142 的准入判定（放行重跑） | 推的：今天登记路径逐文件原文（`admission.py:858`），对 |
| 阳 | `research/prompts/e142-r24-prereg.md`（登记在 `stage-inputs.tsv:52`）里影响结论的那个参数改 | 跑前登记的参数变 | E142/p8g-crates-dump 那一行 | 推的 |
| 阳 | E01 | 工具链 | 全部登记了准入的实验 | 推的（`admission.py:868-871`：工具链进指纹），对 |
| 阴 | T19 | kb 决策文档改一句 | 无（kb 不是代码，底稿 L26 定） | 推的：今天登记路径下逐文件进键 |
| 阴 | N22 | E142 行多登记一条装置不读的路径 | 无 | 推的：今天登记行原文进键（`admission.py:871`） |
| 阴 | E142 装置函数体里只加注释 | .rs 注释 | 无 | 推的：实验分支取文件原文 sha256（`admission.py:858`） |

### L27 E171 修订 11 的复用库（修订 6、8、10 冻结，不改、不当样本对象）

一条判定 = 修订 11 的一个读数单元或一条驱动行；键 = 装置入口起的函数单元 + 变异原文与替换文 + 参数与界 + 依赖版本。

| 类 | 样本 | 为什么 | 作废面 | 今天 |
|---|---|---|---|---|
| 阳 | T15 `e171_three_oracle_collusion_after_the_oracle_fixes.rs:2200` | 装置函数 `first_batch_points` 变 | 执行到它的读数单元 | 推的：今天 dep-info 列到的文件逐个原文（`reuse_library.py:172`），这个装置的全部单元 |
| 阳 | T15 `bad-implementations-expansion-164.tsv:1` 替换文改字 | 变异变 | 只这一行的驱动行单元 | 推的：变异原文进键（`reuse_library.py:546`），对 |
| 阳 | E01 | 工具链 | 全部单元 | 推的 |
| 阴 | T15 在表上方插一行（下面各行号 +1） | 行号挪位 | 无 | 推的：今天 `row {row.number}` 进键（`reuse_library.py:546`） |
| 阴 | T15 `reuse_library.py:88` 轮名 `e171-r1` → `e171-r2` | 库改名 | 无 | 推的：今天就是一份新的空库（`~/.local/share/singlefs/reuse/` 这一轮 `ls` 只有 `e171-r1`） |
| 阴 | T15 `reuse_library.py:89` 只抬 schema 号 | 只抬手写号 | 无 | 推的 |
| 阴 | N01 形态：dep-info 列到的 crate 文件里只加注释 | .rs 注释 | 无 | 推的：今天按文件原文 |

## 4 今天的键在样本上错在哪（按原因归，量的，数据是 1.3 节）

- 文件级与整道级（③）：同文件加没人调的函数（N04、N20、N43、N45 在结论版本上）、测试模块加单测（N06）、别的测试（N19）、无关装置文件（N32）、登记表别的行与注释（N22、N23），都让今天的键换。
- 原文进键（②）：wgsl 注释（N03）、`include_str!` 读的 `.rs` 注释（N21）、`.toml` / `.lock` / `.tsv` / `.cargo` 注释与无关包（N07、N08、N09、N23、N40、N42）、按原文收的 `.py` / `.sh`（N24 判法摘要连注释、N26 / N35 / N41 判法版本整份原文、N38 / N39 Verus 核整目录原文）、litmus 说明文字（N31）。按词法收的那几处判对了：N01、N02（`.rs` 注释）、N25（`.sh` 注释）、N30（`.py` 注释）四个 `changed=0`。
- 名字与路径（⑤）：N10–N14、N34、R03 都换键。
- 手写号（④）：N17、N27 只抬号就换（N18 只换整道键，因为它在停止文件里）。
- 限额进键：N33。
- 漏判（⑦）：S12（GPU 派活回读，`gpu` 不换）、S16（release profile，结论版本不换）、H2（提交之后 `change-touches-crates.sh` 报没碰）。
- 依赖：S13、S14 两次 lock 改动 CPU、观察者两把也换（lock 整份原文）；S15 对 verus-proof 那一格不换是对的（这一格登记的路径里没有 `Cargo.lock`：`stage-inputs.tsv:40` 路径那一列 `grep -c Cargo.lock` 为 0，`:39` 为 1，量的）。

## 5 每道门禁怎么把这组样本做成进快档的自证

共同做法：夹具是一棵小树（不是真仓），每个样本形态各一格，格里只放「施加这一处改动之后、按新键该作废的判定集合」；判定方法照底稿第二节第 9 条 ⑦：跑那一道的 `--list-items` 或判法单元子命令，读「键对不上的判定条数」与红句里报的「变了的单元清单」。阳性格期望集合非空且等于登记值，阴性格期望空集。真仓上的样本（第 2 节）留给改动批之后的攻方当验收跑，不进快档。弄坏开关名接在各道已有的开关变量上（这一轮 `grep -o '[A-Z_]*BREAK[A-Z_]*'` 现查各道的变量名），新起的只有 LKMM 一个变量。

| 门禁 / 共用库 | 自证放哪（快档谁跑） | 夹具里的格（取第 2 节的形态） | 弄坏开关 | 红在哪一格 |
|---|---|---|---|---|
| 共用单元键库，Rust（`judging_closure.rs` 的接替者，`build.rs` 经 `#[path]` 用同一份） | checker 档 `--lib` 单测（快档 crash-case-replay 格那一趟会跑） | 判器体 S01、下游 S03、V5 S04、V11 S05、类型 S07、wgsl S09 / S10、依赖 S13；阴 N01、N03、N04、N05、N06、N08、N10 | `SINGLEFS_UNIT_KEY_BREAK=constants-outside-functions-dropped`；另 `paths-in-key`、`whole-file-units` | V11 格（期望作废、实得沿用）；`paths-in-key` 下 N10 格；`whole-file-units` 下 N04 格 |
| 共用单元键库，Python（`lib_source_digest.py` 的接替者，`.py` def、`.sh` 函数、`.tsv` 被读的行、`.toml` 解析段） | 它自己的 `--selftest`（登记 light） | S29、S33、N22、N23、N24、N25、N30、N42 | `SOURCE_DIGEST_BREAK=registration-table-in-key`；另 `raw-tsv`、`comment-in-python-unit` | N22 格；`raw-tsv` 下 N23 格；`comment-in-python-unit` 下 N24 格 |
| checker-tier-crash-point-replay | 样本档 `fixtures/checker-tier-crash-point-replay.sh/unit-key-samples-{green,red}` | 一份小设点文件（两个测试、一个观察者）加一份假库：S17（只观察者那一项报 1、CPU 报 0）、N19（全 0）、H2（报 1）、H3（报 0）、F1（只中间那一态报 1） | `GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure` | N19 格（CPU 那一项报 1） |
| checker-tier-qemu-device-streams | 样本档（走 `.qemu-prerecorded` 预录那一支：`checker-tier-qemu-device-streams.sh:210-211`） | T10 发布 fourth_transaction 的函数（只 raise-rollback-floor 报 1）、N28（六档 0）、E17（整道那一趟 direct 0） | `GATE_QEMU_DEVICE_STREAMS_BREAK=tier-key-is-the-whole-stage` | T10 格里 file-overwrite 那一档报 1 |
| checker-tier-lkmm | 样本档：两份 litmus 加一个打固定版本串的假 herd7 | S27（那一份 1）、N31（0）、S25（只锚定那份 1） | `GATE_LKMM_BREAK=litmus-key-drops-every-comment`（新变量） | S27 格（期望 1、实得 0） |
| checker-tier-crates-mutation-replay | `crates-mutation-rows.py --selftest` 加格 | S34（同函数两行重判）、S30（全部）、N33、N34、N45（都 0） | `CRATES_MUTATION_ROWS_BREAK=row-key-with-limits` | N33 格 |
| checker-tier-broad-mutation | `mutation-prune.py --selftest` 加格 | S34、T16、N37、N45、T17 | `BROAD_MUTATION_BREAK=catching-test-own-function-only` | T16 格（期望不沿用、实得沿用） |
| checker-tier-formal-verification（Verus 四格） | `verus-proof-check.py --selftest`（假 verus）加格 | S32（只那一项）、T12、N38、N39、S15（证明项 0、toolchain-pinned 格 1） | `VERUS_PROOF_CHECK_BREAK=reuse-key-whole-crate` | N38 格 |
| checker-tier-formal-verification（红例钉表） | `e165-pin-red-examples.py` 的自证加格 | T13 装置函数（判到它的行 1）、T13 账路径改名（0） | `_PIN_RED_EXAMPLES_BREAK` 那个变量加 `row-key-with-ledger-path` | 账路径改名格 |
| checker-tier-research-build-and-replay | 现有样本档 `experiment-replay-reused` 旁加 `replay-row-key-samples`；mutation-tables-compile、research-unit-tests 两格各加一份 | R01–R05、E22；T14 `:1` / `:2`；N06 形态 | `RESEARCH_BUILD_AND_REPLAY_BREAK=row-key-with-product-name`；另 `mutation-compile-key-whole-table`、`unit-tests-without-library` | R03 格；T14 `:2` 格；N06 格 |
| harness-model-differential-and-scenarios | 样本档：六格各一条假测试输出 | 只动其中一格测试执行的单元 | `GATE_MODEL_DIFFERENTIAL_BREAK=one-key-for-every-cell` | 不该动的那五格 |
| code-tooling（research-script-selftests） | 它的样本档 | T21（只那一份自证 1） | `CODE_TOOLING_BREAK=selftest-key-whole-runner-table` | 别的脚本那几格 |
| 准入模块（实验准入、门禁复用，doc-experiments 的 evidence-in-repo 读它） | `admission.py --selftest` 加格 | T19、N22（都 0）、H2（1）、H3（0）、E01（1） | `ADMISSION_BREAK=experiment-key-with-kb-documents`；另 `tree-compare-reuse` | T19 格；`tree-compare-reuse` 下 H2 格 |
| E171 修订 11（`reuse_library.py`） | 它的自证（`REUSE_BREAK`） | T15 插行、轮名、schema 号（都 0）、替换文（1） | `REUSE_BREAK=row-number-in-key` | 插行格 |

## 6 推翻条件与排除掉的解释

推翻条件（什么现象会推翻）：

- 「今天」那一列的量的结论：在一份新拷的副本上（拷仓根 `.gitignore`、带 `PYTHONDONTWRITEBYTECODE=1`）对同一个样本重跑 `tools/samples.py run <号>`，换掉的键与 1.3 节那一行不同，就推翻那一行。探针自己的两头对照：Z00 首尾两次 `changed=0`（没有隐藏状态，确定性探针，跑几轮都只说明这一点）；S04（V5）12 把全换（探针分得出变）。
- crates 侧三种结论版本（`gpu`、`cpu_*`、`obs_mr`）用的是第一轮 crates 路的 python 移植（它与编译表 137/137 相等，第一轮量的）；这一轮没重编。重编一次 `cargo build -p singlefs-checker-tier`（只编不跑）之后 `OUT_DIR/conclusion_code_digests.tsv` 与移植算出的不等，crates 侧「今天」一列全部作废。
- S16 漏判：在副本上把 `Cargo.toml:20` 改成 `false` 再编，版本消息（`crash_identity.rs:647-648` 拼的那一份）变了，就推翻（探针只算了闭包摘要；版本消息另带的是 `kind=`、`code=`、`rustc=`、`cargo=`、`target=` 这几行，读到的代码里没有 profile，后果是推的）。
- S12 漏判：GPU 判器闭包里出现 `crash_judge_dispatch.rs`（今天在不许进的表里：第一轮 C20 引 `judging_closure.rs:678-687`），就推翻。
- 作废面里标「推的」调用链（S03 经 `check_pool_image` 到 `check_journal_record`、S05 到 `extent_upper_leaf_entry_view`、S07 到 `Verdict`、L19 按步序推的档）：新算法打出的单元清单里那条判定没有那个单元，这个样本就从「必须作废」挪到「必须不作废」（对那条判定），样本本身不废，期望集合照新算法的清单改；改之前要先证清单没漏（L13 的 E19 那一格就是防漏的）。
- L13 的条件样本（S01 对 GPU）：新算法的 GPU 单元清单里有 `crash.rs` 的判器函数，期望改成「作废」；没有，期望是「不作废」。两种都不推翻样本，只定期望。

排除掉的解释（各凭哪条观测）：

- 「N01、N02 换了 QEMU 与研究复跑两道的键，是摘要按批现算的取法差」：排除。清掉副本里被跟踪的 `.pyc` 重跑，N01、N02 都是 `changed=0`（1.3 节），那两把换是 1.4 节的污染。
- 「探针有隐藏状态」：排除。重跑那一份首尾两个 Z00 都 `changed=0`；R00 首尾两次逐字相同（1.5 节）。
- 「S15 不动 verus-proof 格是探针没读到 `Cargo.lock`」：排除。同一份探针上 S13、S14 改 `Cargo.lock` 都换了 crates-mutation 整道键；verus-proof 那一行的路径列本来没有 `Cargo.lock`（`grep -c` 为 0，量的）。
- 「`change-touches-crates.sh` 报没碰是因为副本不是 git 仓」：排除。同一副本上没提交的改动它报碰了（ctc-1，rc=0）。
- 「R04 让 E23 换键是因为 E23 用了 `Lru`」：排除。`grep -c "Lru\|aes_gcm" scripts/experiments/e23_journal_geom.rs` 为 0，E140 里 `Lru` 也是 0。

## 7 没做什么

- 没改仓里任何文件、没动判定库与标记、没起门禁、没起 cargo、没跑测试二进制；`qemu-system-x86_64 --version` 想现查版本，被重型测试闸拒了（QEMU 归重型），没绕。
- 副本里做过 git 写操作（`git init`、`git add -A`、几次提交），只在草稿目录的副本自己的 `.git` 里；主仓的 git 一个命令都没写。
- crates 侧 GPU / CPU / 观察者结论版本、门禁整道键、判法摘要、crates 变异判法版本、两条广谱变异键、Verus 共用输入、复跑两行按行键在副本上量了；块键、计划哈希、核对键、给定状态块键、crates 变异行键、广谱全局键、钉表行键、E171 单元键、实验准入指纹没有在副本上算，表里对应格写「推的」并指到代码行。
- 「作废面」一列是新键下的期望，凡是靠调用链的都标了推的；真仓上的样本没有拿任何新算法跑过（新算法还没写）。
- 没量每类改动多久发生一次、作废一次多花多少挂钟。
- 底稿第三节其余 7 问不归这一路，没答。
- 副本：交回前删掉 `copy/`，删之前 `du -sh` 为 `111M`；草稿目录里留着探针（`tools/`）、输出（`runs.out` sha256 `5813a3fb…`、`replay-runs-clean.out` `3ed1a086…`、`baseline.json` `d9898a8f…`、`ctc-*.out`）与污染那一趟的旧输出（`runs-contaminated.out`、`baseline-contaminated.json`，只给 1.4 节作证）。

## 要主 agent 跑的命令

没有。若要核 S16 的推翻条件，要在仓副本里只编不跑一次 `bash scripts/hosts/capped.sh 31 cargo build -p singlefs-checker-tier`（改 `Cargo.toml:20` 前后各一次），比 `OUT_DIR/conclusion_code_digests.tsv` 与版本消息；这一轮派发提示不许起 cargo，没做。
