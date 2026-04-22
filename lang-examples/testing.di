let stream = open(file("Cargo.toml"))!;
for (i in lines(stream)!) {
    printf("%s\n", [i]);
};
