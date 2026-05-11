let add_to_onehundred(x: integer): integer = {
    match (eq(x, 100)) {
        ok o = {
            printf("Base case: %d\n", [x]);
            x
        },
        err e = {
            printf("%d + %d = %d\n", [x, 1, add(x, 1)]);
            add_to_onehundred(add(x, 1))
        },
    }
}

printf("%d\n", [add_to_onehundred(50)]);
