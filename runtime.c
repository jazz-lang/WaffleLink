//

// This runtime works if you run compiler like this: waffle file.waffle -lwaffle_runtime --aot
// How to compile:
// cc -shared runtime.c -lc -o libwaffle_runtime.so 
// Then put shared library into /usr/local/lib and run ldconfig

#include <stdlib.h>
#include <stdio.h>

void printi(int x) 
{
    printf("%i",x);
}

void print_f(float x) {
    printf("%f",x);
}

void print_d(double x) {
    printf("%lf",x);
}

void printl(long x) {
    printf("%li",x);
} 

void printul(unsigned long x) {
    printf("%lu",x);
}

void printui(unsigned int x) {
    printf("%u",x);
}

