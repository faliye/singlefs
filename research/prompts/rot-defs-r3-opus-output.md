# rot-defs-r3 云端攻方（Opus）报告：K2、K3（2026-10-19）

攻击面：正文第三节 K2（前两轮站住的形态撑不撑得住；R1 与 R2 两句放在一起会不会让执行员撞拒而定义没给出路）、K3（前两轮报的数与喂探针输出能不能在副本上复出来；「没核动的」两条有没有被改后的句子写成事实）。不重复第一轮（2026-10-17 那二十处改动与漏改的句）与第二轮（批 4 修补本身、产生结论的方法、`04ceaa904` 四份）攻过的角度。

## 复跑命令与文件哈希

模型目录 `research/prompts/rot-defs-r3-opus-model/`。做的事：把命令文本当 PreToolUse(Bash) 的输入喂给三道判命令的钩子（不执行命令本身），钩子量三版——今天主工作区的、开工快照那一版、R2 R3 换上之前那一版（后两版由 `inputs/rebuild-hooks.py` 从今天的倒推，sha256 与快照、第二轮攻方记的逐个对上）；在草稿目录的一个小包上跑一次 `cargo test --release --bin`；照第二轮判决当时的文件复原一棵根再跑 `cite-check.py`；把第二轮攻方模型拷出来对两版钩子各复跑一趟。主工作区合入锁一律指到草稿目录里不存在的文件，结果不随别的会话占不占锁变。

```
cd /home/fy5090/code/singlefs
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/rot-defs-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/rot-defs-r3-snapshot --work <新的草稿目录>
```

末尾打十一行 `name=compare file=… same=yes|no`，与 `out/` 里留底的逐字节比；本腿最后一趟复跑十一行里十行 `same=yes`，`r2-rerun-pre-swap.out` 一行 `same=no`，原因是那一刻别的会话正在主工作区重编 e165，`target/release/deps/` 下与本体同一份的那个文件不在（第二轮模型 `probe-deps` 那一格写死了它的文件名，第二轮报告已写明这一格随本机重编变）。`evidence/` 里是 2026-10-19 在主工作区现跑的原样输出（钩子自证、弄坏开关、41 条喂法、第二轮模型带锁与不带锁四趟），时刻在另一位工具员换上 `cargo run` 那一批钩子之前，复跑脚本不再起它们（改过的钩子的自证不在副本里起）。

另有两样随本机状态变、留底时照当时的样子：`cases/runner-run-device-deps-test-binary.txt` 写死了 `deps/e181_…-9f6a861789a3f5ea`（重编之后换成 `cargo test` 新编的那一份）；复原第二轮那棵根要用悬空的 git blob（`git gc` 收走之后 `r2-root-sources.out` 会写「取不到」）。

```
823614490f6467fe2233fd3acf1b94310dcb207946b04a124df4857b7e40733a  ./cases/investigator-e165-replay-state.txt
aed87b7d4df35fc34ff5d1ea0b21069b0d953d0f1fa1657099900f2dffab107d  ./cases/investigator-formal-quick-every-pinned-item.txt
3fcdac4ac624bc7fdbea27b85da0bed9d86a4480482698c4c1b2744ca8b2539d  ./cases/investigator-formal-quick-two-pinned-items.txt
5b6736f6d14d2e4d086543e08dd62fca9d3e5cae9131dd2e854cb7cc17e6508e  ./cases.list
0bdffce35e901751b4d732176b63f049a1a17fa85a1c845ca5c45acd530d31b8  ./cases/main-unit-test-commit-prefix.txt
e4625666210b3facec4ead95879dec6fd471e795253700f6c425ead55f574f1e  ./cases/main-unit-test-without-prefix.txt
f8f612c6dbc5b461d422c5eddc4a5091efdd1457f9a6c3ed67f5fd9b0745e075  ./cases/runner-cargo-build-device.txt
d172c543ffeee5784f63b9f5cbf3089b78ccb282a991d53096e4bae2aa4a2264  ./cases/runner-cargo-run-device.txt
bef278bc714138354ef0802cdee08f92d9d933f8735a49d837c05c781a89a00d  ./cases/runner-e165-literal.txt
c5aac33d4a486e5120d187078319192c5d4dbae6ae310c4efb3726d6d0186892  ./cases/runner-e165-loop-variable.txt
059101f8557934373e54cc2c2265275303fad9c46242d95bc5a7e8bf9c640dda  ./cases/runner-run-device-deps-test-binary.txt
7d9272d08965ca7f81bc26be385df739989ee54b9fcd71e36e547d82417fca93  ./cases/runner-run-device-main.txt
26f5ed38749a15237faa9e6014cdd50614dcaad7db34335a5a5afea5e557b973  ./cases/writer-crash-quick-slice-states.txt
49475271fccbddca39bf5cbeac33e757ad8eba2187da45fd4cc581c7e527c2b2  ./cases/writer-crates-quick-break-in-driver-script.txt
92e750a35c0360c722fd5bdcd869a83ed4de59d1f489837ae472d4a83a600b18  ./cases/writer-crates-quick-shard-break.txt
3ab39413e4d85bc1dcc239f67faf51749065b4c8f861f91c420a41d55717d4aa  ./evidence/cite-r2-root.out
ec3818d19422e3c46ac7fc0773db6156c6e4af4fbbbae8354a910f992310587e  ./evidence/cite-r2-today.out
3c625b6c7ad7cbb207088ce29fb23b98df74cd36cbb3737cc78f4f565dee5251  ./evidence/cite-r2-unchanged-since.out
f322c9d3baad038f8358cf0820e5b545471fe947bbcbc6d13a7aff3bf850a163  ./evidence/feed-only.out
7a99ebc3e1d5f01f2caeda9d3cf429b200db916a5702a0e1156258c2e9503f00  ./evidence/guard-break-device-main-replay-allowed.out
d5b2ef05083046592f526a9b298fa05e9b15b0c30c0db979ee44026d904258fb  ./evidence/guard-break-device-replay-registry-unreadable-allowed.out
85df39e1af95fd08b7df775f5d70e3090fe6bdced7896155c5c27d1ce5d9e5f7  ./evidence/guard-break-light-tier-break-variable-ignored.out
cdd00039018b1cf2bd17ebc9b9eed0174cd1c8cba659ae09e5f1cf4028629349  ./evidence/guard-selftest.out
12a06e35cade96bb9d4892635307e3ce01748fb9b9b3695950d7e1aa57c9d6a7  ./evidence/lib-break-device-main-replay-allowed.out
05a0fdf7a3edf60c0d25deb5775b402dec2e63a9f08b8fc4a505a433a5e85c08  ./evidence/lib-break-device-replay-registry-unreadable-allowed.out
a44f80b573f1775cf612025ea6988207ded85059d09d561c4f6839b2323da54e  ./evidence/lib-break-light-tier-break-variable-ignored.out
0d5bf5243855e495d60d63d95d347542e3fa8f96a4d2bc1bcc7fa84a6db9003f  ./evidence/lib-selftest.out
16c8bf5d854f8417a99cd224941ef83cede9e0d03b07d23c8181ef57cf18f399  ./evidence/README.txt
ce0c01cb61bd5277a93698ca0d0d7a457d8a95913f43a8b1618e9260b3400e3e  ./evidence/rerun-preswap.log
c0415d59960bc869d7c21aff8fb839c1fbda01bda220caef624bc5280f6de8d3  ./evidence/rerun-preswap-nolock.log
c710814d7d86933b527f6d447ebc4e9718f31aec8ed2b5dfd0bbec4fdfdf998f  ./evidence/rerun-today.log
1ac195a598ccb0fbde5ed415054d524b026253ac5ff03807f71a02aa82a23640  ./evidence/rerun-today-nolock.log
ec7cb2a313c12b2c1376496c40cafb37bbbd86bb7cb0a1ff9645167b3b58c24c  ./evidence/sec3.out
c9f9b76ea9e9015e724f71ca6db8dbfb7ee18981d64114fb60468b97c6b1b96e  ./inputs/hook-swap-cargo-run-spec.json
55d8396474d0f51a281b865184f5c3823b9ff1f9adc0977e97be8b08c8559f72  ./inputs/hook-swap-feeds.json
748776417a2ccf9bf1608b612c31bd5becf6501f227c0bfbeadbbb412cadcb6c  ./inputs/hook-swap-reverse-first.json
73e45fd6987eacb34c052822ccbd5ab5373bd3af0c429e8f11a875e42d4abec8  ./inputs/hook-swap-reverse-second.json
fee4015927949106f800dd4b72ff20f1901b05dedfaf657b409822e70fb86bd1  ./inputs/rebuild-hooks.py
e81eb797207b65c0939c57dc6f5964a5f99f313d6d5d30ac8e02aa4bd42d52f4  ./out/cite-check.out
9af82d3c63fdacd93eb2bcae874886157efdc7547f7e97e59004bbeb344e4f4c  ./out/cite-check-r2-root.out
13529d96cbdd09967799ef99d851271be32cba25a8af2cf11c9015910a54670f  ./out/feeds-snapshot.out
c2d0d344839bfa16aebb0076620ea517f51814339daa771dc888da2b2100a23c  ./out/hooks.out
64c1089588b89cd1480938e540f6c1d2e1aaa3640d1dbbf363e6166cdcc5d6dd  ./out/probe-snapshot.out
64c1089588b89cd1480938e540f6c1d2e1aaa3640d1dbbf363e6166cdcc5d6dd  ./out/probe-today.out
e7e132fa18d797314023e0ea18a4031fb94607a35b8ad2cb78dd0804589bcc7a  ./out/r2-capped-diff.out
965d7dd6ab7aa57eb16ec0dd10317fc7eecfb49e87804c10013fe5059becc620  ./out/r2-rerun-pre-swap.out
a290243f2e832f34798167351d1415a8352792b15d9509c670e9065ba9099f47  ./out/r2-rerun-today.out
55216aca1cfdef8ec943c81b552d7dd242325657e64ea6febf123e2584ad0bc7  ./out/r2-root-sources.out
09d6267961ab85c75204879bde84e16dc3e2bad41f43144e0857694b7ae04b60  ./out/tiny.out
3f6bcacb323d6df23034686259849d133fd978000c6931af901dc735398b7433  ./probe.py
6cd3541a217707c5ae9b18413b135b6ed86eba99038d86c6e51dae31145456c2  ./tools/rerun.sh
```

## 各格判定一览

| 名字 | 格 | 打中的地方 | 判定 | 量过 / 推的 |
|---|---|---|---|---|
| 本体没人编 | K2（R1） | R1 叫执行员「只执行编好的二进制（`target/release/e<号>_<英文名>`）」，而 R1 那一套步骤里没有一步编出它：`compile-then-swap.py` 编在草稿目录的编译目录、换上的只是源码，`--clean` 再删掉；主 agent 的单测命令 `cargo test --release --bin` 只编测试版。执行员照抄跑到的是旧本体（钩子放行）或不存在的文件 | 打中 | 量过（小包上 `cargo test --release --bin` 不出本体；钩子放行旧本体；主工作区四个装置的源码都比本体新、e184 没有本体）/ 「旧本体与源码内容不同」是推的 |
| 单测前缀 | K2（R1） | R1 叫主 agent 在实验当中带 `SINGLEFS_HEAVY_TESTS=commit` 跑装置单测；共用约束与闸的拒绝说明都写提交之外要先弹窗、带 `=user-request`；`main-agent.md` 没有这一步 | 打中 | 量过（钩子输入：不带前缀拒、带 `=commit` 放）/ 三处原文对读 |
| cargo run 的括注 | K2（R1） | R1 写执行员不起 `cargo run`「（checker 档，重型闸拒）」；闸今天对 checker 档装置的 `cargo run` 放行（只 e152 与 e165 带 `--pinned-cases` 两种除外），拒 deps 测试版时的说明还叫执行员改用 `cargo run --bin` | 打中（轻） | 量过（钩子输入） |
| 循环变量当钉表 | K2（R1 加 R2） | 执行员重跑 E165 照常写 `for family in …; do … --family "$family"`，闸按「带 --pinned-cases」拒，字面写开的同一条放行；拒绝说明的出路是交主 agent，不是「把参数写成字面」 | 打中（轻） | 量过（钩子输入） |
| 钉表重放走轻型 | K2（R2） | R2 把直接起 e165 本体带 `--pinned-cases` 判成提交时的活；同一份活经形式化轻型 `--quick --item pinned:<N>` 点名，360 项全点也放行，调查员照定义第 2、5 步就会走这一条；`--replay-state` 逐行起本体也放行 | 打中 | 量过（钩子输入）/ 门禁那一侧照门禁与脚本源码推的 |
| 第二轮引文数 | K3 | 「核了 119 处引文、22 处裸行号，对上 119 处」 | 复出来了，但只靠悬空 blob | 量过（复原的根上逐字相同；今天的文件上 25 处对不上，全落在之后改过的文件上） |
| 第二轮模型 same=yes | K3 | 四行 `same=yes` | 有条件复出来 | 量过：倒推回换上之前的钩子、锁不被占时三行 `same=yes`，`capped.out` 只差写死的仓根路径；锁被别的会话占着时七格变成锁的拒绝 |
| 工具员报告的数 | K3 | 自检 742 种、1345 种（该拒 255）、三个开关的红格、41 条喂法 0 条不对、第三节探针 | 复出来了（少报一格） | 量过（在快照那一版钩子还在主工作区时现跑）；第三节 R6 那一格在锁被占时退 2 |
| 拒绝说明把推的写成事实 | K3（R3） | 闸对文件头里任何一个 `*_BREAK` 都说「弄坏开关下 --quick 不只跑轻型登记的那几项……照整道算」；`MUTATION_SHARD_RUN_BREAK` 下门禁一行都不判就退 1，崩溃点重放入口平常跑时带它的开关退 2 拒跑 | 打中（轻） | 量过（钩子输入）/ 门禁一侧读源码 |
| 不叫 BREAK 的开关 | K3（R3 的前提） | `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 由调用方设了，崩溃点重放 `--quick` 照原值交给引擎（轻型默认 65536，引擎默认 2^32），闸照轻型放行 | 打中（零轮形态，交用户表） | 量过（钩子输入）/ 落不落到分片的段是推的 |
| 混合补丁 | K3（R5） | 改后验证段第 1 步的句子与脚本源码 | 没打中 | 对读源码（没真跑） |
| 站住的几份 | K2 | `investigator.md`、`experiment-designer.md`、`three-way-defense.md`、`04ceaa904` 四处、第一轮 A 行在今天的钩子上 | 没打中（调查员那一条并进「钉表重放走轻型」） | 量过（第二轮模型今天复跑）/ 对读 |

四句（分不分辨臂、系统看不看得到、满足哪一分句、改法还中不中）在每一节末尾。这一轮没有臂；「分辨」一句答的是这一格分辨的两种说法。

## K2：前两轮站住的形态，照改后的定义走一遍

### 本体没人编

照 R1 改后的执行员定义走（跑前登记把装置写死成入库装置）：

1. 执行员写完装置就停：`.claude/agents/experiment-runner.md:35`：「改完不自己编，停下交回」；编与单测归主 agent：`.claude/agents/experiment-runner.md:32`：「编译与单测是固定的一步，归主 agent」。
2. 主 agent 跑执行员抄给它的两条命令，再续派：`.claude/agents/experiment-runner.md:32`：「主 agent 编、换、跑完单测之后续派，执行员只执行编好的二进制」，路径写死成 `target/release/e<号>_<英文名>`。
3. 第一条命令编在别处、只换源码：`scripts/hosts/compile-then-swap.py:19`：「CARGO_TARGET_DIR 放 <草稿目录>/compile-then-swap/target」；收尾 `--clean` 删掉这个编译目录（同一份定义第 2 步 ④）。第二条命令 `cargo test --release -p singlefs-checker-tier --bin <名>` 只编测试版，不编本体。在草稿目录一个只有一个 bin 的小包上量的（`out/tiny.out` 原样）：

```
name=tiny_cargo_test exit=0 release_main=absent deps_test_binaries=1
```

4. 于是主工作区 `target/release/e<号>_<英文名>` 要么是以前哪一次编的旧本体，要么不在。今天主工作区的样子（按修改时刻比，不打时刻）：

```
name=e180_admission_reading_versus_writable_bytes source_newer_than_binary=yes
name=e181_landing_check_fixed_point_and_mount_lower_bound source_newer_than_binary=yes
name=e182_unreadable_abandoned_root_forks source_newer_than_binary=yes
name=e183_capacity_read_cache_reserve_and_sweep_costs source_newer_than_binary=yes
name=e184_regular_layer_cost_counts binary=absent
```

（命令：对五个装置各比 `crates/singlefs-checker-tier/src/bin/<名>.rs -nt target/release/<名>`。修改时刻新不等于内容变了，e180 与 e182 两份源码的修改时刻相同，像是被同一次 git 操作碰过；「本体与源码内容不同」是推的。）

5. 执行员照第 2 步跑旧本体，三道钩子都放行；改跑主 agent 单测编出来的测试版，按 checker 档拒，拒绝说明叫它用本体或 `cargo run`（`out/probe-today.out` 原样，第二格的 stderr 截到 260 字）：

```
== runner-run-device-main（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
== runner-run-device-deps-test-binary（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    ✗ 重型测试被拒：直接执行 singlefs-checker-tier 包的测试二进制（e181_landing_check_fixed_point_and_mount_lower_bound）：checker 档，默认只在提交时跑；它是装置 e181_landing_check_fixed_point_and_mount_lower_bound 的话�
[run-scope-guard.sh] exit=0
```

那一行拒绝说明往下（`PROBE_LINES=8` 现跑）接的是「装置的 main 用 target/<profile>/e181_… 或 cargo run --bin e181_… 起」——前者是旧的，后者 R1 不许。执行员自己 `cargo build` 钩子放（`runner-cargo-build-device` 那一格退 0，锁不被占时），但定义第 1、2 步都说编归主 agent。

本该看到什么：R1 那一套里有一步编出 `target/release/e<号>_<英文名>`（主 agent 在换上之后 `cargo build --release -p singlefs-checker-tier --bin <名>`，或执行员跑之前核本体不比源码旧）；产出一节第 4 步要的「用最后一次源码改动编出来」（`.claude/agents/experiment-runner.md:39`：「（入库装置那一支编在仓根的 `target/` 下），用最后一次源码改动编出来」）有人做、有人核。都没有：照字面走，产物可能出自上一版装置，而闸、范围声明、产物自查（第 4b 步按臂数行）都看不出来。

四句：
- 分辨：「R1 写的那几步」与「产物真用最后一版源码」两件事；在新建或改过装置的那一次上不同。
- 系统看不看得到：钩子看不到本体新旧（只看命令文本与文件是不是同一份）；执行员看得到（比一次修改时刻），但定义没叫它比。
- 满足哪一分句：K2 第一条「会不会漏做」，与第二条「被拒时定义有没有给出路」（跑测试版被拒时给的两条出路一条旧、一条被 R1 禁）。
- 改法：

| 改法 | 修哪一格 | 标 |
|---|---|---|
| R1 第 2 步 ④ 加一步：主 agent 换上、跑完单测之后在主工作区 `cargo build --release -p singlefs-checker-tier --bin e<号>_<英文名>`，再续派；执行员跑之前核 `src/bin/<名>.rs` 不比本体新，比它新就停下交回 | 两半 | 推的；`cargo build` 钩子放行是量过的（锁不被占时），锁被占时按锁拒 |
| `compile-then-swap.py` 带 `--release`，换上之后把草稿编译目录里的本体硬链接或拷进主工作区 `target/release/` | 「没人编」那一半 | 推的；要改脚本，归 tooling-writer；旧本体照样可能被别的会话在同一位置重编覆盖 |
| 执行员自己编（删掉「改完不自己编」对本体的那一半） | 「没人编」那一半 | 推的；与 R1「编译与单测是固定的一步，归主 agent」相反，等于撤掉 R1 的一半 |

推翻条件：主 agent 的单测命令另带一条 `cargo build`、或 `main-agent.md` 有一步编本体（`grep -c 'compile-then-swap' .claude/main-agent.md` 输出 `0`，那份定义里没有 R1 的这一步）。另量了一次带集成测试（`tests/` 下一条用到 `CARGO_BIN_EXE_tiny` 的测试）的小包：`cargo test --release -p tiny --bin tiny` 退 0，`target/release/tiny` 照样不在（`release_main=absent`；这一格在草稿目录现跑、没进模型）。

### 单测前缀

1. R1 叫执行员抄给主 agent 的第二条命令：`.claude/agents/experiment-runner.md:32`：「和主 agent 带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）跑的单测命令」。这一步发生在实验当中（装置写完、产物跑之前），不在提交流程里。
2. 共用约束：`.claude/agent-common.md:56`：「重型测试只由主 agent 在提交时或用户要求时跑」。闸的拒绝说明：`.claude/hooks/heavy-test-guard.sh:833`：「提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑」。
3. 主 agent 自己的定义里没有这一步：`grep -c 'compile-then-swap' .claude/main-agent.md` 输出 `0`；实验那一行只写「先派一个执行员把装置、单测与变异表写完」。
4. 闸只认前缀、看不到场合（`out/probe-today.out` 原样）：

```
== main-unit-test-without-prefix（-）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    → 照抄（提交流程里跑；用户要求时把 commit 换成 user-request，提交之外要跑先弹窗问用户）：
[run-scope-guard.sh] exit=0
== main-unit-test-commit-prefix（-）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

主 agent 照 R1 抄 `=commit` 就放行，等于在提交之外跑重型而没弹窗；照共用约束与拒绝说明办，每个入库装置的每一次续派之前都要弹一次窗。R1 没说是哪一种，三处说法不是一套。

本该看到什么：R1 写「主 agent 先弹窗、用户同意后带 `=user-request` 跑」，或共用约束与闸把「入库装置单测在实验当中由主 agent 跑」列成一种明写的场合（例如单测按 harness 档算、不带前缀）；`main-agent.md` 实验那一行有这一步。

四句：分辨「R1 的前缀」与「共用约束的场合」，在实验当中跑单测这一次上相反；闸看不到场合（只看前缀的值），主 agent 看得到；满足 K2 第一条「会不会做到本该归主 agent 的重活」的反面——活归主 agent 没错，错在 R1 给的前缀把它算成提交；改法：

| 改法 | 修哪一格 | 标 |
|---|---|---|
| R1 那一句改成「主 agent 弹窗，用户同意后带 `=user-request` 跑」，`main-agent.md` 实验那一行补这一步 | 前缀那一半 | 推的；每个装置多一次弹窗 |
| 入库装置的内联单测改由 harness 档跑（闸对 `--bin <src/bin 下的装置>` 的单测放行，或装置单测挪到不链 checker 档的地方） | 两半 | 推的；动闸与装置布局，归 tooling-writer 与用户定 |

推翻条件：有一处规则或定义写明「实验当中主 agent 跑装置单测算提交流程」（我读了共用约束「不做」一节、`main-agent.md` 全文 grep、闸的拒绝说明）。

### cargo run 的括注

1. R1 那一句：`.claude/agents/experiment-runner.md:32`：「执行员不对它起 `cargo test`、`cargo run`（checker 档，重型闸拒）」。
2. 闸今天的文件头：`.claude/hooks/lib_heavy_tests.py:68`：「`cargo run` 起 checker 档包自己的装置二进制也照旧不判」；另一位工具员今天换上的只收了 e165 带 `--pinned-cases` 那一种（`.claude/hooks/lib_heavy_tests.py:76`：「`cargo run --bin <装置> -- <参数>` 起同一个装置同判」）。
3. 喂钩子（`out/probe-today.out` 原样）：

```
== runner-cargo-run-device（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

括注把推的写成了事实：执行员不会因为它多做什么（定义叫它不起），但上一节那条拒绝说明正叫它「cargo run --bin … 起」，执行员手里两句话相反；读括注的人会以为闸替定义把着这一道。本该看到：括注写成「归主 agent」，不写「重型闸拒」，或闸真拒。

四句：分辨「定义说闸拒」与「闸放」；闸看得到（`--bin` 的值是 checker 档装置）；满足 K3 第二条同形（改后的句子把没核的写成事实），落在 K2 的 R1 那一句上；改法：括注删「重型闸拒」（推的，只改字）；或闸对 checker 档装置的 `cargo run` 一律按 checker 档判（推的，会连带改 `replay.sh` 里 E165 那几个用 `cargo run -p singlefs-checker-tier` 的驱动的判法，归 tooling-writer）。推翻条件：闸改成拒 checker 档装置的 `cargo run`（另一位工具员此刻在改这一处，以今天的为准）。

### 循环变量当钉表（R1 与 R2 放在一起）

K2 第二条问的那一撞：执行员照 R1 跑编好的本体时撞不撞 R2 的拒。装置不是 e165 的不撞（`runner-run-device-main` 退 0）。是 e165 的：E165 已有六次重跑（`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1`：「第 6 次重跑已跑（2026-10-09：r3 档合法一族）」，复跑登记 `scripts/experiment-apparatus/replay.sh:1240` 起的是 `--bounds segment1 --family legal --profile r3`），第七次重跑由执行员照 R1 直接起本体，按族写一个循环是常见写法：

```
== runner-e165-loop-variable（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    ✗ 重型测试被拒：直接执行 checker 档装置 e165_replay_bounded_enumeration 的本体带 --pinned-cases：门禁 checker-tier-formal-verification 的 pinned-red-examples-replay 格的活（e165-pin-red-examples.py --replay 起的就是这一条
[run-scope-guard.sh] exit=0
== runner-e165-literal（experiment-runner）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

（`out/probe-today.out` 原样；前一格是 `for family in legal adversarial; do … --family "$family" --profile r3 > …; done`，后一格把 `legal` 写成字面。）判法是有意偏严：`.claude/hooks/lib_heavy_tests.py:73`：「没展开的词可能整词展开成它的也算」（工具员报告第六节第 2 条也写了「对没展开的词偏严」）。撞上之后：拒绝说明断言命令「带 --pinned-cases」，命令里没有；给的出路只有一条（`.claude/hooks/lib_heavy_tests.py:2153`：「装置本体的别的用法不算；」之后接「要跑写进报告「要主 agent 跑的命令」一节交主 agent」）。真出路是把循环展开成字面，说明里没有，执行员的定义里也没有。照共用约束「被拒了不换写法绕过去」，执行员会把一条本该自己跑的产物命令交回主 agent。

本该看到：拒绝说明分两种写——参数里真有 `--pinned-cases` 的照现在写；只因没展开的词才判的，写「参数里有没展开的词 `$family`，可能展开成 --pinned-cases；写成字面再起，或交主 agent」。

四句：分辨「真带钉表」与「带没展开的词」，闸看得到是哪一种（`device_main_replay_use` 里两支分得开）；满足 K2 第二条「被拒时定义有没有给出路」；改法：拒绝说明按两支分开写（推的，只改说明，归 tooling-writer）；执行员定义 1b 加一句「直接起入库装置本体时参数写成字面」（推的）。两种改法都不碰判法，误拒仍在，只给出路。推翻条件：执行员的定义或共用约束里有「直接执行装置时参数不许用变量」一句（我 grep 了 `experiment-runner.md`、`agent-common.md`，没有）。

### 钉表重放走轻型

R2 的前提：直接起 e165 本体带 `--pinned-cases` 就是形式化门禁红例钉表重放那一格的活，归提交时。同一格的活，子 agent 另有一条照定义走得通的路：

1. 形式化轻型的点名项里有这一格的逐行项：`bash .claude/gate.d/checker-tier-formal-verification.sh --list-items` 里第二列是 `pinned-red-examples-replay` 的行数 `360`（`--list-items` 只读，现跑）。轻型核心部分只是三项：`.claude/gate.d/checker-tier-formal-verification.sh:147`：「QUICK_CORE_ITEMS=(toolchain proof table)」；点名的项并进去：`.claude/gate.d/checker-tier-formal-verification.sh:197`：「requested_items=("${QUICK_CORE_ITEMS[@]}"」；重放那一格把点名的行交给脚本：`.claude/gate.d/checker-tier-formal-verification.sh:352`：「item_arguments_of pinned-red-examples-replay」，下一行起的就是 `e165-pin-red-examples.py --replay`。
2. 调查员照定义走到这里：第 2 步：`.claude/agents/investigator.md:31`：「收窄到点名的目标或格再复现」；第 5 步：`.claude/agents/investigator.md:34`：「轻型里形式化那一条」……「照跑」。输入是「钉表重放格红」的，收窄到 `--quick --item pinned:<N>` 就是照定义走。
3. 闸放行，两项、360 项全点都放（`out/probe-today.out` 原样）：

```
== investigator-formal-quick-two-pinned-items（investigator）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
== investigator-formal-quick-every-pinned-item（investigator）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
== investigator-e165-replay-state（investigator）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

第三格是逐行的另一条路：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:31`：「（与 `--replay-state` 打的那一行是同一段代码）」，`--replay-state <域>:<N>:<序号>` 起本体，R2 不判。

规则那一侧说轻型的点名是「按自己改到的文件」（`.claude/rules/verification.md:49`：「跑的人按自己改到的文件点名」），闸不判点名项与改动的关系，也不数项。于是 R2 拦下的只是一个入口：同一份活经轻型全格照跑，经 `--replay-state` 逐行照跑。两套说法必有一套错：要么钉表重放少量几行本来就可以给子 agent 跑（R2 偏严、误拒调查员复现），要么轻型的 `--item` 漏了这一格（R2 的前提要连轻型一起收）。

本该看到：R2 的判法与轻型的点名口径是一套——「钉表重放的行」在两处要么都放、要么都拒；或闸文件头写明「经轻型点名的钉表行不算」及理由。

四句：分辨「R2：钉表重放归提交时」与「轻型：点名项随便点」两种说法，在 `--item pinned:<N>` 这一条上相反；闸看得到（项名前缀 `pinned:`）；满足 K2 第一条「前两轮站住的形态」里调查员那份（第二轮判它站得住）走一遍会做到 R2 定成提交时的活；改法：

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 轻型认点名项时 `pinned:` 前缀的项照 R2 判重型 | 「轻型漏」那一半 | 推的；调查员复现钉表红就只剩交主 agent 一条路 |
| R2 放宽：本体带 `--pinned-cases` 而钉表只有几行（或经轻型点名）的放行，整表照重型 | 「R2 偏严」那一半 | 推的；要给「几行」一个数，没有依据 |
| 两边都不改，闸文件头与 `verification.md` 写明轻型点名钉表行是许可的、R2 只管整表 | 说法那一半 | 推的；`--replay-state` 那条照旧 |

推翻条件：形式化轻型在 `--quick` 下另有一步把 `pinned:` 项剔掉（我读了第 185–205 行与重放那一格，没读 `e165-pin-red-examples.py` 的 `--item` 处理全文；「真会起重放」这一半是按源码推的，没真跑）。

## K3：前两轮报出的数能不能核

### 第二轮引文数（复出来了，只靠悬空 blob）

第二轮判决：`research/prompts/rot-defs-r2-main-verification.md:13`：「核了 119 处引文、22 处裸行号，对上 119 处」。照它的命令在今天的文件上跑，末行（`evidence/cite-r2-today.out`）：

```
  ✗ 核了 119 处引文、22 处裸行号，25 处对不上，22 处没判（其中裸行号没有引文可比 22 处；逐处列在上面）
```

25 处全落在第二轮判决之后改过的文件上（R1–R13 改的定义、两份钩子、另一位会话改的 `scripts/crash/crash-point-replay.sh`）。加 `--unchanged-since 1792341124`（第二轮判决文件的修改时刻）只判没改过的：`  ✓ 核了 119 处引文、22 处裸行号，对上 63 处，没判 78 处（其中裸行号没有引文可比 4 处；逐处列在上面）`。

要核剩下的 56 处，得有第二轮当时的文件。第二轮开工快照只存了哈希（`research/prompts/rot-defs-r2-snapshot/` 下只有 `defs-sha256.txt`、`kb-sha256.txt` 两份），倒不回原文。我在 git 的悬空 blob 里按 sha256 找到了快照记的六份（五份定义与 `lib_heavy_tests.py` 快照那一版；别的七份在 HEAD 或暂存区里原样还在）。复原的根里两份钩子用判决那一刻的版本（`30f7e232…`，第二轮判决第一节写明 cite-check 是对着「此刻那一版」跑的，用 hook-swap 留下的反向规格倒推），再加 HEAD 里的几份与一份悬空的 `crash-point-replay.sh`（`out/r2-root-sources.out` 逐份写了从哪取），复原一棵根再跑，末行（`out/cite-check-r2-root.out`）：

```
  ✓ 核了 119 处引文、22 处裸行号，对上 119 处，没判 22 处（其中裸行号没有引文可比 22 处；逐处列在上面）
```

与判决那一行逐字相同。数是对的；能核是碰巧：那几个 blob 没被 `git gc` 收走，hook-swap 的反向规格还留在工具员的草稿目录里（我拷进了 `inputs/`）。第三轮的判决要引第二轮的数时，若照规则「在副本里复跑」，下一次 gc 之后就核不动了。

四句：分辨「数对」与「数能核」；判决那一刻看得到（文件都在）；满足 K3 第一条「照报告里的命令在副本上能不能复出来」——能，靠快照之外的东西；改法：开工快照除哈希外拷一份被判文件与被引钩子的原文（推的；快照目录会大，十几份定义与两份钩子，几百 KB）；或判决里写明 cite-check 跑在哪一份文件上（`--unchanged-since` 的时刻）（推的）。

### 第二轮模型的四行 same=yes（有条件复出来）

第二轮判决：`research/prompts/rot-defs-r2-main-verification.md:21`：「name=compare file=capped.out same=yes」（连同上面三行）。把第二轮攻方模型拷出来复跑：

- 今天的钩子、锁不被占（`out/r2-rerun-today.out`）：`probe.out`、`probe-deps.out` 两行 `same=no`，`quick-sim.out`、`capped.out` 两行 `same=yes`。`probe.out` 差的是 R2、R3 有意翻过来的三格（e165 本体带钉表、两道 `--quick` 设弄坏开关，退 0 变退 2）与两句拒绝说明的字（补了 `--check-table`、崩溃点重放那一句补了弄坏开关）；`probe-deps.out` 是 deps 下那一份 e165 本体，同样翻成退 2。
- 倒推回 R2 R3 换上之前的钩子（lib `30f7e232…`，正是第二轮攻方报告里写的「今天」那一版）、锁不被占（`out/r2-rerun-pre-swap.out`）：

```
name=compare file=probe.out same=yes
name=compare file=probe-deps.out same=yes
name=compare file=quick-sim.out same=yes
name=compare file=capped.out same=no
```

`capped.out` 只差路径（`out/r2-capped-diff.out`）：第二轮模型把 `capped.sh` 的报错原样存了，里面带仓根的绝对路径，换一个 `--repo` 就 `same=no`。
- 锁被占时（我第一趟复跑时主工作区合入锁正被另一个会话的 `merge-and-verify.sh` 占着），换上之前的钩子那一趟也是 `probe.out same=no`：六格的拒绝说明变成「主工作区合入锁被别的进程占着」，其中四格（执行员 research 那一支单测、两格声明夹进包装、实现员合成一条）退出码从 0 变 2（`evidence/rerun-preswap.log` 只留了比对四行，那一趟的 `probe.out` 随草稿删了；我当场看过逐格差异）。

所以第二轮判决那四行 `same=yes` 成立的条件有两个没写下来：钩子是 `30f7e232…` 那一版、主 agent 复跑的那一刻没人占主工作区合入锁。第二个条件三方腿的复跑模板也没管（`scripts/agents/three-way-rerun-template.sh` 不设 `SINGLEFS_WORKSPACE_LOCK_FILE`）；我的模型在 `probe.py` 里把锁指到草稿目录里不存在的文件。

四句：分辨「模型判的东西」与「复跑那一刻主工作区的状态」；钩子看得到锁（它就是按锁拒的）；满足 K3 第一条；改法：模板里 export `SINGLEFS_WORKSPACE_LOCK_FILE=<草稿目录>/no-such-lock`（推的，归 tooling-writer；只对喂钩子的模型有意义），腿留底的输出把仓根换成占位再比（推的；我的模型这样做了，量过：`r2-capped-diff.out` 里两边都换成了 `@REPO@`、`@WORK@`）。

### 工具员报告第二、三节的数（复出来了，少报一格）

开工时（`sha256sum -c research/prompts/rot-defs-r3-snapshot/defs-sha256.txt` 里两份钩子都 OK，即快照那一版还在主工作区）现跑，原样在 `evidence/`：

- `python3 .claude/hooks/lib_heavy_tests.py --selftest` 退 0：「自检通过（查了 742 种」（`evidence/lib-selftest.out`）；`bash .claude/hooks/heavy-test-guard.sh --selftest` 退 0：「自检通过（查了 1345 种，其中该拒 255 种）」（`evidence/guard-selftest.out`）。与报告相同。
- 三个新开关逐个打开，红格数（lib 数「✗ lib_heavy_tests 自检」行、guard 数「✗ 自检」行）：`device-main-replay-allowed` 5 与 3；`device-replay-registry-unreadable-allowed` 1 与 1；`light-tier-break-variable-ignored` 5 与 4。报告写第二个开关「lib 自检红 1 格」，没写 guard 那边也红 1 格（是总括的「lib_heavy_tests.py 的自检」那一格，见 `evidence/guard-break-device-replay-registry-unreadable-allowed.out`）。只是少报，不影响判。
- 41 条喂法只喂不换：「✓ 只喂不换：.claude/hooks/heavy-test-guard.sh 喂了 41 条，0 条不对」（`evidence/feed-only.out`）；复跑脚本对倒推出的快照那一版树再喂一次，同一行（`out/feeds-snapshot.out`）。
- 第三节的六格与 R10 两格照报告的命令喂真钩子（`evidence/sec3.out`）：退出码都与报告相同，只有 `rerun-wrapped-declaration-outside`（R6 那一格）在锁被占时退 2（「主工作区合入锁被别的进程占着……：cargo test」），锁指走之后退 0。

另：这几样在同一天下午之后就复不出了——另一位工具员换上 `cargo run` 那一批，两份钩子的 sha256 已变（今天 lib `b7f0c416…`），自检种数随之变；改过的钩子的自证不在副本里起，复跑脚本没法回到快照那一版重跑自检（只能重喂 41 条）。

### 「没核动的」R3：拒绝说明把推的写成了事实

第二轮判决第四节：`research/prompts/rot-defs-r2-main-verification.md:70`：「「轻型开关」门禁那一侧照整道跑，都是读源码推的」。改后的定义里没有一句靠它（`verification.md` 没补那一句，工具员报告第六节第 4 条），靠它的是闸的拒绝说明：`.claude/hooks/lib_heavy_tests.py:1533`：「弄坏开关下 --quick 不只跑轻型登记的那几项」，对每一道门禁文件头里的每一个 `*_BREAK` 都这么说。

门禁一侧对着读：

- `MUTATION_SHARD_RUN_BREAK` 在 crates 变异门禁文件头里，闸照它拒；门禁平常跑时见它就一行不判退 1：`.claude/gate.d/checker-tier-crates-mutation-replay.sh:251`：「MUTATION_SHARD_RUN_BREAK 只在 scripts/mutation/mutation-shard-run.sh --selftest 起的进程里认」，`.claude/gate.d/checker-tier-crates-mutation-replay.sh:255`：「弄坏开关只在那份自证起的进程里生效，这一趟什么都没判」。
- 崩溃点重放入口平常跑时带它的开关就退 2：`scripts/crash/crash-point-replay.sh:231`：「平常跑时有一个非空就退 2 拒跑」。

喂钩子（`out/probe-today.out` 原样）：

```
== writer-crates-quick-shard-break（implementation-writer）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=2
    ✗ 重型测试被拒：门禁 checker-tier-crates-mutation-replay.sh（带 --quick，但环境里设了它文件头写的弄坏开关 MUTATION_SHARD_RUN_BREAK：弄坏开关下 --quick 不只跑轻型登记的那几项，不认轻型，照整道算；要
[run-scope-guard.sh] exit=0
```

判法偏严、方向安全（工具员报告第六节第 3 条认了偏严），错在说明：对这一个变量，「不只跑轻型登记的那几项、照整道算」与门禁源码相反（门禁一项不跑）。照源码读、可能让 `--quick` 变宽的开关是 `GATE_MUTATION_BREAK=quickrunsallrows`、`FORMAL_VERIFICATION_BREAK=quick-core-unselected` 与共用库的 `STAGE_CELLS_BREAK=check-ignored`（`.claude/gate.d/lib/stage-cells.sh` 第 136 行把选格换成全部格）这几种值（按源码读的，没真跑）。

四句：分辨「说明断言的门禁行为」与「门禁源码」；闸看得到变量名与值，判不出值的意思；满足 K3 第二条「改后的句子有没有把推的写成事实」——定义里没有，钩子说明里有；改法：说明改成「设了文件头里的弄坏开关，判不出这一趟跑多大，不认轻型」（推的，只改字，归 tooling-writer）。推翻条件：`MUTATION_SHARD_RUN_BREAK` 有一个值在平常跑时生效并放宽选行（我读的是那道门禁第 186–256 行）。

### 不叫 BREAK 的开关（R3 的前提没罩住，零轮形态）

R3 的前提是「环境变量能让 `--quick` 跑出轻型以外的量」；判法只认文件头里的 `*_BREAK`。崩溃点重放入口另有一个变量：`scripts/crash/crash-point-replay.sh:725`：「调用方没设就设成一块，设了照用」，轻型的一块是 `scripts/crash/crash-point-replay.sh:247`：「LIGHT_REGISTERED_SEGMENT_SLICE_STATES=65536」，不设时引擎照 2^32 判（`scripts/crash/crash-point-replay.sh:102`：「本机没设就不送、第二台照引擎的默认（2^32）判」）。

```
== writer-crash-quick-slice-states（implementation-writer）
[bash-command-detector.sh] exit=0
[heavy-test-guard.sh] exit=0
[run-scope-guard.sh] exit=0
```

（命令：`SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=4294967296 SINGLEFS_RUN_SCOPE='种类=quick; …' bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/crash/crash-point-replay.sh --quick`。）照源码读，这一趟分片的段每趟判 2^32 态而不是 65536 态；轻型点到的项里有没有分片的段、真跑多久，推的，没跑（崩溃那一条轻型子 agent 不起）。共用约束本来就不叫子 agent 起崩溃轻型，所以照定义走的子 agent 不会撞上；会撞上的是主 agent 照实现员报告「要主 agent 跑的命令」原样抄、而那条命令里带了这个变量。这是这一轮新冒出来的形态，照规则不再为它开一轮，交用户表、标「零轮」。改法（推的）：闸的 `light_tier_break_variable` 另认这一个变量名；或入口在 `--quick` 下不认调用方的值。

### 「没核动的」R5：混合补丁（没打中）

改后的验证段第 1 步：`.claude/main-agent.md:51`：「脚本先逐份核补丁已打在树上，核不过退 1」，「测试挑包与重编核对都按给的目录算」。对着源码：`--skip-apply` 时逐份 `scripts/gate/merge-and-verify.sh:190`：「apply --check --reverse "$patch"」，核不过 `die` 退 1；挑包读的是 `CHECK_PATCH_DIRS`（第 127 行定义，第 828–829 行在清空 `dirs` 之前填好），`--skip-apply` 下照样有。句子说的与源码一致，没把推的写成事实；「两条命令依次跑都绿」那一半（第二条在已打了第一批的树上再打）仍没真跑，第二轮判决记的那一条照旧没核动。

## K2 第一条里没打中的几份

照改后的定义换一个子 agent 走一遍，第二轮判「站得住」的几格：

- `investigator.md`：第 2 步收窄（输入是整包 `cargo test` 的，在副本里收成 `-p <包> --test <目标>` 带 target 声明）走得通、不撞闸；第 5 步交回重型走得通。走到钉表重放格时的那一条并进上面「钉表重放走轻型」。
- `experiment-designer.md`：第 31 行占号之后定英文名，与 R1 的「bin 名、产物名一律用英文名」一致；它不写装置的跑法，R1 那几步不经过它。`04ceaa904` 改的 `test-script-discipline.md` 路径今天在 `.claude/rules/` 下、两节标题在（第 62、87 行）。
- `three-way-defense.md`（除 R8）：第 1 步 `grep -nF`、R8 补的第 4 步与写范围的冻结副本只读，照走没有撞闸的步；三份三方定义里 `tools/rerun.sh` 各出现一次、写法相同。
- `investigator.md` 第 16 行引的小节：`evidence-discipline.md` 第 73 行标题逐字在。
- 第二轮判决第二节 K1 前两行（第一轮 A1、A3、A9 那几条命令）：第二轮模型今天复跑，锁不被占时这几格的退出码与第二轮一样（`kb-scribe-write`、`runner-research-unit-test`、`implementer-combined` 都退 0），差的只有 R2、R3 有意翻的三格。锁被占时 `runner-research-unit-test`、`implementer-combined` 被锁拒——这是锁，不是定义；实现员在自己的副本里跑，碰不到主工作区的锁，执行员 research 那一支在主工作区 `scripts/experiment-apparatus` 下起 cargo，会碰到（定义没提锁，出路在锁的拒绝说明里，等或换副本）。

## 没打中的形状

照 K2、K3 试过、判成没问题的：

- R3 的驱动脚本：把 `export GATE_MUTATION_BREAK=quickrunsallrows` 与 crates 变异轻型写进一份脚本，命令只写 `bash <脚本>`。闸读进脚本、照拒（`writer-crates-quick-break-in-driver-script` 退 2）。
- R2 的字面写法：e165 本体带 `--bounds … --family legal --profile r3` 字面写的放行（`runner-e165-literal` 退 0），R2 没误伤正常的 E165 重跑写法。
- 别的装置本体（e183）直接起：放行（`runner-run-device-main` 退 0），R2 只拦 e165。
- 今天另一位工具员换上 `cargo run` 那一批之后，我的十四格在今天的钩子与倒推出的快照那一版上逐格相同（`out/probe-today.out` 与 `out/probe-snapshot.out` 的 sha256 相同）；`cargo run --bin e165_replay_bounded_enumeration -- --pinned-cases <表>` 今天退 2（在草稿目录现喂的，没进模型）。
- R5：见上一节，句子与源码一致。
- R4（第 14 条监控行改成指到 crash-test skill 第 3 步）：只对读了，没造日志；第 3 步讲的是逐阶段抄 ✓、✗ 行与退出码 77，第 4 步讲「作废」，第 14 条「三栏」里「作废」放哪一栏没写——不在我的攻击面里，只记一笔，没攻。
- 「登记读不出时装置本体一律按重放算」：若 `e165-pin-red-examples.py` 的 `DEVICE_BINARY`、`DEVICE_FLAG` 两行改了写法，所有 checker 档装置本体（e183 这类）都会被拒，执行员照 R1 跑产物全撞。按源码推的，没造这个状态；工具员报告第六节第 2 条认了这是接受的误拒。

取样范围：钩子输入十四格（四种 agent 类型）× 两版钩子；第二轮模型十二格 × 三版钩子（今天、快照、换上之前）× 锁占与不占；41 条喂法 × 两版；cite-check 三种根。没试：真起执行员、调查员、主 agent 去走；真跑门禁、崩溃轻型、merge-and-verify。

## 这条腿自己的限度

- 「量过」到钩子这一层与一个小包的 `cargo test`；门禁那一侧（轻型点名钉表行真起重放、`SLICE_STATES` 真放大、`STAGE_CELLS_BREAK` 真变宽）是读源码推的。
- 钩子输入里的 `agent_type` 是按类型名填的，与前两轮同。
- 开工之后两份钩子被另一位工具员换过（`sha256sum -c` 的开工快照对不上的是 `lib_heavy_tests.py`、`heavy-test-guard.sh`、`doc-decisions.sh`；后者是别的会话的改动，我没用到）。K3 的自检、开关、喂法、第三节探针是在换之前现跑的；K2 的十四格在换之前与换之后各量一次（今天的、倒推出的快照那一版），结果相同。
- 「本体没人编」里主工作区本体比源码旧，是按修改时刻比的；内容是否不同没核。主 agent 实际跑单测那几趟前后本体也被编过（本体的修改时刻落在主 agent 跑单测的那一段里），是谁编的、怎么编的我查不到（检出记录只记范围声明那一行）；如果主 agent 实际另带了一条 `cargo build`，打中的是「定义没写这一步」，不是「事情没做」。
- 主工作区 `target/` 被别的会话随时重编（复跑时 e165 的 deps 那一份正在重编），涉及 `target/` 下文件的格随本机状态变。
- 「没打中」照规则一次不算数；这是第三轮攻方的一次抽样。

## 没做什么

- 没判 K1（辩方那一格）；没读禁读清单里的 `research/prompts/rot-defs-r3-sonnet-output.md`；没替主 agent 采纳；没改任何定义、钩子、门禁与规则。
- 没跑门禁阶段、轻型三条、重型；跑过的只读命令：`checker-tier-formal-verification.sh --list-items`。跑过的 light 自证：`lib_heavy_tests.py --selftest`、`heavy-test-guard.sh --selftest`（各带三个开关再跑一次），在主工作区。
- cargo 只在草稿目录的两个小包上起过（`cargo test --release --bin`，经 capped.sh 与 2G 内存包装），编译目录已删。
- 要主 agent 跑的命令：没有必须的；要核「钉表重放走轻型」门禁那一侧，主 agent 可在副本里跑 `SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=门禁 checker-tier-formal-verification.sh 的轻型（--quick）; 范围=轻型核心部分加点名的 1 项; 为什么=核点名钉表行时重放格真起装置' bash scripts/hosts/run-with-memory-cap.sh 16G bash .claude/gate.d/checker-tier-formal-verification.sh --quick <副本根> --item pinned:26`，看输出里有没有重放格「编装置」与逐行重放的那几行（子 agent 能起形式化轻型，我没起：它会编 checker 档装置与跑 Verus，挂钟没估）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/rot-defs-r3-opus/` 里留着探针草稿、复跑日志与 `k3/` 下的中间输出，没有仓副本与编译目录。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/rot-defs-r3-opus-output.md` 末行（写这一节之前跑的）：

```
  ✓ 核了 34 处引文、2 处裸行号，对上 34 处，没判 2 处（其中裸行号没有引文可比 2 处；逐处列在上面）
```
