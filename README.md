# aprovecha – Make the most of Provecho

Download and decode recipes from [provecho.co](https://www.provecho.co/platform) and creator sites like [breadbakebeyond.com](https://www.breadbakebeyond.com/@breadbakebeyond/site)!  
Works without subscription.

Results are the original JSON files with all available data, including:
* Name and notes
* Steps
* Ingredients (with separate name, unit, quantity and image) and tools
* Links to video and image resources
* Time to cook/prepare
* Servings
* Ratings
* Creation/modification date

This tool is inofficial and not associated with Provecho or any of its creators, including usernames mentioned in examples.

## Download compiled binaries

Instead of [building the tool yourself](#build), you can download statically linked binaries for macOS and Linux from the [**latest&nbsp;release**](https://github.com/YourMJK/aprovecha/releases/latest).

## Usage (automatic)

Example: **`$ aprovecha auto --username breadbakebeyond —-out recipes/`**

Full usage:

```
All-in-one subcommand for automatically downloading and decoding all recipes of a given user

Usage: aprovecha auto [OPTIONS] --username <USERNAME> --out <DIRECTORY>

Options:
  -b, --build-id <BUILD_ID>  The current Next.js build ID of the website (changes regularly).
                             Will be scraped from main site if not specified
  -u, --username <USERNAME>  The username (without leading @) from which to download the recipes
  -i, --interval <ms>        How long to wait between each request (as to not upset the server) in milliseconds [default: 1000]
      --freq-map <FILE>      A JSON file containing absolute frequencies (probabilities) of the most common characters (see subcommand `frequency`).
                             If not specified, the default table for English JSON output will be used
  -l, --length <n>           Specify the number of characters in the key (if known). Otherwise, period of input will be analyzed to determine key length
      --max-length <n>       Check key lengths from 1 up to n. Try increasing this number if no key could be found [default: 32]
  -f, --force                Overwrite existing output files
  -o, --out <DIRECTORY>      The output directory path
  -h, --help                 Print help
```

## Usage (manual)

If the `auto` subcommand fails at some stage, you can try each step manually.

| Subcommand | Description|
| :-------------- | :-- |
| **`auto`**      | All-in-one subcommand for automatically downloading and decoding all recipes of a given user |
| **`list`**      | List available recipes from provecho.co |
| **`download`**  | Download JSON recipe files from provecho.co |
| **`key`**       | Extract the encryption key from a JSON recipe file through statistical analysis |
| **`decode`**    | Decode a JSON recipe file using the encryption key |
| **`frequency`** | Analyze the character frequencies of known valid output files and return the result as a JSON dictionary |


### 1. Download encoded JSON recipe files

If you don't specify any recipe names, all available recipes of the given user will be downloaded.  
Use the subcommand `list` to list all recipe names.

Example: **`$ aprovecha download --username breadbakebeyond —-out recipes/`**

Full usage:

```
Download JSON recipe files from provecho.co

Usage: aprovecha download [OPTIONS] --username <USERNAME> --out <DIRECTORY> [NAME]...

Arguments:
  [NAME]...  The (slug) names of the recipes to download.
             If none are provided, all available recipes will be downloaded

Options:
  -b, --build-id <BUILD_ID>  The current Next.js build ID of the website (changes regularly).
                             Will be scraped from main site if not specified
  -u, --username <USERNAME>  The username (without leading @) from which to download the recipes
  -i, --interval <ms>        How long to wait between each request (as to not upset the server) in milliseconds [default: 1000]
  -f, --force                Overwrite existing output files
  -o, --out <DIRECTORY>      The output directory path
  -h, --help                 Print help
```

### 2. Find encryption key

Example: **`$ aprovecha key recipes/my-recipe.json`**

Full usage:

```
Extract the encryption key from a JSON recipe file through statistical analysis

Usage: aprovecha key [OPTIONS] <RECIPE FILE>

Arguments:
  <RECIPE FILE>  The input JSON file from which to extract the key

Options:
      --freq-map <FILE>  A JSON file containing absolute frequencies (probabilities) of the most common characters (see subcommand `frequency`).
                         If not specified, the default table for English JSON output will be used
  -l, --length <n>       Specify the number of characters in the key (if known). Otherwise, period of input will be analyzed to determine key length
      --max-length <n>   Check key lengths from 1 up to n. Try increasing this number if no key could be found [default: 32]
      --no-check         Do not check if the found key actual produces valid JSON when applied to the input file
      --debug            Print debug information such as confidence per found key character
  -h, --help             Print help
```

### 3. Decode recipe files using the key

Example: **`$ aprovecha decode --key "abcd" --out decoded/ recipes/*`**

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

## Build

To build the package, the Rust toolchain needs to be installed:  
https://rust-lang.org/tools/install

Build and copy executable to `bin/`:
```
$ make
```
Automatically install executable into `/usr/local/bin/`:
```
$ make install
```

Alternatively, you can just run `$ cargo build` to build into `target/`.
