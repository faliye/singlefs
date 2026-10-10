# m3-batch56-code-r1 判决：删除线第 56 批代码轮第一轮（2026-10-10）

被判的改动（按路径点名）：`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`；测试与变异表见 `research/prompts/_m3-batch56-code-r1-diff.md`。正文 `research/prompts/_m3-batch56-code-r1-body.md`。

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch56-code-r1/repo/` 与主工作区各 `sha256sum -c research/prompts/m3-batch56-code-r1-snapshot/crates-sha256.txt`；仓根 `kb-sha256.txt` | crates 14 份（含变异表）两边全对、kb 6 份全对，腿跑着时没人改过 |
| 正推腿 `research/prompts/m3-batch56-code-r1-sonnet-output.md`（309 行，sha256 aad8dfd0a90a…）、攻方 `research/prompts/m3-batch56-code-r1-opus-output.md`（376 行，sha256 6b939ff720d0…） | `cite-check.py` 两份合判，`--background` 背景材料；`--root` 冻结副本时 11 处 kb 引文因副本里没有 kb 判不了，改 `--root` 仓根（第 56 批文件仓根与冻结副本逐字节相同）重判 | 「✓ 核了 99 处引文，对上 99 处，没判 0 处（逐处列在上面）」 |
| 攻方模型 | 拷到 `~/.local/share/singlefs/work/m3-batch56-code-r1-verify/model/`，`rerun.sh … all` | 见第六节 |
| 主 agent 现查 | 读冻结副本 | `crates/singlefs-checker/src/walk.rs` 第 12045–12046 行：装了旋钮那一支只判 `record_count <= records_the_knob_allows`，不判非最右那片装满；`.claude/kb/decisions/18-块里携带什么信息.md` 第 578 行「3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」 |
| 合入后验证（主 agent 在主工作区跑，不是腿的数） | `merge-and-verify.sh`、checker 档 CPU 快档、带 gpu 特性那一趟、13 行没证过的变异点名复跑 | harness 194 个测试二进制 1973 条全过；CPU 快档 61 个目标 659 过 0 红；GPU 那一趟 61 个目标 1020 过 0 红、`NOT_RUN_NO_GPU_ADAPTER` 0；变异「抓到 13 条、没红 0 条、无效 0 条」 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| K1-1 新次序成员与合法镜像 | 一致：33 张差分镜像只有台阶行代晚于根 4 张成员变（该变），合法 41 + 35 条用例没拒 | 没打中：30 个种子 331 次干净重挂、794 次回退 0 拒；G1 两行变异各红自己 | 两条腿各自造镜像、同向。攻方单次没打中，不拿它支撑「没有别的次序问题」 |
| K1-2 三条路次序 | 一致；冷走读先读记账树、水位行判只有重建有，是旧差别 | 同，旧差别不归这批 | 一致。正推另报「冷走读分配记录代与跨度挪回数条目之后 35 条全绿」：实现员自定的次序没有用例钉，补（第三节 H4） |
| K2-1 合法镜像误报 | 一致：63 + 30 个崩溃前缀镜像与随机历史带旋钮入口 0 红 | 没打中：旋钮下 4000 次判 0 违例；「崩溃恢复抛弃段里写的容器」层 0 全量最小窗口 1 048 588 态、估约 25 小时，没跑全量，13 态估时样本 0 违例 | 两条腿同向，但点名要造的那一形两腿都没跑到全域：不作结论，等提交前用户定的全量（层 0 全量与崩溃放量）罩它，第四节记 |
| K2-2 坏镜像漏报 | 规则没说：产品支一片容器不判；跨版本容器集包含没判；首片号 0 没有用例 | **打中**（不分辨批次）：装了旋钮时 I-9.22、I-9.37 只判 ≤ 旋钮值，不判非最右那片装满；旋钮 1 写出的镜像交旋钮 2、3 判 Holds，写侧早关一片的变异下带旋钮判 600 次 0 违例；GPU 同漏（`crash_judge_checker_scan.wgsl` 第 3177 行） | 攻方那一格坐实（主 agent 现查第一节两行）：D18 已定项 24 第 3 条「满了才关」在旋钮下的那一半没有判法，墓碑 I-9.22 同形。补（H1）。正推三条：首片号 0 补用例（H3）；产品支一片容器不判是上一轮记下的线索，不算打中；跨版本容器集包含是新判据，记欠账（第四节） |
| K2-3 CPU 与 GPU | 读码一致；GPU 产品支三判没有红例 | 没打中（读码）；旋钮常量全 0 那一份就是产品入口（`crash_judge_dispatch.rs` 第 517 行） | 一致；GPU 产品支调用点（`crash_judge_checker_scan.wgsl` 第 3194、3215 行）没有用例，归 H2 |
| K3 旋钮进 GPU | 一致；四种旋钮变体展开量都是 388 815 | — | 过 |
| K4-1 参照 | 一致；11 种改坏 9 种红，2 种推等价 | 四种改坏 3 种红，「并过之后也照样借」推等价 | 两条腿同向；没红的那几种两腿都推为等价，参照不改 |
| K4-2 等价族 | 一致：放宽到 221 256 棵树、1 744 830 次删 0 处不同，去掉前提立刻分叉 | 没打中：160 042 形、2 095 099 次删 0 处不同；另报非最右叶恒 ≥ 117 条（8 种子 × 1200 次删），「删空那片两边都有邻居」不可达 | 两条腿各自扩族、同向，等价留档站得住；攻方那条观察只说明等价测试里那一支走的是不可达形，不改测试 |
| K5 树 ID 水位 | 一致；不读的参数规则没说 | 没打中；G5 那一行变异红 | 过；不读的参数照上一轮主 agent 判的留着 |
| K6 判别力 | 37 行里跑了 27 行都红；缺口四处：首片判「≤ 1」照绿、产品支三判去掉照绿、冷走读分配记录次序挪回照绿、GPU 产品支两个调用点无人看守 | **打中**：产品支（I-9.21、I-9.36）不判三句的变异下 `checker_known_bad_images` 与容器树长高两份 253 条全绿 | 两条腿一致坐实产品支三判没有会红的用例（H2）；其余三处归 H3、H4、H2。正推说没跑完的四种全文件复跑（c3、c789、t789、ca）主 agent 没补跑：改法是补会红的用例，不依赖它们照绿 |

## 三、采纳的改法（第 57 批）

| 编号 | 改法 | 验收 | 来源 |
|---|---|---|---|
| H1 | 装了旋钮那一支（I-9.22 墓碑、I-9.37 谱系与退役）加判「不是最右那片记录数 == 旋钮值」，CPU 与 GPU 同判；`.claude/kb/invariants.md` I-9.22 一行随写回改 | 攻方 `probe-crate/tests/k2_knob_bad_images.rs` 那一形（旋钮 1 写、旋钮 2 / 3 判）做成坏镜像，三棵树各红自己；写侧 `w-close-one-early` 变异下旋钮随机历史红；合法旋钮历史不红；CPU、GPU 各一行变异证红 | 攻方 K2-2 |
| H2 | 产品支（没装旋钮：I-9.21、I-9.36）G2 三判补会红的用例：直接调判法的单测（交 (容器号, 记录数) 表）三棵树各三形；GPU 产品支两个调用点各补一条会红的对拍，做不到写明卡在哪交主 agent | 攻方 `c-product-skips-three` 与正推「产品支三判去掉」在新用例上红；GPU 两个调用点各一行变异证红 | 两条腿 K6 |
| H3 | 首片容器号 0 的坏镜像（三棵树，旋钮支与产品支各一） | 「首片判 ≤ 1」变异红 | 正推 K2.2、K6 |
| H4 | 冷走读里「分配记录代与跨度」排在两个数之前：造一张只有这一次序分得出的坏镜像钉住，造不出就写成等价性测试留档 | 「挪回数条目之后」变异红，或等价测试在案 | 正推 K1.2、K6 |

H1、H2 动 GPU 扫描内核 `crash_judge_checker_scan.wgsl`，派之前先与根环线商定（它的崩溃点重放合并正改 GPU 判器那几份）。

## 四、零轮与欠账

- H1–H4 的改法被攻过零轮，第 57 批合入后开它的代码轮。
- 单次没打中、本判决不作结论的格：K1-1 别的次序问题、K2-1（崩溃恢复抛弃段里写的容器那一形两腿都没跑到全域）。提交前弹窗请用户跑全量时，把这一形写进问句的出处。
- 记欠账（随 kb 写回一批）：① 跨版本容器集包含（后一版的容器集含前一版的，只增不删的树应当成立）没有判法，新判据，正推读码；② 产品支只有一片容器时三判不判（上一轮主 agent 判过不挡本轮，转欠账）。
- 下一批线索（不挡本轮，上一轮记过的）：「每盘一条」排在分配记录代之前；`crates/mutations.tsv` 文件头「门禁 59 号」。

## 五、交用户表

无。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 24 | 不影响 | 2026-10-10 不受影响：第 3 条「满了才关」在旋钮下照写，缺的是池级 checker 的判法（H1），不改条款 |
| D23（journal的角色与格式） 已定项 14 | 不影响 | 2026-10-10 不受影响：树 ID 水位抽函数前后行为相同 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-10 不受影响：M3 参照与等价留档两腿都没打中 |

## 六、复跑核对（攻方模型）

攻方模型拷到 `~/.local/share/singlefs/work/m3-batch56-code-r1-verify/model/`，在空工作目录 `run/` 从头跑 `SINGLEFS_SCRIPTS=research/scripts OPUS_THREADS=15 OPUS_MEMORY_CAP=8G bash model/rerun.sh run all`（冻结副本取缺省路径），rc=0，compare 原样末行「compare：21 份，不一致 0 份」，21 份逐份「一致」。第二节引的攻方数（K1 重挂与回退 0 拒、K2 旋钮下 4000 次判 0 违例、旋钮 1 写旋钮 2 / 3 判 Holds、`w-close-one-early` 与 `fix-knob-full` 两臂、K4 等价族、产品支三判去掉照绿）都在这 21 份里，主 agent 复跑与腿交的一致。
