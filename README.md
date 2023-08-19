# stream-more

More utils to operate Stream in rust

# StreamMore

`StreamMore` extends the functionality of Streams, providing additional methods for merging and sorting.


## Features

- **kmerge_by:** creates a k-way merge Stream by merging the given Streams according to a provided closure function.

- **kmerge_max:** merges Streams by choosing the "greatest" item.

- **kmerge_min:** merges Streams by choosing the "smallest" item.

## Examples

Here are some examples of how to use the functions provided by StreamMore.

**Merge streams in customized order**:

```rust
use futures::StreamExt;
use futures::executor::block_on;
use futures::stream::iter;
use stream_more::StreamMore;

let x = iter([1, 3]);
let y = iter([2, 4]);

let m = x.kmerge_by(|a,b| a < b)
         .merge(y);

let got = block_on(m.collect::<Vec<u64>>());
assert_eq!(vec![1, 2, 3, 4], got);
```


**Merge and choose smallest item**:

```rust
let x = iter([3, 2]);
let y = iter([4, 1]);
let z = iter([5]);

let m = x.kmerge_min()
         .merge(y)
         .merge(z);

let got = block_on(m.collect::<Vec<u64>>());
assert_eq!(vec![1, 2, 3, 4, 5], got);
```

