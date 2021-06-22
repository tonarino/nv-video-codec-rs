#include <cuviddec.h>
#include <nvcuvid.h>
#include <nvEncodeAPI.h>
// #include <ColorSpace.h> // bindgen does not like template functions

// Creates and returns CUDA context.
CUcontext CreateCudaContext(int iGpu);

// Minimal example to trigger parser callbacks.
void ParseFrame(const uint8_t* frame, int size);
