#include "wrapper.h"
#include <iostream>

int CUDAAPI HandleVideoSequenceProc(void*, CUVIDEOFORMAT*) { std::cerr << "**HandleVideoSequenceProc()" << std::endl; return 0; }
int CUDAAPI HandlePictureDecodeProc(void*, CUVIDPICPARAMS*) { std::cerr << "**HandlePictureDecodeProc()" << std::endl; return 0; }
int CUDAAPI HandlePictureDisplayProc(void*, CUVIDPARSERDISPINFO*) { std::cerr << "**HandlePictureDisplayProc()" << std::endl; return 0; }
int CUDAAPI HandleOperatingPointProc(void*, CUVIDOPERATINGPOINTINFO*) { std::cerr << "**HandleOperatingPointProc()" << std::endl; return 0; }

CUcontext CreateCudaContext(int iGpu) {
    if (cuInit(0) != CUDA_SUCCESS) {
        std::cerr << "cuInit() failed." << std::endl;
        return NULL;
    }

    int nGpu = 0;
    if (cuDeviceGetCount(&nGpu) != CUDA_SUCCESS) {
        std::cerr << "cuDeviceGetCount() failed." << std::endl;
        return NULL;
    }

    if (iGpu < 0 || iGpu >= nGpu) {
        std::cerr << "GPU ordinal out of range. Should be within [" << 0 << ", " << nGpu - 1 << "]" << std::endl;
        return NULL;
    }

    CUdevice cuDevice = 0;
    if (cuDeviceGet(&cuDevice, iGpu) != CUDA_SUCCESS) {
        std::cerr << "cuDeviceGet() failed." << std::endl;
        return NULL;
    }

    CUcontext cuContext = NULL;
    if (cuCtxCreate(&cuContext, 0, cuDevice) != CUDA_SUCCESS) {
        std::cerr << "cuCtxCreate() failed." << std::endl;
        return NULL;
    }

    return cuContext;
}

void ParseFrame(const uint8_t* frame, int size) {
    std::cerr << "**ParseFrame()\n";

    CUVIDPARSERPARAMS videoParserParameters = {};
    videoParserParameters.CodecType = cudaVideoCodec_HEVC;
    videoParserParameters.ulMaxNumDecodeSurfaces = 1;
    videoParserParameters.ulClockRate = 1000;
    videoParserParameters.ulMaxDisplayDelay = 1;
    videoParserParameters.pUserData = NULL;
    videoParserParameters.pfnSequenceCallback = HandleVideoSequenceProc;
    videoParserParameters.pfnDecodePicture = HandlePictureDecodeProc;
    videoParserParameters.pfnDisplayPicture = HandlePictureDisplayProc;
    videoParserParameters.pfnGetOperatingPoint = HandleOperatingPointProc;

    CUvideoparser parser = NULL;
    cuvidCreateVideoParser(&parser, &videoParserParameters);

    CUVIDSOURCEDATAPACKET packet = { };
    packet.payload = frame;
    packet.payload_size = size;
    packet.flags = CUVID_PKT_TIMESTAMP;
    packet.timestamp = 0;
    cuvidParseVideoData(parser, &packet);
}
