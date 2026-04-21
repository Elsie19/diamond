let ~internal printf(format: string, txt: string): unit = ();
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
dump_var(bla);
