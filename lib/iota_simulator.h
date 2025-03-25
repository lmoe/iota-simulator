#ifndef IOTA_SIMULATOR_H
#define IOTA_SIMULATOR_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SimulatorHandle SimulatorHandle;

typedef struct {
    uint8_t* data;
    size_t length;
} ByteArray;


SimulatorHandle* simulator_create();

void simulator_destroy(SimulatorHandle* handle);

ByteArray simulator_execute(SimulatorHandle* handle, const uint8_t* request_data, size_t request_len);

void simulator_free_byte_array(ByteArray array);

#ifdef __cplusplus
}
#endif

#endif // IOTA_SIMULATOR_H