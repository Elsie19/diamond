let last_broken(lst: [any]): any = {
    for (i in lst) {
        i;
    }
};

let bla_two = last_broken([1, 2, 3]);

let my_file = file("hello.csv");

dump_var(bla_two);
dump_var(my_file);

let file = create(my_file)!;
let stream = open(file)!;
dump(stream, "hello, world!")!;
