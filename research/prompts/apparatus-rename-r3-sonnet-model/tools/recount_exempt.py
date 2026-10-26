#!/usr/bin/env python3
"""J3：长活豁免表「6 行换算之后与基准逐字节相同」不经 lib_long_run.py 重数：基准表里每行 (路径, sha256) → 把基准那份脚本整串换前缀 → 与此刻同路径的脚本、此刻表里的 sha256 比。
用法：recount_exempt.py --repo <仓根> --base <基准>"""
import argparse
import hashlib
import re
import subprocess

OLD, NEW = "scripts/experiments", "scripts/experiment-apparatus"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--base", required=True)
    arguments = parser.parse_args()
    run = lambda *a: subprocess.run(["git", "-C", arguments.repo, *a], capture_output=True)
    table = ".claude/long-run-live-hosts-exempt"
    rows = lambda text: [re.split(r"\s{2,}", line.strip())[:2] for line in text.split("\n") if line.strip() and not line.startswith("#")]
    base_rows = rows(run("show", f"{arguments.base}:{table}").stdout.decode())
    current_rows = {path: sha for path, sha in rows(open(f"{arguments.repo}/{table}", encoding="utf-8").read())}
    good = 0
    for old_path, old_sha in base_rows:
        new_path = NEW + old_path[len(OLD):] if old_path.startswith(OLD + "/") else old_path
        data = run("show", f"{arguments.base}:{old_path}").stdout
        replaced = data.replace(OLD.encode(), NEW.encode())
        with open(f"{arguments.repo}/{new_path}", "rb") as handle:
            current = handle.read()
        ok = (hashlib.sha256(data).hexdigest() == old_sha and replaced == current and hashlib.sha256(current).hexdigest() == current_rows.get(new_path))
        good += ok
    print(f"name=recount_exempt base_rows={len(base_rows)} current_rows={len(current_rows)} rows_that_are_path_only={good}")


main()
