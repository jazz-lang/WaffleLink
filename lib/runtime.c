//

// This runtime works if you run compiler like this: waffle file.waffle -lwaffle_runtime --aot
// How to compile:
// cc -shared runtime.c -lc -o libwaffle_runtime.so 
// Then put shared library into /usr/local/lib and run ldconfig

#include <stdlib.h>
#include <stdio.h>



extern void printi(int x) 
{
    printf("%i",x);
}

extern void print_f(float x) {
    printf("%f",x);
}

extern void print_d(double x) {
    printf("%lf",x);
}

extern void printl(long x) {
    printf("%li",x);
} 

extern void printul(unsigned long x) {
    printf("%lu",x);
}

extern void printui(unsigned int x) {
    printf("%u",x);
}

extern const char* int_to_str(int i) {
    char *str = malloc(20);
    sprintf(str,"%d",i);

    return str;
}

extern const char* long_to_str(long i) {
    char *str = malloc(80);
    sprintf(str,"%li",i);
    return str;
}

extern const char* ulong_to_str(unsigned long i) {
    char *str = malloc(80);
    sprintf(str,"%lu",i);
    return str;
}

extern const char* uint_to_str(unsigned int i) {
    char *str = malloc(80);
    sprintf(str,"%u",i);
    return str;
}

extern const char* float32_to_str(float f) {
    char *str = malloc(40);
    sprintf(str,"%f",f);
    return str;
}

extern const char* float64_to_str(double f) {
    char *str = malloc(80);
    sprintf(str,"%lf",f);
    return str;
}
