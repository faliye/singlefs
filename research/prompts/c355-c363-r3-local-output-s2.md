1. The release chain is unavoidable because updating a leaf requires updating its ancestors, but sorting and merging records into the fewest leaves bounds the chain length to the tree height, which is finite.

2. Borrow from the reserve causes the publish to never complete; borrow freed blocks whose free generation has not passed the floor breaks F5 and correctness; push empty publish and retry has no breaks; switch to read-only breaks F4.

3. Push empty publish and retry; if empty publishes cannot be completed due to insufficient space.
