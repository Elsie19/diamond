let file = match (nth(args(), 0)) {
    ok o = o,
    err e = panic("pass in file path", []),
}

let how_many = match (nth(args(), 1)) {
    ok o = o,
    err e = panic("pass in how many lines you want to read", []),
}

let how_many = match (atoi(how_many)) {
    ok o = o,
    err e = panic("pass a valid number", []),
}

{
    # Assume we can read from the file.
    let lines = lines(STREAM)!;
    let needed = only(lines, how_many);
    for (line in needed) printf("%s\n", [line]);
} < match (open(file(file))) {
    ok o = o,
    err e = panic("Invalid file '%s'", [file]),
}
