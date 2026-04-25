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
    for (i in lst) {
        i
    }
}
```

```rust
let last(lst: [any]): any = for (i in lst) {
    i
}
```

```rust
let last(lst: [any]): any = for (i in lst) i;
```
</details>
</details>

## Make a program that reads the first `n` lines of a file

### Requirements
* File is passed in as an argument to the program

```rust

```
