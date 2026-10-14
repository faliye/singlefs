# m3-defs-b3-r2 攻方腿（Opus）报告（2026-10-13）

攻击面：正文「三、九格要答的」R1、R2、R5、R7、R9；第二轮分工（第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方）。只读两份冻结副本：改后 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`（开工对 `research/prompts/m3-defs-b3-r2-snapshot/frozen-copy-sha256.txt` 跑 `sha256sum -c`：30 份 OK、0 份不对），改前 `~/.local/share/singlefs/work/m3-defs-b3-r1/repo/`。引文写冻结副本的绝对路径。没读禁读清单里的文件（调度记录在扫 records/ 的探针里按文件名跳过）。

## 复跑

```
FROZEN_ROUND1=~/.local/share/singlefs/work/m3-defs-b3-r1/repo \
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-defs-b3-r2-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-defs-b3-r2/repo --work <新的草稿目录>
```

末尾八行 `name=compare file=<r1-r2|r5-r2|r7-r2|r7-scan|r2-regex|r1-r1|r5-r1|r7-r1> same` 即与 `out/` 逐字节相同（`FROZEN_ROUND1` 不给就只比前五份）。不起 cargo、不跑任何 `--selftest` 与门禁格；钩子都是喂 JSON 给入口（派发闸 `CLAUDE_PROJECT_DIR` 指探针自己建的样本根；重型测试闸另把钩子进程的 `SINGLEFS_WORK_ROOT` 指到 `<工作目录>/r5-work-root`）。这一次两趟（`rerun-a` 生成 `out/`，`rerun-b` 新目录复跑）第二趟八行都是 `same`。

门禁格那三份小树（`gate-samples/`）子 agent 不起，要主 agent 跑：`bash research/prompts/m3-defs-b3-r2-opus-model/tools/run-gate-samples.sh ~/.local/share/singlefs/work/m3-defs-b3-r2/repo <新的工作目录>`，推的结局在各目录的 `README.expect.txt`，下面各格也抄了。

模型目录各文件 sha256（`find . -type f | sort | xargs sha256sum` 原样）：

```
8ebe33e8c4b8c51d6bbfb3564cd0413a4debf9b3508980d1864d425ba231cf7f  ./gate-samples/code-tooling/r2-thread-gaps/.claude/cargo-command-prefix
845fe767b0f0d4ec3b54c1c44933b8bfa3527869e6d147f4d49d09d5683e3509  ./gate-samples/code-tooling/r2-thread-gaps/.claude/heavy-thread-default
344964f00cd233c367ef9430781413a187f389ca36af5adc8aa645ec753e12d1  ./gate-samples/code-tooling/r2-thread-gaps/.claude/heavy-thread-exempt
77bad418d0801076c5d8122ca2f1cd3fb871b7708c6bc25b8980570f227fbb0c  ./gate-samples/code-tooling/r2-thread-gaps/crates/demo/src/device_after_cfg_test.rs
b0c1420245c4e3fece3d307b0cf16fec28a0b6b5146c38487d597eef0a192466  ./gate-samples/code-tooling/r2-thread-gaps/crates/demo/src/device_control.rs
f976c6d764d5fa1b6753e4401f33772d517beabe50ddfac6540ed99fd231fbb1  ./gate-samples/code-tooling/r2-thread-gaps/.gate-cells
69664563a6f920ad5c3d9cf611338f0e17c37737b559f4ef82f9a1f767f63118  ./gate-samples/code-tooling/r2-thread-gaps/README.expect.txt
b397bcee603c43786f7edc805e8de14669b712b70c60fdcf6ddff1a7c66675cd  ./gate-samples/code-tooling/r2-thread-gaps/scripts/experiments/mentions_capped.py
57af600cfbba204b6329399bf25d3de4e6ad51e667c90e2fcd19b66ea8ff1b8c  ./gate-samples/code-tooling/r2-thread-gaps/scripts/experiments/own_control.py
be551550ee9685a1075711913058b6dfdb89aa17ab67f707480c1dc03ab251be  ./gate-samples/code-tooling/r2-thread-gaps/scripts/hosts/capped.sh
08c851ffbc9eeafdf90450c67df14b738fc1fe3d99adea7b9e4810da85b47be5  ./gate-samples/code-tooling/r2-thread-gaps/scripts/hosts/env_prefix.sh
6184842a8e812aa1ecf231d20c3e79b11e55031d21485a189d871f76f50e90c2  ./gate-samples/doc-process-records/r7-date-not-iso/.gate-cells
c8a0d8efb7181e8876dd4fd6c2fc279c6b3a4e0a2c24f2e622e7bef39a0c8874  ./gate-samples/doc-process-records/r7-date-not-iso/records/r7.md
2f4d43b02d1d8258e566efb77f656e38fd932d9d3f6d1272bde2db153684c319  ./gate-samples/doc-process-records/r7-date-not-iso/research/prompts/abandoned-rounds.tsv
ff2a5d1e397d2cb876d947544b48b9dc78b3b2002be809220258859fdeae3153  ./gate-samples/doc-process-records/r7-date-not-iso/research/prompts/_r-victim-r1-body.md
6184842a8e812aa1ecf231d20c3e79b11e55031d21485a189d871f76f50e90c2  ./gate-samples/doc-process-records/r7-duplicated-rows/.gate-cells
c8a0d8efb7181e8876dd4fd6c2fc279c6b3a4e0a2c24f2e622e7bef39a0c8874  ./gate-samples/doc-process-records/r7-duplicated-rows/records/r7.md
8bbb3823b8fa38d5f590892f07b46a0c58f98c582488711b1ee2c65bdb7ac0b9  ./gate-samples/doc-process-records/r7-duplicated-rows/research/prompts/abandoned-rounds.tsv
ff2a5d1e397d2cb876d947544b48b9dc78b3b2002be809220258859fdeae3153  ./gate-samples/doc-process-records/r7-duplicated-rows/research/prompts/_r-victim-r1-body.md
6184842a8e812aa1ecf231d20c3e79b11e55031d21485a189d871f76f50e90c2  ./gate-samples/doc-process-records/r7-no-source/.gate-cells
c8a0d8efb7181e8876dd4fd6c2fc279c6b3a4e0a2c24f2e622e7bef39a0c8874  ./gate-samples/doc-process-records/r7-no-source/records/r7.md
32af32803de6a2ea7725969ea57062146e737046dc1de71536c3d64a56ac6b65  ./gate-samples/doc-process-records/r7-no-source/research/prompts/abandoned-rounds.tsv
ff2a5d1e397d2cb876d947544b48b9dc78b3b2002be809220258859fdeae3153  ./gate-samples/doc-process-records/r7-no-source/research/prompts/_r-victim-r1-body.md
6afd5f5a5d4a87c9b456c4c6004782c6f4d1f6baa5aa52f8cc2768831a8f25ac  ./gate-samples/doc-process-records/README.expect.txt
7dab0c219d3c0182005f809f212947f8672460788db80199e38621a142eae579  ./gate-samples/doc-text/pv-big-untracked/.gate-cells
a0daed5b08813f2c3a70acbd198bd08b3e2f99c12bfe91ec18174e6b55908499  ./gate-samples/doc-text/pv-big-untracked/multi-host.env
a619bc5ee4432588bc22a81b435f4d01b14e3f90361f87075dfb0169bf3ddc24  ./gate-samples/doc-text/pv-big-untracked/.private-values-hostname
bb239f1845f220d5fbb54d4ff0bd7d954385e2df328a6dab9ce5a3fbb76ea59e  ./gate-samples/doc-text/pv-big-untracked/setup.sh
7dab0c219d3c0182005f809f212947f8672460788db80199e38621a142eae579  ./gate-samples/doc-text/pv-filename-and-content/.gate-cells
a0daed5b08813f2c3a70acbd198bd08b3e2f99c12bfe91ec18174e6b55908499  ./gate-samples/doc-text/pv-filename-and-content/multi-host.env
a619bc5ee4432588bc22a81b435f4d01b14e3f90361f87075dfb0169bf3ddc24  ./gate-samples/doc-text/pv-filename-and-content/.private-values-hostname
e21fb4daa61f87e88fe98f484885c6f42277d8147c5b3eaaf3d4eab00c9b1216  ./gate-samples/doc-text/pv-filename-and-content/setup.sh
7dab0c219d3c0182005f809f212947f8672460788db80199e38621a142eae579  ./gate-samples/doc-text/pv-filename-only/.gate-cells
a0daed5b08813f2c3a70acbd198bd08b3e2f99c12bfe91ec18174e6b55908499  ./gate-samples/doc-text/pv-filename-only/multi-host.env
a619bc5ee4432588bc22a81b435f4d01b14e3f90361f87075dfb0169bf3ddc24  ./gate-samples/doc-text/pv-filename-only/.private-values-hostname
e02f940ee0f4b9082da98fc95a7aad9eeb45a27912109a80abad018fe3138be5  ./gate-samples/doc-text/pv-filename-only/setup.sh
7dab0c219d3c0182005f809f212947f8672460788db80199e38621a142eae579  ./gate-samples/doc-text/pv-nul-head/.gate-cells
a0daed5b08813f2c3a70acbd198bd08b3e2f99c12bfe91ec18174e6b55908499  ./gate-samples/doc-text/pv-nul-head/multi-host.env
a619bc5ee4432588bc22a81b435f4d01b14e3f90361f87075dfb0169bf3ddc24  ./gate-samples/doc-text/pv-nul-head/.private-values-hostname
3a099e67c805cc058a61f1ac3f5da887455d71fc465ba39043606aee8af2d45c  ./gate-samples/doc-text/pv-nul-head/setup.sh
4e60a16650a31d76e96610b68fd7b03879cc69f94644b42374514c23cd5dcd7b  ./gate-samples/doc-text/README.expect.txt
f73378b0f475dedfc61fa2738ca7feba57110bdfb6dd4072dbab273e0ccdb1ed  ./out/r1-r1.txt
9ccdc1391da20856a93c8f3ca39ee8c9790ada3b6d2713a64768209a05943458  ./out/r1-r2.txt
20d03f1f0720e0a3ff569ed1c251582f5a69dcbda1cc557390eaddce0a41bb17  ./out/r2-regex.txt
1e826f4f13c71b038cd03c0a3bb330ced81a6a972efc2556ed285f324e1eb1b6  ./out/r5-r1.txt
9d0c0dc0f8ca0e043e4bdbb2bb0fb40674dddbb2ebb2f98109f818dfba62ab7f  ./out/r5-r2.txt
7063f019bc205e9e5ef55e0162a88011f6394b14b264fefa87cd03e0b0945efd  ./out/r7-r1.txt
84ec1da5497443aeed1bdadc5bf2e45f66c982b6fae05cbb7ddf02b91f3c1cc4  ./out/r7-r2.txt
a1098b8c58fd07d9fa7f078ea4871e139cb09f35b658ad00e5d243c8a753bb2d  ./out/r7-scan.txt
7cabc346ae0cb18f83ac15f88af318f3fba0ec1dc8493bc42d1eb703b181430a  ./probes/config-words-in-repo.py
f9d33687ee01c59b45a05c6b198a25a395b6235e0a519891977f1108a26fc463  ./probes/config-words.py
d2cf024630807be97c023ff637a6a870ce8747a5f921e67a4504b822c96de01b  ./probes/r1-thread-limit.sh
626c0349b85ec6418310562c41c544b606d3310eff9680bc65abc9d46eb12f64  ./probes/r2-regex-level.py
68c881b2ae1cb3ea3e68ed645df63d8e8a9cc1cb12b1784eeb916a514e89cc13  ./probes/r5-heavy-guard.sh
27e097077d25b74a0d8a21ab9896ecc1f6786bd4baf3015098c5018bcbd23779  ./probes/r7-abandoned.sh
f88e1c60ca6278bcbddb48e68b32a24ce4e03f6a2ab74463607cd4553129d85a  ./probes/r7-records-scan.py
264db3e36d4f5358973086db9f6fb0561e1a4e73bf0bf709c458fc010a5708f2  ./tools/rerun.sh
dce3c6c845a93ffd80a1c95936a4c35dc1ecb0189b1fa1847d02b7845e6fea32  ./tools/run-gate-samples.sh
```

`gate-samples/` 里的 `multi-host.env`、`.private-values-hostname` 与样本里的主机名、路径都是编的（从冻结副本门禁样本 `private-values-red` 拷来），不对应真机器。冻结副本根下真的 `multi-host.env` 只被探针读来比对计数，报告与模型目录里一个值都没写。

## 各格判定一览

| 格 | 编号 | 判定 | 打中的历史（一句） | 量过 / 推的 | 跑前条款哪一行 | 改前（第一轮冻结副本）中不中 |
|---|---|---|---|---|---|---|
| R1 | A1 | 打中 | 「线程上限：31（本机；第二台十六）」「31（第二台减半）」「31（两台合计）」「31（实际用三分之一）」⑮ 都放行；定义写括注里不写数字、双机两台各取这一个数 | 量过（`out/r1-r2.txt`） | 第二行后半「定义说的拒绝钩子不拒」 | 也放（第一轮 ⑮ 只比开头整数）；改法 G1 只挡住阿拉伯数字那一支，中文数字那一支改后又中 |
| R1 | A2 | 打中 | 派 tooling-writer 写「线程上限：<登记值>」（不是 `<填 …>`）：⑬ 不认、⑮ 当占位跳过，「没写这一行也拒」被绕过 | 量过 | 第二行后半（钩子文件头「认不出数的都拒」） | 也放（第一轮不要求这一行）；是 G1 新加的「缺行拒」没罩住 |
| R1 | A3 | 打中 | 派 clerk、three-way-defense 去编译（两份定义都许）不写线程上限一行，⑮ 放行；之后子 agent 起命令时三处文字给三种做法（共用约束只许 capped.sh N、verification 第 26 行说没这一行用 default、capped.sh 拒绝说明说没给写 default） | 量过（放行）+ 推的（三处文字） | 第二行（定义之间矛盾，照哪一句都违反另一句） | 不中：改前共用约束许 default，三处不矛盾；矛盾是 G2 带进来的 |
| R1 | A4 | 打中（弱） | 已判行理由写「机器忙，别的会话在跑提交阶段，让出 CPU」放行；main-agent 第 53 行同一行里「机器忙时也固定」与「别的会话要让出 CPU 时写别的数」两句，主 agent 看得到的只有负载，分不出是哪一种 | 量过（放行）+ 推的 | 第二行（照定义做却违反「机器忙也固定 31」） | 钩子改前也放；「别的会话要让出 CPU」那半句是 H3 新写的 |
| R2 | B1 | 打中（门禁格，要主 agent 跑） | 「读自己的线程变量」那一路遇到行首 `#[cfg(test)]` 就 break，只罩一条 use 的也算；仓里 18 份 rust 文件第一处 `#[cfg(test)]` 后面不是 mod（e168 装置第 118 行就是） | 正则层量过 + 小树推的 | 第二行后半（格文件头「测试模块不算」，实际把生产代码也跳过） | 不中（这一路是 G3 新加的） |
| R2 | B2 | 打中（门禁格，要主 agent 跑） | 「有没有回退」按整份文件找 capped.sh 等字样；装置读 SAMPLE_OWN_JOBS 没设就 KeyError，同文件另有一处字符串 `scripts/hosts/capped.sh` 就不点名 | 正则层量过 + 小树推的 | 第二行后半（「没设时也不回退」判的不是这个变量） | 不中（G3 新加） |
| R2 | B3 | 打中（另立一笔账） | 命令前缀赋值、`-j 8`、`--test-threads 8`、`:=8` 自己钉线程数，格一种都不认；真树里 replay.sh 有 22 行 `E1xx_…THREADS=N` 钉数，含 `E161_THREADS=10`（G3 给 capped.sh 补的 E161_THREADS 在 replay 里到不了） | 正则层量过 | 第二行后半（登记文件头「不经这里自己定并行度默认值的写法」由这一格判） | 也中（THREAD_DEFAULT_FORMS 两版逐字相同） |
| R5 | C1 | 打中 | 工作根底下仓副本里 light 自证的正文改成起 checker 档测试，重型测试闸照放（改前拒） | 量过（`out/r5-r2.txt`、`out/r5-r1.txt`） | 第二行（子 agent 起重型；共用约束没写这一处放行不读正文） | 不中：改前副本里的自证一律拒；是 G7 那一处放宽带进来的 |
| R5 | C2 | 打中（另立一笔账） | `env -C <仓副本>/crates/singlefs-checker-tier cargo test`（`--chdir=` 同）从草稿目录起，放行；同一包换成 `cd` 或 `--manifest-path` 都拒 | 量过 | 第二行后半（钩子文件头「包几层都一样」，没写 env -C 罩不住） | 也中（两版都 rc=0） |
| R7 | D1 | 打中 | 登记行出处指 records/ 里与这一轮同一行的「别的轮的叫停原话」、开轮的调度行、引规则原文的行、用户「先放一放」的行，⑥ 都收、快照不再挡派写回员 | 量过（`out/r7-r2.txt`）；records 里能这样用的行计数量过（`out/r7-scan.txt`） | 第二行后半：规则第 121 行写别的轮的叫停原话 ⑥ 不收 | 也中（第一轮 halted 引同一句照收，`out/r7-r1.txt`；第一轮 F1 已点出「别的轮的叫停原话挡不住」，改后又中） |
| R7 | D2 | 打中（门禁格那一半要主 agent 跑） | 同一张表两处判法不一：日期不是 YYYY-MM-DD、同一轮两行，⑥ 收、门禁格红；没写出处，⑥ 不收、门禁格绿 | ⑥ 量过；门禁格推的 | 正文 R7 第二问「一处收、一处拒的行」 | 不中（改前 ⑥ 不读这张表） |
| R9 | E1 | 打中（门禁格，要主 agent 跑） | 新加的未跟踪产物超过 8 MiB，私有值在第 8 MiB 之后：格只读前 8 MiB，判绿；格文件头写新文件整份都算新加 | 推的（读码） | 第二行后半 | 不中（新格） |
| R9 | E2 | 打中（门禁格，要主 agent 跑） | 私有值只在新文件的文件名里：不判；文件名与内容都带值时，红行照「只报文件、行号与键名」把带值的文件名打出来 | 推的 | 第二行后半（「不打印值」） | 不中 |
| R9 | E3 | 打中（弱，推的） | 词源只有 multi-host.env 与 hostname；共用约束第 47 行写私有值也在 ~/.config/singlefs/，清场脚本的内容也算私有，格不读那里 | 推的；那里 12 份文件的词数量过（不打值） | 第二行后半（共用约束说这一格判新加的行） | 不中 |
| R9 | — | 没打中 | 没有 multi-host.env 退 77：stage-cells 混合时整道退 0、77 那一格经 GATE_NOT_RUN_FILE 进 gate.sh 的「本次未跑一部分」，不说成判过；第 9 条「有别人没审过的改动就不登记」另有「用 SendMessage 协商」出路 | 读码 | — | — |

## R1　⑮ 线程上限

喂法（`probes/r1-thread-limit.sh`）：样本根是冻结副本 `.claude` 与 `scripts` 的拷贝，`CLAUDE_PROJECT_DIR` 指它、`SINGLEFS_THREAD_CAP` 不设，JSON 是 `{"tool_name": "Agent", "tool_input": {"subagent_type": <类型>, "prompt": <提示>}}`。tooling-writer 的提示原文（线程那几行换着填，其余每格相同）：

```
改一份脚本的拒绝说明。要改的文件：scripts/hosts/capped.sh。出口：自证绿。报告放在本次的草稿目录里（草稿目录在工作根下）。
<线程那几行>
重型测试：不跑
开工先读 `.claude/agent-common.md`
预计挂钟：2 小时内
```

退出码（`out/r1-r2.txt` 是改后，`out/r1-r1.txt` 是改前，原样逐行；thread_msg 是 stderr 里「线程上限」出现的行数）：

```
name=r1 case=ctl-31 type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=ctl-31-note type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=ctl-second-host-digit type=tooling-writer rc=2 thread_msg=2|改前 rc=0
name=r1 case=ctl-no-line type=tooling-writer rc=2 thread_msg=2|改前 rc=0
name=r1 case=note-second-host-chinese-number type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=note-second-host-halved type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=note-two-hosts-total type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=note-fraction-words type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=angle-placeholder type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=angle-placeholder-any type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=second-line-prefixed type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=second-line-bracket-label type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=second-line-backtick type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=valve-machine-busy type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=valve-any-reason type=tooling-writer rc=0 thread_msg=0|改前 rc=0
name=r1 case=clerk-compile-no-line type=clerk rc=0 thread_msg=0|改前 rc=0
name=r1 case=defense-compile-no-line type=three-way-defense rc=0 thread_msg=0|改前 rc=0
name=r1 case=fullwidth-digits type=tooling-writer rc=2 thread_msg=2|改前 rc=0
name=r1 case=superscript-digits type=tooling-writer rc=2 thread_msg=2|改前 rc=2
```

对照四格（ctl-）与第一轮判决 G1 的意图一致：阿拉伯数字的第二台、缺行都拒，登记值放行。

### A1　括注里写中文数字或比例词，⑮ 放行（量过）

被判的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:53`：「N 只写一个十进制整数，后面至多接一段全角括号的中文括注（括注里不写数字，半角括号不认）」；同一行：「双机时两台各取这一个数」。钩子文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:57`：「整数后面只许接一段中文括注（全角括号，括注里没有数字）」。判法 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1003`：「THREAD_LIMIT_FORMULA_MARKS = re.compile(r"÷|/|一半|除以")」，数字只认 `[0-9]+`（第 1002 行 `THREAD_LIMIT_NUMBER`）。

历史：主 agent 派 tooling-writer，第二台内存小，照第 53 行「后面至多接一段中文括注」写成「线程上限：31（本机；第二台十六）」；⑮ 放行（note-second-host-chinese-number rc=0）。子 agent 读到第二台用 16，违反「双机时两台各取这一个数」。同形的还有「31（第二台减半）」「31（两台合计）」「31（实际用三分之一）」，四格都 rc=0。

四句：
- 分不分辨臂：分。「括注里不许出现数词（十、百、半、分之、合计这类）」或「括注只许出现在一张白名单里（本机、登记值这类）」的形态在这几格上拒（推的）；改前的形态（只比开头整数）同样放，所以这一格是 G1 没修全，不是改过头。
- 系统看得到吗：看得到，括注原文就在提示里。
- 满足的分句：钩子文件头那一句「括注里没有数字」与第 53 行「括注里不写数字」——「十六」是不是「数字」是这一格的关键，按「阿拉伯数字」读就只剩「双机两台各取这一个数」被违反、钩子没声称管它，按「文字没写到」记。
- 改法：见末尾改法表 P1（推的，被攻过零轮）。

### A2　尖括号写的「占位」绕过「没写这一行也拒」（量过）

字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:60`：「不等于登记值、认不出数的都拒」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:62`：「N 是 <填 …> 占位的交 ⑬」。代码只认 `<填` 是占位（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:365`：「SKELETON_PLACEHOLDER = re.compile(r"<填[^>\n]*>")」），而 ⑮ 把一切 `<` 开头的值都当占位丢掉（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1045`：「stated = [value for value in written if not value.startswith("<")]」），写了这一行又被丢掉，「缺行」与「认不出数」两条都不触发（第 1047 行「if not stated and not missing:」）。

历史：主 agent 手写提示（不经骨架），写「线程上限：<登记值>」或「线程上限：<按机器忙闲定>」；两格 rc=0。子 agent 读不出数，按共用约束 `capped.sh N` 没有 N，只能自己挑（见 A3 的三种说法）。

四句：分辨臂——分（「`<` 开头而不是 `<填` 的按认不出数拒」不中，推的）；看得到——看得到；分句——钩子文件头「认不出数的都拒」，这一格是「定义说拒而钩子不拒」；改法——P2（推的，零轮）。

### A3　类型表外、定义许编译的类型不写这一行：三处文字三种做法（放行量过，矛盾推的）

类型表 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1005`：「THREAD_LIMIT_REQUIRED_AGENTS = {"implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack"}」。表外两份定义许编译、跑：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/clerk.md:27`：「要编译、跑编译出来的代码的，照共用约束「不做」一节看负载、经内存包装、带线程上限」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/three-way-defense.md:34`：「要改、要编译先拷进自己的草稿目录」。

提示原文（clerk 一格；three-way-defense 一格同形，见探针）：

```
把一个 harness 档的慢档用例编出来跑一遍，报告放在本次的草稿目录里（草稿目录在工作根下）。
重型测试：不跑
开工先读 `.claude/agent-common.md`
预计挂钟：2 小时内
```

clerk-compile-no-line rc=0、defense-compile-no-line rc=0。子 agent 开跑时手里三句话：

1. `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:61`：「capped.sh default 只给主 agent 与脚本内部用」（同一行前半要它「一律用 bash scripts/hosts/capped.sh N <命令> 起，N 照派发提示的「线程上限：N」」，而提示里没有 N）。
2. `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/verification.md:26`：「没有这一行的经 scripts/hosts/capped.sh default 取登记的每台默认重活线程数」。
3. `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/hosts/capped.sh:52`：「照 N，没给写 default」（前半句说派发提示给了线程上限一行的照那个数；这是 capped.sh 写错上限时的拒绝说明，子 agent 被拒就读到它）。

照 2、3 做违反 1；照 1 做没有数可用（自己挑一个数又违反「默认 31」的定案）。改前 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md` 那一条许 `capped.sh default`，三句不打架：矛盾是改法 G2「default 只给主 agent」带进来的，verification 第 26 行与 capped.sh 拒绝说明没跟着改。

四句：分辨臂——分（G2 那一句加「派发提示没有这一行的，照 verification 第 26 行用 default」，或类型表加 clerk、three-way-defense，任一种不中，推的）；看得到——子 agent 看得到三句话、看得到提示里没这一行；分句——跑前条款第二行「照定义做却违反」——照 1 违反 2，照 2 违反 1；改法——P3（推的，零轮）。

### A4　「机器忙也固定」与「别的会话要让出 CPU 写别的数」在同一行，主 agent 分不出（放行量过，冲突推的）

`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:53`：「机器忙时也固定这个数，不看忙闲」；同一行后面：「用户或别的会话要让出 CPU 时，只改新派的子 agent（派发提示写别的数，带一行「线程上限已判：<理由>」）与主 agent 新起的命令」；共用约束同义（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:61`：「用户或别的会话要让出 CPU 时，主 agent 新派的写别的数」）。

历史：别的会话在跑提交阶段，主 agent 看到资源提醒钩子报本机忙；它没收到任何会话的消息，只看得到负载。照「别的会话要让出 CPU」写「线程上限：8」加「线程上限已判：机器忙，别的会话在跑提交阶段，让出 CPU」，⑮ 放行（valve-machine-busy rc=0；理由写「见上」也放，valve-any-reason rc=0）；照「机器忙时也固定」就该写 31。

四句：分辨臂——这一格判的是文字；「让出」那半句限定成「用户或别的会话明说（消息、弹窗里的原话）」就不中（推的）。看得到——判别子观测不到：主 agent 做决定时看得到的是负载与资源提醒，两种情形（别的会话要求让出 / 只是机器忙）在这一刻逐条相同，除非有一条消息。照 `evidence-discipline.md`「判据自己也会写错」这是「判别子观测不到」，要改判据（P4）。分句——第二行「照定义做却违反用户定案（机器忙也固定 31）」；钩子放行是设计如此（已判行理由不审），不另算。改前：钩子同样放（`out/r1-r1.txt` valve 两格 rc=0），「别的会话要让出 CPU」那半句是 H3 新写的，改前文字只有「机器忙时也固定」一句，不冲突。

### A5　其余形状（不另算打中）

- 第二台另起一行但标签不在行首「线程上限：」：「第二台线程上限：16」「线程上限（第二台）：16」「`线程上限：16`（第二台照这一行）」三格都 rc=0（改前也 0）。⑮ 只判行首标签；定义只说「线程上限：」这一行只写一个整数，没声称别的行会被判，按「文字没写到」记，不算钩子不拒。
- 全角数字「３１」：改后拒（rc=2），改前放（`\d` 认全角，int 得 31）；上标「³¹」两版都拒。只会多拒，不是绕过。
- H3「在跑的不动」与「派出去之后」第 39 行「改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息」：用户要把默认值本身改掉（改 `.claude/heavy-thread-default` 登记）走第 39 行，临时让出走第 53 行，两句管的不是同一件事；第 49 行「只在改了定义、共用约束或规则时收一条消息」与第 53 行一致。没造出冲突的历史。
- 六种类型的表与「任务带重活」：sweep、kb-scribe、three-way-forward、three-way-materials、prior-art 的定义里没有编译、跑代码的步骤（`grep -c 'cargo\|capped\.sh\|编译\|run-with-memory-cap\|原型'` 分别是 0、0、0、0、0），不算漏；漏的只有 A3 那两种。

## R2　capped 与装置自己的线程变量

### 清单与装置逐个对（量过：`grep` 冻结副本 crates/、scripts/ 里点名的线程变量与各自的读法）

capped.sh 设的 9 个变量里有 E161_THREADS（G3 补的）。仓里装置读、而 capped.sh 不设的：

| 变量 | 谁读 | 没设时 | 门禁那一格怎么过 |
|---|---|---|---|
| E165_THREADS、E168_THREADS | 两份 checker 档装置 | 先回退 SINGLEFS_THREAD_CAP（capped.sh 设它），再 available_parallelism | 文件里有 SINGLEFS_THREAD_CAP 字样，算回退；available_parallelism 那一行在排除表「代码批」 |
| E177_THREADS、E178_THREADS | scripts/experiments 两份装置 | SINGLEFS_THREAD_CAP，再登记值 | 算回退 |
| SINGLEFS_CRASH_AMPLIFICATION_THREADS | crash_amplification.rs | 直接 available_parallelism（本机 32，不是 31） | 排除表「代码批」 |
| SINGLEFS_CRASH_JUDGE_CPU_THREADS | 两份 GPU / CPU 判器测试 | available_parallelism | 排除表「代码批」 |
| E168_LARGE_SLICE_THREADS | E168 装置 | 常量 1（内存护栏） | 排除表「代码批」 |
| E175_WORKER_THREADS | 装置源码在 research/results/ 下的 device-source.rs | 必须设（replay.sh 设 4） | 不在扫描目录（scripts、.claude/gate.d、.claude/hooks、crates）里，格看不到 |

这一张表的前五行都已在主 agent 第二节或排除表里，不另算打中；第六行是「从未被看过的地方」，但它要求变量必须设、不会按核数起，后果只是 `capped.sh N` 到不了它（推的）。

### B1、B2　G3 新加的「读自己的线程变量」那一路有两处判得比文件头说的宽（正则层量过，整格要主 agent 跑）

文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/code-tooling.sh:1865`：「rust 行首 #[cfg(test)] 往后的测试模块不算」。代码 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/code-tooling.sh:2145`：「if language == "rust" and code.startswith("#[cfg(test)]") and broken != "thread-default-test-module-unskipped":」之后 break——不看它后面是不是 `mod`。仓里 rust 文件第一处行首 `#[cfg(test)]` 后面接的不是 mod 的有 18 份（我的扫描，判法：那一行之后三行内第一个非空行不以 `mod ` 起头），其中 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:118`：「#[cfg(test)]」只罩一条条目，E168_THREADS 的读法在它后面第 303 行起。E168 今天另有回退与排除表行，所以真树上不漏判；新装置照这个文件的样子写就漏。

回退判法 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/code-tooling.sh:2156`：「if not CAP_OR_REGISTRY.search("\n".join(code_lines)):」——整份文件里出现 capped.sh、SINGLEFS_THREAD_CAP 等任一字样就算「回退了」，不看是不是这个变量的回退；python 里的字符串 `"scripts/hosts/capped.sh"` 被 kept 留下（`.*capped\.sh` 整串匹配），也算。

正则层核对（`probes/r2-regex-level.py`，正则原样抄自冻结副本那一份，跑前比该文件 sha256；`out/r2-regex.txt` 原样）：

```
name=r2-regex gate-file-sha256-matches=1
name=r2-regex case=thread-default-forms:env-prefix-assignment hit=0
name=r2-regex case=thread-default-forms:cargo-dash-j hit=0
name=r2-regex case=thread-default-forms:test-threads-literal hit=0
name=r2-regex case=thread-default-forms:colon-equals-default hit=0
name=r2-regex case=thread-default-forms:control-colon-dash-default hit=1
name=r2-regex case=thread-default-forms:real-replay-e161-pin hit=0
name=r2-regex case=env-prefix-counted-as-shell-local hit=1
name=r2-regex case=cfg-test-on-single-use-breaks hit=1
name=r2-regex case=capped-path-string-kept hit=1
name=r2-regex case=capped-path-string-counts-as-fallback hit=1
```

小树 `gate-samples/code-tooling/r2-thread-gaps/`（要主 agent 跑 `tools/run-gate-samples.sh`），推的结局：退 1，只点名两处对照（`crates/demo/src/device_control.rs` 第 2 行、`scripts/experiments/own_control.py` 第 3 行）；`crates/demo/src/device_after_cfg_test.rs`（第 1 行 `#[cfg(test)]` 只罩一条 use，第 5 行读 SAMPLE_WORKER_THREADS 没设就 panic）与 `scripts/experiments/mentions_capped.py`（SAMPLE_OWN_JOBS 没设就 KeyError，同文件另有一处 capped.sh 字符串）不点名。

四句（B1、B2 同）：分辨臂——分（「cfg(test) 之后第一行是 mod 才跳过」「回退判法看同一个变量名在回退式里」不中，推的）；看得到——看得到，代码就在扫描对象里；分句——第二行后半，格文件头「测试模块不算」「没回退到 SINGLEFS_THREAD_CAP 或登记值」写的是这个变量，代码判的是整份文件；改前——不中（这一路是 G3 新加的）。

### B3　自己钉线程数的 shell 写法，格一种都不认（另立一笔账）

登记文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/heavy-thread-default:5`：「里不经这里自己定并行度默认值的写法」由这一格判。格认的「写死的线程数默认」只有 `${X:-N}`、argparse 的 `default=N`、名字带 DEFAULT 的常量、`"threads": N` 几种（`THREAD_DEFAULT_FORMS`，改前改后逐字相同）。命令前缀赋值 `E161_THREADS=10 cargo …`、`cargo build -j 8`、`cargo test -- --test-threads 8`、`: "${CARGO_BUILD_JOBS:=8}"` 都不认（上面 `thread-default-forms:*` 五格 hit=0，对照 `${…:-8}` hit=1）；命令前缀赋值还被新加的 SHELL_ASSIGNMENT 当成文件自己的局部变量（`env-prefix-counted-as-shell-local hit=1`）。

真树：`scripts/experiments/replay.sh` 里 22 行这样钉数（`grep -c 'E1[0-9][0-9]_[A-Z_]*THREADS=[0-9]'`），含 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/scripts/experiments/replay.sh:722`：「E161_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh」——G3 给 capped.sh 补 E161_THREADS 是为了 `capped.sh N` 到得了 E161，replay 里这一行把它盖成 10。replay.sh 第 720 行写了「线程数钉 10」，是有意的钉数，算不算「自己定并行度默认值」由主 agent 判。

四句：分辨臂——这一格判的是门禁的覆盖面，不是臂；看得到——看得到；分句——登记文件头那一句；改前——也中（`THREAD_DEFAULT_FORMS` 两版逐字相同，`diff` 无输出），照跑前条款第三行另立一笔账。

## R5　重型测试闸的三处放宽与收严

喂法（`probes/r5-heavy-guard.sh`）：`<工作目录>/r5-work-root/hook-root` 是冻结副本 `.claude` 与 `scripts` 的拷贝（钩子与「主仓」登记表都是这一份）；钩子进程的环境 `SINGLEFS_WORK_ROOT=<工作目录>/r5-work-root`、`CLAUDE_PROJECT_DIR=<hook-root>`；JSON 是 `{"tool_name": "Bash", "cwd": <目录>, "agent_type": <类型>, "tool_input": {"command": <命令>}}`。仓副本 `r5-work-root/copy-a` 只放两样：与主仓逐字相同的 `.claude/hooks/selftest-registry.tsv`，和正文换成下面这三行的 `scripts/agents/agent-cost.py`（主仓登记成 light）：

```
#!/usr/bin/env python3
import subprocess
subprocess.run(["cargo", "test", "-p", "singlefs-checker-tier", "--", "--include-ignored"])
```

探针包 `hook-root/probes/linked`（包名 attack-probe，path 依赖一个只有 Cargo.toml 的 `singlefs-checker-tier`）；`r5-work-root/repo-copy` 只拷了冻结副本根与 crates/ 下的各份 Cargo.toml（钩子只读清单，不编）。退出码（改后 | 改前，原样）：

```
name=r5 case=copy-light-selftest-body-heavy rc=0 checker_msg=0|改前 rc=2
name=r5 case=copy-light-selftest-body-heavy-abs rc=0 checker_msg=0|改前 rc=2
name=r5 case=copy-outside-work-root rc=2 checker_msg=1|改前 rc=2
name=r5 case=main-light-selftest rc=0 checker_msg=0|改前 rc=0
name=r5 case=p-cwd-probe rc=2 checker_msg=2|改前 rc=0
name=r5 case=p-cwd-probe-src-subdir rc=2 checker_msg=2|改前 rc=0
name=r5 case=p-glob rc=2 checker_msg=2|改前 rc=0
name=r5 case=p-version rc=2 checker_msg=2|改前 rc=0
name=r5 case=p-env-C rc=0 checker_msg=0|改前 rc=0
name=r5 case=p-pushd rc=2 checker_msg=2|改前 rc=0
name=r5 case=p-cd-semicolon rc=2 checker_msg=2|改前 rc=0
name=r5 case=p-manifest-eq rc=2 checker_msg=2|改前 rc=0
name=r5 case=bench-cwd-probe rc=2 checker_msg=2|改前 rc=0
name=r5 case=run-cwd-probe-bin rc=2 checker_msg=2|改前 rc=2
name=r5 case=envC-from-scratch-into-probe rc=0 checker_msg=0|改前 rc=0
name=r5 case=copy-tier-cd-bare rc=2 checker_msg=2|改前 rc=2
name=r5 case=copy-tier-manifest-path rc=2 checker_msg=2|改前 rc=2
name=r5 case=copy-tier-envC rc=0 checker_msg=0|改前 rc=0
name=r5 case=copy-tier-envC-long rc=0 checker_msg=0|改前 rc=0
```

### C1　仓副本里 light 自证的正文换成起 checker 档测试，照放（量过）

字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_heavy_tests.py:131`：「三条都满足的单独一个 --selftest 放行」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_heavy_tests.py:133`：「罩不住的：副本里的脚本正文可以被改成起重活」。钩子文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/heavy-test-guard.sh:85`：「工作根（~/.local/share/singlefs/work）底下仓副本里的那一份放行」。共用约束只写了另两处放宽：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:81`：「直接执行它编出来的二进制、脚本里起的 cargo，这一样不判」——没写「仓副本里的自证不读正文、只认主仓登记行」。

历史：
1. tooling-writer 在草稿目录的仓副本里改 `scripts/agents/agent-cost.py`，给它的 `--selftest` 加一格，那一格里调了真的门禁阶段（或忘了把驱动换成假的），这条路现在起 checker 档测试；登记表的行它没动（登记归它改，但它没意识到自证变重了）。
2. 照 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:91`：「登记为 light 的，照钩子拒绝说明里写的起法改了谁就跑谁到绿」，在副本根里跑 `python3 scripts/agents/agent-cost.py --selftest`。
3. 闸放行（copy-light-selftest-body-heavy rc=0；绝对路径、换成 implementation-writer 也 rc=0）。checker 档测试由子 agent 起了，没带 `SINGLEFS_HEAVY_TESTS`，看门狗进程那一层是唯一的兜底。

同一条命令改前 rc=2（副本里的自证一律按「解到的不是登记表登记的那一份」拒，交回主 agent）。攻方腿自己的定义同样许在副本里跑（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agents/three-way-attack.md:53`：「副本与自己的模型可以编译、跑」），这条路对攻方腿一样开着。

四句：
- 分辨臂：分。「副本里那一份脚本的 sha256 与主仓那一份相同才放」的形态在这段历史上拒（推的；代价是改过的脚本的自证照旧要交主 agent，这一处放宽就白做了）；「放、但共用约束写明副本里的自证照登记行放、不看正文，自证变重的先改登记行」的形态不拦，只把责任写到定义里。
- 看得到吗：判别子半观测不到：「这份脚本的 --selftest 这条路起不起重活」钩子静态看不出（主仓那一份也是靠人审的登记行）；但「副本那一份与主仓那一份是不是同一份内容」钩子看得到，只是没比。
- 分句：跑前条款第二行（重型测试只由主 agent 跑，这是用户定的口径）；钩子文件头自己承认罩不住，共用约束那一条没写，按「钩子放的、定义没写」记。
- 改法：P5（推的，零轮）。

### C2　`env -C` 换目录再起 cargo，闸按 JSON 里的 cwd 判（量过，另立一笔账）

从草稿目录（没有 Cargo.toml）起 `bash …/run-with-memory-cap.sh 8G env -C <repo-copy>/crates/singlefs-checker-tier cargo test`：rc=0（`copy-tier-envC`，`--chdir=` 写法同）；同一个包换成 `cd … && cargo test`（`copy-tier-cd-bare`）、`--manifest-path`（`copy-tier-manifest-path`）都 rc=2。这就是在仓副本里跑 checker 档测试。钩子文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/lib_heavy_tests.py:61`：「经 bash -c、run-with-memory-cap.sh、capped.sh 包几层都一样」，罩不住的那一段列了直接执行探针二进制、git 依赖，没列 `env -C`。

四句：分辨臂——不分：改前改后同样 rc=0，照跑前条款第三行另立一笔账；看得到——看得到（`-C` / `--chdir` 就在命令行上）；分句——第二行后半（「包几层都一样」）；改法——P6（推的，零轮）。

### R5 里没打中的

- `-p` 先找当前目录：探针目录里 `cargo test -p attack-probe`、子目录 `src/` 里起、`-p 'attack*'`、`--package attack-probe@0.1.0`、`pushd`、`cd …;` 分号、`--manifest-path=` 等号写法、`cargo bench`：改后全拒（改前除 `cargo run --bin` 外全放）。G7 这一处修到了，没找到改过头的格（不依赖 checker 档的探针放行，见钩子自证格，我没重喂）。
- 副本里改登记行、多一行同名登记、副本在工作根外面：钩子自证格已有，我只重喂了「工作根外面」一格（copy-outside-work-root rc=2）。
- 符号链接：副本里的脚本是指向主仓那一份的链接，真路径就是主仓，按主仓判；指向工作根外面的，真路径落在外面，拒。造不出放错的（推的：读 selftest_work_copy_problem 里 realpath 与工作根比的那几行，没喂样本）。

## R7　叫停轮只登记一处

喂法（`probes/r7-abandoned.sh`，形态沿用第一轮攻方 d6 探针）：样本根是冻结副本 `.claude`、`scripts` 的拷贝，加一个还开着的轮 r-victim-r1（`_r-victim-r1-body.md` 在、快照点名 `.claude/kb/victim.md`、没判决）；派 kb-scribe 改 `.claude/kb/victim.md`（提示里重型测试、开工先读、规格检查已判都齐，只剩 ⑥ 会拒）。records/r7.md 六行（原样）：

```
# 调度记录（样本）
- 用户：「这一轮先不判了，停掉吧」。去向：r-victim-r1 两条腿停掉。
- 用户叫停 r-other-r1：「这一轮先停掉不做了」；r-victim-r1 照常跑，等腿交回。
- 开 r-victim-r1，攻方腿攻击面「R1、R2、R5、R7、R9 这几格」。
- r-victim-r1 正文引了规则原文「每一次工具调用写进文件的内容不超过 150 行」。
- 用户说「r-victim-r1 先放一放，等我看完判决再接着跑」。
```

每格登记表只留一行 `r-victim-r1<TAB>2026-10-13<TAB>用户叫停（records/r7.md 第 N 行）<TAB>不写判决`。退出码（改后 | 改前，原样）：

```
name=r7 case=no-registration rc=2|改前 rc=2
name=r7 case=legit-user-quote rc=0|改前 rc=2
name=r7 case=other-rounds-halt-quote-same-line rc=0|改前 rc=2
name=r7 case=dispatch-log-line rc=0|改前 rc=2
name=r7 case=rule-quote-line rc=0|改前 rc=2
name=r7 case=user-said-pause-only rc=0|改前 rc=2
name=r7 case=date-not-iso rc=0|改前 rc=2
name=r7 case=duplicated-rows-second-good rc=0|改前 rc=2
name=r7 case=no-source rc=2|改前 rc=2
name=r7 case=halted-other-rounds-quote rc=2|改前 rc=0
```

### D1　同一行里别的轮的叫停原话、开轮行、引规则的行、「先放一放」都收（量过）

字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/rules/three-way-inference.md:121`：「出处不许指这一轮自己的材料」，同一行列到「别的轮的叫停原话。派发闸 ⑥ 读这张表」，并写出处不合这几条的 ⑥ 不收；同一行末尾：「那句话是不是用户为这一轮说的，由主 agent 核」。钩子文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:38`：「行号对不上的都不收，那一轮照旧算开着」（前一行列了别的轮的叫停原话）。判法只要那一行有轮名（前后不连轮名字符）和一段六字以上的「」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:809`：「if not name_edges.search(cited_text)」，之后只看「」长度）。

历史：主 agent 要派写回员改 victim.md，被 ⑥ 挡；照拒绝说明在登记表加一行，出处写 records/r7.md 第 3 行（用户叫停的是 r-other-r1，同一行写了 r-victim-r1 照常跑）——收（rc=0）。第 4 行是主 agent 自己写的开轮记录、第 5 行是引规则原文、第 6 行是用户说先放一放，三格也都收。之后这一轮的快照不再挡派发，腿交回时它们判的 victim.md 已被改。

量（`probes/r7-records-scan.py`，冻结副本 records/ 下 95 份，调度记录按禁读跳过；`out/r7-scan.txt` 原样）：

```
name=r7-scan records_files=95 source_lines=557 round_names=180
name=r7-scan source_lines_without_halt_words=504 round_names_only_in_lines_without_halt_words=146
```

即今天的 records 里有 504 行按 ⑥ 能当出处、行里却没有叫停、停掉、停了、撂下、不判任何一个词，146 个轮名只出现在这样的行里——这些轮任何一个开着时，主 agent 引那一行就能把它关掉。

四句：
- 分辨臂：分。「那一行里除了这一轮的名字，不许有别的轮名」能拒第 3 行（推的）；「那一行要有叫停一类的词」能拒第 4、5 行，拒不了第 6 行（「先放一放」不是叫停词，但「放一放」也可以写成「停一下」）。
- 看得到吗：第 3 行看得到（同一行两个轮名）；第 4–6 行这句话是不是「用户为这一轮叫停」钩子看不到，规则自己也写了由主 agent 核——这一半是「判别子观测不到」，不算钩子的错。
- 分句：第 3 行满足跑前条款第二行后半：规则第 121 行把别的轮的叫停原话列进 ⑥ 不收的那几条，钩子收了。第 4–6 行只满足「由主 agent 核」那一句管的范围，按文字没写到记。
- 改前：第一轮那一版 ⑥ 不读登记表，同一张表各格都 rc=2；改成第一轮的写法（halted 引 records/r7.md 里别的轮的那句原话）照收（halted-other-rounds-quote 改前 rc=0），第一轮 F1 已报「挡不住引一句别的轮的用户叫停原话」——这是改后又中。
- 改法：P7（推的，零轮）。

### D2　同一张表，⑥ 与门禁 abandoned-rounds 格判法不一（⑥ 量过，门禁格推的）

⑥ 只判四列不空（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/hooks/runner-dispatch-guard.sh:779`：「if len(fields) != ABANDONED_ROUND_COLUMNS or any(not field.strip() for field in fields):」），有一行出处收得了就收；门禁格另判日期（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-process-records.sh:803`：「if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", fields[1].strip()):」）与同一轮两行（第 810 行 duplicated_registrations），却不看出处。

| 行 | ⑥（量过） | 门禁格（推的；小树 `gate-samples/doc-process-records/`，要主 agent 跑） |
|---|---|---|
| 日期写「10月13日」，出处合格 | 收（date-not-iso rc=0） | 退 1：第 1 行日期不是 YYYY-MM-DD |
| 同一轮两行：第一行出处行号超出、第二行合格 | 收（duplicated-rows-second-good rc=0） | 退 1：登记了不止一行 |
| 没写出处（「原话见调度记录」） | 不收（no-source rc=2） | 退 0：登记撂下 1 轮 |

第二种是照拒绝说明做出来的：⑥ 第一次不收时，notes 里写「第 N 行的出处……」，主 agent 补一行合格的比改原行顺手；⑥ 放了、写回员派出去了，提交时门禁格才红。第三种反过来：门禁绿，派发一直被挡。

四句：分辨臂——分（门禁格也调 ⑥ 那一份出处判法、⑥ 也判日期与重复，任一种不中，推的）；看得到——两边读的是同一个文件；分句——正文 R7 第二问要的「一处收、一处拒的行」；改前——不中（改前 ⑥ 不读这张表，两边本来就各管一半，第一轮 F3 判的是那一种）。

## R9　私有值与豁免表

门禁格子 agent 不起；下面 E1、E2 的四份小树在 `gate-samples/doc-text/`，形态照冻结副本门禁样本 `private-values-red`（拷了它的假 `multi-host.env`、`.private-values-hostname`、`.gate-cells`），`setup.sh` 建 git 仓提交一版干净的记录、再加这一轮的新文件。要主 agent 跑 `tools/run-gate-samples.sh`。

### E1　新加的未跟踪文件只读前 8 MiB（推的，读码）

格文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-text.sh:47`：「新文件整份都算新加」。代码 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-text.sh:336`：「READ_LIMIT = 8 * 1024 * 1024」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-text.sh:421`：「content = handle.read(READ_LIMIT)」，未跟踪文件的「新加的行」就是这 8 MiB 切出来的行。

历史：执行员跑一条长产物，日志里每段打一行进度，末尾打一行「第二台 <PEER_SSH_HOST 的值> 跑完了」（`scripts/hosts/peer-host-lib.sh` 第 87 行拼 rsync 目标就是「值:路径」）；产物 9 MiB，落进 research/results/ 还没提交。门禁这一格只读到第 8 MiB，判绿。仓里今天已有 10 份超过 8 MiB 的产物（主工作区 `find research/results -type f -size +8M | wc -l` 得 10，最大一份 115269707 字节），这种大小是常态。

小树 `pv-big-untracked` 推的结局：退 0，「✓ 查了这次改动里 1 份文本文件新加的 … 行 … 没有一处命中」。

### E2　文件名里的值不判；文件名带值又命中内容时，红行把值打出来（推的）

格只比行文（第 421 行起读内容、按行比）。格文件头 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-text.sh:49`：「输出：只报文件、行号与键名」，打印那一行 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-text.sh:451`：「print(f"      {path}:{number}  键名：」——`path` 里带值就打出来。管不到的那一句（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/gate.d/doc-text.sh:51`：「管不到的：换了写法的值」，后面列了大小写、拆开、别称、commit message、短词）没列文件名。

小树推的结局：`pv-filename-only`（值只在文件名里）退 0；`pv-filename-and-content`（文件名与第 1 行都带值）退 1，红行是「research/results/e999-<假主机名>.out:1  键名：PEER_SSH_HOST、QUIESCE_STOP_COMMAND（值不打印）」——值经路径进了门禁日志。

### E3　词源只有 multi-host.env 与 hostname（推的；计数量过，不打值）

`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/agent-common.md:47`：「真实值只在仓根被 git 忽略的 multi-host.env 与 ~/.config/singlefs/ 里」，同一行列了「清场脚本的内容」，并写「门禁 doc-text 的 private-values 格判新加的行」。格只读 multi-host.env（`SINGLEFS_MULTI_HOST_CONFIG` 或主工作树根下那一份）与 hostname。本机 `~/.config/singlefs/` 下 12 份文件，按 8 字以上切词，有 342 个词不在 multi-host.env 的词里（`probes/config-words.py`，只打计数；复跑 `python3 -I probes/config-words.py <冻结副本>/multi-host.env ~/.config/singlefs`）；其中带执行位的 5 份（按路径排第 5、8、9、10、11 份，是脚本）各有 69、76、138、124、135 个这样的词；拿第 8 份（换一种切法得 70 个）去冻结副本里找，50 个一份文件都不出现（`probes/config-words-in-repo.py`，只打计数）。它们里有多少是主机名、服务名、路径，我没看（看就要打值），交主 agent 判。若有，清场脚本里的服务名、路径被原样贴进报告时这一格看不到。

四句（E1–E3）：分辨臂——这一格判的是门禁覆盖面；看得到——E1、E2 看得到（文件就在改动范围里，名字也在）；E3 格看得到 ~/.config/singlefs/ 但不读；分句——第二行后半：格文件头「新文件整份都算新加」「只报文件、行号与键名」、共用约束「真实值只在 … 与 ~/.config/singlefs/ 里」；改前——不中（新格）；改法——P10（推的，零轮）。

### R9 里没打中的

- 没有 multi-host.env 退 77：`.claude/gate.d/lib/stage-cells.sh` 第 37–39 行，有格 77、别的格 0 时整道退 0、77 那一格往 `GATE_NOT_RUN_FILE` 报一行，上游 `gate.sh` 第 207 行收成「本次未跑一部分」——不说成判过。
- 大小写、拆开写、URL 编码、commit message：格文件头「管不到的」已列，不算打中。
- 前 8 KiB 带 NUL 的文本整份按二进制跳过（第 422 行）：形状成立（小树 `pv-nul-head` 推的结局退 0、「二进制 1 份」），但主工作区 research/results 下文本后缀的文件前 8 KiB 带 NUL 的是 0 份（我数的），按没打中记。
- 第 9 条「有别人没审过的改动就不登记」：两个会话各在同一份定义里有没审过的改动、各自只有用户豁免时，两边都不登记、都等对方——`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo/.claude/main-agent.md:27` 同一条另有「别人没提交的改动挡着这一批的活时，用 SendMessage 发给那个会话」协商、送不到弹窗的出路，没造出走不通的历史。
- 第一轮那两处已推到远端的私有值：冻结副本里仍在（records/2026-09-30-里程碑三开工第一轮.md 第 1004 行有 PEER_SSH_HOST 的值，research/prompts/m3-batch10x-code-r1-opus-output.md 第 295、302 行有本机 hostname；我只数行号），正文表里「只在工作区换掉」在冻结副本这一版里还没落地。是不是在别处换了，主 agent 现查。

## 改法与各格（全部只在我的模型上想过、没实现、被攻过零轮；每格都是「推的」）

| 改法 | 内容 | A1 | A2 | A3 | A4 | B1 | B2 | B3 | C1 | C2 | D1 | D2 | E1 | E2 | E3 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P1 | ⑮ 的括注只许一张白名单里的词（本机、登记值、默认这类），或括注里出现十、百、千、半、分之、合计、减、各一类数词就拒 | 修（推的） | — | — | — | — | — | — | — | — | — | — | — | — | — |
| P2 | ⑮ 只把 `<填` 开头的当占位交 ⑬，别的 `<…>` 按认不出数拒 | — | 修（推的） | — | — | — | — | — | — | — | — | — | — | — | — |
| P3 | 共用约束 G2 那一句补「派发提示没有线程上限一行的，照 verification 第 26 行用 capped.sh default」，或类型表加 clerk、three-way-defense | — | — | 修（推的） | — | — | — | — | — | — | — | — | — | — | — |
| P4 | main-agent 第 53 行「让出 CPU」限定成「用户或别的会话在消息、弹窗里明说」，并写「只看到负载忙不算」 | — | — | — | 修（推的；钩子不审理由，仍靠主 agent） | — | — | — | — | — | — | — | — | — | — |
| P5 | 仓副本里的自证另比「副本那一份脚本与主仓那一份 sha256 相同」才放；或保留放宽、在共用约束「执行前拒绝的写法」写明「不读正文，自证变重的先把副本登记行改成 heavy」 | — | — | — | — | — | — | — | 前一种修、后一种只补文字（推的） | 不碰 | — | — | — | — | — |
| P6 | 重型测试闸认 `env -C`、`env --chdir=` 换的目录 | — | — | — | — | — | — | — | 不碰 | 修（推的） | — | — | — | — | — |
| P7 | ⑥ 出处那一行除这一轮外不许有别的轮名；规则第 121 行把「别的轮的叫停原话」从 ⑥ 不收的那几条里挪到「由主 agent 核」那一句 | — | — | — | — | — | — | — | — | — | 第 3 行修、第 4–6 行只改文字（推的） | 不碰 | — | — | — |
| P8 | 门禁 abandoned-rounds 格与 ⑥ 共用一份登记行判法（日期、重复、出处三样两边都判） | — | — | — | — | — | — | — | — | — | 不碰 | 修（推的） | — | — | — |
| P9 | heavy-thread-defaults：cfg(test) 后第一行是 mod 才跳过；回退判法看同一个变量名出现在回退式里；另认命令前缀赋值、`-j N`、`--test-threads N`、`:=N` | — | — | — | — | 修（推的） | 修（推的） | 修，但 replay.sh 22 行要另登记或改（推的） | — | — | — | — | — | — | — |
| P10 | private-values：未跟踪文件整份读（或超过上限时判红而不是截断）；改动范围里的路径也比；红行里的路径命中私有值时打成「〔文件名含私有值〕」；词源加 ~/.config/singlefs/ | — | — | — | — | — | — | — | — | — | — | — | 修（推的） | 修（推的） | 修（推的；误报没量） |

A3 用 P3 修文字，但 A3 那段历史里的子 agent 若照 P3 用 `capped.sh default`、用户又设了 `SINGLEFS_THREAD_CAP`，回到第一轮 A1 的分叉（推的）。

## 要主 agent 跑的命令

1. 门禁格三份小树：工作目录随意，不用环境变量，不起 cargo（门禁格里的 python 与 git），内存与线程用不上：
   `bash research/prompts/m3-defs-b3-r2-opus-model/tools/run-gate-samples.sh ~/.local/share/singlefs/work/m3-defs-b3-r2/repo <新的工作目录>`
   看每份 `name=gate-sample … rc=` 那一行和下面的 ✓ / ✗ 行，对 `gate-samples/*/README.expect.txt`（与上面 B1、B2、D2、E1、E2 抄的推的结局相同）。推翻条件：pv-big-untracked、pv-filename-only 判红，或 r2-thread-gaps 点名了 env_prefix.sh、device_after_cfg_test.rs、mentions_capped.py 之一，或 r7-no-source 判红、r7-date-not-iso / r7-duplicated-rows 判绿——那一格的打中就不成立。
2. 复跑（核我的读数）：见开头「复跑」。

## 没打中的形状（试过的写法与取样范围）

- R1：⑮ 喂了 19 种提示（`out/r1-r2.txt` 每行一种），三种类型；全角数字、上标两种只会多拒；第二台另起一行的三种标签⑮ 不管，按文字没写到记；H3 与第 39、49 行没造出冲突。
- R2：capped.sh 清单与装置逐个对了 27 个线程变量名（`grep` 带引号的全大写名字，含自证用的桩名），已在排除表或回退的不另算；E175 的装置在 research/results 下，扫描目录不含它，后果只是 `capped.sh N` 到不了（它必须显式设）。
- R5：`-p` 九种写法、bench、`cargo run --bin`、副本在工作根外面，改后都拒；符号链接推的不中。
- R7：halted 那一支改后不再认（halted-other-rounds-quote 改后 rc=2）；出处指仓外、带 `..`、指这一轮自己的材料，钩子自证格已覆盖，我没重喂。
- R9：77 那一格、短词、commit message、第 9 条，见 R9 那一节。

## 这条腿自己的限度

- 门禁格（B1、B2、D2 的门禁那一半、E1、E2）一格都没跑，结局是读码推的；B1、B2、B3 的正则层是把正则原样抄出来判单行，不是跑整格。
- 派发闸、重型测试闸喂的是钩子入口，没带主会话记录（`transcript_path`），⑦ 并发、⑯ 写者锁、回执这几条没参与；提示里有意不写行首「草稿目录：」以免 ⑯ 建锁。
- C1 的后果（子 agent 真的起了 checker 档测试）没真跑，只喂了命令行；副本里那份脚本没执行过。
- E3 里 ~/.config/singlefs/ 那些词是不是私有值我没看，只给了计数。
- R7 的 records 计数用的是我写的轮名正则（以 `-r<N>` 结尾）与叫停词表，只是量级。
- 每格只抽了一次样；没打中的格按规则要另抽一次才算「没打中」。

## 没做什么

- 不判 R3、R4、R6、R8（辩方腿的格）；没替主 agent 采纳；样本根与副本上的读数不算入库装置上的数。
- 没跑任何 `--selftest`、门禁阶段、cargo、重型测试；没编译。
- 改法一个都没实现。
- 没读禁读清单里的文件；records 扫描按文件名跳过了调度记录。

## 草稿目录

`~/.local/share/singlefs/work/m3-defs-b3-r2-opus/`：探针的两趟复跑目录 `rerun-a`、`rerun-b`（各是冻结副本 `.claude` 与 `scripts` 的几份拷贝，加一份只含 Cargo.toml 的仓副本）交回前删掉；留下 `scan/` 两份计数脚本（已拷进模型目录 `probes/`）与几份 diff。
