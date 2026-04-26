# Practice Problems

Here I'll give you some practice problems, each with a bunch of hints if you're stuck, and finally the solution.

## Create a `last` function

Create a function that will accomplish this pseudocode:

```
let arr = [1, 2, 3, 4, 5, 6];
let last_elem = last(arr);
assert(last_elem, 6);
```

<details>
    <summary>Stuck? Click here for some hints.</summary>

Think about a specific feature of the language.

<details>
    <summary>Still stuck? Click here for the answer!</summary>

Remember that for loops yield their last value, so the answer is any one of the following:

```rust
let last(lst: [any]): any = {
    for (elem in lst) {
        elem
    }
}
```

```rust
let last(lst: [any]): any = for (elem in lst) {
    elem
}
```

```rust
let last(lst: [any]): any = for (elem in lst) elem;
```
</details>
</details>

## Make a program that reads the first `n` lines of a file

This should be equivalent in functionality to the command `head -$N file`.

### Requirements
* File is passed in as an argument to the program

<details>
    <summary>Stuck? Click here for the solution.</summary>

```rust
let file_path = match (nth(args(), 0)) {
    ok o = o,
    err e = panic("pass in file path\n", []),
}

let how_many = match (nth(args(), 1)) {
    ok o = o,
    err e = panic("pass in how many lines you want to read\n", []),
}

let how_many = match (atoi(how_many)) {
    ok o = o,
    err e = panic("pass a valid number\n", []),
}

{
    let lines = lines(STREAM)!;
    let needed = only(lines, how_many);
    for (line in needed) printf("%s\n", [line]);
} < match (open(file(file_path))) {
    ok o = o,
    err e = panic("Invalid file '%s'\n", [file]),
}
```

</details>
