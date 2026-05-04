let file = file(match (nth(args(), 0)) {
                ok o = o,
                err e = panic("expected file to be passed", []),
             });

{
    for (lines in enumerate(lines(STREAM)!)) {
        let idx = nth(lines, 0)!;
        let line = nth(lines, 1)!;
        match (eq(idx, 0)) {
            ok o = {
                let csv_header_split = split(line, ",");
                printf("%s\n", [join(csv_header_split, "|")]);
            },
            err e = {
                let line_split = split(line, ",");
                printf("%s\n", [join(line_split, "ZZZ")]);
            },
        }
    }
} < open(file)!;
