# agg.txt 按 (族, 读法, 树) 合起来：段数、r 或 m 峰值 > 0 的格数、r / m 峰值的最大值
{ k=$1" "$6" "$7; split($8,h,"="); n[k]+=h[2]; split($9,a,"="); split($11,b,"=");
  if (a[2]+0>0) rc[k]++; if (b[2]+0>0) mc[k]++; if (a[2]+0>R[k]) R[k]=a[2]+0; if (b[2]+0>M[k]) M[k]=b[2]+0; c[k]++ }
END { for (k in n) print k, "cells="c[k], "histories="n[k], "cells_r_pos="rc[k]+0, "cells_m_pos="mc[k]+0, "r_max="R[k]+0, "m_max="M[k]+0 }
