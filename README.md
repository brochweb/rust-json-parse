# rs-json

| file                     | implementation | time (secs) | memory (KB) |
|:------------------------:|:--------------:|:-----------:|:-----------:|
| tests/ascii_strings.json | rs-json        | 0.188       | 118192      |
| tests/ascii_strings.json | serde_json     | 0.093       | 101568      |
| tests/numbers.json       | rs-json        | 0.111       | 85952       |
| tests/numbers.json       | serde_json     | 0.173       | 108144      |
| tests/random.json        | rs-json        | 0.303       | 242704      |
| tests/random.json        | serde_json     | 0.449       | 177584      |
| tests/food.json          | rs-json        | 0.004       | 1824        |
| tests/food.json          | serde_json     | 0.004       | 1744        |
| tests/geojson.json       | rs-json        | 0.069       | 60592       |
| tests/geojson.json       | serde_json     | 0.108       | 66336       |
