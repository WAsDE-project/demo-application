#ifndef SHARED_H
#define SHARED_H

typedef struct {
    char* message;
    int x;
} SharedStruct;

__attribute__((visibility("default")))
extern SharedStruct shared; // handled as GOT.mem

extern char* statically_shared_function(); // handled as env import
extern char* (*pointer_shared_function)(); // handled as GOT.mem import

#endif
