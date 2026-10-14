# m3-defs-b3-r3 攻方腿（Opus）报告（2026-10-13）

攻击面：正文「一」那张表 T1、T2、T3、T4；第三轮分工「已定」项撑不撑得住，只攻前两轮站住的形态（R4、R6、R8 与第二轮辩方第 1 节判「修到」的条目）。只读冻结副本：第三轮 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/`（开工对 `research/prompts/m3-defs-b3-r3-snapshot/frozen-copy-sha256.txt` 跑 `sha256sum -c`：35 份 OK、非 OK 0 份），第一轮 `…/m3-defs-b3-r1/repo/`，第二轮 `…/m3-defs-b3-r2/repo/`（只在 T3 上补跑了一遍）。引文写冻结副本的绝对路径，行号是这一次 `grep -nF` 现取的。没读禁读清单里的文件。冻结副本根下的 `multi-host.env` 没拷、没读。

## 复跑

```
FROZEN_ROUND1=~/.local/share/singlefs/work/m3-defs-b3-r1/repo \
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-defs-b3-r3-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-defs-b3-r3/repo --work <新的草稿目录>
```

`tools/rerun.sh` 拷自 `scripts/agents/three-way-rerun-template.sh`，只改了 `rerun_steps`：把第三轮与第一轮冻结副本各拷 `.claude`、`scripts`、`CLAUDE.md` 进 `--work`，在两棵树上跑 `probes/` 四份探针，末尾八行 `name=compare file=<探针>-<r3|r1>.txt same` 即与 `out/` 逐字节相同。不起 cargo、不跑任何 `--selftest` 与门禁格；钩子都是喂 JSON 给入口或走 `--check-prompt`，`CLAUDE_PROJECT_DIR` 指拷出来的树，`SINGLEFS_WORK_ROOT`、`DISPATCH_GUARD_STATE_ROOT`、`AGENT_HOOK_DETECTIONS` 都指 `--work` 里（不写真的工作根与检出记录）。这一次两趟：`rerun-a` 生成 `out/`，`rerun-b` 新目录复跑，八行都是 `same`（原样输出见末尾「复跑原样输出」）。

模型目录各文件 sha256（`cd research/prompts/m3-defs-b3-r3-opus-model && find . -type f | sort | xargs sha256sum` 原样）：

```
e30bc68d30085935d410536bcc7a11a0932326e2a90e474dc35defa893a858c8  ./out/t1_peer_env-r1.txt
70cec7e0eab667cb361dd18a0e13a5f0fc72842439be614fafa8cc8109830e56  ./out/t1_peer_env-r3.txt
f0436a9d79a045c07eb1e624b8b1252a459822ece913e87e4e5e1af95700c78a  ./out/t2_nested-r1.txt
0770f6a5449a2b9491c2b8687e3c7ba89d97015bf00f0e08bc2aef77f5d9ae47  ./out/t2_nested-r3.txt
7bb78a7e90b4d37fc736875d6af8fe23fd9b518cd1919561dd9e3af7f5731e64  ./out/t3_dispatch_forms-r1.txt
9937de887c0394d6df667e3817f7bc0b38ee790e9e2643d7a31e818e8250bda8  ./out/t3_dispatch_forms-r3.txt
f1aa9cd6e5c9932edc3b46284f439409ccc84b2032d0ce191934d0c09366b862  ./out/t3_long_run-r1.txt
6b472236db1006bca234cde54479f13aabf4f5d4bbfecf6ba09005a45b8269f3  ./out/t3_long_run-r3.txt
60f73ca7e568022ab057d5f1cad5fe0d91f0dfb6a24489935425f80b6f00599a  ./probes/t1_peer_env.sh
bca765907bf26c7715348f41b2cc0be22baf0d93f7a4cbec5103f5af5d044585  ./probes/t2_nested.py
f68bc88f3d1a7f6c3c761b5c01fa320e1da68c0f4eb4119d30f7e0679e15a09f  ./probes/t3_dispatch_forms.py
752102c9ea637ff0ce1edc3428375264a982d943015205b5efc77052320f6f88  ./probes/t3_long_run.py
e76a2d405eaafdbd42fdc0c3fb991e87fed5e39d346d32adb46b4e0a376a370f  ./tools/rerun.sh
```

门禁格这一轮一格都没用到，没有要主 agent 跑的样本树。

## 各格判定一览

| 格 | 编号 | 判定 | 打中的历史（一句） | 量过 / 推的 | 跑前条款哪一行 | 改前（第一轮冻结副本）中不中 |
|---|---|---|---|---|---|---|
| T3 | P1 | 打中 | 派 experiment-runner 写「预计挂钟：超过 2 小时」与骨架那句「多机：」；执行员照定义第 1d 步把这三行做进脚本、写文件头长活声明、跑 `detached-run.sh long-run-check` 到退 0：照定义写的文件头（登记值读法）long-run-check 退 5、host-dispatch.py ⓪ 判成只占一台；能过 long-run-check 的文件头要点名 host-resources.py（现算读法），同一句写进派发提示 ⑭ 又拒。同一句「多机」，⑭ 与文件头两处判法相反，没有一句两处都过 | 量过（`out/t3_long_run-r3.txt`；第二轮冻结副本上同样，草稿里跑过） | 第二行「照定义做却违反用户定案」（要过闸就得把「每台线程按 host-resources.py 此刻空闲现算」写进文件头，用户定的是「机器忙时也固定、不看忙闲」）与「钩子拒而定义没写」（第 1d 步没写文件头要点名 host-resources.py） | 不中：第一轮定义第 1d 步写的就是现算，同一句在 ⑭ 与文件头都过（`out/t3_long_run-r1.txt` live 那三行 rc 0）；冲突是改法 G11 把定义与 ⑭ 改成登记值、文件头判法没跟着改带进来的 |
| T3 | P2 | 打中 | 「多机：…每台线程取 .claude/heavy-thread-default 的登记值除以此刻同时在跑的重活数」⑭ 放行；「只在本机一台跑，不读 --emit-hosts；线程按 nproc 取，不用 .claude/heavy-thread-default」⑭ 也放行。⑭ 的登记值读法只看两个词在不在、两个现算词不在，否定句、除法、别的现算说法都认不出 | 量过（`out/t3_dispatch_forms-r3.txt`） | 第二行后半「定义说拒而钩子不拒」（⑮ 自己的拒绝说明写着「不按整机核数除以同时在跑的重活数算，机器忙也不降」） | 除法那一句改前拒（第一轮要点名 host-resources.py）；「只在本机一台、线程按 nproc」这一形状改前同样放行（第一轮写法换成也点名 host-resources.py 的否定句，rc 0）：形态改前也中，另立一笔账 |
| T3 | P3 | 打中（弱） | 「预计挂钟：2 小时内（第一段）；第二段超过 2 小时，另派」只认开头，⑭ 当 2 小时内放行，不要三行 | 量过 | 第二行后半 | 也中（第一轮同样 rc 0）：另立一笔账 |
| T2 | W1 | 打中（弱） | 子目录 `e900/segment-1` 的持锁者还在跑，把父目录 `e900` 派给另一个子 agent，⑯ 照放、在父目录建新锁；新来的那一个照共用约束第 46 行「先 du -sh 列出…不再用的先删」在自己草稿目录里删副本，Bash 里的删没有闸；Write 被第九道拦（rc 2），删不拦 | ⑯ 与第九道量过（`out/t2_nested-r3.txt`）；删那一步推的 | 第二行后半（⑯ 头注「持锁的子 agent 还在跑就拒」只判点名那一层） | 也中（第一轮 `out/t2_nested-r1.txt` 逐行同）：另立一笔账 |
| T1 | A1 | 打中（弱） | 子 agent 照共用约束第 61 行把重活写成 `capped.sh 31 <命令>`，命令是 `python3 scripts/hosts/host-dispatch.py -- cargo test …`；派发器按内存余量挑中第二台时经 `multi-host-run.sh --peer` 起，`capped.sh` 设的线程变量一个都到不了第二台，cargo 取第二台的核数；登记文件头写「双机时每台各取这一个数」 | 环境不带过去量过（peer-host-lib.sh 自己的「第二台是本机」模式，`out/t1_peer_env-r3.txt`）；第二台上实际线程数推的（没连第二台） | 第一行的反面：规则让人以为是登记值、实际不是 | 也中（第一轮同样）：另立一笔账 |
| T4 | — | 没打中 | R4 两句（第 58、60 行）与单机路径 `--test-threads 1`：轻型崩溃域实测态数远低于 20 万；门禁与入口单机分支 `--test-threads 1` 都在 `--` 之后、照特性里有 gpu 才加；只顺手记一行 K6 新文字与第 60 行的冲突（零轮，见 T4 一节） | 读码 + 留存产物计数 | — | — |
| T2 | — | 没打中 | 写者锁其余形状（同一份提示并行派、回执路径、嵌套写、交回之后接手、看门狗「被停的同样算」）见「没打中的形状」 | 读码 + 部分量过 | — | — |

## T3　⑭ 长活三行与多机线程（R8 站住的形态）

### P1　同一句「多机」，派发闸与脚本文件头判法相反；照执行员定义做，long-run-check 到不了退 0（量过）

可复现的历史（每一步指到许可它的那一句）：

1. 主 agent 派 `experiment-runner`，提示从骨架 `python3 scripts/agents/dispatch-prompt.py experiment-runner` 起，填「预计挂钟：超过 2 小时」与分片、续跑、多机三行；多机一行照骨架原句：「台数从 layer0-shard-configuration-check.sh --emit-hosts 读、每台线程取 .claude/heavy-thread-default 的登记值」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/agents/dispatch-prompt.py:90`），也就是 `.claude/main-agent.md` 要的「每台线程取登记的每台默认重活线程数，不现算」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/main-agent.md:55`）。⑭ 放行（下面原样输出 `dispatch-14 … hosts=registered rc=0`）。提示原文在探针 `probes/t3_long_run.py` 里拼（`prompt-registered.md`）。
2. 执行员照第 1d 步把跑法写成脚本，照提示的三行做：「每台线程取登记值（`.claude/heavy-thread-default`），不现算、不写常量」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/agents/experiment-runner.md:33`）；文件头写长活声明，`# long-run-hosts:` 照实写脚本做的事，就是第 1 步那一句。
3. 同一步要求「交回前跑 `bash scripts/hosts/detached-run.sh long-run-check -- <起法>` 到退 0」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/agents/experiment-runner.md:33`）。它退 5，拒绝说明要的正是现算：「机器清单只从 layer0-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存从 host-resources.py --json 现算」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/lib_long_run.py:120`）。这一半是有意留着的：「脚本文件头的 # long-run-hosts: 仍按 lib_long_run.py 的旧判法（点名 host-resources.py），只有派发提示走登记值这一种」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/runner-dispatch-guard.sh:59`），写法说明也这么写：「脚本文件头这一行按现算读：台数从 layer0-shard-configuration-check.sh --emit-hosts 读、额度从 host-resources.py --json 现算，不写死」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/detached-run.sh:54`）。
4. 执行员要退 0 只有三条路，条条错：
   - 脚本真按 host-resources.py 现算每台线程：那个数是「available_threads=max(0, cpu["cores"] - used)」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/host-resources.py:356`），即此刻空闲，违反第 1d 步「不现算」与用户定案「机器忙时也固定 31、不看忙闲」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/heavy-thread-default:6`）；
   - 脚本照登记值、文件头照抄 detached-run.sh 的写法写「额度从 host-resources.py --json 现算」：声明与脚本做的事不符；
   - 文件头只在否定句里提一下 host-resources.py（`registered-negated`）：文件头判法按字面找词，过了（rc 0），但同一句写回派发提示 ⑭ 拒（rc 2）。
5. 交回之后主 agent 照 `.claude/main-agent.md` 起长活：「被起的脚本文件头要有长活声明，写法在 `scripts/hosts/detached-run.sh` 文件头 start 的 `--wall-clock` 一段」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/main-agent.md:43`）；`start --wall-clock over-2h` 判的是同一个函数（第 3 步那份 lib），照定义写的文件头退 5、什么都不起。改走统一派发器的，`host-dispatch.py` ⓪ 用同一个现算判法读文件头（「long_run_lib.judge(declared, long_run_lib.HEADER_SPELLING)」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/host-dispatch.py:560`），判不齐就「return "single", f"{script} 文件头的长活声明不齐」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/host-dispatch.py:562`），超过 2 小时的只占一台按它文件头 ⓪ 退 244（这一步推的，没起派发器，下面 `host-dispatch-header` 行是同一个 judge 的判定）。
6. 设计员那一侧同一个口径：「每台线程取登记值（`.claude/heavy-thread-default`），不现算、不写常量」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/agents/experiment-designer.md:34`），登记写的跑法一样过不了文件头判法。

第三轮冻结副本上的原样输出（`out/t3_long_run-r3.txt`，每行截到 200 字节）：

```
name=header tree=r3 hosts=registered rc=1 first=✗ 长活声明（<WORK>/long-registered.sh）：# long-run-hosts: 一行没点名 host-resources.py：机器清单只从 layer0-shard-configuration-ch
name=host-dispatch-header tree=r3 hosts=registered problems=1 verdict=single
name=long-run-check tree=r3 hosts=registered rc=5 first=✗ 长活声明（<WORK>/long-registered.sh）：# long-run-hosts: 一行没点名 host-resources.py：机器清单只从 layer0-shard-configur
name=dispatch-14 tree=r3 hosts=registered rc=0 long_run_refusals=0 first=-
name=header tree=r3 hosts=live rc=0 first=✓ 长活声明齐了（<WORK>/long-live.sh，判了 3 个键）：分片：按登记的取样点切片，片数取多机配置的台数；续跑：同一条
name=host-dispatch-header tree=r3 hosts=live problems=0 verdict=multi
name=long-run-check tree=r3 hosts=live rc=0 first=-
name=dispatch-14 tree=r3 hosts=live rc=2 long_run_refusals=1 first=✗ 派 experiment-runner 的提示写了预计挂钟超过 2 小时，长活声明不齐：
name=header tree=r3 hosts=registered-negated rc=0 first=✓ 长活声明齐了（<WORK>/long-registered-negated.sh，判了 3 个键）：分片：按登记的取样点切片，片数取多机配置�
name=host-dispatch-header tree=r3 hosts=registered-negated problems=0 verdict=multi
name=long-run-check tree=r3 hosts=registered-negated rc=0 first=-
name=dispatch-14 tree=r3 hosts=registered-negated rc=2 long_run_refusals=1 first=✗ 派 experiment-runner 的提示写了预计挂钟超过 2 小时，长活声明不齐：
```

三句的原文：registered 是骨架那一句加「，不现算、不写常量」；live 是第一轮定义第 1d 步那一句「每台的线程与内存读 `scripts/hosts/host-resources.py --json` 现算，不写常量」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agents/experiment-runner.md:32`）；registered-negated 是 registered 把「不现算、不写常量」换成「不调 host-resources.py 现算」。

改前：第一轮冻结副本上（`out/t3_long_run-r1.txt`）live 那一句四处都过（`header … rc=0`、`host-dispatch-header … verdict=multi`、`long-run-check … rc=0`、`dispatch-14 … rc=0`），第一轮定义第 1d 步写的就是 live，照定义做走得通；第二轮冻结副本上与第三轮逐格相同（草稿里跑过：`long-run-check … hosts=registered rc=5`、`dispatch-14 … hosts=registered rc=0`、`dispatch-14 … hosts=live rc=2`）。所以是第二轮站住的 R8 形态（G11：多机一行与两份定义改读登记值）带进来、第二轮没被看见的冲突。第二轮辩方 S24 写过「脚本文件头仍走现算读法，并存，文字已分开」，没看到执行员第 1d 步把两种读法串在同一条工作流上。

打中之后四句：

- 分不分辨臂：分辨。两条读法（登记值、现算）各自单看都自洽，打中的是同一条工作流先后经两处、两处要的相反；只把文件头判法也改成登记值读法的那条改法清掉它，改回全用现算的清掉冲突却违反用户定案。
- 被判的系统看不看得到：看得到。两处判的都是写出来的那一行字，执行员手里有拒绝说明。
- 满足的是哪一个分句：正文「四、跑前条款」第二行「给出那样一条可达历史」，形状是正文第三节攻方那一行的「钩子拒而定义没写」（long-run-check 拒定义要的文件头，定义没写文件头要点名 host-resources.py），走现算那条出路则是「照定义做却违反用户定案」。
- 跑前条款给的改法：第二轮 K1–K11 没有一条碰 `lib_long_run.py` 的文件头判法、`detached-run.sh`、`host-dispatch.py` ⓪ 或第 1d 步，在这几格上全部还中（按改法表逐条对的，推的）。

### P2　⑭ 的登记值读法只按字面找两个词，除法与「只用一台、线程按 nproc」照放（量过）

⑭ 判多机一行：点名两个词，「unnamed = [read for read in reads if read not in hosts]」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/lib_long_run.py:111`），现算只认两个词，「LIVE_COMPUTE = re.compile(r"host[-_]resources|available_threads")」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/lib_long_run.py:63`）。⑮ 自己的拒绝说明写着用户定的口径「不按整机核数除以同时在跑的重活数算，机器忙也不降」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/runner-dispatch-guard.sh:1070`）。

派 experiment-runner，提示里多机一行写下面几种，其余各行齐（提示在探针 `probes/t3_dispatch_forms.py` 里拼），`--check-prompt` 原样输出（`out/t3_dispatch_forms-r3.txt`）：

```
name=dispatch-14-form tree=r3 case=divide-by-running rc=0 refusals=0 first=-
name=dispatch-14-form tree=r3 case=one-host-nproc rc=2 refusals=1 first=✗ 派 experiment-runner 的提示写了预计挂钟超过 2 小时，长活声明不齐：
name=dispatch-14-form tree=r3 case=one-host-nproc-registered-words rc=0 refusals=0 first=-
name=dispatch-14-form tree=r3 case=wall-clock-first-segment rc=0 refusals=0 first=-
name=dispatch-14-form tree=r3 case=control-registered rc=0 refusals=0 first=-
```

- `divide-by-running`：「台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值除以此刻同时在跑的重活数」——正是用户否掉的算法，放行。
- `one-host-nproc-registered-words`：「只在本机一台跑，不读 --emit-hosts；线程按 nproc 取，不用 .claude/heavy-thread-default」——台数、线程两样都不是登记值，放行。
- `one-host-nproc` 多带一句「也不读 host-resources.py」就被拒：拒的理由是出现了禁词，不是语义。

改前（`out/t3_dispatch_forms-r1.txt`）：

```
name=dispatch-14-form tree=r1 case=divide-by-running rc=2 refusals=1 first=✗ 派 experiment-runner 的提示写了预计挂钟超过 2 小时，长活声明不齐：
name=dispatch-14-form tree=r1 case=one-host-nproc rc=0 refusals=0 first=-
name=dispatch-14-form tree=r1 case=one-host-nproc-registered-words rc=2 refusals=1 first=✗ 派 experiment-runner 的提示写了预计挂钟超过 2 小时，长活声明不齐：
name=dispatch-14-form tree=r1 case=wall-clock-first-segment rc=0 refusals=0 first=-
name=dispatch-14-form tree=r1 case=control-registered rc=2 refusals=1 first=✗ 派 experiment-runner 的提示写了预计挂钟超过 2 小时，长活声明不齐：
```

「只用一台、线程按 nproc」换成点名第一轮那两个词的否定句（`one-host-nproc`）改前同样放行：按字面找词这个形态改前也中，另立一笔账；除法那一句改前被拒，是登记值读法下才放的。

四句：分辨臂（只打登记值读法的判法实现，不打「每台取登记值」这个定案）；系统看得到（判的就是那一行字）；分句是第二行后半「定义说拒而钩子不拒」（⑭ 头注第 57 行起写「不许写现算」，这里写了现算没拒）；改法：K1–K11 没有一条碰 ⑭ 的多机判法，还中（推的）。

### P3　预计挂钟一行只认开头（量过，弱）

「预计挂钟：2 小时内（第一段）；第二段超过 2 小时，另派」：判法只认开头「WALL_CLOCK_WITHIN_VALUE = re.compile(r"^2\s*小时内")」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/lib_long_run.py:74`），当 2 小时内放行，不要三行（上面 `wall-clock-first-segment rc=0`）。改前同样 rc 0，另立一笔账。可达性弱：设计员按段写挂钟（`.claude/agents/experiment-designer.md` 第 3c 步），主 agent 派一段时照抄两段的估计才会写成这样。

## T2　写者锁（R6 站住的形态）与看门狗

### W1　子目录的持锁者还在跑，把父目录派给另一个子 agent 照放；Bash 里的删没有闸（⑯ 与第九道量过，删那一步推的，弱）

可复现的历史（探针 `probes/t2_nested.py`：假工作根、假主会话记录都在 `--work` 里，派发闸与写闸都经 stdin 入口喂 JSON）：

1. 主 agent 派 clerk 甲，「草稿目录：<工作根>/e900/segment-1/」：⑯ 建锁（`step=child-first rc=0 lock_child=True`）；记录里补甲的派发回执（还在跑）。甲在里面建 `repo-copy/`。
2. 对照：同一子目录再派一件，⑯ 拒（`step=same-child-again rc=2`），⑯ 头注「目录里已有锁、持锁的子 agent 还在跑就拒」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/runner-dispatch-guard.sh:69`）只看点名那一层。
3. 派 clerk 乙，「草稿目录：<工作根>/e900/」（例：分段跑完派一个汇总的）：⑯ 放行、在父目录建新锁，子目录的锁留着（`step=parent-while-child-running rc=0 lock_parent=True lock_child_kept=True`）。
4. 乙用 Write 写子目录被第九道拒（`parent-holder-writes-child rc=2`），写自己那一层放行；甲照常写自己的子目录（`child-holder-writes-child rc=0`）。
5. 乙照共用约束建副本之前「先 `du -sh` 列出自己草稿目录下已有的副本与编译目录，不再用的先删」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/agent-common.md:46`）：在它看来 `segment-1/repo-copy` 是自己草稿目录下一份不是它建的副本，用 Bash 删掉，没有闸——「Bash 里的写这里看不见」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/hooks/lib_writer_lock.py:30`）。共用约束只写了另一个方向：「子目录另派给别的子 agent 之后，父目录的持锁者不写那个子目录」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/agent-common.md:45`），子目录先派、父目录后派时乙不知道下面有别人的锁。

原样输出（`out/t2_nested-r3.txt`）：

```
name=dispatch tree=r3 step=child-first rc=0 lock_child=True
name=dispatch tree=r3 step=same-child-again rc=2
name=dispatch tree=r3 step=parent-while-child-running rc=0 lock_parent=True lock_child_kept=True
name=write tree=r3 case=parent-holder-writes-own rc=0 lock_refusal=-
name=write tree=r3 case=parent-holder-writes-child rc=2 lock_refusal=✗ <WORK>/work-root/e900/segment-1/repo-copy/notes.md 在
name=write tree=r3 case=child-holder-writes-child rc=0 lock_refusal=-
```

改前：`out/t2_nested-r1.txt` 除 `tree=r1` 外逐行相同，也中，另立一笔账。四句：不分辨 R6 的臂（嵌套四格的写判对，打的是派发那一刻不往下看）；系统看得到（⑯ 派发时可以往下找 `.writer-lock`）；分句是第二行后半（弱：⑯ 文字本来只说点名那一层，乙删副本那一步要乙判「不再用的」判错）；改法：K1–K11 没有碰 ⑯ 的，还中（推的）。

## T1　线程数「默认 31、双机各 31」

### A1　`capped.sh 31` 套在统一派发器外面，派到第二台时线程变量一个都带不过去（环境量过，第二台的数推的，弱）

历史：

1. 子 agent 照共用约束起重活：「的每条命令一律用 `bash scripts/hosts/capped.sh N <命令>` 起」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/agent-common.md:61`）。它要按两台余量挑机器跑一条 cargo，用内存包装文件头点名的派发器：「选哪台机器、按历史预测要多少归统一派发器 scripts/hosts/host-dispatch.py」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/run-with-memory-cap.sh:11`），照字面写成 `bash scripts/hosts/capped.sh 31 python3 scripts/hosts/host-dispatch.py -- cargo test …`。
2. 派发器挑中第二台，经 `multi-host-run.sh --peer` 起；那边的环境「本机别的环境变量不带（真的第二台经 ssh 也拿不到），要设写成 -- env 变量=值 <命令>」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/multi-host-run.sh:26`），库里「环境照 ssh 那样只给最少几样」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/peer-host-lib.sh:15`）。
3. 第二台上 `CARGO_BUILD_JOBS`、`RUST_TEST_THREADS`、`SINGLEFS_THREAD_CAP` 都没设，cargo 取第二台的核数；登记文件头却写「双机时每台各取这一个数」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/heavy-thread-default:3`）。挑哪台按内存余量现判，同一条命令这一次 31、下一次第二台核数。

原样输出（`out/t1_peer_env-r3.txt`；用 peer-host-lib.sh 自己的「第二台是本机」模式演示，与 ssh 同一份最小环境，不连第二台、不读多机配置）：

```
name=peer-env tree=r3 side=local RUST_TEST_THREADS=31 CARGO_BUILD_JOBS=31 SINGLEFS_THREAD_CAP=31
name=peer-env tree=r3 side=peer RUST_TEST_THREADS=unset CARGO_BUILD_JOBS=unset SINGLEFS_THREAD_CAP=unset
name=peer-env-doc tree=r3 multi-host-run.sh line=26
```

改前：`out/t1_peer_env-r1.txt` 除 `tree=r1` 外相同，也中，另立一笔账。可达性弱：规则与定义里没有一处叫子 agent 用 `host-dispatch.py`（冻结副本 `.claude/rules`、`.claude/agents`、共用约束、`main-agent.md` 里 `grep -rn host-dispatch` 0 处），只有脚本文件头点名它；正确写法是 `host-dispatch.py -- bash scripts/hosts/capped.sh 31 cargo …`（`capped.sh` 随树拷过去），没有一处写这个次序。是不是与第一、二轮已列的「双机驱动」同一件，由主 agent 判：已列的是 `crash-amplification-two-hosts.sh` 自己现算线程，这里是通用派发器丢环境。

四句：不分辨 31 这个定案（打的是传数的通道）；系统看不看得到：子 agent 起命令时看不到这次会派到哪台；分句：正文 T1 格的「实际线程数不是登记值而规则让人以为是」；改法：K1–K11 没有碰派发器的，还中（推的）。

## T4　verification 两句（R4）与 `--test-threads 1` 的单机路径：没打中

试过的形状与结局：

- 第 58 行「日常验证（harness 档与轻型三条）的崩溃状态数压在 20 万量级」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/rules/verification.md:58`）与第 46、49 行「轻型跑全部、不剪枝」会不会互斥：数了留存的轻型跑法产物（仓外 `~/.local/share/singlefs/crash-amplification/runs/crash-point-replay-*/pipeline.log`，2026-10-10 至 2026-10-11 的几趟，读到的是当时的数）的 `CRASH_AMPLIFICATION_TOTAL` 行，现判态数 0 到 639、复用 0 到 308，离 20 万差三个量级；harness 档按第 27 行不该有崩溃枚举。不互斥。
- 单机路径 `--test-threads 1` 的位置：入口第 487 行与门禁第 768 行都放在 `--` 之后、只在特性里有 gpu 时加（入口第 467 行、门禁第 734 行），与第 59 行写的射程相同；没造出单机路径漏带的历史。
- 第 60 行「崩溃放量与对拍驱动照多机配置用两台与全部卡」里的「对拍驱动」在冻结副本 `.claude`、`scripts` 里只出现在这一行（`grep -rn 对拍驱动` 1 处），指不到具体脚本；只是措辞，不算打中。
- 顺手看到（K6 零轮，只记一行）：第 59 行新写的「GPU 对拍测试的 CPU 一侧读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设取核数（`available_parallelism`），不读登记值」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/rules/verification.md:59`）与紧接着第 60 行站住的「用例内部的 CPU 线程开满默认线程（登记的每台默认重活线程数）」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/rules/verification.md:60`）对同一类用例说了两个数。

## 改法与各格（全部只在我的模型上想过、没实现、被攻过零轮；每格都是「推的」）

| 改法 | 改什么 | 修哪一格 | 量过 / 推的 |
|---|---|---|---|
| Q1 | `lib_long_run.py` 的文件头判法也改成登记值读法（点名 `--emit-hosts` 与 `heavy-thread-default`、不许现算词），`detached-run.sh` 文件头与 `host-dispatch.py` ⓪ 的说明随之改；仓里现有文件头点名 host-resources.py 的脚本逐个改写或改写成 `# long-run-unshardable:` | P1 全部三处（long-run-check、start、派发器 ⓪） | 推的 |
| Q2 | 只改第 1d 步与设计员第 3c 步的文字，写明文件头另按现算写 | P1 的「定义没写」一半；「看忙闲」那一半照旧，违反用户定案，不建议 | 推的 |
| Q3 | ⑭ 多机一行改成只认一句规范写法（骨架那一句），或现算词表加「除以、在跑、nproc、核数、忙闲、负载、available_parallelism」 | P2；规范写法那一种也罩住按字面找词的否定句 | 推的 |
| Q4 | 预计挂钟一行整行只许一个挂钟词，出现「超过 2 小时」就按超过判 | P3 | 推的 |
| Q5 | ⑯ 派发时往下找点名目录里还在跑的持锁者，找到就拒；或共用约束第 46 行补「删之前看子目录有没有别人的 `.writer-lock`」 | W1 | 推的 |
| Q6 | `host-dispatch.py`（或 `multi-host-run.sh --peer`）把 `capped.sh` 那几个线程变量带过去，或在 `capped.sh` 与派发器文件头写明次序 `host-dispatch.py -- capped.sh N <命令>` | A1 | 推的 |

## 现场观察（冻结副本之外，没核原因：主工作区的钩子、脚本与登记在禁读清单里）

- 写者锁在现役环境里一把都没建：`cd ~/.local/share/singlefs/work && echo "目录数 $(ls -d */ | wc -l)，带 .writer-lock 的 $(for d in */; do [ -e "$d.writer-lock" ] && echo "$d"; done | wc -l)"` 原样输出「目录数 73，带 .writer-lock 的 0」；这一轮两条腿的草稿目录在派发时都还不存在（我开工时 `ls` 报 No such file，第一次写之前是我自己 `mkdir -p` 建的），`ls -A m3-defs-b3-r3-sonnet | grep -c writer` 输出 0。冻结副本的 ⑯ 派发放行时应当建目录与锁（`runner-dispatch-guard.sh` 第 69 行起）。可能是主工作区还没合入批 4 以后的 ⑯，或现役钩子没走到建锁那一支；主 agent 现查。若是后者，T2 整格在现役环境里不成立。
- 主工作区此刻有一条不给 `--threads` 的合入后验证按 32 个线程跑：`ps -o pid,args -u "$(id -u)" | grep -F 'capped.sh 32'` 第一行原样（截到 230 字节）「3871506 bash /home/fy5090/code/singlefs/scripts/gate/../hosts/run-with-memory-cap.sh 24G bash /home/fy5090/code/singlefs/scripts/gate/../hosts/capped.sh 32 cargo test --offline --no-fail-fast -p singlefs-format -p singlefs-core -p」；它的父进程是 `bash scripts/gate/merge-and-verify.sh --memory 24G <补丁目录>`（没给 `--threads`），属另一个会话，进程环境里没有 `SINGLEFS_THREAD_CAP`。冻结副本里 `merge-and-verify.sh` 不给 `--threads` 取登记值 31。主工作区那一份取到 32 的原因（登记不同、脚本未合入、或别的）我没查。

## 没打中的形状（试过的写法与取样范围）

- T2 写者锁：同一份提示同一条消息并行派两次到同一目录（第二次走「重试」那一支接手、两个都起；第九道只让先写的那个认领，另一个被拒，没违反「同一时刻只许一个写」，读码推的，没喂）；子 agent 钩子输入里的 `transcript_path` 是不是主会话记录（看了现役检出记录 `~/.local/share/singlefs/work/agent-hook-detections.jsonl` 里一条真的子 agent 记录，是主会话的 `.jsonl`，回执查得到）；子 agent 自己的记录路径与 `is_running` 拼的 `<会话>/subagents/agent-<号>.jsonl` 对不对（本机现有记录就是这个布局）；交回之后新派一个接着做（交回算结束，接手对）；被 TaskStop 停掉的、任务通知 failed / killed / stopped 的算结束（⑯ 与看门狗两边都认，读码）；看门狗「被停的同样算」两处（交回之后后台还在跑、草稿根超阈值报告）与 `agent-watch.py` 文件头第 27、52、116 行一致。另一个方向的嵌套（父目录先派、子目录后派）是第一轮 S13 那四格，不重交。
- T1：`run-with-memory-cap.sh` 用 `systemd-run --user --scope`，环境原样继承，`capped.sh` 套在它里外两种次序都到得了命令；`merge-and-verify.sh`、`compile-then-swap.py` 冻结副本里都经 `capped.sh` 取登记值；广谱变异驱动内部自己经 `capped.sh`；E171 线道数切分（登记值 ÷ 线道）合计仍是登记值。`crates/` 里读 `available_parallelism` 的几处都在排除表「代码批」行，按正文不交。
- T3：骨架原句在 ⑭ 过（control-registered rc 0）；中文数字、卡数这类额度判法是第二轮 S26，不重交；`--check-prompt` 只判了 experiment-runner 一种类型（⑭ 对十种类型同一个判法，按代码推同样）。
- T4：见 T4 一节四条。
- 取样范围：P1 三种文件头写法 × 三处判法 × 三棵冻结副本（第二轮只在草稿里跑）；P2/P3 五种提示 × 两棵；W1 一段六步历史 × 两棵；A1 三个变量 × 两棵。

## 这条腿自己的限度

- 每格只抽了一次样；「没打中」的 T4 与 T2 其余形状按规则要另抽一次才算。
- P1 第 5 步派发器退 244 那一段没起 `host-dispatch.py`（要多机配置），用的是它第 560 行调的同一个 judge；`detached-run.sh start` 没起（子 agent 被拒），用的是同一个判法函数 `long-run-check`。
- W1 删副本那一步是按共用约束推的，没让一个真的子 agent 去删；A1 第二台上的实际线程数没量（没连第二台）。
- 喂派发闸都没带真的主会话记录，⑦ 并发、限额窗口那几条没参与；⑯ 用的是探针自己写的假记录。

## 没做什么

- 不判 T5、T6（正推腿的格）；没替主 agent 采纳；样本树与副本上的读数不算入库装置上的数。
- 没跑任何 `--selftest`、门禁阶段、cargo、重型测试；没编译。改法一个都没实现。
- 没读禁读清单里的文件；主工作区那两条现场观察只看了进程表与目录列表，没读主工作区的钩子、脚本与登记。

## 草稿目录

`~/.local/share/singlefs/work/m3-defs-b3-r3-opus/`：三棵树的拷贝 `trees/`（冻结副本 `.claude`、`scripts`、`CLAUDE.md`，不含 `crates/`、`.git`、`multi-host.env`）、探针的试跑目录 `try-*`、两趟复跑 `rerun-a`、`rerun-b`，交回前全部删掉。

## 复跑原样输出（`rerun-b`，末尾八行）

```
name=compare file=t1_peer_env-r1.txt same
name=compare file=t1_peer_env-r3.txt same
name=compare file=t2_nested-r1.txt same
name=compare file=t2_nested-r3.txt same
name=compare file=t3_dispatch_forms-r1.txt same
name=compare file=t3_dispatch_forms-r3.txt same
name=compare file=t3_long_run-r1.txt same
name=compare file=t3_long_run-r3.txt same
```
