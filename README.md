# aprovecha – Make the most of Provecho

Download and decode recipes from [provecho.co](https://www.provecho.co/platform) and creator sites like [breadbakebeyond.com](https://www.breadbakebeyond.com/@breadbakebeyond/site)!

## Usage

### 1. Download encoded JSON recipe files

Example: **`$ aprovecha download --username breadbakebeyond —-out recipes/`**

Full usage:

```
Download JSON recipe files from provecho.co

Usage: aprovecha download [OPTIONS] --username <USERNAME> --out <DIRECTORY>

Options:
  -b, --build-id <BUILD_ID>  The current Next.js build ID of the website (changes regularly). Will be scraped from main site if not specified
  -u, --username <USERNAME>  The username (without leading @) from which to download the recipes
  -i, --interval <ms>        How long to wait between each request (as to not upset the server) in milliseconds [default: 1000]
  -f, --force                Overwrite existing output files
  -o, --out <DIRECTORY>      The output directory path
  -h, --help                 Print help
```

### 2. Find encryption key using subcommand `key`

Example: **`$ aprovecha key --freq-file eng.json recipes/my-recipe.json`**

Full usage:

```
Extract the encryption key from a JSON recipe file through statistical analysis

Usage: aprovecha key [OPTIONS] --freq-map <FILE> <RECIPE FILE>

Arguments:
  <RECIPE FILE>  The input JSON file from which to extract the key

Options:
      --freq-map <FILE>  A JSON file containing absolute frequencies of the most common characters
  -l, --length <n>       Specify the number of characters in the key (if known). Otherwise, period of input will be analyzed to determine key length
      --max-length <n>   Check key lengths from 1 up to n. Try increasing this number if no key could be found [default: 32]
      --no-check         Do not check if the found key actual produces valid JSON when applied to the input file
  -h, --help             Print help
```

### 3. Decode recipe file using the key

Example: **`$ aprovecha decode --key "abcd" --output decoded/ recipes/*`**

Full usage:

```
Decode a JSON recipe file using the encryption key

Usage: aprovecha decode [OPTIONS] --key <KEY> --out <DIRECTORY> [RECIPE FILE]...

Arguments:
  [RECIPE FILE]...  The input JSON files to decode

Options:
  -k, --key <KEY>        The repeated key for the XOR cipher (usually 24 characters long)
  -f, --force            Overwrite existing output files
  -o, --out <DIRECTORY>  The output directory path
  -h, --help             Print help
```
