/*
 * Reason: Because rust cannot handle /dev devices correctly.
 */
#include <stdint.h>
#include <string.h>
#include <unistd.h>

#include <fcntl.h>

// Internal file descriptor to talk with host.
static int fd = -1;
static char buffer[1024];

/*
 * init_comms - Initializes the communication layer to be used for host -> guest comms.
 * @returns - Returns a boolean value on if we succeeded opening the file or not.
 *
 * Side effects
 * - Opens a long lasting file descriptor.
 */
int32_t init_comms()
{
    if (fd != -1)
        return 0;

    fd = open("/dev/virtio-ports/hostcommunications", O_RDWR | O_CLOEXEC);

    return fd != -1;
}

/*
 * read_comms - Reads an internal buffer of size 1024 for communication with the host.
 * @returns - Pointer to internal buffer.
 *
 * Side effects
 * - Uses a static buffer, meaning if someone were to hold onto this buffer after the fact
 *     they would be able to alter/spy on long lasting communications. Ensure all access to
 *     this function ONLY occurs under the read_string function, and use that one.
 * - NOTE: NOT THREAD SAFE EITHER.
 * - NOTE: If communication channel is not initialized, it will return the empty string.
 */
const char *read_comms()
{
    if (fd == -1) return NULL;
    memset(buffer, 0, 1024 * sizeof(char));
    read(fd, buffer, 1024 * sizeof(char));
    return buffer;
}

/*
 * write_command - Writes a command into the host -> guest communication chardev.
 * @param str - String to write into host -> guest buffer.
 * @returns - Boolean value if the right is done or not.
 *
 * Side effects
 * - Communicates to host device a message.
 */
int32_t write_comms(const char *str)
{
    return (write(fd, str, strlen(str)) > 0);
}
