#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main() {
    printf("Starting GStreamer Log Viewer via wrapper...\n");
    
    // Set environment variables
    setenv("HOME", "/app/data", 1);
    setenv("PORT", "3000", 1);
    setenv("RUST_BACKTRACE", "1", 1);
    setenv("RUST_LOG", "info", 1);
    
    // Execute the actual binary
    char *args[] = {"/app/code/gst-log-viewer", NULL};
    execv("/app/code/gst-log-viewer", args);
    
    // If we get here, execv failed
    perror("execv failed");
    return 1;
}