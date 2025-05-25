# customasm

## Main Repository:
`https://github.com/hlorenzi/customasm`
## Documentation:
`https://github.com/hlorenzi/customasm/wiki`

## Command-Line Usage:
`customasm <INPUT-FILES...> [options] <OUTPUT-GROUPS...>`

Specify multiple OUTPUT-GROUPS using the -- separator.

Examples:  
* `customasm main.asm`
* `customasm main.asm -f hexdump -o main.txt`
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

If the argument is a string, escape characters
incompatible with the command-line. For example,
spaces can be escaped as `\x20`.

Examples:  
* `-f annotated`  
* `-f annotated,group:4`  
* `-f annotated,base:8,group:3`  
* `-f list,before:"begin\x20data\n"`  

## Formats:
* `binary`  
    Code compatible with your target machine.

* `annotated,base:16,group:2`  
    Annotates the output data with excerpts
    from the source code.
* `annotatedbin`  
    Same as: `annotated,base:2,group:8`  

* `binstr`  
    Uninterrupted string of binary digits.
* `hexstr`  
    Uninterrupted string of hexadecimal digits.
* `bindump`  
    Memory-dump style encoded as binary digits.
* `hexdump`  
    Memory-dump style encoded as hexadecimal digits.

* `mif`  
    Memory Initialization File format.
* `intelhex,addr_unit:8`  
    Intel HEX format. `addr_unit` can be 8, 16, or 32.

* `list,base:16,group:2,between:"",group2:16,between2:"",before:"",after:""`  
    Customizable list format. Digits of the selected `base`
    will be grouped in amounts given by `group`,
    separated by the string given by `between`.
    Optionally, there will be another round of
    grouping by `group2` and `between2`. Output will be
    prefixed and suffixed by the strings given by
    `before` and `after`.

* `deccomma`  
    Bytes encoded as decimal literals
    separated by commas.
* `hexcomma`  
    Bytes encoded as hexadecimal literals
    separated by commas.
* `decspace`  
    Bytes encoded as decimal literals
    separated by spaces.
* `hexspace`  
    Bytes encoded as hexadecimal literals
    separated by spaces.

* `decc`  
    Bytes encoded as decimal literals
    wrapped in a C-style declaration.
* `hexc`  
    Bytes encoded as hexadecimal literals
    wrapped in a C-style declaration.

* `logisim8`  
    For use with the Logisim logic simulator program.
* `logisim16`  
    For use with the Logisim logic simulator program.

* `addrspan`  

* `tcgame,base:16,group:2`  
    Annotates the output data with snippets
    of the source code in a format compatible
    with the assembly editor for the game
    "Turing Complete". Supports base 2 and 16.
    Comments out annotations with `#` and prefixes
    each group with `0x` or `0b`.
* `tcgamebin`  
    Same as: `tcgame,base:2,group:8`

* `symbols`  
    Lists all defined symbols with their resolved values.
* `mesen-mlb`  
    Symbol file for usage with the Mesen NES emulator.