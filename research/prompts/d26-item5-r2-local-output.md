Clause 1 breaks: A unit born at txg=10 is overwritten at txg=25. Snapshots exist at txg=10, 20, 30. When snapshot 20 is destroyed, its prev_txg is 10. The deadlist entry for the unit (birth=10) has birth <=10 so it is not freed. When snapshot 30 is destroyed, prev_txg=20. Birth=10 <=20 so not freed. This deadlist entry persists indefinitely causing a leak. The scope note incorrectly claims only one consumer reads birth as physical age; snapshot destruction uses birth for accounting decisions.

Clause 2 breaks: If deadlist entries carry a 14-byte hint, space is wasted without benefit since freeing always uses the central map. For 10,000 deadlist entries this adds 140KB of unnecessary storage. The choice cannot be deferred because including the hint is strictly worse than omitting it.

Clause 3 breaks: Encoding B allows a single key's value to grow unbounded. A file overwritten 1 million times with 1 million snapshots retained creates a value with 1 million version entries. This exceeds practical storage limits and makes the system unusable.

Clause 4 breaks: With destination policy undefined and lowest free slot first (scattered), compacting 1000 locations with fanout 100 rewrites 1000 leaves instead of the claimed ceil(1000/100)=10 leaves. The cost formula is invalid without specifying a destination policy.
