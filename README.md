# rs-json

An optimized JSON parser in Rust that parses a slice `&[u8]` to a `JsonValue` enum. It doesn’t parse to a Rust struct.

This is a research project, it is not tested for production, but only provided as an example Rust program to optimize. Suggestions for improving reliability, speed or memory usage are welcome.


|           file           | implementation | time (secs) | memory (KB) |
| :----------------------: | :------------: | :---------: | :---------: |
| tests/ascii_strings.json |    rs-json     |    0.090    |   101008    |
| tests/ascii_strings.json |   serde_json   |    0.093    |   101568    |
|    tests/numbers.json    |    rs-json     |    0.106    |    85504    |
|    tests/numbers.json    |   serde_json   |    0.173    |   108144    |
|    tests/random.json     |    rs-json     |    0.327    |   240432    |
|    tests/random.json     |   serde_json   |    0.449    |   177584    |
|     tests/food.json      |    rs-json     |    0.005    |    1824     |
|     tests/food.json      |   serde_json   |    0.004    |    1744     |
|    tests/geojson.json    |    rs-json     |    0.047    |    59264    |
|    tests/geojson.json    |   serde_json   |    0.108    |    66336    |

A [Broch Web Solutions](https://www.brochweb.com/) project