let ~internal @file(path: string): file = ();
let ~internal @printf(format: string, txt: string): unit = ();
let ~internal @sprintf(format: string, txt: string): string = ();
let ~internal @dump(stream: stream, txt: string): unit = ();
let ~internal @nth(arr: [string], nth: integer): result(string, string) = ();
let ~internal @panic(fmt: string): unret = ();
let ~internal @open(file: file): result(stream, string) = ();
let ~internal @create(file: file): result(file, string) = ();
let ~internal @split(line: string, char: string): [string] = ();
let ~internal @length(arr: [string]): integer = ();
let ~internal @join_str(arr: [string], char: string): string = ();
let ~internal @skip(arr: [string], skip: integer): [string] = ();

# returns unit type.
let @tee(file: stream, txt: string) = {
    @printf("%s", txt);
    @dump(file, txt);
};

let @bar(): integer = 5;

# TODO: Remove
let ARGV = ["one", "two", "three"];
let STREAM = ["bla"];

# set file to the first file inputted.
let file = @file(match (@nth(ARGV, 0)) {
                ok o = o,
                err e = @panic("expected file to be passed"),
             }); # ty : file
let output = @open(@create(@file("kvs.txt"))!)!; # ty : stream

{
    let first_line = @nth(STREAM, 0)!; # ty : string
    let csv_header_split = @split(first_line, ","); # ty : [string]
    let csv_length = @length(csv_header_split); # ty : integer
    let header = @sprintf("%s\n", @join_str(csv_header_split, ",")); # ty : string
    @tee(output, header);

    # main loop.
    for (line in @skip(STREAM, 0)) {
        let line_split = @split(line, ",");
        let txt = @sprintf("%s\n", @join_str(line_split, ","));
        @tee(output, txt);
    }
} < @open(file);
