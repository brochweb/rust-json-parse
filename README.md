# rust-json-parse

An optimized JSON parser in Rust that parses a slice `&[u8]` to a `JsonValue` enum. It doesn’t parse to a Rust struct.

This is a research project, it is not tested for production, but only provided as an example Rust program to optimize. Suggestions for improving reliability, speed or memory usage are welcome.

For the moment nightly-only because it relies on portable SIMD.


|           file           | implementation  | time (secs) | memory (KB) |
| :----------------------: | :-------------: | :---------: | :---------: |
| tests/ascii_strings.json | rust-json-parse |    0.030    |   101152    |
| tests/ascii_strings.json |   serde_json    |    0.063    |   120416    |
| tests/ascii_strings.json |    simd-json    |    0.052    |   124896    |
|    tests/numbers.json    | rust-json-parse |    0.058    |    85680    |
|    tests/numbers.json    |   serde_json    |    0.073    |   126384    |
|    tests/numbers.json    |    simd-json    |    0.080    |   161216    |
|    tests/random.json     | rust-json-parse |    0.196    |   241504    |
|    tests/random.json     |   serde_json    |    0.209    |   198080    |
|    tests/random.json     |    simd-json    |    0.213    |   259696    |
|     tests/food.json      | rust-json-parse |    0.002    |    2000     |
|     tests/food.json      |   serde_json    |    0.003    |    2016     |
|     tests/food.json      |    simd-json    |    0.003    |    2240     |
|    tests/geojson.json    | rust-json-parse |    0.038    |    59424    |
|    tests/geojson.json    |   serde_json    |    0.057    |    89344    |
|    tests/geojson.json    |    simd-json    |    0.051    |   105760    |

A [Broch Web Solutions](https://www.brochweb.com/) project