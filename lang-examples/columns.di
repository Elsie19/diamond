let [internal] @file(path: string): result(file, string) = ();
let [internal] @printf(format: string, txt: string): unit = ();
let [internal] @dump(stream: stream, txt: string): unit = ();
let [internal] @nth(arr: [string], nth: integer): result(string, string) = ();
let [internal] @panic(fmt: string): unret = ();

# returns unit type.
let @tee(file: stream, txt: string) = {
    @printf("%s", txt);
    @dump(file, txt);
};

let @bar(): integer = 5;

# TODO: Remove
let ARGV = ["one", "two", "three"];

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
        @assert_eq(@length(line_split), csv_length);
        let txt = @sprintf("%s\n", @join_str(line_split, ","));
        @tee(output, txt);
    }
} < @open(file)!;
