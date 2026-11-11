# aprovecha – Make the most of Provecho
Download and decode recipes from [provecho.co](https://www.provecho.co/platform) and creator sites like [breadbakebeyond.com](https://www.breadbakebeyond.com/@breadbakebeyond/site)!

## Usage

### 1. Download encoded JSON recipe files
*TODO*

### 2. Find encryption key using subcommand `key`
Example: **`$ aprovecha key --freq-file eng.json my-recipe.json`**

Full usage:

```
Extract the encryption key from a JSON recipe file through statistical analysis

Usage: aprovecha key [OPTIONS] --freq-file <FILE> <FILE>

Arguments:
  <FILE>  The input JSON file from which to extract the key

Options:
      --freq-file <FILE>  A JSON file containing absolute frequencies of the most common characters
  -l, --length <n>        Specify the number of characters in the key (if known). Otherwise, period of input will be analyzed to determine key length
      --max-length <n>    Check key lengths from 1 up to n. Try increasing this number if no key could be found [default: 32]
      --no-check          Do not check if the found key actual produces valid JSON when applied to the input file
  -h, --help              Print help
```

### 3. Decode recipe file using the key
Example: **`$ aprovecha decode --output . --key "abcd" my-recipe.json`**

Full usage:

```
Decode a JSON recipe file using the encryption key

Usage: aprovecha decode [OPTIONS] --output <DIRECTORY> --key <KEY> [FILE]...

Arguments:
  [FILE]...  The input JSON files to decode

Options:
  -o, --output <DIRECTORY>  The output directory path
  -k, --key <KEY>           The repeated key for the XOR cipher (usually 24 characters long)
  -f, --force               Overwrite existing output files
  -h, --help                Print help
```
