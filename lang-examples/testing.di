let file = match (nth(args(), 0)) {
    ok o = o,
    err e = panic("pass in file path", []),
}

let how_many = match (nth(args(), 1)) {
    ok o = match (atoi(o)) {
        ok o = o,
        err e = panic("pass a valid number", []),
    },
    err e = panic("pass in how many lines you want to read", []),
}


{
    let lines = lines(STREAM)!; # panic if unsuccessful
    let needed = only(lines, how_many);
    for (line in needed) printf("%s\n", [line]);
} < match (open(file(file))) {
    ok o = o,
    err e = panic("Invalid file '%s'", [file]),
}
