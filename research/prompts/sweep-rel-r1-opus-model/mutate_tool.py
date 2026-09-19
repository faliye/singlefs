#!/usr/bin/env python3
"""sweep-rel-r1 攻方腿（Opus）：A3 —— 改窄了工具、门禁 47 号跑的两条（--selftest、--benchmark）照绿吗。
每个变异把 stale-candidates.py 拷进草稿目录、做一处恰好命中一次的替换，在仓根跑两条命令；不碰仓里的脚本。
用法：cd 仓根 && python3 research/prompts/sweep-rel-r1-opus-model/mutate_tool.py 草稿目录"""
import os, subprocess, sys
draft = sys.argv[1]
SRC = open('research/scripts/stale-candidates.py', encoding='utf-8').read()
MUTANTS = [
    ('M1 候选表落盘时丢掉 records/ 的行（只改 write_candidates）',
     "    for group, old_fact, new_fact, location, line in candidates:\n        output.write(",
     "    for group, old_fact, new_fact, location, line in candidates:\n        if location.startswith('records/'):\n            continue\n        output.write("),
    ('M2 --facts 出候选表前按载体去重，同一行只留第一件事实',
     "            candidates = build_candidates(repository_root, arguments.target, read_fact_table(arguments.facts))\n",
     "            candidates = build_candidates(repository_root, arguments.target, read_fact_table(arguments.facts))\n            seen = set(); candidates = [c for c in candidates if not (c[3] in seen or seen.add(c[3]))]\n"),
    ('M3 结束在工作区时不收未跟踪的新文件（all_paths 的 --others）',
     "                 + run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\\n'))",
     "                 )"),
    ('M4 --groups 区间丢掉最后一组（range 少 1）',
     "range(low, high + 1)", "range(low, high)"),
    ('M5 核对器不再核判定是不是五种之一',
     "if not verdict.startswith(LINE_VERDICTS) or len(reason)", "if len(reason)"),
    ('M6 理由下限 8 → 3',
     "MINIMUM_REASON_LENGTH = 8", "MINIMUM_REASON_LENGTH = 3"),
    ('M7 冒充新立只看基准命中、不再看新事实里的六个词',
     "if base_hits or NOT_NEWLY_ADDED_WORDS.search(fact['新事实']):", "if base_hits:"),
    ('M8 check-facts 去掉「基准零命中」闸（对照组，预期被 selftest 抓）',
     "            empty.append(fact['编号'])", "            pass"),
    ('M9 候选只收长度 ≤ 400 字的行（build_candidates）',
     "                if search_term_matches(compiled_parts, line):\n                    candidates.append(",
     "                if len(line) <= 400 and search_term_matches(compiled_parts, line):\n                    candidates.append("),
    ('M10 现状句停在任何「## 历史」「## 经过」开头的标题（改 current_state_lines 的停点；这段历史上候选数不变，算空变异）',
     "        if line.strip() == HISTORY_SECTION_TITLE:", "        if line.startswith('## 历史') or line.startswith('## 经过'):"),
]
print('| 变异 | --selftest 退出码 | --benchmark 退出码与末行 |')
print('|---|---|---|')
for label, old, new in MUTANTS:
    assert SRC.count(old) == 1, (label, SRC.count(old))
    path = os.path.join(draft, 'mutant-' + label.split()[0] + '.py')
    with open(path, 'w', encoding='utf-8') as fh:
        fh.write(SRC.replace(old, new))
    s = subprocess.run(['python3', path, '--selftest'], capture_output=True, text=True)
    b = subprocess.run(['python3', path, '--benchmark'], capture_output=True, text=True)
    blast = [l for l in b.stdout.split('\n') if l.strip()][0].strip()
    print(f'| {label} | {s.returncode} | {b.returncode}：{blast} |')
