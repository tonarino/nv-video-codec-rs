#include "wrapper.h"
#include <iostream>

int CUDAAPI HandleVideoSequenceProc(void*, CUVIDEOFORMAT*) { std::cerr << "**HandleVideoSequenceProc()\n"; return 0; }
int CUDAAPI HandlePictureDecodeProc(void*, CUVIDPICPARAMS*) { std::cerr << "**HandlePictureDecodeProc()\n"; return 0; }
int CUDAAPI HandlePictureDisplayProc(void*, CUVIDPARSERDISPINFO*) { std::cerr << "**HandlePictureDisplayProc()\n"; return 0; }
int CUDAAPI HandleOperatingPointProc(void*, CUVIDOPERATINGPOINTINFO*) { std::cerr << "**HandleOperatingPointProc()\n"; return 0; }

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
