let stream = open(file("Cargo.toml"))!;
let lines = lines(stream)!;
printf("%s\n", [last(lines)]);
