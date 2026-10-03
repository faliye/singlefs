| 历史（组内条数） | const | first | probe512 | probe8192 | refuse | search | search-mkfsclears | today | unique |
|---|---|---|---|---|---|---|---|---|---|
| C:y=512:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN |
| C:y=512:n=2 等 2 段 | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | N/NN |
| C:y=65536:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/00[I-8.5]+N/00[I-8.5] | N/NN+N/NN |
| C:y=65536:n=2 等 2 段 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/00[I-8.5] | N/NN |
| C:y=8192:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/00[I-8.5]+N/00[I-8.5] | N/NN+N/NN |
| C:y=8192:n=2 等 2 段 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/00[I-8.5] | N/NN |
| S:x=512:y=65536:u=0:reuse=both:fsid=diff:n=0 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+S!noroot/SS[I-7.1] | N/NN+N/NN | N/SS[I-7.1]+S!noroot/SS[I-7.1] | R!noroot/00[I-8.5]x2 |
| S:x=512:y=65536:u=0:reuse=both:fsid=diff:n=2 等 4 段 | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/SS[I-7.1] | R!noroot/00[I-8.5] |
| S:x=512:y=65536:u=0:reuse=both:fsid=same:n=0 | S/SS+S/SS[I-7.7] | S/SS+S/SS[I-7.7] | S/SS+S/SS[I-7.7] | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+S/SS | N/NN+N/NN | N/SS[I-7.7]+S/SS | R!noroot/00[I-8.5]x2 |
| S:x=512:y=65536:u=0:reuse=both:fsid=same:n=2 等 2 段 | S/SS[I-7.7] | S/SS[I-7.7] | S/SS[I-7.7] | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/SS[I-7.7] | R!noroot/00[I-8.5] |
| S:x=512:y=65536:u=0:reuse=disk0:fsid=diff:n=0 | S!noroot/S0[I-7.1]x2 | R!noroot/SN[I-1.4, I-7.1, I-7.5]+R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1]x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+R!noroot/SN[I-1.4, I-7.1, I-7.5] | N/NN+N/NN | N/S0[I-7.1]+R!noroot/S0[I-7.1] | N/0N+N/0N |
| S:x=512:y=65536:u=0:reuse=disk0:fsid=diff:n=2 等 4 段 | S!noroot/S0[I-7.1] | R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1] | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/S0[I-7.1] | N/0N |
| S:x=512:y=65536:u=0:reuse=disk0:fsid=same:n=0 | S/S0x2 | S/SN[I-7.5]+S/SN[I-7.7, I-7.5] | S/S0x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+S/SN[I-7.5] | N/NN+N/NN | N/S0+S/S0 | N/0N+N/0N |
| S:x=512:y=65536:u=0:reuse=disk0:fsid=same:n=2 等 2 段 | S/S0 | S/SN[I-7.7, I-7.5] | S/S0 | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/S0 | N/0N |
| S:x=512:y=65536:u=1:reuse=both:fsid=diff:n=0 等 2 段 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S!noroot/SS[I-7.1]x2 | N/NN+N/NN | S!noroot/SS[I-7.1]x2 | R!noroot/00[I-8.5]x2 |
| S:x=512:y=65536:u=1:reuse=both:fsid=same:n=0 | S/SSx2 | S/SSx2 | S/SSx2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S/SSx2 | N/NN+N/NN | S/SSx2 | R!noroot/00[I-8.5]x2 |
| S:x=512:y=65536:u=1:reuse=both:fsid=same:n=2 等 2 段 | S/SS | S/SS | S/SS | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/SS | R!noroot/00[I-8.5] |
| S:x=512:y=65536:u=1:reuse=disk0:fsid=diff:n=0 等 2 段 | S!noroot/S0[I-7.1]x2 | R!noroot/SN[I-1.4, I-7.1, I-7.5]+R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1]x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | R!noroot/SN[I-1.4, I-7.1, I-7.5]+R!noroot/SN[I-1.4, I-7.1, I-7.5] | N/NN+N/NN | R!noroot/S0[I-7.1]x2 | N/0N+N/0N |
| S:x=512:y=65536:u=1:reuse=disk0:fsid=same:n=0 | S/S0x2 | S/SN[I-7.5]+S/SN[I-7.5] | S/S0x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S/SN[I-7.5]+S/SN[I-7.5] | N/NN+N/NN | S/S0x2 | N/0N+N/0N |
| S:x=512:y=65536:u=1:reuse=disk0:fsid=same:n=2 等 2 段 | S/S0 | S/SN[I-7.5] | S/S0 | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/S0 | N/0N |
| S:x=512:y=65536:u=5:reuse=both:fsid=diff:n=2 等 2 段 | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | S!noroot/SS[I-7.1] | N/NN | S!noroot/SS[I-7.1] | R!noroot/00[I-8.5] |
| S:x=512:y=65536:u=5:reuse=both:fsid=same:n=0 | S/SSx2 | S/SSx2 | S/SSx2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S/SSx2 | N/NN+N/NN[I-7.7] | S/SSx2 | R!noroot/00[I-8.5]x2 |
| S:x=512:y=65536:u=5:reuse=both:fsid=same:n=2 等 2 段 | S/SS | S/SS | S/SS | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | S/SS | N/NN | S/SS | R!noroot/00[I-8.5] |
| S:x=512:y=65536:u=5:reuse=disk0:fsid=diff:n=2 等 2 段 | S!noroot/S0[I-7.1] | R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1] | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | R!noroot/SN[I-1.4, I-7.1, I-7.5] | N/NN | R!noroot/S0[I-7.1] | N/0N |
| S:x=512:y=65536:u=5:reuse=disk0:fsid=same:n=0 | S/S0x2 | S/SN[I-7.5]+S/SN[I-7.7, I-7.5] | S/S0x2 | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S/SN[I-7.5]+S/SN[I-7.7, I-7.5] | N/NN+N/NN[I-7.7] | S/S0x2 | N/0N+N/0N |
| S:x=512:y=65536:u=5:reuse=disk0:fsid=same:n=2 等 2 段 | S/S0 | S/SN[I-7.5] | S/S0 | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | S/SN[I-7.5] | N/NN | S/S0 | N/0N |
| S:x=512:y=8192:u=0:reuse=both:fsid=diff:n=0 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+S!noroot/SS[I-7.1] | N/NN+N/NN | N/SS[I-7.1]+S!noroot/SS[I-7.1] | R!noroot/00[I-8.5]x2 |
| S:x=512:y=8192:u=0:reuse=both:fsid=diff:n=2 等 4 段 | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/SS[I-7.1] | R!noroot/00[I-8.5] |
| S:x=512:y=8192:u=0:reuse=both:fsid=same:n=0 | S/SS+S/SS[I-7.7] | S/SS+S/SS[I-7.7] | S/SS+S/SS[I-7.7] | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+S/SS | N/NN+N/NN | N/SS[I-7.7]+S/SS | R!noroot/00[I-8.5]x2 |
| S:x=512:y=8192:u=0:reuse=both:fsid=same:n=2 等 2 段 | S/SS[I-7.7, I-7.17] | S/SS[I-7.7, I-7.17] | S/SS[I-7.7, I-7.17] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/SS[I-7.7, I-7.17] | R!noroot/00[I-8.5] |
| S:x=512:y=8192:u=0:reuse=disk0:fsid=diff:n=0 | S!noroot/S0[I-7.1]x2 | R!noroot/SN[I-1.4, I-7.1, I-7.5]+R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+R!noroot/SN[I-1.4, I-7.1, I-7.5] | N/NN+N/NN | N/S0[I-7.1]+R!noroot/S0[I-7.1] | N/0N+N/0N |
| S:x=512:y=8192:u=0:reuse=disk0:fsid=diff:n=2 等 4 段 | S!noroot/S0[I-7.1] | R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/S0[I-7.1] | N/0N |
| S:x=512:y=8192:u=0:reuse=disk0:fsid=same:n=0 | S/S0x2 | S/SN[I-7.5]+S/SN[I-7.7, I-7.5] | S/S0x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+S/SN[I-7.5] | N/NN+N/NN | N/S0+S/S0 | N/0N+N/0N |
| S:x=512:y=8192:u=0:reuse=disk0:fsid=same:n=2 等 2 段 | S/S0[I-7.17] | S/SN[I-7.7, I-7.17, I-7.5] | S/S0[I-7.17] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/S0[I-7.17] | N/0N |
| S:x=512:y=8192:u=1:reuse=both:fsid=diff:n=0 等 2 段 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | S!noroot/SS[I-7.1]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | S!noroot/SS[I-7.1]x2 | N/NN+N/NN | S!noroot/SS[I-7.1]x2 | R!noroot/00[I-8.5]x2 |
| S:x=512:y=8192:u=1:reuse=both:fsid=same:n=0 | S/SSx2 | S/SSx2 | S/SSx2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | S/SSx2 | N/NN+N/NN | S/SSx2 | R!noroot/00[I-8.5]x2 |
| S:x=512:y=8192:u=1:reuse=both:fsid=same:n=2 等 2 段 | S/SS[I-7.17] | S/SS[I-7.17] | S/SS[I-7.17] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/SS[I-7.17] | R!noroot/00[I-8.5] |
| S:x=512:y=8192:u=1:reuse=disk0:fsid=diff:n=0 等 2 段 | S!noroot/S0[I-7.1]x2 | R!noroot/SN[I-1.4, I-7.1, I-7.5]+R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/SN[I-1.4, I-7.1, I-7.5]+R!noroot/SN[I-1.4, I-7.1, I-7.5] | N/NN+N/NN | R!noroot/S0[I-7.1]x2 | N/0N+N/0N |
| S:x=512:y=8192:u=1:reuse=disk0:fsid=same:n=0 | S/S0x2 | S/SN[I-7.5]+S/SN[I-7.5] | S/S0x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | S/SN[I-7.5]+S/SN[I-7.5] | N/NN+N/NN | S/S0x2 | N/0N+N/0N |
| S:x=512:y=8192:u=1:reuse=disk0:fsid=same:n=2 等 2 段 | S/S0[I-7.17] | S/SN[I-7.17, I-7.5] | S/S0[I-7.17] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/S0[I-7.17] | N/0N |
| S:x=512:y=8192:u=5:reuse=both:fsid=diff:n=2 等 2 段 | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | S!noroot/SS[I-7.1] | N/NN | R!noroot/00[I-8.5] | S!noroot/SS[I-7.1] | N/NN | S!noroot/SS[I-7.1] | R!noroot/00[I-8.5] |
| S:x=512:y=8192:u=5:reuse=both:fsid=same:n=0 | S/SSx2 | S/SSx2 | S/SSx2 | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | S/SSx2 | N/NN+N/NN[I-7.7] | S/SSx2 | R!noroot/00[I-8.5]x2 |
| S:x=512:y=8192:u=5:reuse=both:fsid=same:n=2 等 2 段 | S/SS[I-7.17] | S/SS[I-7.17] | S/SS[I-7.17] | N/NN | R!noroot/00[I-8.5] | S/SS[I-7.17] | N/NN | S/SS[I-7.17] | R!noroot/00[I-8.5] |
| S:x=512:y=8192:u=5:reuse=disk0:fsid=diff:n=2 等 2 段 | S!noroot/S0[I-7.1] | R!noroot/SN[I-1.4, I-7.1, I-7.5] | S!noroot/S0[I-7.1] | N/NN | R!noroot/00[I-8.5] | R!noroot/SN[I-1.4, I-7.1, I-7.5] | N/NN | R!noroot/S0[I-7.1] | N/0N |
| S:x=512:y=8192:u=5:reuse=disk0:fsid=same:n=0 | S/S0x2 | S/SN[I-7.5]+S/SN[I-7.7, I-7.5] | S/S0x2 | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | S/SN[I-7.5]+S/SN[I-7.7, I-7.5] | N/NN+N/NN[I-7.7] | S/S0x2 | N/0N+N/0N |
| S:x=512:y=8192:u=5:reuse=disk0:fsid=same:n=2 等 2 段 | S/S0[I-7.17] | S/SN[I-7.17, I-7.5] | S/S0[I-7.17] | N/NN | R!noroot/00[I-8.5] | S/SN[I-7.17, I-7.5] | N/NN | S/S0[I-7.17] | N/0N |
| S:x=65536:y=512:u=0:reuse=both:fsid=diff:n=0 等 2 段 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=512:u=0:reuse=both:fsid=diff:n=2 等 10 段 | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] |
| S:x=65536:y=512:u=0:reuse=disk0:fsid=diff:n=0 等 2 段 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | N/0N+N/0N |
| S:x=65536:y=512:u=0:reuse=disk0:fsid=diff:n=2 等 10 段 | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | N/0N |
| S:x=65536:y=512:u=1:reuse=both:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+S!noroot/SS[I-7.1] | N/NN+N/NN | N/NN+S!noroot/NN | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=512:u=1:reuse=both:fsid=same:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+S/SS | N/NN+N/NN | N/NN+S/NN | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=512:u=1:reuse=disk0:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+R!noroot/SN[I-1.4, I-7.1] | N/NN+N/NN | N/NN+R!noroot/NN | N/0N+N/0N |
| S:x=65536:y=512:u=1:reuse=disk0:fsid=same:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | N/NN+S/SN | N/NN+N/NN | N/NN+S/NN | N/0N+N/0N |
| S:x=65536:y=512:u=5:reuse=both:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S!fail/SS[I-2.1, I-3.1, I-4.8, I-7.2, I-7.4]x2 | N/NN+N/NN | S!fail/NN+S!fail/NN | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=512:u=5:reuse=both:fsid=diff:n=2 | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | S!fail/SS[I-2.1, I-3.1, I-4.8, I-7.2, I-7.4] | N/NN | S!fail/NN | R!noroot/00[I-8.5] |
| S:x=65536:y=512:u=5:reuse=both:fsid=same:n=0 | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S!fileNone/SSx2 | N/NN+N/NN[I-7.7] | S!fileNone/NN+S!fileNone/NN[I-7.7] | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=512:u=5:reuse=both:fsid=same:n=2 | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | S!fileNone/SS[I-2.1, I-3.1, I-4.8, I-7.4, I-8.10] | N/NN | S!fileNone/NN | R!noroot/00[I-8.5] |
| S:x=65536:y=512:u=5:reuse=disk0:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | R!noroot/SN[I-1.4, I-2.1, I-3.1, I-4.8, I-7.2, I-7.4]+R!noroot/SN[I-1.4, I-2.1, I-3.1, I-4.8, I-7.2, I-7.4] | N/NN+N/NN | R!noroot/NN+R!noroot/NN | N/0N+N/0N |
| S:x=65536:y=512:u=5:reuse=disk0:fsid=diff:n=2 | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | R!noroot/SN[I-1.4, I-2.1, I-3.1, I-4.8, I-7.2, I-7.4] | N/NN | R!noroot/NN | N/0N |
| S:x=65536:y=512:u=5:reuse=disk0:fsid=same:n=0 | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 | S!fileNone/SN[I-2.1, I-3.1, I-4.8, I-7.4, I-7.7]+S!fileNone/SN[I-2.1, I-3.1, I-4.8, I-7.4] | N/NN+N/NN[I-7.7] | S!fileNone/NN+S!fileNone/NN[I-7.7] | N/0N+N/0N |
| S:x=65536:y=512:u=5:reuse=disk0:fsid=same:n=2 | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] | R!noroot/00[I-8.5] | S!fileNone/SN[I-2.1, I-3.1, I-4.8, I-7.4, I-8.10] | N/NN | S!fileNone/NN | N/0N |
| S:x=65536:y=8192:u=0:reuse=both:fsid=diff:n=0 等 2 段 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/00[I-8.5]+N/00[I-8.5] | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=8192:u=0:reuse=both:fsid=diff:n=2 等 10 段 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/00[I-8.5] | R!noroot/00[I-8.5] |
| S:x=65536:y=8192:u=0:reuse=disk0:fsid=diff:n=0 等 2 段 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/00[I-8.5]+N/00[I-8.5] | N/0N+N/0N |
| S:x=65536:y=8192:u=0:reuse=disk0:fsid=diff:n=2 等 10 段 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | N/NN | N/00[I-8.5] | N/0N |
| S:x=65536:y=8192:u=1:reuse=both:fsid=diff:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+S!noroot/SS[I-7.1] | N/NN+N/NN | N/00[I-8.5]+S!noroot/00[I-8.5] | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=8192:u=1:reuse=both:fsid=same:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+S/SS | N/NN+N/NN | N/00[I-8.5]+S/00[I-8.5] | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=8192:u=1:reuse=disk0:fsid=diff:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+R!noroot/SN[I-1.4, I-7.1] | N/NN+N/NN | N/00[I-8.5]+R!noroot/00[I-8.5] | N/0N+N/0N |
| S:x=65536:y=8192:u=1:reuse=disk0:fsid=same:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+S/SN | N/NN+N/NN | N/00[I-8.5]+S/00[I-8.5] | N/0N+N/0N |
| S:x=65536:y=8192:u=5:reuse=both:fsid=diff:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | S!fail/SS[I-2.1, I-3.1, I-4.8, I-7.2, I-7.4]x2 | N/NN+N/NN | S!fail/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=8192:u=5:reuse=both:fsid=diff:n=2 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | S!fail/SS[I-2.1, I-3.1, I-4.8, I-7.2, I-7.4] | N/NN | S!fail/00[I-8.5] | R!noroot/00[I-8.5] |
| S:x=65536:y=8192:u=5:reuse=both:fsid=same:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | S!fileNone/SSx2 | N/NN+N/NN[I-7.7] | S!fileNone/00[I-8.5]x2 | R!noroot/00[I-8.5]x2 |
| S:x=65536:y=8192:u=5:reuse=both:fsid=same:n=2 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | S!fileNone/SS[I-2.1, I-3.1, I-4.8, I-7.4, I-8.10] | N/NN | S!fileNone/00[I-8.5] | R!noroot/00[I-8.5] |
| S:x=65536:y=8192:u=5:reuse=disk0:fsid=diff:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/NN+N/NN | R!noroot/00[I-8.5]x2 | R!noroot/SN[I-1.4, I-2.1, I-3.1, I-4.8, I-7.2, I-7.4]+R!noroot/SN[I-1.4, I-2.1, I-3.1, I-4.8, I-7.2, I-7.4] | N/NN+N/NN | R!noroot/00[I-8.5]x2 | N/0N+N/0N |
| S:x=65536:y=8192:u=5:reuse=disk0:fsid=diff:n=2 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | R!noroot/SN[I-1.4, I-2.1, I-3.1, I-4.8, I-7.2, I-7.4] | N/NN | R!noroot/00[I-8.5] | N/0N |
| S:x=65536:y=8192:u=5:reuse=disk0:fsid=same:n=0 | R!noroot/00[I-8.5]x2 | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | N/NN+N/NN[I-7.7] | R!noroot/00[I-8.5]x2 | S!fileNone/SN[I-2.1, I-3.1, I-4.8, I-7.4, I-7.7]+S!fileNone/SN[I-2.1, I-3.1, I-4.8, I-7.4] | N/NN+N/NN[I-7.7] | S!fileNone/00[I-8.5]x2 | N/0N+N/0N |
| S:x=65536:y=8192:u=5:reuse=disk0:fsid=same:n=2 | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | N/NN | R!noroot/00[I-8.5] | S!fileNone/SN[I-2.1, I-3.1, I-4.8, I-7.4, I-8.10] | N/NN | S!fileNone/00[I-8.5] | N/0N |
| S:x=8192:y=512:u=0:reuse=both:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S!noroot/SS[I-7.1]x2 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 |
| S:x=8192:y=512:u=0:reuse=both:fsid=diff:n=2 等 5 段 | N/NN | N/NN | N/NN | S!noroot/SS[I-7.1] | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] |
| S:x=8192:y=512:u=0:reuse=both:fsid=same:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S/SS+S/SS[I-7.7] | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | R!noroot/00[I-8.5]x2 |
| S:x=8192:y=512:u=0:reuse=both:fsid=same:n=2 等 2 段 | N/NN | N/NN | N/NN | S/SS[I-7.7] | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] |
| S:x=8192:y=512:u=0:reuse=disk0:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S!noroot/S0[I-7.1]x2 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | N/0N+N/0N |
| S:x=8192:y=512:u=0:reuse=disk0:fsid=diff:n=2 等 5 段 | N/NN | N/NN | N/NN | S!noroot/S0[I-7.1] | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | N/0N |
| S:x=8192:y=512:u=0:reuse=disk0:fsid=same:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S/S0x2 | R!noroot/00[I-8.5]x2 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | N/0N+N/0N |
| S:x=8192:y=512:u=0:reuse=disk0:fsid=same:n=2 等 5 段 | N/NN | N/NN | N/NN | S/S0 | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | N/0N |
| S:x=8192:y=512:u=1:reuse=both:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S!noroot/SS[I-7.1]x2 | R!noroot/00[I-8.5]x2 | N/NN+S!noroot/SS[I-7.1] | N/NN+N/NN | N/NN+S!noroot/NN | R!noroot/00[I-8.5]x2 |
| S:x=8192:y=512:u=1:reuse=both:fsid=same:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S/SSx2 | R!noroot/00[I-8.5]x2 | N/NN+S/SS | N/NN+N/NN | N/NN+S/NN | R!noroot/00[I-8.5]x2 |
| S:x=8192:y=512:u=1:reuse=both:fsid=same:n=2 等 3 段 | N/NN | N/NN | N/NN | S/SS | R!noroot/00[I-8.5] | N/NN | N/NN | N/NN | R!noroot/00[I-8.5] |
| S:x=8192:y=512:u=1:reuse=disk0:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S!noroot/S0[I-7.1]x2 | R!noroot/00[I-8.5]x2 | N/NN+R!noroot/SN[I-1.4, I-7.1] | N/NN+N/NN | N/NN+R!noroot/NN | N/0N+N/0N |
| S:x=8192:y=512:u=1:reuse=disk0:fsid=same:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S/S0x2 | R!noroot/00[I-8.5]x2 | N/NN+S/SN | N/NN+N/NN | N/NN+S/NN | N/0N+N/0N |
| S:x=8192:y=512:u=5:reuse=both:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S!noroot/SS[I-7.1]x2 | R!noroot/00[I-8.5]x2 | S!noroot/SS[I-7.1]x2 | N/NN+N/NN | S!noroot/NN+S!noroot/NN | R!noroot/00[I-8.5]x2 |
| S:x=8192:y=512:u=5:reuse=both:fsid=diff:n=2 | N/NN | N/NN | N/NN | S!noroot/SS[I-7.1] | R!noroot/00[I-8.5] | S!noroot/SS[I-7.1] | N/NN | S!noroot/NN | R!noroot/00[I-8.5] |
| S:x=8192:y=512:u=5:reuse=both:fsid=same:n=0 | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | S/SSx2 | R!noroot/00[I-8.5]x2 | S/SSx2 | N/NN+N/NN[I-7.7] | S/NN+S/NN[I-7.7] | R!noroot/00[I-8.5]x2 |
| S:x=8192:y=512:u=5:reuse=both:fsid=same:n=2 | N/NN | N/NN | N/NN | S/SS | R!noroot/00[I-8.5] | S/SS | N/NN | S/NN | R!noroot/00[I-8.5] |
| S:x=8192:y=512:u=5:reuse=disk0:fsid=diff:n=0 | N/NN+N/NN | N/NN+N/NN | N/NN+N/NN | S!noroot/S0[I-7.1]x2 | R!noroot/00[I-8.5]x2 | R!noroot/SN[I-1.4, I-7.1]+R!noroot/SN[I-1.4, I-7.1] | N/NN+N/NN | R!noroot/NN+R!noroot/NN | N/0N+N/0N |
| S:x=8192:y=512:u=5:reuse=disk0:fsid=diff:n=2 | N/NN | N/NN | N/NN | S!noroot/S0[I-7.1] | R!noroot/00[I-8.5] | R!noroot/SN[I-1.4, I-7.1] | N/NN | R!noroot/NN | N/0N |
| S:x=8192:y=512:u=5:reuse=disk0:fsid=same:n=0 | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | N/NN+N/NN[I-7.7] | S/S0x2 | R!noroot/00[I-8.5]x2 | S/SN+S/SN[I-7.7] | N/NN+N/NN[I-7.7] | S/NN+S/NN[I-7.7] | N/0N+N/0N |
| S:x=8192:y=512:u=5:reuse=disk0:fsid=same:n=2 | N/NN | N/NN | N/NN | S/S0 | R!noroot/00[I-8.5] | S/SN | N/NN | S/NN | N/0N |
histories=189 groups=99
