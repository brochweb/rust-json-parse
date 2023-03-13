# rust-json-parse

An optimized JSON parser in Rust that parses a slice `&[u8]` to a `JsonValue` enum. It doesn’t parse to a Rust struct.

This is a research project, it is not tested for production, but only provided as an example Rust program to optimize. Suggestions for improving reliability, speed or memory usage are welcome.


|           file           | implementation  | time (secs) | memory (KB) |
| :----------------------: | :-------------: | :---------: | :---------: |
| tests/ascii_strings.json | rust-json-parse |    0.044    |   101584    |
| tests/ascii_strings.json |   serde_json    |    0.079    |   104352    |
| tests/ascii_strings.json |    simd-json    |    0.073    |   152016    |
|    tests/numbers.json    | rust-json-parse |    0.094    |    86128    |
|    tests/numbers.json    |   serde_json    |    0.119    |   109216    |
|    tests/numbers.json    |    simd-json    |    0.100    |   152336    |
|    tests/random.json     | rust-json-parse |    0.298    |   242096    |
|    tests/random.json     |   serde_json    |    0.341    |   180688    |
|    tests/random.json     |    simd-json    |    0.317    |   243104    |
|     tests/food.json      | rust-json-parse |    0.004    |    2352     |
|     tests/food.json      |   serde_json    |    0.004    |    2176     |
|     tests/food.json      |    simd-json    |    0.005    |    2464     |
|    tests/geojson.json    | rust-json-parse |    0.045    |    59792    |
|    tests/geojson.json    |   serde_json    |    0.070    |    67568    |
|    tests/geojson.json    |    simd-json    |    0.068    |    84016    |

A [Broch Web Solutions](https://www.brochweb.com/) project