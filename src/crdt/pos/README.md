# Position Identifiers





## Terminology

Noted that all of the various papers and implementations of these algorithms are inconsistent with their terminology around position vs. identifiers and levels vs. depth, etc.… with respect to each other. This crate is internally consistent with its naming but may not match any particular (other) work.



## Small String Optimization

The bytes that represent the position identifier start are initially stored inline but, as they grow in the size, fallback to being store on the heap.

Both [TinyVec](https://crates.io/crates/tinyvec) and [SmallVec](https://crates.io/crates/smallvec) have the same minimum size as a `Vec` — 24 bytes on 64-bit platforms. This implementation manages to get that down to 16 bytes. For a large number of identifier, this savings adds up. Not just in memory usage, but **cache utilization** as well.



### Safety 

This crate uses `unsafe` as `Position` is a Rust [`union`](https://doc.rust-lang.org/reference/items/unions.html) to accomplish this optimization. However:

1. The `unsafe` subset of the code is purposefully kept [small/simple](mod.rs) to simplify manual auditing.
2. [Property testing](https://github.com/BurntSushi/quickcheck#readme) is done to ensure that it works on a large variety of values.
3. These tests are run under [Miri](https://github.com/rust-lang/miri#readme) ([on every push](https://github.com/bwoods/CRDT/actions)) to help confirm the correctness of the `unsafe` code.

    ![](https://github.com/bwoods/CRDT/actions/workflows/ci.yml/badge.svg)



The rest of the crate is `#![deny(unsafe_code)]` and no `unsafe` code is brought in from its dependancies.[^itertools]

[^itertools]: The itertools crate [does contain](https://github.com/search?q=repo%3Arust-itertools%2Fitertools%20unsafe&type=code) a small bit of `unsafe` code, but not in any of the algorithms used here.



## Interleaving

All Logoot/LSEQ implementations (including this one) have a well known fault — when two collaborators insert new items in the *exact same position* their results will be interleaved (usually sub-sorted by site id).

>   The interleaving anomaly in Logoot and LSEQ has been independently pointed out in our draft manuscript [^8], by Sun et al. [^19], and by a Stack Overflow user [^3]. From conversations with various members of the CRDT community it appears that the anomaly has been known in the community folklore for some time, but to our knowledge there is no published work that clearly explains the problem or proposes solutions.
>
>   — [Interleaving anomalies in collaborative text editors](https://martin.kleppmann.com/papers/interleaving-papoc19.pdf)

It is widely believed that this anomaly is not a real problem in collaborative on-line text editing, as users will immediately correct the mistake, but it should be noted. 



[^3]: Archagon. 2017. Logoot CRDT: interleaving of data on concurrent edits to the same spot? https://stackoverflow.com/questions/45722742/logoot-crdt-interleaving-of-data-on-concurrent-edits-to-the-same-spot
[^8]: Martin Kleppmann, Victor B. F. Gomes, Dominic P. Mulligan, and Alastair R.Beresford. 2018. OpSets: Sequential Specifications for Replicated Datatypes (Extended Version). https://arxiv.org/abs/1805.04263
[^19]: Chengzheng Sun, David Sun, Agustina, and Weiwei Cai. 2018. Real Differences between OT and CRDT for Co-Editors. https://arxiv.org/abs/1810.02137

