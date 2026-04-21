# "Everything is an expression"

In Diamond, everything is an expression. What that means is that **everything returns a value**. If you can write a piece of code in Diamond, it can be used as a value. Every construct is an expression as well, so:

* For loops
* Groupings (`{ these things }`)
* Function definitions
* Match expressions
* Variable definitions

All return values.

For instance:

```diamond
let last(lst: [string]): string = for (line in lst) {
        line
};
```

Is a clever way to return the last element of a list. It will loop through the list, returning only the last element.

> [!TIP]
> If you don't want to return something, you can always suffix any expression with a `;`!
