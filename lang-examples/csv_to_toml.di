let record_header_str = sprintf("[[record]]\n", []);
let nline = sprintf("\n", []);

let file = file(match (nth(args(), 0)) {
    ok o = o,
    err e = panic("expected file to be passed", []),
});

let out = open(create(file("out.toml"))!)!;

let all_lines = lines(open(file)!)!;

let header_line = nth(all_lines, 0)!;
let keys = split(header_line, ",");

{
    for (row in skip(all_lines, 1)!) {
        let vals = split(row, ",");

        dump(out, record_header_str);

        for (pair in enumerate(keys)) {
            let idx = nth(pair, 0)!;
            let key = nth(pair, 1)!;
            let val = nth(vals, idx)!;

            let line = sprintf("%s = '%s'\n", [key, val]);
            dump(out, line);
        }

        dump(out, nline);
    }
}
