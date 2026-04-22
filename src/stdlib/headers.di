let ~internal printf(format: string, args: [any]): integer = ();
let ~internal sprintf(format: string, args: [any]): string = ();
let ~internal puts(format: string): unit = ();
let ~internal atoi(str: string): result(integer, string) = ();
let ~internal itoa(str: integer): string = ();
let ~internal panic(format: string, args: [any]): unret = ();
let ~internal dump_var(var: any): unit = ();
let ~internal testing_branch(int: integer): result(integer, integer) = ();
let ~internal nth(arr: [any], nth: integer): result(any, string) = ();
let ~internal file(path: string): file = ();

# Get last element of list.
#
# This works because in 
let last(lst: [any]): any = {
    for (i in lst) {
        i
    }
};
