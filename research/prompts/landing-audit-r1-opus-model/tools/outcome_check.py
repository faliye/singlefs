#!/usr/bin/env python3
"""攻方腿 landing-audit-r1 的 K4 复算：照调度记录那条命令的口径（agent-cost.py outcome --since 2026-10-01 --jobs 12）算，再多给三样：
  1. 按截止时刻切（只留首条记录不晚于截止时刻的子 agent），看调度记录那组数是不是某一个时刻的快照；
  2. 「交回」改按送达判（SubagentHandback 调过、且它的 tool_result 不是错误、不是 success:false；认法照 .claude/hooks/continuation-guard.sh
     的 handback_delivered，这里另写一份），数出 outcome 当成交回、其实一次都没送达的子 agent；
  3. 返工字样换回调度记录第一批写的五个词，并排数。
agent-cost.py 用的是 --agent-cost 给的那一份（草稿目录里的拷贝），只调它的 collect_outcome、folded 这几个函数，不改它。
用法：outcome_check.py --agent-cost <拷贝> --root <会话目录> --repo <仓根> --since 2026-10-01 --jobs 12 --cutoff <unix 秒>…
"""
import argparse, collections, datetime, glob, importlib.util, json, os, re, sys

UTC = datetime.timezone.utc  # clock-times:allow 会话记录的时间戳是 UTC
FIVE_WORDS = re.compile("修|重跑|续做|改名|改号")

def load(path):
    spec = importlib.util.spec_from_file_location("agent_cost_copy", path)
    module = importlib.util.module_from_spec(spec)
    sys.modules["agent_cost_copy"] = module   # 工作进程按名字找回 scan_job
    spec.loader.exec_module(module)
    return module

def result_is_error(block):
    if block.get("is_error") is True:
        return True
    content = block.get("content")
    if isinstance(content, list):
        content = "".join(part.get("text") or "" for part in content if isinstance(part, dict))
    try:
        parsed = json.loads(content) if isinstance(content, str) else None
    except ValueError:
        parsed = None
    return isinstance(parsed, dict) and parsed.get("success") is False

def delivered(path):
    ids = set()
    for line in open(path, encoding="utf-8", errors="replace"):
        if "SubagentHandback" not in line and not any(i in line for i in ids):
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        message = record.get("message")
        if not isinstance(message, dict) or not isinstance(message.get("content"), list):
            continue
        for block in message["content"]:
            if not isinstance(block, dict):
                continue
            if message.get("role") == "assistant" and block.get("type") == "tool_use" and block.get("name") == "SubagentHandback" and block.get("id"):
                ids.add(block["id"])
            elif message.get("role") == "user" and block.get("type") == "tool_result" and block.get("tool_use_id") in ids and not result_is_error(block):
                return True
    return False

def table(module, agents):
    total = sum(a["cost"] for a in agents) or 1.0
    rows, used = [], set()
    for label, buckets in module.RECORD_TABLE_ROWS:
        chosen = [a for a in agents if a["bucket"] in buckets]
        used.update(buckets)
        rows.append((label, len(chosen), 100 * sum(a["cost"] for a in chosen) / total))
    rest = [a for a in agents if a["bucket"] not in used]
    rows.append(("其余", len(rest), 100 * sum(a["cost"] for a in rest) / total))
    return rows, total

def report(module, agents, label, transcripts):
    rows, total = table(module, agents)
    print(f"== {label}：{len(agents)} 个子 agent，折算合计 {total / 1e6:.2f} 百万")
    for name, count, share in rows:
        print(f"   {name}\t{count}\t{share:.1f}%")
    buckets = collections.Counter(a["bucket"] for a in agents)
    print("   桶：" + "、".join(f"{b} {buckets[b]}" for b, _ in module.BUCKET_GLOSS if buckets[b]))
    rework = [a for a in agents if a["rework"]]
    five = [a for a in agents if FIVE_WORDS.search(a["description"])]
    print(f"   返工字样（agent-cost 现词表）{len(rework)} 个，占个数 {100 * len(rework) / max(len(agents), 1):.1f}%，占花费 {100 * sum(a['cost'] for a in rework) / total:.1f}%")
    print(f"   返工字样（调度记录第一批写的五个词）{len(five)} 个，占个数 {100 * len(five) / max(len(agents), 1):.1f}%，占花费 {100 * sum(a['cost'] for a in five) / total:.1f}%")
    cuts = collections.Counter(a["cut"] for a in agents if not a["handed_back"])
    print(f"   没交回 {sum(cuts.values())} 个：" + "、".join(f"{k} {v}" for k, v in sorted(cuts.items())))
    if transcripts is not None:
        undelivered = [a for a in agents if a["handed_back"] and not delivered(transcripts[a["agent"]])]
        by_bucket = collections.Counter(a["bucket"] for a in undelivered)
        by_type = collections.Counter(a["type"] for a in undelivered)
        print(f"   outcome 当成交回、交回一次都没送达的 {len(undelivered)} 个，按桶：" + "、".join(f"{k} {v}" for k, v in by_bucket.most_common())
              + "；按类型：" + "、".join(f"{k} {v}" for k, v in by_type.most_common()))
        alone = collections.Counter()
        repair_next = collections.Counter()
        for a in rework:
            found = {m.group(0).lower() for m in module.REWORK_WORDS.finditer(a["description"])}
            if len(found) == 1:
                alone[next(iter(found))] += 1
                if found == {"修"}:
                    repair_next.update(m.group(0) for m in re.finditer(r"修.", a["description"]))
        print("   返工字样只靠一个词命中的：" + "、".join(f"{k} {v}" for k, v in alone.most_common()))
        print("   只靠「修」命中的里「修」后一字：" + "、".join(f"{k.strip() or '修+空白'} {v}" for k, v in repair_next.most_common(8)))
        for a in undelivered[:12]:
            print(f"     {a['agent']}\t{a['type']}\t{a['start'].isoformat()}\t{a['bucket']}\t{a['description'][:40]}")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent-cost", required=True)
    parser.add_argument("--root", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--since", default="2026-10-01")
    parser.add_argument("--jobs", type=int, default=12)
    parser.add_argument("--cutoff", nargs="*", default=[])
    arguments = parser.parse_args()
    module = load(arguments.agent_cost)
    since = datetime.datetime.fromisoformat(arguments.since).replace(tzinfo=UTC)
    now = datetime.datetime.now(UTC)
    agents = module.collect_outcome(arguments.root, since, arguments.repo, now, arguments.jobs)
    transcripts = {os.path.basename(p)[6:-6]: p for p in glob.glob(os.path.join(arguments.root, "*", "subagents", "agent-*.jsonl"))}
    print(f"此刻（unix 秒）{int(now.timestamp())}")
    report(module, agents, "此刻全部", transcripts)
    for cutoff_text in arguments.cutoff:
        cutoff = datetime.datetime.fromtimestamp(int(cutoff_text), UTC)
        report(module, [a for a in agents if a["start"] <= cutoff], f"首条记录不晚于 unix 秒 {cutoff_text}", None)

main()
