let ~internal printf(format: string, args: [any]): integer = ();
let ~internal sprintf(format: string, args: [any]): string = ();
let ~internal puts(format: string): unit = ();
let ~internal atoi(str: string): result(integer, string) = ();
let ~internal itoa(str: integer): string = ();
let ~internal panic(format: string, args: [any]): unret = ();
let ~internal dump_var(var: any): unit = ();
let ~internal testing_branch(int: integer): result(integer, integer) = ();

let bla = 0;
let foobar = "hello";
let woah_there = bla;

let func(int: integer): string = {
    itoa(int)
};

let bla = for (i in [1, 2, 3]) {
    i
};

let IMHEREHOES = func(bla);
printf("Hello, World!\n", []);
printf("IMHEREHOES is `%s`\n", [IMHEREHOES]);
printf("number is `%d` but as string is `%s`!\n", [bla, IMHEREHOES]);

for (char in ["H", "e", "l", "l", "o", ",", " ", "W", "o", "r", "l", "d", "!", "\n"]) {
    printf("%s", [char]);
};

let num = match (testing_branch(1)) {
    ok o = o,
    err e = panic("NUMBER RETURNED IS `%d`\n", [e]),
};
