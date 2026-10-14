#!/usr/bin/env bash
# 攻方样本（X7）：豁免表登记一份按现算写文件头的脚本 live.sh（带 sha256），提交为基准；之后改 live.sh 的正文（槽数从现算改成写死 8），再把表里的 sha256 换成新文件的（重钉），表的行数与路径都不变。
# exempt-check --base-table 基准那一版：只比路径，判绿。另做两样：chmod -x（sha256 不变）、换成 CRLF 行尾（sha256 变、不重钉）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude scripts
cat > scripts/live.sh <<'S'
#!/usr/bin/env bash
# long-run-shards: 按行切，一行一片
# long-run-resume: 同一条命令再起只补还没有结果文件的行
# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存从 host-resources.py --json 现算
slots=$(python3 scripts/hosts/host-resources.py --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["available_threads"])')
echo "$slots"
S
printf '%s\n' '# 样本豁免表' "scripts/live.sh  $(sha256sum scripts/live.sh | cut -d' ' -f1)  # 样本：冻结入口，按现算读" > .claude/long-run-live-hosts-exempt
git add -A && git commit -qm base
git show HEAD:.claude/long-run-live-hosts-exempt > base-table.txt
sed -i 's/^slots=.*/slots=8/' scripts/live.sh
printf '%s\n' '# 样本豁免表' "scripts/live.sh  $(sha256sum scripts/live.sh | cut -d' ' -f1)  # 样本：冻结入口，按现算读" > .claude/long-run-live-hosts-exempt
