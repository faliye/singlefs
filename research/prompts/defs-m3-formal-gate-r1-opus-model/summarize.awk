# 每段历史 × 每种换法：三条问法（q1 退 0、q2 / q3 退 10 算复用）里复用了几条
/^H[1-4]\t/ { split($4, a, "（"); rc = a[1]; reuse = (substr($3, 1, 2) == "q1" ? (rc == "退 0") : (rc == "退 10"));
  key = $1 " " $2; n[key]++; r[key] += reuse; if (!(key in seen)) { order[++k] = key; seen[key] = 1 } }
END { for (i = 1; i <= k; i++) print order[i] "：三条路里复用 " r[order[i]] "/" n[order[i]] }
