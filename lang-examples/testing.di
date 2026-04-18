let ~internal @printf(format: string, txt: string): unit = ();
let ~internal @atoi(str: string): result(integer, string) = ();

let @testing(): integer = {
    for (line in ["1", "2", "3"]) {
        @atoi(line)!
    }

    0;
};
