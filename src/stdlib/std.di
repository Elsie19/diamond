let @dump(file: stream, text: string): unit = {
    rust { @rust_dump(file, text); }
}

let @file(path: string): result(file, string) = {
    rust { @rust_file(path) }
};
