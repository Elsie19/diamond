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
let ~internal chars(string: string): [string] = ();
let ~internal only(arr: [any], up_to: integer): [any] = ();
let ~internal len(probs_arr: any): integer = ();
let ~internal enumerate(arr: [any]): [[any]] = ();

###################
#    SYSTEM       #
###################
let ~internal exit(code: integer): unret = ();
let ~internal args(): [string] = ();

let ~internal panic(format: string, args: [any]): unret = ();
let ~internal dump_var(var: any): unit = ();

let ~internal max(fst: integer, snd: integer): integer = ();
let ~internal min(fst: integer, snd: integer): integer = ();

################
#    RESULT    #
################
let ~internal ok(val: any): result(any, any) = ();
let ~internal err(val: any): result(any, any) = ();

let last(lst: [any]): any = for (i in lst) i;
