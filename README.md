# rust-json-parse

An optimized JSON parser in Rust that parses a slice `&[u8]` to a `JsonValue` enum. It doesn’t parse to a Rust struct.

This is a research project, it is not tested for production, but only provided as an example Rust program to optimize. Suggestions for improving reliability, speed or memory usage are welcome.


|           file           | implementation  | time (secs) | memory (KB) |
| :----------------------: | :-------------: | :---------: | :---------: |
| tests/ascii_strings.json | rust-json-parse |    0.044    |   101584    |
| tests/ascii_strings.json |   serde_json    |    0.079    |   104352    |
| tests/ascii_strings.json |    simd-json    |    0.072    |   152016    |
|    tests/numbers.json    | rust-json-parse |    0.094    |    86128    |
|    tests/numbers.json    |   serde_json    |    0.100    |   109232    |
|    tests/numbers.json    |    simd-json    |    0.099    |   152352    |
|    tests/random.json     | rust-json-parse |    0.298    |   242096    |
|    tests/random.json     |   serde_json    |    0.320    |   180720    |
|    tests/random.json     |    simd-json    |    0.311    |   243136    |
|     tests/food.json      | rust-json-parse |    0.004    |    2352     |
|     tests/food.json      |   serde_json    |    0.004    |    2192     |
|     tests/food.json      |    simd-json    |    0.004    |    2480     |
|    tests/geojson.json    | rust-json-parse |    0.045    |    59792    |
|    tests/geojson.json    |   serde_json    |    0.064    |    67600    |
|    tests/geojson.json    |    simd-json    |    0.059    |    84000    |

A [Broch Web Solutions](https://www.brochweb.com/) project