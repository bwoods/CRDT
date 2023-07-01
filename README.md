# CRDT

>   Expanding on the previous [Small String Optimization](https://github.com/bwoods/immutable-sso) work to (re)implement a full [Conflict-free replicated data type](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type) for text.



## Algorithm

-   Logoot
-   LSEQ



### Small String Optimization

The bytes that represent the position identifier start are initially stored inline but, as they grow in the size, fallback to being store on the heap.

Both [TinyVec](https://crates.io/crates/tinyvec) and [SmallVec](https://crates.io/crates/smallvec) have the same minimum size as a `Vec` — 24 bytes on 64-bit platforms. This implementation manages to get that down to 16 bytes. For a large number of strings, this savings adds up. Not just in memory usage, but **cache utilization** as well.



## Safety 

This crate uses `unsafe` as it use a Rust [`union`](https://doc.rust-lang.org/reference/items/unions.html) internally to accomplish this. However:

1. The `unsafe` subset of the code is purposefully kept [small/simple](src/crdt/pos/mod.rs) to simplify manual auditing.
2. [Property testing](https://github.com/BurntSushi/quickcheck#readme) is done to ensure that it works on a large variety of values.
3. These tests are run under [Miri](https://github.com/rust-lang/miri#readme) ()[on every push](https://github.com/bwoods/CRDT/actions)) to help confirm the correctness of the `unsafe` code.

​    ![](https://github.com/bwoods//CRDT/actions/workflows/ci.yml/badge.svg)



## Usage

This repo can be added to your `Cargo.toml` file directly.

```yaml
[dependencies.sso]
git = "https://github.com/bwoods/CRDT"
```



## License

Distributed under the terms of both the MIT license and the Apache License (Version 2.0)

See [LICENSE-APACHE](LICENSE-APACHE.md) and [LICENSE-MIT](LICENSE-MIT.md) for details.



## References

-   Stéphane Weiss, Pascal Urso, Pascal Molli. [Logoot: A Scalable Optimistic Replication Algorithm for Collaborative Editing on P2P Networks](papers/Logoot, A Scalable Optimistic Replication Algorithm for Collaborative Editing on P2P Networks.pdf). 29th IEEE International Conference on Distributed Computing Systems - ICDCS 2009, Jun 2009, Montreal, Canada. pp.404-412, 10.1109/ICDCS.2009.75. [inria-00432368](https://inria.hal.science/inria-00432368)
-   Brice Nédelec, Pascal Molli, Achour Mostefaoui, Emmanuel Desmontils. [LSEQ: an Adaptive Structure for Sequences in Distributed Collaborative Editing](papers/LSEQ, an Adaptive Structure for Sequences in Distributed Collaborative Editing.pdf). 13th ACM Symposium on Document Engineering (DocEng), Sep 2013, Florence, Italy. pp.37–46, 10.1145/2494266.2494278. [hal-00921633](https://hal.science/hal-00921633)

