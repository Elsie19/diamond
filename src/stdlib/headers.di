################
#     FILE     #
################
let ~internal file(path: string): file = ();
let ~internal create(path: file): result(file, string) = ();
let ~internal open(path: file): result(stream, string) = ();
let ~internal dump(stream: stream, contents: string): result(unit, string) = ();
let ~internal lines(stream: stream): result([string], string) = ();
let ~internal skip(stream: stream, n: integer): result([string], string) = ();

################
#    PRINTF    #
################
let ~internal printf(format: string, args: [any]): integer = ();
let ~internal sprintf(format: string, args: [any]): string = ();
let ~internal puts(format: string): unit = ();


###################
#   CONVERSIONS   #
###################
let ~internal atoi(str: string): result(integer, string) = ();
let ~internal itoa(str: integer): string = ();

###################
#     ARRAYS      #
###################
let ~internal nth(arr: [any], nth: integer): result(any, string) = ();
let ~internal split(string: string, char: string): [string] = ();

###################
#    SYSTEM       #
###################
let ~internal exit(code: integer): unret = ();
let ~internal args(): [string] = ();

# Get size of something.
#
# Arrays:  size of array.
# Integer: integer size.
# String:  string length.
# Unit:    0.
# Result:  1.
# Stream:  length of file.
# File:    length of filepath.
let ~internal len(probs_arr: any): integer = ();
let ~internal enumerate(arr: [any]): [[any]] = ();

let ~internal panic(format: string, args: [any]): unret = ();
let ~internal dump_var(var: any): unit = ();
let ~internal testing_branch(int: integer): result(integer, integer) = ();

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
