let stream = open(file("Cargo.toml"))!;
let lines = skip(stream, 1)!;
for (line in lines) {
    printf("%s\n", [line]);
};
# printf("%s\n", [last(lines)]);
