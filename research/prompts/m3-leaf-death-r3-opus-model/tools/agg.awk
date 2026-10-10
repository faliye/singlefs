# 按 (路径, 变体, 读法, 树) 合起来：段数、r / m / q 峰值的最大值、r > 0 与 m > 0 的段数。输入：sweep 的输出行。
/name=cell/ {
    delete v
    for (i = 1; i <= NF; i++) { n = index($i, "="); if (n > 0) v[substr($i, 1, n - 1)] = substr($i, n + 1) }
    k = "path=" v["path"] " variant=" v["variant"] " reader=" v["reader"] " tree=" v["tree"]
    h[k]++
    if (v["r_peak"] + 0 > R[k]) R[k] = v["r_peak"] + 0
    if (v["m_peak"] + 0 > M[k]) M[k] = v["m_peak"] + 0
    if (v["q_peak"] + 0 > Q[k]) Q[k] = v["q_peak"] + 0
    if (v["r_peak"] + 0 > 0) rp[k]++
    if (v["m_peak"] + 0 > 0) mp[k]++
}
END { for (k in h) print k, "histories=" h[k], "r_max=" R[k] + 0, "m_max=" M[k] + 0, "q_max=" Q[k] + 0, "histories_r_pos=" rp[k] + 0, "histories_m_pos=" mp[k] + 0 }
