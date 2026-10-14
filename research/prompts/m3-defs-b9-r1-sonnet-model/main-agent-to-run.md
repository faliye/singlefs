# 要主 agent 跑的命令（门禁格子 agent 起不了；以下我没跑）

冻结副本：`~/.local/share/singlefs/work/m3-defs-b9-r1/repo/`（下称 F）。工作目录、线程数、内存上限照下面各条。

1. 门禁 code-tooling 的样本档（长活声明豁免表只缩不涨的三个样本：`long-run-header-exempt-repin-red`、`long-run-header-exempt-base-green`、`long-run-header-exempt-sha256-red`）。
   - 命令：`cd F && bash scripts/hosts/capped.sh 31 nice -n 19 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item code-tooling.sh`
   - 看哪几行：这三个样本各判对；`.claude/hooks/selftest-registry.tsv:150` 把这一道登记成 samples-light。
   - 对应我的喂法：`tools/z7_exempt_cases.py` 的「改了脚本再重钉」「基准与现在逐字相同」「表里 sha256 与脚本不符而基准相同」三例（库这一层）。

2. 门禁 doc-process-records 的样本档（叫停登记行出处带〔〕原话的样本：`abandoned-rounds-source-green`、`abandoned-rounds-source-red`）。
   - 命令：`cd F && bash scripts/hosts/capped.sh 31 nice -n 19 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item doc-process-records.sh`
   - 看哪几行：两个样本各判对（red 样本逐行点名十行，green 样本报「挪到第几行」）。
   - 对应我的喂法：`tools/z4_abandoned_cases.py`（库这一层，门禁格与派发闸共用同一份库）。

3. 门禁格取基准那一段（`cell_long_run_header_exempt`，`.claude/gate.d/code-tooling.sh:2439-2455`）在「重钉进了基准之后再跑」时的行为（Z7-01 第三处，只读代码得出，没跑）：
   - 造法：`T=$(mktemp -d); cp -a F/.claude/gate.d/fixtures/code-tooling.sh/long-run-header-exempt-repin-red/. "$T"/; cd "$T" && bash setup.sh`（setup.sh 建 git 仓、提交改正文之前的版本、再把重钉的版本放回工作区）。
   - 先跑：`cd "$T" && bash F/.claude/gate.d/code-tooling.sh "$T" --check long-run-header-exempt`，预期退 1、点名「豁免表重钉了」（即样本 repin-red 的 expect）。
   - 再提交重钉：`cd "$T" && git add -A && git -c user.name=t -c user.email=t@t commit -qm repin`，再跑同一条，推演退 0、成功句写「与基准那一版比没涨」（基准取 HEAD，已含重钉）。跑出来不是这样，Z7-01 第三处改判。
   - 内存：python 与 git 而已，`bash scripts/hosts/run-with-memory-cap.sh 2G` 包上即可。
