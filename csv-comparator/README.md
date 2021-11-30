# CSV Comparator

This program is meant to be used as part of the `ink-waterfall` CI in order to see how
the size of different smart contracts changes over time. It does this by taking two CSV
files containing rows in the following form: `contract name, unoptimzed size, optimized size`
and outputing the difference between the new contract sizes and the old contract sizes.

## Usage

```bash
cargo run old-sizes.csv new-sizes.csv old-gas.csv new-gas.csv
```

The CSV formatted output will be written to `STDOUT`.
