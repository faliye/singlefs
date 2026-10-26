# m3-close-gov-b17-r1 判决（治理轮第一轮，2026-10-23）

被判对象：批 17 治理文件改动，正文 `research/prompts/_m3-close-gov-b17-r1-body.md`，两份 diff 在 `research/prompts/m3-close-gov-b17-r1-snapshot/`（`governance.diff` 改 `.claude/rules/implementation-workflow.md`、`.claude/rules/verification.md`、`.claude/rules/three-way-inference.md`、`.claude/skills/crash-test/commit-stages.md`；`ji8-hooks-and-verification.diff` 改 `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/rules/verification.md` one-scenario 一行）。腿：正推（Sonnet）`research/prompts/m3-close-gov-b17-r1-sonnet-output.md`（808c4d97…），攻方（Opus）`research/prompts/m3-close-gov-b17-r1-opus-output.md`（cdddccd3…）。

## 一、核查

1. 开工快照：`judged-sha256.txt` 10 行里 `.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py` FAILED，其余 OK；`kb-sha256.txt` 全 OK。两份钩子是别的会话在腿跑着的时候改的（加的是标准输入喂 shell 那一判与「没判全放行也记检出」），两腿各自在新旧两版上喂过 `replay.sh` 相关的命令，结果逐行相同（攻方报告第 G4 节、正推报告注意一段）；腿引这两份的行号按新版现查。
2. 正文与背景材料在腿开跑之后改过一次路径：`scripts/experiments/replay.sh` → `scripts/experiment-apparatus/replay.sh`（全仓目录改名，`.claude/gate.d/path-moves.tsv` 第 6 行），每份 2 处，只差路径（body 98671567… → e98c2748…，background a0afd1b7… → 5907c30b…）。
3. `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-gov-b17-r1-background.md`：「✓ 核了 148 处引文、41 处裸行号，对上 148 处，没判 41 处（其中裸行号没有引文可比 40 处…）」。
4. 主 agent 现查：
   - D13 已定项 17 定案（`.claude/kb/decisions/13-验证路线.md` 第 435 行）对边级写的是「按实测排，不沿它收；现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条，第二台那一路只出提示，按这条边排出键」，「它们的单元不进任何一份判定的键」只挂在文件级那一句上；自证三格写的是「边级排除要配三格自证」，没有「新加」。两腿 G1 的比对坐实。
   - `.claude/main-agent.md` 第 30 行第 11 条「哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用」：新加的「留到下一轮判决写完再删」与它字面顶牛，两腿 G2 一致。
   - `scripts/crash/lib_verdict_library.py` 第 88 行定义 `NOT_JUDGING_FILES`、第 223 行在判法单元指纹里排掉；`scripts/mutation/lib_mutation_units.py` 里没有一处引 `NOT_JUDGING_FILES`（`grep -n` 0 命中）。攻方 G1-g「crates 变异判法单元指纹没做文件级排除」坐实。
   - `EXCLUDED_PROGRAM_EDGES` 在 `scripts/gate/lib_function_units.py` 第 7150 行。
5. 复跑：见第五节。

## 二、逐格判定

| 格 | 正推 | 攻方 | 主 agent 判 |
|---|---|---|---|
| G1 复述 | 骨架照定案，5 处出入：多 2（边级也写「单元不进键」；「新加」是规则自加的限定），少 3（文件级缺「管怎么存、怎么算键，不管判对错」；verification 缺边级「不沿它收」；verification 缺模块级语句四样清单）；「现在只有那一条」改指 `EXCLUDED_PROGRAM_EDGES` | 同样报边级多挂「单元不进键」、自证两处不一致、「现在只有那一条」该指登记位；另报 crates 变异判法单元指纹 961 个单元里 496 个出自 `lib_function_units.py`（`lib_mutation_units.py` 那条路没做文件级排除） | **采两腿**，出入照正推那 5 处改，两处复述写成同一套（自证要求只写一处、另一处指过去）。「现在只有那一条」换成指 `EXCLUDED_PROGRAM_EDGES`。G1-g 是代码问题：判法单元指纹多收了算键的库，结果是多作废、不少作废（偏保守），但与 D13 已定项 17 文件级排除不符，进下一批工具员 |
| G2 复跑副本保留 | 与第 11 条字面矛盾；收工闸对含仓副本或编译目录的副本拦一次再放，没有如实的交付物写法能留；建议限定成「留输出与模型目录拷贝，仓副本、编译目录这一步用完就删」 | 同报与第 11 条打架、收工闸每次拦；另报跨会话没人能删 | **采两腿**：改成正推的限定写法——判决复跑那一步用完就删仓副本与编译目录，只留复跑输出与模型目录拷贝到下一轮判决写完；这样不碰第 11 条与收工闸 |
| G3 GPU 那一趟 | 24 项里 20 项相同；多 1：`gpu-driver-error` 套进「加大限时 / 上限」不对症；少 3：清场退 78 只写一种原因、两台分片末尾「本机补 GPU 趟」默认本机有卡、编译命令少 `--locked`（无害） | 同报 `gpu-driver-error` 出路不对症；另报两台分片时第二台的 GPU 判定不随 `--import-results` 导回（读源码量的）、GPU 复用哨兵红时 ③ 退 1 没写 | **采两腿**：commit-stages 第 2b 步改 `gpu-driver-error` 的出路、补清场退 78 的几种原因、补哨兵红退 1、两台分片那一句写成「在有卡的机器上补 GPU 趟」。第二台 GPU 判定不导回是驱动代码问题，进下一批工具员（驱动此刻 f6 在改，排在它之后） |
| G4 重型闸放行 | 按写的做；放过的三样都不是新增：路径尾部相同的拷贝（`lib_heavy_tests.py` 按尾部认，与文件头「别处同名拷贝读正文」不符）、`xargs` / `find -exec`（已登记的老口子）、先拷再起（真钩子读正文拦下）；one-scenario 目录对得上 | **打中**：副本里路径尾部是 `scripts/experiment-apparatus/replay.sh`、正文 `--list-items` 那一支起 cargo 的文件带 `--list-items` 放行（钩子现喂），同一份正文换个路径就被拒 | **采**：两腿量到的是同一个口子（按路径尾部认、不核正文），攻方给了起 cargo 的实例。改成只认本仓那一份（与 `--selftest` 那条一样比 sha256 或比真路径），进下一批工具员。G4-b（`BASH_ENV`）、`xargs` 那一类是老口子，不归这一批 |

## 三、去向

两腿在四格上互不矛盾、各补了对方没走到的形。打中的全部采：

| 批 | 改什么 | 谁改 |
|---|---|---|
| 修红一批（治理文本） | G1 五处出入与登记位指针、G2 限定写法、G3 commit-stages 四处 | tooling-writer，在副本里改、交补丁 |
| 下一批工具员（脚本） | G1-g `lib_mutation_units.py` 判法单元指纹排掉 `NOT_JUDGING_FILES`；G3-a 第二台 GPU 判定随导入导回；G4-a `lib_heavy_tests.py` 对 `replay.sh --list-items` 只认本仓那一份 | tooling-writer；`broad-mutation.py`、`crates-mutation-rows.py` 一带此刻 f6 在换，`lib_heavy_tests.py` 别的会话在改，先问再派 |

改完再开第二轮，攻第一轮的修补本身与这一轮没看过的地方（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）。

## 四、另登

| 项 | 去向 |
|---|---|
| 正推一次复跑 hook_feed 的 diff 计数多 1、原因没查到，之后 4 趟一致（正推报告第 20 行） | 主 agent 复跑一趟逐字相同；记「未稳」，不采它当证据 |
| `verification.md`、`.claude/agent-common.md`、`heavy-test-guard.sh` 文件头没点名 `replay.sh` 的重型与 `--list-items` 豁免（正推 G4） | 并进修红一批的文本那一份 |

## 五、复跑

两份模型都拷进 `~/.local/share/singlefs/work/m3-close-gov-b17-r1-verify/` 再跑，`--frozen` 照腿给的快照目录。

- 攻方：`bash scripts/hosts/run-with-memory-cap.sh 2G bash <副本>/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <仓根>/research/prompts/m3-close-gov-b17-r1-snapshot --work <副本>/opus-work`，退 0：`name=probe_same_as_model lines=27`、`name=cleanup_same_as_model lines=5`；`crates_judging_units_differs`：首行 961 → 966 个单元、`crates-mutation-rows.py` 222 → 227，差在 f6 腿交回之后换上的那一份；`lib_function_units.py` 496 个不变，G1-g 的结论不受影响。
- 正推：`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <仓根>/research/prompts/m3-close-gov-b17-r1-snapshot --work <副本>/sonnet-work`，退 0，三行 `name=rerun_compare file=feed.txt result=identical`、`hook_feed.txt result=identical`、`probe.txt result=identical`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-10-23 不受影响：判的是规则复述跟不跟定案，定案不改；G1-g 是实现与定案不符，改实现 |
