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
let ~internal create(path: file): result(file, string) = ();
let ~internal open(path: file): result(stream, string) = ();

# Get last element of list.
#
# This works because for loops are
# expressions that return the last
# element.
let last(lst: [any]): any = {
    for (i in lst) {
        i
    }
};
