# customasm

## Main Repository:
`https://github.com/hlorenzi/customasm`
## Documentation:
`https://github.com/hlorenzi/customasm/wiki`

## Command-Line Usage:
`customasm <INPUT-FILES...> [options] <OUTPUT-GROUPS...>`

Specify multiple OUTPUT-GROUPS using the -- separator.

Examples:  
* `customasm main.asm -f binary -o main.bin -- -f symbols -o symbols.txt`
* `customasm main.asm --iters=3 -f annotated -p -- -f symbols -- -f binary`

## Global Options:
* `-q, --quiet`  
    Suppress progress reports.  
* `-v, --version`  
    Display version information.  
* `-h, --help`  
    Display this information.  
* `-t, --iters=NUM`  
    The maximum number of resolution iterations to attempt.  
    (Default: 10)  
* `-dNAME, --define=NAME`
* `-dNAME=VALUE, --define=NAME=VALUE`
    Overwrites a constant definition with the given value,
    or `true` if none is given.
* `--color=on/off`  
    Whether to style the output with colors.  
    (Default: on)  
* `--debug-iters`  
    Print debug info during resolution iterations.  
* `--debug-no-optimize-static`  
    Prevent optimization of statically-known values.  
* `--debug-no-optimize-matcher`  
    Prevent optimization of the instruction matcher algorithm.  

## Output Options:
* `-f, --format=FORMAT`  
    The format of the output file. See below for possible values.  
* `-o, --output=FILENAME`  
    The name of the output file.  
* `-p, --print`  
    Print the output to the screen instead of writing to a file.  

## Format Usage:
Use the format names below with or without
the extra parameters, separated by commas,
with no whitespace in between.

Examples:  
* `-f annotated`  
* `-f annotated,group:4`  
* `-f annotated,base:8,group:3`  

## Formats:
* `binary`  

* `annotated,base:16,group:2`  
    Annotates the output data with snippets
    of the source code.  

* `annotatedbin`  
    Same as: `annotated,base:2,group:8`  

* `binstr`  
* `hexstr`  
* `bindump`  
* `hexdump`  

* `mif`  
* `intelhex`  

* `deccomma`  
* `hexcomma`  
* `decspace`  
* `hexspace`  

* `decc`  
* `hexc`  

* `logisim8`  
* `logisim16`  

* `addrspan`  

* `symbols`  
* `mesen-mlb`  