function foo(x,y) {
    print(x,y);
    return x + y;
}

for (let i = 0;i < 100000;i ++) {
    foo(1,2);
    foo(3.4,5);
    foo("21",2);
    foo("x",4);
}