# CRDT

>   Expanding on the previous [Small String Optimization](https://github.com/bwoods/immutable-sso) work to (re)implement a full [Conflict-free replicated data type](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type) for text.



## Safety 

This crate uses `unsafe` as it use a Rust [`union`](https://doc.rust-lang.org/reference/items/unions.html) internally to lower the overhead. However

1. The code is purposefully kept [small/simple](src/crdt/mod.rs) to simplify manual auditing.
2. [Property testing](https://github.com/BurntSushi/quickcheck#readme) is done to ensure that it works on a large variety of strings.
3. Every test is run under [Miri](https://github.com/rust-lang/miri#readme) [on every push](https://github.com/bwoods/CRDT/actions) to help check the sanity of the `unsafe` code.

​    ![](https://github.com/bwoods//CRDT/actions/workflows/miri.yml/badge.svg)



## Usage

This repo can be added to your `Cargo.toml` file directly.

```yaml
[dependencies.sso]
git = "https://github.com/bwoods/CRDT"
```



## License

Distributed under the terms of both the MIT license and the Apache License (Version 2.0)

See [LICENSE-APACHE](LICENSE-APACHE.md) and [LICENSE-MIT](LICENSE-MIT.md) for details.
