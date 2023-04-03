# rust-json-parse

An optimized JSON parser in Rust that parses a slice `&[u8]` to a `JsonValue` enum. It doesn’t parse to a Rust struct.

This is a research project, it is not tested for production, but only provided as an example Rust program to optimize. Suggestions for improving reliability, speed or memory usage are welcome.


|           file           | implementation  | time (secs) | memory (KB) |
| :----------------------: | :-------------: | :---------: | :---------: |
| tests/ascii_strings.json | rust-json-parse |    0.038    |   101184    |
| tests/ascii_strings.json |   serde_json    |    0.097    |   120480    |
| tests/ascii_strings.json |    simd-json    |    0.075    |   124944    |
|    tests/numbers.json    | rust-json-parse |    0.087    |    85696    |
|    tests/numbers.json    |   serde_json    |    0.109    |   126432    |
|    tests/numbers.json    |    simd-json    |    0.109    |   161248    |
|    tests/random.json     | rust-json-parse |    0.276    |   241504    |
|    tests/random.json     |   serde_json    |    0.338    |   198128    |
|    tests/random.json     |    simd-json    |    0.330    |   259728    |
|     tests/food.json      | rust-json-parse |    0.003    |    2032     |
|     tests/food.json      |   serde_json    |    0.004    |    2048     |
|     tests/food.json      |    simd-json    |    0.003    |    2208     |
|    tests/geojson.json    | rust-json-parse |    0.041    |    59440    |
|    tests/geojson.json    |   serde_json    |    0.077    |    89296    |
|    tests/geojson.json    |    simd-json    |    0.071    |   105680    |

A [Broch Web Solutions](https://www.brochweb.com/) project