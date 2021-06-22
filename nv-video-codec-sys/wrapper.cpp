#include "wrapper.h"
#include <iostream>

int CUDAAPI HandleVideoSequenceProc(void*, CUVIDEOFORMAT*) { std::cerr << "**HandleVideoSequenceProc()" << std::endl; return 0; }
int CUDAAPI HandlePictureDecodeProc(void*, CUVIDPICPARAMS*) { std::cerr << "**HandlePictureDecodeProc()" << std::endl; return 0; }
int CUDAAPI HandlePictureDisplayProc(void*, CUVIDPARSERDISPINFO*) { std::cerr << "**HandlePictureDisplayProc()" << std::endl; return 0; }
int CUDAAPI HandleOperatingPointProc(void*, CUVIDOPERATINGPOINTINFO*) { std::cerr << "**HandleOperatingPointProc()" << std::endl; return 0; }

CUcontext CreateCudaContext(int iGpu) {
    ck(cuInit(0));
    int nGpu = 0;
    ck(cuDeviceGetCount(&nGpu));
    if (iGpu < 0 || iGpu >= nGpu) {
        std::cerr << "GPU ordinal out of range. Should be within [" << 0 << ", " << nGpu - 1 << "]" << std::endl;
        return NULL;
    }

    CUcontext cuContext = NULL;
    createCudaContext(&cuContext, iGpu, 0);
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
