#!/usr/bin/env python3
"""扫子 agent 会话记录：Write 工具调用的目标文件名里带 report / summary / findings / analysis 的，
结果是否被工具层以 "Subagents should return findings" 拒。
用法：scan_report_names.py <会话根> <起始日期> <分片号> <分片数> > 输出（制表符分隔）"""
import json, os, sys, glob, datetime
root, since, shard, nshards = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
since_ts = datetime.datetime.strptime(since, "%Y-%m-%d").timestamp()
KEYS = (b"report", b"summary", b"findings", b"analysis")
files = sorted(glob.glob(os.path.join(root, "*", "subagents", "agent-*.jsonl")))
for i, path in enumerate(files):
    if i % nshards != shard:
        continue
    try:
        if os.path.getmtime(path) < since_ts:
            continue
    except OSError:
        continue
    uses = {}
    with open(path, "rb") as f:
        lines = f.readlines()
    for line in lines:
        if b'"name":"Write"' not in line:
            continue
        low = line.lower()
        if not any(k in low for k in KEYS):
            continue
        try:
            rec = json.loads(line)
        except Exception:
            continue
        content = rec.get("message", {}).get("content")
        if not isinstance(content, list):
            continue
        for b in content:
            if isinstance(b, dict) and b.get("type") == "tool_use" and b.get("name") == "Write":
                fp = (b.get("input") or {}).get("file_path", "")
                base = os.path.basename(fp).lower()
                if any(k.decode() in base for k in KEYS):
                    uses[b["id"]] = fp
    if not uses:
        continue
    seen = set()
    for line in lines:
        if b"tool_result" not in line:
            continue
        if not any(u.encode() in line for u in uses):
            continue
        try:
            rec = json.loads(line)
        except Exception:
            continue
        content = rec.get("message", {}).get("content")
        if not isinstance(content, list):
            continue
        for b in content:
            if isinstance(b, dict) and b.get("type") == "tool_result" and b.get("tool_use_id") in uses:
                c = b.get("content")
                text = json.dumps(c, ensure_ascii=False) if not isinstance(c, str) else c
                rejected = "Subagents should return findings" in text
                seen.add(b["tool_use_id"])
                print("\t".join([os.path.basename(path), uses[b["tool_use_id"]], "拒" if rejected else "放", "err" if b.get("is_error") else "ok"]))
    for u, fp in uses.items():
        if u not in seen:
            print("\t".join([os.path.basename(path), fp, "无结果", ""]))
