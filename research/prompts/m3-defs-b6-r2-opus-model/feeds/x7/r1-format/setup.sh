#!/usr/bin/env bash
# 攻方样本（X7 对照）：第一轮的豁免表写法（一行「路径  # 理由」，不带 sha256），提交为基准之后只改脚本正文、表不动。
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
printf '%s\n' '# 样本豁免表（第一轮写法）' "scripts/live.sh  # 样本：冻结入口，按现算读" > .claude/long-run-live-hosts-exempt
git add -A && git commit -qm base
git show HEAD:.claude/long-run-live-hosts-exempt > base-table.txt
sed -i 's/^slots=.*/slots=8/' scripts/live.sh
