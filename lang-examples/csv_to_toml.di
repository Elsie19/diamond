let record_header_str = sprintf("[[record]]\n", []);
let nline = sprintf("\n", []);
let pats = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

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

            let check = pattern_pos(val, 0, pats);
            let line = match (check) {
                ok o  = sprintf("%s = %s\n", [key, val]),
                err e = sprintf("%s = '%s'\n", [key, val]),
            };

            dump(out, line);
        }

        dump(out, nline);
    }
}
