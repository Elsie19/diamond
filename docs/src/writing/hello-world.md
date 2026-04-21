# Hello, World!

There are a couple ways I think you can write "Hello, World!" in Diamond, so in no particular order:

## The Usual Way

```
printf("Hello, World!\n", []);
```

You'll learn more about this later, but you probably noticed the weird empty array. Because Diamond does not have variadic arguments, I decided to go the [Zig route](https://ziglang.org/documentation/master/#Hello-World:~:text=s%7D!%5Cn%22%2C-,.%7B%22World%22%7D,-\)%3B%0A%7D), except instead of an anonymous comptime struct, I went with a simpler array, because Diamond does not have structures.

## With `puts`

```
puts("Hello, World!\n");
```

You'll find that a lot of functions that you'd expect from C are in Diamond.

## With Arrays

```
printf("%s, %s!\n", ["Hello", "World"]);
```

Similiar to [the usual way](#the-usual-way) but the arguments are passed in by the array, just to show you that you can.

## With Loops

```
for (char in ["H", "e", "l", "l", "o", ", ", "W", "o", "r", "l", "d", "!", "\n"]) {
    printf("%s", [char]);
};
```
