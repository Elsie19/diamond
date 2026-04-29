# set file to the first file inputted.
let file = file(match (nth(args(), 0)) {
                ok o = o,
                err e = panic("expected file to be passed\n", []),
             }); # ty : file

{
    let lines = lines(STREAM)!; # ty : [string]
    let first_line = nth(lines, 0)!; # ty : string
    let csv_header_split = split(first_line, ","); # ty : [string]
    printf("%s\n", [join(csv_header_split, ",")]); # ty : unit

    # main loop.
    for (line in skip(lines, 1)!) {
        let line_split = split(line, ","); # ty : [string]
        printf("%s\n", [join(line_split, ",")]); # ty : unit
    }

} < open(file)!;
