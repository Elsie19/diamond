# Learning Diamond

<h1>💎</h1>

## Introduction

Diamond is a minimalistic, streaming and string oriented DSL. There are no structures, no if statements, etc. Its purpose is to take input from files and transform it into some output, where the output can be a couple different things; more on that later.

It is minimalistic in the sense that except for function declarations, there are no explicit types at all.

Furthermore, there is no[^1] mutability in the language.

Putting all of this into a list (+ some extra stuff I'll talk about later):

* Immutable
* Strongly typed
* <img src="https://i.imgflip.com/aq2r57.jpg" width="400" height="300" />

## Getting Started

### The Commandline

The Diamond command-line is very simple. It takes a program to run, and optionally, as many arguments that you want to pass to it. For instance:

```bash
di parse_csv.di names.csv
```

### Hello, World!

There are a couple ways of doing a `Hello, World!` program, so I will go through them in no particular order:

#### The Usual

```rust
puts("Hello, World!\n");
```

#### Printf

```rust
printf("%s, %s!\n", ["Hello", "World"]);
```

#### Characters

```rust
let string = "Hello, World!";
for (char in chars(string)) {
    printf("%s", [char]);
};
printf("\n", []);
```

## Type System

Diamond has only 7[^2] types:

1. [Strings](#string)
2. [Unsized Integers](#integer)
3. [Arrays](#array)
4. [Unit](#unit)
5. [Results](#result)
6. [Files](#file)
7. [Streams](#stream)

and *technically*:

8. [Any](#any)
8. [Unret](#unret)

We are now going to go over each type in detail.

### String

Strings are the backbone of Diamond. They are UTF-8 encoded, and internally immutable.

You can create them with double quotes:

```rust
let my_string = "Hello!";

let return_string(): string = {
    "hello"
};
```

<details>
<summary><i>Click here to learn more about the design choices of string.</i></summary>

#### Internal Representation

Internally, `string` is stored as an [`Rc<str>`](`std::rc::Rc<str>`). This has a couple benefits over a plain [`String`]:

##### Sizing

A Rust `String` has three components, which you can check [here](std::string::String#representation), but because I know you didn't click, they are the pointer to the data, the capacity, and the length (*hint, it's literally just a dynamic array*). This is not ideal for a DSL that statically stores known values, without mutability. Thus, if we can get rid of the capacity field alone (because we will never be modifying that string), we can save 8 bytes, for the [`usize`] that we are no longer going to need. If you don't believe me, you can run this code to check:

    let size_of_rc_str = size_of::<Rc<str>>();
    let size_of_string = size_of::<String>();
    assert_eq!(size_of_rc_str, 16);
    assert_eq!(size_of_string, 24);
    assert!(size_of_rc_str < size_of_string);

Here's a table also:

|    Type   | Mutable | Capacity | Length | Size |
|:---------:|:-------:|:--------:|:------:|:----:|
|  `String` |    ✅   |     ✅   |    ✅  |  24  |
| `Rc<str>` |    ❌   |     ❌   |    ✅  |  16  |

##### Who Has Access?

Arguably, this is the most important part of why `Rc<str>` is being used over `String`. Because strings are immutable, they should be able to be owned by any piece of code that can get ahold of it, but that's not allowed in Rust. Things cannot statically have multiple owners. If we used `String`, we would have to clone everywhere we wanted to use that string, but it's not going anywhere!!!! `Rc` allows us to use reference counting for cheap cloning, because now instead of `my_string.clone()`, we can do `Rc::clone(&my_string)`, and we'll get back an "owned value", but we haven't actually moved the string anywhere nor done any reallocations. This also means that:

    let my_string = "Hello!";
    let is_this_a_copy = my_string;

Is actually not fully copying the string! It's just a pointer to that first string.

</details>

### Integer

The integer type is what I consider a "*secondary type*". Very few functions will to output them, but many will accept them. They are stored as [`usize`], or for C people, that's a `size_t`. There are no floats, or doubles, or anything else.

```rust
let num_min = 0;
let num_max = 18446744073709551615;

let return_integer(): integer = {
    0
};
```

### Array

Arrays can be non-homogenous (*although I don't recommend that, more later*).

```rust
let my_str_arr = ["Hello", ",", " ", "World", "!"];
let my_num_arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
let my_nested_arr = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9],
];
```

#### Please please don't do non-homogenous arrays I'm begging you please don't

I had to make a concession in the type system to accomodate the [`any`](#any) type, specifically arrays of `any`, so `[any]`. The bad news is that I no longer type check array homogenity, so this code passes type-checking:

```rust
let some_function(arr: [string]) = ();

let my_stupid_arr = ["hello", 67, ()];

some_function(my_stupid_arr);
```

Some functions, namely [`enumerate`](crate::interpreter::functions::arrays::enumerate), do actually return non-homogenous arrays (*non-homogenous arrays within an array in fact*), but these are not user generated, and can be considered safe to use.

The reason why I had to make this concession was for [`printf`](crate::interpreter::functions::printf::printf)'s second argument, which is an array that can take anything inside it, so you can do things like:

```rust
printf("string => %s\nnumber => %d\n", ["hello", 420]);
```

### Unit

Unit types are inspired directly by Rust and OCaml. They generally indicate an absence of a value, while still being a value itself. If you are coming from C, you can think about it as `void`, except you can instantiate them as well as pass them to functions. Functions that do not return anything will implicitly return units.

```rust
let unit = ();

let return_nothing(): unit = {
    puts("returned nothing");
};

let return_nothing_also() = {
    ()
};

let return_nothing_also_smaller() = ();

let return_not_butter(): unit = puts("I can't believe it's not butter!\n");
```

### Result

Result types are borrowed from Rust. They can contain either a success or an error value. As of right now (5/24/26), you cannot construct a result type in Diamond, but functions can return them.

```rust
let some_result_function(): result(integer, string) = {
    some_function_that_returns_result()
};
```

Later, I will show you how to [`match`](#match) them, so you can operate on the possible success or error value, along with how to *unwrap* them.

### File

The `file` type can be thought of as an intermediary between `string` and `stream`, so:

```text
string ~> file ~> stream
```

```rust
let my_file = file("names.csv");
```

Think of them as tentative `stream`s almost.

### Stream

Streams are wrappers around the Rust [`File`](`std::fs::File`) type. You can both read and write to and from them.

```rust
let my_file = file("names.csv");
let my_stream = open(my_file)!;
dump(my_stream, "appended!");
```

### Any

This type only exists at the type-checking level. It's an escape hatch for functions that need to take any type (*duh*), but since I ain't doing generics in a DSL, this was the next best thing. All types match on the `any` type, so if `T = "any type"`, the relation is `T <: any`. You cannot construct an `any`, but you can pass it along:

```rust
let return_any(val: any): any = val;
```

For functions that you haven't defined yourself that *do* return or accept `any`, check their docs. I'm sure I gave a layout of what they return.

### Unret

`unret` has the exact same semantics as `any`, except that the function will never return back control flow. Think of it as an early return from anywhere, that happens to match whatever type it needs to in order to pass type-checking.

[^1]: Streams could be considered mutable, but for the purposes of learning, don't worry about it.

[^2]: Diamond technically has two more types:

    8. Any

    9. Unret

    But these are type erased after type-checking, and thus only exist at the parsing and type-checking level, and not the interpreting level. So a more accurate wording of "Diamond only has 7 types" would be, "Diamond only has 7 types that are used when interpreting".
