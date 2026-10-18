"""放过的状态在被判那次挂载之前是不是已经红：数（候选，判）× 盘上此刻池级 checker 红不红。
用法：python3 tally.py opus <输出文件…> | python3 tally.py sonnet <输出文件…>
打出：name=r3_tally <腿> candidate=<候选> judge=pass|refuse state_red=red|green rows=N（一个状态两种盘序各数一行）；\n攻方腿另打每个走到那一判的状态上映射根凭据与指针槽凭据的四种组合 name=r3_tally opus credentials … rows=N"""
import collections, re, sys

kind, *paths = sys.argv[1:]
count = collections.Counter()
for path in paths:
    for line in open(path, encoding="utf-8"):
        if not line.startswith("name=c834_state"):
            continue
        fields = dict(re.findall(r"(\w+)=(\S+|\[[^\]]*\])", line))
        colour = "green" if fields["state_red"] == "-" else "red"
        if kind == "opus":
            credentials = dict(re.findall(r"(mapping_root_newer|pointer_per_kind):(true|false)", fields["creds"]))
            if credentials:
                count[("credentials", f"mapping_root_newer={credentials['mapping_root_newer']}", f"pointer_per_kind={credentials['pointer_per_kind']}")] += 1
            for cell in fields["cands"].split(";"):
                candidate, verdict, _ = cell.split(":", 2)
                if candidate == "all" or candidate not in ("pointer_per_kind", "k6", "always_refuse", "today", "k3", "pointer_r1"):
                    continue
                count[(candidate, verdict, colour)] += 1
        else:
            judge = re.search(r"judge=(\S+)", line).group(1)
            if judge == "not_reached":
                continue
            verdict = ",".join(sorted(set(re.findall(r"verdict=(\w+)", judge))))
            count[(fields["candidate"], verdict, colour)] += 1
for (candidate, verdict, colour), rows in sorted(count.items()):
    if candidate == "credentials":
        print(f"name=r3_tally {kind} credentials {verdict} {colour} rows={rows}")
    else:
        print(f"name=r3_tally {kind} candidate={candidate} judge={verdict} state_red={colour} rows={rows}")
