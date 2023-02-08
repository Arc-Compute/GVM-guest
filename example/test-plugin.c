#include <stdio.h>

char* start() {
    printf("Starting test plugin\n");
    return NULL;
}

char* cmd_process(const char *cmd_process) {
    printf("Processing %s\n", cmd_process);
    return "Processed";
}

char* stop() {
    printf("Stop test plugin\n");
    return NULL;
}
