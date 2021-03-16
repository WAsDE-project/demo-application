
#include <stdio.h>
#include "shared.h"

__attribute__((import_module("host")))
void* dlopen(const char *);
__attribute__((import_module("host")))
void* dlsym(void*, const char *);

__attribute__((import_module("main")))
int check_error(void* handle);

int multiply(int x, int y) {
    return x*y;
}

int (*multiply_wrapper(void))(int x, int y) { // generates GOT.func.multiply
    return &multiply;
}

void run() {
    printf("Side started!\n");
    printf("%s\n", shared.message);

    printf("%p\n", &dlopen); // generates GOT.func.dlopen
    printf("%p\n", &statically_shared_function); // generates GOT.func.statically_shared_function
    printf("%p\n", &pointer_shared_function); // does not use GOT.func
    printf("%p\n", &run); // generates GOT.func.run

    printf("%s\n", pointer_shared_function());
    printf("2*5: %i\n", multiply_wrapper()(2, 5));

    printf("%s\n", statically_shared_function());

    void* handle = dlopen("main"); // normally dlopen(NULL) would work
    if (check_error(handle)) {
        char* (*dynamically_shared_function)() = dlsym(handle, "dynamically_shared_function");
        if (check_error(dynamically_shared_function))
            printf("%s\n", dynamically_shared_function());
    }
    printf("Side ended!\n");
}
