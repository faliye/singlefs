1. Yes, the implementation can sort all new and rewritten allocation records by key in the first round and merge them into the fewest leaves. This avoids the unbounded chain because each leaf is updated only once per publish, making the chain bounded by the number of unique leaves affected.

2. 
- Borrow from the reserve: causes a publish that can never complete because the reserve was already subtracted at admission and there is no extra space to borrow.
- Borrow freed blocks whose free generation has not passed the floor: breaks correctness and F5 because blocks needed for rollback states might be overwritten.
- Push an empty publish and retry: does not break correctness, F4, or F5; the publish can eventually complete or report ENOSPC after 8 attempts as per F6.
- Switch the file system to read-only: breaks F4 because df reports free space that is not writable.

3. Push an empty publish and retry. The single observation that would change this pick is if empty publishes consume more space than they free, making it impossible to advance the free generation floor and reuse space.
