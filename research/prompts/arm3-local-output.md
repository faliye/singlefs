1. N=6 R=3 S=2 D=2 K=2. Publish six times creating roots in slots 0-5. Publish seventh time overwriting slot 0. Accounting for generation 1 (slot 0) is removed per E's D+1=3. Generation 5 still references blocks from generation 1. System reallocates those blocks for new data. When accessing generation 5 the data is corrupted due to overwritten blocks.

2. Lowering D during rollback: rollback starts with D=3 to access generation 3. D lowered to 2 mid-rollback. System tries to read generation 3 blocks but they were reallocated as K=2 only protects last two generations. Corruption occurs. Raising D during rollback: rollback starts with D=2 accessing generation 2. D raised to 3 mid-rollback. System tries to read generation 3 blocks which were reallocated because K=2 didn't protect them. Corruption occurs.

3. Yes. If D=2 forever E requires accounting to retain 3 generations. Capping promise at 2 without E would retain N generations for accounting. For N>3 this means less accounting data stored on disk under E.

4. E moves the contradiction into D's choice. F3 originally allowed rollback to any valid root in ring (including fake candidates). E ties rollback depth D to K so only D generations are rollback candidates. But if D is set below N fake candidates still exist for D < N. The contradiction is not resolved but shifted to whether D is chosen large enough to avoid fake candidates.

5. Yes. With: N=4 R=3 S=2 (N=6 but use N=4 for simplicity) D=2 K=2. After 5 publishes accounting retains 3 generations. Generation 5 references blocks from generation 1. Accounting for generation 1 removed. System reallocates those blocks. Accessing generation 5 shows corrupted data. This contradicts E's promise of safe rollback within D.
