/*  Hooks let you customize what happens
 *  when an application reads from / writes to
 *  a network connection.
 *  The default behavior is to read from stdin /
 *  write to stdout but this can easily be changed
 *  in the functions below.
 */

#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/shm.h>

#include "util.h"
#include "desock.h"
#include "hooks.h"
#include "syscall.h"

#define ENV_VAR "LIBDESOCK_PACKET_BUFFER"

typedef struct {
    size_t cursor;
    size_t size;
    char data[];
} PacketBuffer;

static PacketBuffer* packet_buffer = NULL;

__attribute__((constructor))
static void init_packet_buffer (void) {
    char* shm_id = getenv(ENV_VAR);
    
    if (shm_id) {
        char* endptr = NULL;
        unsigned long id = strtoul(shm_id, &endptr, 0);
        
        if (endptr == NULL || *endptr != 0) {
            _error("Invalid shm id in " ENV_VAR ": %s\n", shm_id);
        }
        
        packet_buffer = shmat(id, NULL, 0);
        
        if (packet_buffer == NULL || packet_buffer == (void*) -1) {
            _error("Could not attach to packet buffer: %ld\n", id);
        }
    }
}

/*  This function is called whenever a read on a network
 *  connection occurs. It MUST return the number of bytes
 *  written to buf or -1 if an error occurs.
 */
ssize_t hook_input (char* buf, size_t size) {
    if (packet_buffer) {
        size_t cursor = packet_buffer->cursor;
        size_t rem_bytes = packet_buffer->size - cursor;
        size = (size < rem_bytes) ? size : rem_bytes;
        
        memcpy(buf, &packet_buffer->data[cursor], size);
        packet_buffer->cursor = cursor + size;
        
        return (ssize_t) size;
    } else {
        return syscall_cp(SYS_read, 0, buf, size);
    }
}

/*  This function is called whenever a write on a network
 *  connection occurs. It MUST return the number of bytes
 *  written or -1 if an error occurs.
 */
ssize_t hook_output (const char* buf, size_t size) {
    (void) buf;
    return (ssize_t) size;
}

/*  This function is called whenever libdesock internally
 *  searches through the input stream. It MUST behave like
 *  the lseek() function in the sense that on success, it
 *  must return the resulting offset and on error it 
 *  must return -1.
 *  The supplied offset always is relative to the current
 *  stream position.
 */
off_t hook_seek (off_t offset) {
    if (packet_buffer) {
        packet_buffer->cursor += (size_t) ((ssize_t) offset);
        return packet_buffer->cursor;
    } else {
        return lseek(0, offset, SEEK_CUR);
    }
}
