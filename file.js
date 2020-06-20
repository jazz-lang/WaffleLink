"use strict";
function foo() {
    for (let i = 0; i < 600; i++) {
        let object = {};
        x[1] = 2;
    }
}

let map = {};
map[0] = 1;
map[1] = 2;
map[2] = 3;
map[3] = {};
gc();

print("Done");