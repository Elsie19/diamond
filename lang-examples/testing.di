let string = "Hello, World!";
for (char in chars(string)) {
    printf("%s", [char]);
}
printf("\n", []);

let foo = let bar = let baz = "hello";

dump_var(foo);
dump_var(bar);
dump_var(baz);
