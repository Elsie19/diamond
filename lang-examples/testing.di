let bla = 0;
let foobar = "hello";
let my_list = [1, 2, 3, 4];
let woah_there = bla;

let func(int: integer): string = {
    itoa(int)
};

let last(lst: [any]): any = {
    for (i in lst) {
        i
    }
};

let bla = for (i in [1, 2, 3]) {
    i
};

let last_num = last([1, 2, 3]);

printf("last_num is `%d`\n", [last_num]);

let IMHEREHOES = func(bla);
printf("Hello, World!\n", []);
printf("IMHEREHOES is `%s`\n", [IMHEREHOES]);
printf("number is `%d` but as string is `%s`!\n", [bla, IMHEREHOES]);

# let num = match (testing_branch(1)) {
#     ok o = o,
#     err e = panic("NUMBER RETURNED IS `%d`\n", [e]),
# };

for (char in ["H", "e", "l", "l", "o", ",", " ", "W", "o", "r", "l", "d", "!", "\n"]) {
    printf("%s", [char]);
};

printf("found 3rd element to be `%d`\n", [nth(my_list, 2)!]);

let last_broken(lst: [any]): any = {
    for (i in lst) {
        i;
    }
};

let bla = last_broken([1, 2, 3]);

dump_var(bla);
