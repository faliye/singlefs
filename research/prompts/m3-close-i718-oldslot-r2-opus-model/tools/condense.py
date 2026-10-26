import sys, re
abbrev = {"holds": "G", "violated": "R", "not_applicable": "NA"}
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("cell="):
        if line.strip(): print(line.rstrip())
        continue
    fields = dict(re.findall(r"(\S+?)=(\[[^\]]*\]|\([^)]*\)|Some\([^)]*\)\)|\S+)", line))
    names = ["today","jia","jia_any_lap","ding_r1","ding_r2_same_instance","ding_r2_any_instance","ding_r2_raw_values"]
    v = " ".join(abbrev.get(fields.get(n,"?"),"?") for n in names)
    print(f"{fields['cell']:<52} {fields['image']:<36} N={fields['newest']:<9} t={fields['t']:<3} p={fields['p']:<3} next={fields['next_slot']:<14} rec={fields['record_same_instance'][0]}{fields['record_any_instance'][0]} | {v} | red={fields['today_red']}")
