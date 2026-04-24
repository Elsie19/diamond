let args = args();

for (i in args) {
    printf("arg => %s\n", [i]);
};

let stream = open(file("Cargo.toml"))!;
for (line in lines(stream)!) {
    printf("%s\n", [line]);
};
