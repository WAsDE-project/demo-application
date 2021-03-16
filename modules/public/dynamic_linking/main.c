
#include <stdio.h>
#include <stdlib.h>
#include "shared.h"

__attribute__((import_module("host")))
void* dlopen(const char *);
__attribute__((import_module("host")))
void* dlsym(void*, const char *);
__attribute__((import_module("host")))
char* dlerror();

__attribute__((used))
void* alloc(size_t size) {
    return malloc(size);
}

SharedStruct shared = { "Hello world!", 100 };

char* shared_function() {
    return "pointer";
}

char* (*pointer_shared_function)() = &shared_function;

__attribute__((visibility("default")))
int check_error(void* handle) {
    if (!handle) {
        printf("%s\n", dlerror());
        return 0;
    }
    return 1;
}

__attribute__((visibility("default")))
char* statically_shared_function() {
    return "static";
}

__attribute__((visibility("default")))
char* dynamically_shared_function() {
    return "dynamic";
}

__attribute__((used))
int main() {
    printf("Main started!\n");
    void* handle = dlopen("side");
    if (check_error(handle)) {
        void (*run)() = dlsym(handle, "run");
        if (check_error(run))
            run();
    }
    printf("Main ended!\n");
    return 0;
}
