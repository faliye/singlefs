#!/usr/bin/env bash
# 引文取法：每行「文件<TAB>要找的串」，在文件里 grep -nF，恰好一处命中才打「文件:行」与那一整行；0 处或多处打出来并标明，不当引文用。
# 用法：bash cite.sh < citations.tsv
cd /home/user/singlefs || exit 2
while IFS=$'\t' read -r file key; do
  [[ -z "$file" || "$file" == \#* ]] && continue
  hits="$(grep -nF -- "$key" "$file")"; count="$(grep -cF -- "$key" "$file")"
  if [[ "$count" == 1 ]]; then printf '%s:%s\n' "$file" "$hits"; else printf '!! %s 命中 %s 处：%s\n' "$file" "$count" "$key"; fi
done
