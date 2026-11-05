# d137-gov-r1 攻方腿（Opus）：G3 过度外推、内部矛盾、最脆弱的一条

轮名 d137-gov-r1，第一轮；前几轮判决：无。背景材料 `research/prompts/_d137-gov-r1-body.md`。
开工时核过：快照 `research/prompts/d137-gov-r1-snapshot/files-sha256.txt` 里 41 份与主工作区此刻逐份 sha256 相同（`sha256sum -c` 无一行不是 OK）。
草稿目录 `~/.local/share/singlefs/work/d137-gov-r1-opus`；模型目录 `research/prompts/d137-gov-r1-opus-model/`。
禁读清单（正推腿报告与模型）没读。

## 复跑

整份复跑（在新 shell，工作目录随意；两个模型都只调钩子与 core_usage 的纯函数、喂钩子，不起 cargo、不碰盘，故不必经内存包装）：

```
bash research/prompts/d137-gov-r1-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen <冻结副本> --work <新的空草稿目录> \
  --model research/prompts/d137-gov-r1-opus-model
```

本腿交回前在新 shell 里整份跑过一次，退 0（`--repo` 指主工作区、`--frozen` 指 `research/prompts/d137-gov-r1-snapshot`）。

模型文件 sha256：

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | e68c928b539bfec8dd76bda402dbb0aa26061ef9087d4df77fc062c1a53acda2 |
| prototypes/decl_silencing_model.py | 6506d81dcefe1f54e7ad16249e4c1c80e31e1c3a37935c30144d8c60f4e9fef0 |
| prototypes/bash_env_forwarding_escape.sh | 30a363bdab176318a7383d567597fb18261c952a41cb6f96fb5819e8eae97a8d |
| prototypes/feed_hook.py | dde743a18523a32e55a76646c66e91df7c070b61fd892d56e87dc9014871b986 |
| out/decl_silencing.txt | 88244353aa700adafba070060a0477ccfb11d25c3404eb85fe7f09991bcbd57b |
| out/bash_env_escape.txt | 89eff0eab6b6bbda1f0d932dc15f867f6abf514c1565f26cb4a6706bc1727183 |

## 分片进度


- 片一（过度外推与越位）：做完，打中 2 条。
- 片二（内部矛盾与循环依赖）：做完，打中 2 条。
- 片三（最脆弱的一条）：做完，打中 1 条主、1 条次（与片二的声明静音同源，同一机制两个面）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G3-过度外推-① 声明了串行段只记不叫醒不漏报 | 打中（量过） | 任一条 declared_in_window>0 就整条低核静音，含「只声明 1 秒随即闭合」与「声明一次永不闭合」两种普通写法，真卡住 599/600 秒也叫不醒 |
| G3-过度外推-② 认不出的一律拒（偏严） | 打中（量过） | 「偏严」只覆盖命令文本这一层；BASH_ENV + 转发函数复合起来，checker 档 cargo 被静默放行、连检出都不记 |
| G3-内部矛盾-① core_usage 文件头与看门狗门槛不一致 | 打中（量过＋推） | core_usage.py 第 56 行「超过整趟一半才另注」，看门狗第 3476 行 >0 就静音：两处对「声明到什么程度算数」口径不一 |
| G3-内部矛盾-② 看门狗跨会话单元认法 vs detached-run 口径 | 打中（推） | detached-run 单元由脚本起、主会话记录里没有 --label 时被当成别人的只记不叫醒，自己的长活低核会漏报 |
| G3-最脆弱 声明静音 + BASH_ENV 绕过 | 打中（量过） | 见片三 |

## 片一　过度外推与越位

### ① 「只记不叫醒也不漏报」越出量过的范围：声明一次就把真卡住静音

被判断言在三处：
- `~/.local/share/singlefs/work/d137-watch/result.md`「第 1 条 ①：看门狗噪声」表里「声明了串行段的不叫醒」一行，自证格 `selftest_core_usage`、`selftest_core_usage_quiet`。
- `~/.local/share/singlefs/work/d137-watch-follow/result.md`「第 1 条」表把 main-agent.md「派出去之后」那一句改成「声明了串行段的看门狗只在报告里记一行、不叫醒，那一行带串行段秒数与理由」。
- `scripts/hosts/core_usage.py`（sha256 ec112292）第 142 行 `DEFAULT_NEXT_STEP` 收尾「看门狗对声明了串行段的低核读数只在报告里记一行、不叫醒主 agent」。

看门狗里真正的判法（`scripts/agents/agent-watch.py`，sha256 9334380a）两处都是「declared_in_window 大于 0 就静音」，不是「超过一半」：
- 第 3476 行 `elif verdict["status"] == "low" and verdict["declared_in_window"] > 0 and BROKEN_DETECTION != "declaredwakes":`（本机重活那一半）。
- 第 3536 行 `if (record.get("declared_in_window") or 0) > 0 and BROKEN_DETECTION != "declaredwakes":`（告警记录那一半）。

反例（量过，模型 `research/prompts/d137-gov-r1-opus-model/prototypes/decl_silencing_model.py`，只调 `core_usage.judge_series`、`parse_declarations`，不起 cargo、不碰盘）：造一段「单线程忙等 600 秒、只用 1 核」的样本（真卡住的形状，在用核数 1.0 远低于阈值 7.8），比三种声明。原样输出 `research/prompts/d137-gov-r1-opus-model/out/decl_silencing.txt`：

```
name=decl_silencing tag=no-declaration-stuck status=low cores=1.0 threshold=7.8 declared_in_window=0 declared_fraction=0.00 watchdog_silenced=False
name=decl_silencing tag=one-open-declaration-never-closed status=low cores=1.0 threshold=7.8 declared_in_window=600 declared_fraction=1.00 watchdog_silenced=True
name=decl_silencing tag=one-second-serial-then-closed status=low cores=1.0 threshold=7.8 declared_in_window=1 declared_fraction=0.00 watchdog_silenced=True
```

- 「声明一次永不闭合」：程序只写一条 `<秒>\t此刻串行：<8 字理由>`、再不写「此刻并行」，`serial_intervals`（core_usage.py 第 347 行起）把未闭合的段算到当下，declared_in_window = 整窗口 → 看门狗此后每个窗口都静音，哪怕进程死锁。
- 「声明 1 秒随即闭合」：declared_fraction 只有 0.00（远不到一半），declared_in_window=1，看门狗照样整条静音。

越位在哪：报告与文件头把「声明了串行段只记不叫醒」说成一条稳妥的降噪，隐含「声明得当才静音」；实测是任一条 >0 的声明就让这个重活从此叫不醒主 agent，`selftest_core_usage` 的样本只验了 `declared_in_window=5.0`（d137-watch/result.md「红绿样本」里 declared-unit 那一格）证明静音会触发，没验「真卡住 + 琐碎/陈旧声明仍静音」这一面。打中的是用户口径里 ①「真卡住、真漏跑的情形从此叫不醒主 agent」。推翻条件：`judge_series` 对 declared_in_window>0 不再整条静音、改成按 declared_fraction 过半才静音。

### ② 「认不出的一律拒（偏严）」越出命令文本这一层

`.claude/hooks/heavy-test-guard.sh`（sha256 83aacdab）文件头「认不出的写法（用户 2026-10-29 弹窗定『认不出的一律拒（偏严）』…）」那一段，和 `~/.local/share/singlefs/work/d137-hooks/result.md`「第 2 条」把 Z1/Z2/Z9/Z16 一律归到「认不出的一律拒」。实测「偏严」只在命令文本这一层成立：同文件头「看不见的（照常放行、不记检出）」一段自己列了 make、python subprocess、eval 参数、脚本正文里变量拼的命令等照放。这几种是「已知没认」，不报；但同类里有一种不在已知名单、却同样静默放行的写法——见片三 ②（BASH_ENV + 转发函数）。这一条与片三 ② 是同一个打中，列在此处点明「偏严」这个措辞的射程被说宽了。

## 片二　内部矛盾与循环依赖

### ① core_usage 文件头与看门狗对「声明到什么程度算数」的门槛不一致

- `scripts/hosts/core_usage.py`（sha256 ec112292）第 56 行：`声明不让告警不报：核数过少照报 ✗，只是标出这一窗口里声明的串行段有几秒、理由是什么；声明的串行段累计超过整趟（起跑到此刻）一半的，另注一句「超过一半」。`
- `scripts/agents/agent-watch.py`（sha256 9334380a）第 3476、3536 行：declared_in_window > 0 就把这条低核读数判成只记不叫醒。

矛盾：core_usage 这一侧把「声明」摆成「不让告警不报」、只有「超过整趟一半」才特别注一句，读这段文件头的人会以为一半是那条有意义的线；看门狗这一侧只要 >0 就整条静音，没有一半这道门槛。两份治理文件对同一件事（声明到什么程度不叫醒主 agent）给了不同的口径。片一 ① 的 `one-second-serial-then-closed`（declared_fraction=0.00、<一半）就落在两者的缝里：core_usage 连「超过一半」都不注，看门狗却已经静音。判定：打中（core_usage 第 56 行量过、看门狗第 3476 行量过，「读文件头的人会怎么理解」是推的）。推翻条件：把看门狗的静音门槛对齐成 declared_fraction 过半，或把 core_usage 第 56 行改成「任一条声明即不叫醒」。

### ② 看门狗跨会话单元认法与 detached-run「脱离会话」的口径对不上

- `scripts/hosts/detached-run.sh`（sha256 7dbbbcc7）第 3 行：`长跑脚本脱离会话跑：挂到用户级 systemd 的一个临时服务单元下，不在调用方的进程树里，调用方被停、会话的后台时限到点、会话关掉都不影响它；`
- 看门狗认「本会话起过哪些单元」只认主会话记录里主 agent 的 Bash 命令 `detached-run.sh start … --label <名>`（`scripts/agents/agent-watch.py` 第 3313 行 `session_unit_labels`、第 3330 行 `foreign_unit`）。认全了而某单元名不在里面，就当别的会话的、只记不叫醒（第 3527 行 `foreign_seen`）。

矛盾：detached-run 明说单元可由脚本在会话里起、脱离进程树；而 `d137-watch/result.md`「没做什么」自己也写「由脚本自己起、又没在任何命令里写过名字的单元认不出……会被当成别人的只记一行」。于是一条由脚本（而非主 agent 直接 Bash）起的、属于本会话的长活，一旦在用核数过低，看门狗会把它当成别人的单元静音——这正是「脱离会话」口径与「只认主 agent Bash --label」认法的裂缝，落在用户口径「真卡住、真漏跑叫不醒」里。判定：打中（代码位置量过，「会漏报本会话长活」是推的，没在真多会话环境跑）。推翻条件：看门狗按单元登记目录（detached-run 的登记）而非主会话 Bash 文本认归属。

### 循环依赖

没找到会死循环的依赖链。看门狗 import core_usage、lib_heavy_tests（agent-watch.py 第 2460、3151 行），core_usage import lib_work_root；import 方向单向、无环。内存包装的 `memory_peaks_lib.pending_bytes` 与包装排队口径一致（都跳过 `<unit>.card-wait` 标记，`memory_peaks_lib.py` 第 548 行注释「同包装排队那一步的口径」），互相读的是同一份账、不互相调用，没有循环。

## 片三　最脆弱的一条

两条同源，都在看门狗/重型闸「认不出就从宽」的一侧。

### 主：声明静音——一条普通脚本写一次串行声明就能让自己此后叫不醒

最脆弱的是 `scripts/agents/agent-watch.py` 第 3476、3536 行的 declared_in_window>0 整条静音。普通到什么程度：任何经 `scripts/hosts/detached-run.sh start` 或 `multi-host-run.sh --peer` 起的长活，环境里都已设好 `SINGLEFS_SERIAL_DECLARATIONS`（core_usage.py 第 54 行「谁设这个变量」）；脚本里 `core_usage.py declare-serial <理由>` 写一条、之后不 `declare-parallel`，看门狗此后每个窗口都把它的低核读数静音。`scripts/crash/gpu-quiesce-run.sh` 第 391 行 `gpu_quiesce_serial_declare` 正是这个用法——它在命令退出时才 `gpu_quiesce_serial_withdraw`（第 412 行），命令若在卡上死锁、`withdraw` 不触发，声明一直开口，看门狗再不叫醒。反例与原样输出同片一 ①（`out/decl_silencing.txt` 的 `one-open-declaration-never-closed` 行：declared_in_window=600、watchdog_silenced=True）。

分辨臂：这一条不分辨用户在弹窗里定的任何一条臂（钩子判法/偏严三类/撤旧根/等卡/准入各臂），它是看门狗降噪那一批改动自身的病根，用户动作没写死进模型（模型只造样本与声明文本，不涉及用户选项）。被判的系统当时看得到的东西：看门狗在做静音决定那一刻看得到 declared_in_window 的具体秒数与 declared_fraction，区分得出「过半」与「只 1 秒」，所以这不是「判别子观测不到」，是判据门槛取宽了。满足的是用户口径 ①「只记不叫醒……有没有真卡住、真漏跑的情形从此叫不醒主 agent」字面那一句。

### 次：BASH_ENV + 转发函数复合，静默放过 checker 档 cargo

`.claude/hooks/heavy-test-guard.sh`（sha256 83aacdab）单独认得 BASH_ENV 启动文件（文件头「shell 启动时先读的文件」一段），也单独认得转发函数（文件头「脚本里自己的转发函数」一段）；两者复合——把 bash 包进一个转发函数、BASH_ENV 指向一份起 checker 档测试包的脚本——重型闸看不见那份启动文件，静默放行，连检出都不记。模型 `research/prompts/d137-gov-r1-opus-model/prototypes/bash_env_forwarding_escape.sh`（经 `feed_hook.py` 用独立进程组喂钩子），原样输出 `research/prompts/d137-gov-r1-opus-model/out/bash_env_escape.txt`：

```
name=bash_env_escape form=direct
rc=2
rc=0
name=bash_env_escape form=forwarding-function detections_after=0
```

- `BASH_ENV=sub/startup.sh bash -c true`：rc=2（重型闸拒）。
- `f() { bash -c true; }; BASH_ENV=sub/startup.sh f`：rc=0（放行），且检出记录 0 行（静默，不是「放行但记检出」）。

分辨臂：这一条打中用户口径「认不出的一律拒（偏严）」那一臂，正是它许可的——`heavy-test-guard.sh` 文件头说「认不出的一律拒」，而这里既没拒、也没记检出，落进「看不见」的缝。它不是「已知没认」名单里的四种（登录 shell profile、系统级启动文件、别的程序里起 shell、`bash x.sh` 正文里起 claude），是同类的一种新写法。被判系统当时看得到的东西：命令文本里有 `BASH_ENV=…` 与函数定义，看得到；是判法没把启动文件跟进转发函数里的那一层 bash。这一条标「量过」。推翻条件：heavy-test-guard 把 BASH_ENV 启动文件跟进经转发函数起的 bash，这条命令改判 rc=2 或至少记一条检出。

## 打中之后的四句（声明静音，主打中）

1. 分不分辨臂：不分辨用户任何一臂，是看门狗降噪改动自身的病根。
2. 系统当时看不看得到判别它的东西：看得到——declared_in_window 的秒数与 declared_fraction 都在 verdict 里，区分得出「过半」与「只 1 秒」。
3. 满足的是判据字面哪一个分句：用户口径 ①「只记不叫醒……有没有真卡住、真漏跑的情形从此叫不醒主 agent」。
4. 跑前条款给的每个改法在打中的格上还中不中：本腿只提一个改法（见下），它正对这一格。

## 这条腿自己提的改法（被攻过零轮，只在我的模型上量过）

把看门狗静音门槛从 declared_in_window>0 改成 declared_fraction 过半（与 core_usage.py 第 56 行的「超过整趟一半」对齐）。只在 `decl_silencing_model.py` 上推演过，没落进 `agent-watch.py`、没跑它的自证与开关轮，被攻过零轮。它修的格：片一 ①、片二 ①。各格量过/推的：

| 格 | 这个改法修不修得到 | 量过/推的 |
|---|---|---|
| 片一 ① one-second-serial-then-closed（declared_fraction=0.00） | 修得到：过半才静音，0.00 不再静音 | 量过（模型里 declared_fraction 已算出 0.00） |
| 片一 ① one-open-declaration-never-closed（declared_fraction=1.00） | 修不到：永不闭合的声明 fraction=1.00 仍过半、仍静音——真死锁的那种照旧叫不醒 | 量过（模型里 fraction=1.00） |
| 片二 ① 文件头与看门狗门槛不一致 | 修得到：两边都用过半 | 推的 |

注意：这个改法对「声明一次永不闭合」那种真死锁无效（fraction 到 1.0），交出去时写明「对永不闭合的声明这一格不起作用」。更稳的另一路（未实现、推的）：声明设一个到期时间，过期的声明不再静音，逼程序周期性续声明，死锁的续不了就恢复叫醒。

## 没打中的形状

- 循环依赖：看门狗/core_usage/lib_work_root/memory_peaks_lib 的 import 与读账方向都单向，没找到环（取样范围：agent-watch.py 第 2460、3151 行两处 import，memory_peaks_lib.py 第 548 行注释口径）。
- 等卡标记被普通命令滥用：`RUN_WITH_MEMORY_CAP_CARD_WAIT_MARK` 命令「只写不读」，命令可自建标记把自己从账上摘掉——但这是 `run-with-memory-cap.sh` 文件头第 130 段已写的「已知的局限」（涨过头由盯账与 oom_score_adj 接住），不另报。
- `pending_bytes` 跳过标记：`memory_peaks_lib.py` 第 548 行与包装第 1858 行两处口径一致，没找到对不上。
- 撞顶 scope 读早了（⑧）、failed-Result-success（另一种读错）：两条都在文件头写了实测复现条件与改法，没找到它们判错的新反例（没在真整机高负载下复跑，归限度）。
- 准入退码（用户第五条）：没攻，不在本腿分到的三面里的核心，且正推腿 G1 覆盖。
- BASH_ENV 的别的复合写法：试过 `make`、`python3 -c os.system`、`git`、`exec/command/nice/timeout/env` 前缀（取样 16 种，`feed_hook.py` 喂）——前缀类都照剥后判（rc=2），make/python 是已知没认；只有转发函数这一种复合是新的静默放行。

## 这条腿自己的限度

- 声明静音、BASH_ENV 两条都在拷进草稿目录的最小钩子树（`~/.local/share/singlefs/work/d137-gov-r1-opus/try/tree`，从主仓 `.claude/hooks`、`.claude/scripts`、`.claude/singlefs-ai-sop/scripts`、`.claude/heavy-thread-default` 拷）上喂钩子、或直接 import `core_usage` 的纯函数量的，主工作区一个文件没改。模型目录里 `rerun.sh` 跑的是 `$REPO_ROOT`（主工作区此刻的代码），不是副本。
- 声明静音模型只造了 samples 与声明文本，没真起一个单线程死锁进程让看门狗整趟跑；判的是 `judge_series` 的返回与看门狗第 3476 行的条件，没经 `agent-watch.py` 的 `--selftest` 整趟。真多会话、真 systemd 单元下的跨会话漏报（片二 ②）没在真环境复现，标「推的」。
- 撞顶 scope 读早了那一类要真整机高负载才复现，本腿没跑。
- 没读正推腿的报告与模型（禁读清单），没做逐条对照。

## 没做什么

- 不判 G1、G2 两格（正推腿），不替主 agent 采纳。
- 副本/最小树上量出的数不算入库装置上的数；本腿提的改法被攻过零轮。
- 没跑重型测试、没起 cargo、没碰 GPU；所有探针是喂钩子（独立进程组、超时杀整组）与 import 纯函数，没经内存包装（都不是编译出来的代码）。
- 交回前 `ps` 核过自己起的样本进程：喂钩子的 bash/python 都在独立进程组、同步 communicate、超时 killpg，没有留下的进程。
