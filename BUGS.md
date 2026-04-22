# Bugs

## Typechecking and IR do not agree on assignments

It's not a problem per se, but there is no such thing as `Type::Unit` in the IR.
For instance:

```
let last_broken(lst: [any]): any = {
    for (i in lst) {
        i;
    }
};

let bla = last_broken([1, 2, 3]);
dump_var(bla);
```

Prints:

```
Integer(3)
```

Conceptually, and typecheckingly, `bla` would be `Type::Unit` because `last_broken` returns `Type::Unit`, and you're right, it does.
But when it interprets, there is no such thing as `Type::Unit`. It will evaluate to `Type::Integer`. On the IR level, `bla` is `Type::Integer`.

You can inspect it yourself with `dump_var(bla)`, and you'll see it leaks through.
Any function that takes `Type::Any` can inspect the variable that shouldn't be assigned.

I think I can add a `ret_ty` field to all expressions and then pass that along to fix this.
