let last_broken(lst: [any]): any = {
    for (i in lst) {
        i;
    }
};

let bla_two = last_broken([1, 2, 3]);

dump_var(bla_two);
