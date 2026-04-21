let ~internal printf(format: string, txt: string): unit = ();
let ~internal atoi(str: string): result(integer, string) = ();
let ~internal itoa(str: integer): string = ();
let ~internal panic(msg: string): unret = ();

let testing(): integer = {
    for (line in ["1", "2", "3"]) {
        let bar = 0;
        let num = match (atoi(line)) {
            ok o = {
                let line = 69;
                printf("%s\n", itoa(line));
                o
            },
            err e = panic("oopsies"),
        };
        num
    }

    0
};

let bla(): integer = {
    0
};
