package lib

// #cgo CFLAGS: -I${SRCDIR}
// #cgo linux LDFLAGS: -L${SRCDIR}/. -L${SRCDIR}/lib -Wl,-rpath=/home/luke/dev/iota/iota-simulator/native/target/release/ -llsimulacrum -ldl -lm -pthread
// #cgo darwin LDFLAGS: -L${SRCDIR}/lib -L${SRCDIR}/../native/target/release  -llsimulacrum -ldl -lm -pthread
// #cgo windows LDFLAGS: -L${SRCDIR}/lib -L${SRCDIR}/../native/target/release  -llsimulacrum -lws2_32 -luserenv
// #include <stdlib.h>
// #include "iota_simulator.h"
import "C"
import (
	"encoding/json"
	"errors"
	"fmt"
	"runtime"
	"unsafe"
)

// Simulator wraps the Rust simulator
type Simulator struct {
	handle *C.SimulatorHandle
}

type FFIRequest struct {
	Method string `json:"method"`
	Args   any    `json:"args"`
}

// FFIResponse is a generic response container
type FFIResponse[T any] struct {
	Success      bool   `json:"success"`
	Data         T      `json:"data"`
	ErrorMessage string `json:"error_message,omitempty"`
}

// NewSimulator creates a new simulator instance
func NewSimulator() (*Simulator, error) {
	handle := C.simulator_create()
	if handle == nil {
		return nil, errors.New("failed to create simulator")
	}

	sim := &Simulator{handle: handle}

	runtime.SetFinalizer(sim, (*Simulator).Destroy)

	return sim, nil
}

// Destroy releases the simulator resources
func (s *Simulator) Destroy() {
	if s.handle != nil {
		C.simulator_destroy(s.handle)
		s.handle = nil
	}
}

func executeCall[T any](s *Simulator, method string, args interface{}) (T, error) {
	var nilT T

	if s.handle == nil {
		return nilT, errors.New("simulator has been destroyed")
	}

	// Create the request
	request := FFIRequest{
		Method: method,
		Args:   args,
	}

	// Serialize the request to JSON
	requestJSON, err := json.Marshal(request)
	if err != nil {
		return nilT, fmt.Errorf("failed to serialize request: %v", err)
	}

	// Convert Go slice to C array
	var cRequestData *C.uint8_t
	var cRequestLen C.size_t

	if len(requestJSON) > 0 {
		cRequestData = (*C.uint8_t)(unsafe.Pointer(&requestJSON[0]))
		cRequestLen = C.size_t(len(requestJSON))
	}

	// Call the FFI function
	result := C.simulator_execute(s.handle, cRequestData, cRequestLen)

	// Ensure we free the result when we're done
	defer C.simulator_free_byte_array(result)

	// Check for errors
	if result.data == nil {
		return nilT, errors.New("FFI call failed")
	}

	// Convert the result back to a Go slice
	resultBytes := C.GoBytes(unsafe.Pointer(result.data), C.int(result.length))

	fmt.Println(string(resultBytes))
	// Parse the JSON response
	var response FFIResponse[T]
	if err := json.Unmarshal(resultBytes, &response); err != nil {
		return nilT, fmt.Errorf("failed to parse response: %v", err)
	}

	// Check for success
	if !response.Success {
		if response.ErrorMessage != "" {
			return nilT, errors.New(response.ErrorMessage)
		}
		return nilT, errors.New("operation failed")
	}

	return response.Data, nil
}
