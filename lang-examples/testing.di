let ~internal printf(format: string, args: [any]): integer = ();
let ~internal sprintf(format: string, args: [any]): string = ();
let ~internal puts(format: string): unit = ();
let ~internal atoi(str: string): result(integer, string) = ();
let ~internal itoa(str: integer): string = ();
let ~internal panic(msg: string): unret = ();
let ~internal dump_var(var: any): unit = ();

let bla = 0;
let foobar = "hello";
let woah_there = bla;

# let func(int: integer): string = {
#     itoa(int)
# };

let bla = for (i in [1, 2, 3]) {
    i
};

let IMHEREHOES = itoa(bla);
printf("number is `%d` but as string is `%s`!\n", [bla, IMHEREHOES]);
