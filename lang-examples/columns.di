# returns unit type.
let tee(file: stream, txt: string) = {
    printf("%s", txt);
    dump(file, txt);
};

let bar(int: integer): integer = int;
let baz(int: integer): integer = bar(int);

let last(lst: [string]): string = for (line in lst) {
        line
};

let testing(): integer = {
    for (line in ["one", "two", "three"]) {
        printf("%s", line);
    }

    0
};

# TODO: Remove
let ARGV = ["one", "two", "three"];

# set file to the first file inputted.
let file = file(match (nth(ARGV, 0)) {
                ok o = o,
                err e = panic("expected file to be passed"),
             }); # ty : file
let output = open(create(file("kvs.txt"))!)!; # ty : stream

{
    let first_line = nth_stream(STREAM, 0)!; # ty : string
    let csv_header_split = split(first_line, ","); # ty : [string]
    let csv_length = len(csv_header_split); # ty : integer
    let header = sprintf("%s\n", join_str(csv_header_split, ",")); # ty : string
    tee(output, header);

    # main loop.
    for (line in skip_stream(STREAM, 0)) {
        let line_split = split(line, ",");
        let txt = sprintf("%s\n", join_str(line_split, ","));
        tee(output, txt);
    }

    for (line in ["foo", "bar", "baz"]) {
        printf("%s", line);
    }

} < open(file)!;
