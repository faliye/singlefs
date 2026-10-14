# 工具与治理批的零轮形态：第一轮判决（2026-10-13）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 V7 V8 L1 L2 L3 L4 L5 L6 L7 L8 F1 F2 F3 F4 F5 F6 F7 F8 F9 K1 K5 K7 K8 K9 K10 K11 W01 W06 Q1 Q3 -->

正文 `research/prompts/_m3-defs-b6-r1-body.md`；腿：正推 `research/prompts/m3-defs-b6-r1-sonnet-output.md`（sha256 e34887bc…9931，模型 `research/prompts/m3-defs-b6-r1-sonnet-model/`）、攻方 `research/prompts/m3-defs-b6-r1-opus-output.md`（sha256 65be73af…9314，模型 `research/prompts/m3-defs-b6-r1-opus-model/`）。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/clerk.md`、`.claude/agents/investigator.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-defense.md`。

## 核查记录（主 agent 做）

- 开工快照：冻结副本 `~/.local/share/singlefs/work/m3-defs-b6-r1/repo/` 对 `research/prompts/m3-defs-b6-r1-snapshot/frozen-copy-sha256.txt` 27 份非 OK 0 行。
- `cite-check.py <两份报告> --root <冻结副本> --background research/prompts/_m3-defs-b6-r1-background.md`：「✓ 核了 120 处引文，对上 120 处，没判 0 处」。
- 正推模型拷进草稿目录经 `run-with-memory-cap.sh 4G` 复跑：十二行十一 same，`lr.out` 只差路径没换成占位、判定相同。攻方模型经 `run-with-memory-cap.sh 2G`（腿报 8G 在包装里排队不起）复跑：21 份全 same。
- 门禁格主 agent 现跑（`SINGLEFS_HEAVY_TESTS=commit`）：正推三棵树——legacy-sourceless、other-name-unrecognised 两棵 abandoned-rounds 格绿，exempt-table 红在「指向不存在的路径」「已经按登记值写齐」两行、现算那一行放。攻方四样（两份冻结副本各一遍）——heavy-thread-defaults 小树零轮那一版只点名 `baseline_read.py`、`tests.rs`、`if_default.sh` 与样本里的两份库，零轮之前那一版另点名 `wrap_helper.py`、`alias_env.py`、`copied_env.py`、`use_alias.rs`、`vars_iter.rs`、`project_wrapper.rs`、`harness_wrapper.rs`、`or_default.sh`、`and_default.sh` 九份；private-values `pv-tar-mirror-red` 零轮那一版退 1、输出里出现样本的（编的）私有值 1 处，零轮之前那一版退 0；abandoned-rounds `abandoned-legacy-dispatch-date` 两版都绿；long-run-header-exempt：零轮那一版拷贝里放进攻方的修订 12 入口（文件头一行改登记值读法、正文仍 exec 修订 11 的现算编排），long-run-header-exempt、heavy-thread-defaults 两格与 `detached-run.sh long-run-check` 三条都退 0。

## 逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| V1 ⑮ 括注、类型表 | 站不住 | 括注里中文数词按字符判：误伤定义自己的原话「（全仓唯一的登记位）」「（双机时两台各取这一个数）」与「统一」「万一」「十分」（攻方 V1 甲、正推，零轮新造的误伤），放过「（机器忙时按负载降）」「（小机器对折）」「（双倍）」「（第２台１６）」「（按核数取）」（攻方 V1 乙两版都放、正推）；括注里的尖括号 `31（<登记值>）` 放行，与「别的尖括号写法按认不出数拒」冲突（正推）；「线程上限已判」能顶缺行，定义没写这条路（正推）；钩子注释说 three-way-forward 定义许编译，定义里没有（正推） | 改法 L1 |
| V2 让出 CPU | 站得住 | 两句都是给主 agent 的规矩，钩子判不了，正推逐句一致 | — |
| V3 heavy-thread-defaults 读自己线程变量 | 站不住（零轮更糟） | 九种自然写法（仓里现成的 `count_from_environment` 包装、harness 的 `process_environment_variable`、`os.environ` 的别名与拷贝、`std::env::vars()`、`use std::env::var as`、shell 的 `|| X=8`、`&& X=8`）零轮那一版一处不点名、零轮之前全点名（主 agent 跑小树坐实）；`#[cfg(test)] mod tests;` 之后的读两版都漏 | 改法 L2 |
| V4 `--test-threads 1` 文字 | 站得住 | 名单与源码相同（正推）；名单手写、没有检查与共用库绑定，记账 | 另立账 |
| V5 重型测试闸 | 部分站不住 | 拒绝说明写「由主 agent 在集成副本里跑」，主 agent 不带前缀照做也被拒（攻方 V5 乙，零轮之前放行）；副本里只改入口 import 的库，入口自证照放（V5 甲，两版一样，文件头已写罩不住）；`env -iC`、`env -S '…'` 放行（V5 丙，两版一样） | 改法 L3；V5 甲另立账 |
| V6 叫停轮登记 | 站不住 | 两个根：别的轮名只认 `-r<数字>` 形与登记过的名字，`m3-close-b1` 照收（正推，主 agent 跑树坐实门禁绿）；登记表表头的日期是「腿发出那天」，判法按它免出处，2026-10-12 之前发腿、之后叫停的轮门禁绿而 ⑥ 不收（攻方、正推，主 agent 跑树坐实门禁绿） | 改法 L4 |
| V7 private-values | 站不住（零轮更糟） | 私有值本身带 `/`（真配置 8 个长词里 6 个，攻方只数不打）时逐段打码打不到，红行把值原样打出；零轮之前不比路径、不打（主 agent 跑样本坐实） | 改法 L5 |
| V8 长活文件头 | 部分站不住 | 只改文件头一行、正文仍起现算编排的入口三道全过，老实照拷的反被拒（攻方 V8 甲，零轮之前相反）——文件头写的读法与脚本做的事是否一致，静态判法看不到（判别子观测不到）；⑭ 词表拒定义自己的原话「不看忙闲」与「在跑」「核数」，放「登记值的一半」「按此刻余量少给」（攻方 V8 乙）；`main-agent.md` 第 55 行没写词表与「否定句也算」（正推）；E171 六份「被 sha256 钉住」无人核、现行文件与找得到的钉都不同（正推） | 改法 L6、L7 |

## 改法（全部被攻过零轮；进批 7，改完开这一题第二轮）

| 改法 | 改什么 | 修的格 |
|---|---|---|
| L1 | ⑮ 不再认括注：「线程上限：」一行只许一个十进制整数，说明另起一行（「线程上限说明：」不判）；类型表与骨架、定义同步写清「线程上限已判」也顶缺行；钩子注释改实情（three-way-forward 不编译就从「许编译」那句里删掉，或定义写明它跑模型） | V1 |
| L2 | heavy-thread-defaults 退回零轮之前「引号里的全大写线程变量名出现就算读」的判法，W01 那一处（`dispatch-prompt.py` 读派发闸表的那一行）进 `.claude/heavy-thread-exempt` 一行；`|| X=N`、`&& X=N` 认成钉数；`#[cfg(test)] mod tests;`（外置模块）不跳过余下部分 | V3 |
| L3 | 重型测试闸仓副本那一支的拒绝说明写全起法（主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 前缀在集成副本里跑）；`env` 的 `-i`、`-S`/`--split-string` 照拆再判 | V5 |
| L4 | 「别的轮名」按仓里认得的轮名全集判（`research/prompts/` 下 `_<轮>-body.md`、`<轮>-snapshot/`、判决文件与登记表的轮名），不按 `-r<数字>` 形；W06 的免出处不按日期判，改成点名：登记表里现有的四行（c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1）列进库里一张写死的旧行名单，只缩不涨，门禁格与 ⑥ 同一套（名单里的不要求出处，名单外的一律要） | V6 |
| L5 | private-values 路径比对按私有值整串在整条路径里找，命中的那一段子串打成「〔私有值〕」，不逐段比 | V7 |
| L6 | ⑭ 多机一行与文件头 `# long-run-hosts:` 只认骨架那一句规范写法（逐字，后面可接「；」与不含数字的说明），不再用现算词表；定义与 `main-agent.md` 第 55 行写这一句原文 | V8 乙 |
| L7 | 豁免表每行加被豁免脚本的 sha256（exempt-check 比对，对不上判红）；文件头读法与脚本行为一致由写的人保证，定义写明钩子只判写法；E171 下一次修订的入口不进豁免表 | V8 甲 |

另立一笔账：V4 名单手写没有检查绑定；V5 甲副本里 import 的库被改闸看不到；三种新类型（clerk、three-way-forward、three-way-defense）多数派发不带重活却要写线程上限，出路是已判行（L1 写清）。上一题三轮判决的另立账照旧。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。
