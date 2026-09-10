1. c2 is a consumer that demands K greater than 2. Its requirement value is the in-flight record limit, which is a number not defined in the repository but could be set to a value exceeding 2 in practice.

2. The void clause requires both a requirement over 2 generations and inability to abort. A consumer needing 5 generations could be argued to abort and re-anchor, making the conjunction false. This makes the clause unfalsifiable in practice because any consumer needing more than 2 generations might be abortable, preventing the void clause from ever firing.

3. C claims K and ring depth are determined by c1 and R. Ring depth is R times S. F2 states S is not a format decision and its upper bound comes from I-7.4 pinning. C silently decides S by assuming it is fixed by c1 and R, but S depends on other factors. This contradicts F2.

4. If the two K variables are treated as one: accounting requires K=10 but I-7.4 only needs K=2. Setting K=10 wastes resources. If I-7.4 needs K=5 and accounting needs K=2, setting K=2 causes I-7.4 to fail. If treated as two separate variables: accounting K set to 2 when it needs 5 causes accounting failure, but I-7.4 K set to 5 works correctly.

5. F13 states after a crash runtime K is set to ring depth. Ring depth depends on R and S, where S is determined by I-7.4 pinning. This makes ring depth a consumer of itself because ring depth determines K. C claims K is only determined by c1 and R, but ring depth (via S) overrides this, breaking C's claim.
