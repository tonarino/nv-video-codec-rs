# Copyright 2020 NVIDIA Corporation.  All rights reserved.
#
# Please refer to the NVIDIA end user license agreement (EULA) associated
# with this source code for terms and conditions that govern your use of
# this software. Any use, reproduction, disclosure, or distribution of
# this software and related documentation outside the terms of the EULA
# is strictly prohibited.

# 3.7 is required for FindVulkan module support in CMake.
cmake_minimum_required(VERSION 3.7)

project(NvCodec)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(NVCODEC_PUBLIC_INTERFACE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../Interface)
set(NVCODEC_UTILS_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../Utils)
set(NV_CODEC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

find_package(CUDA)
find_library(CUVID_LIB nvcuvid)
find_library(NVENCODEAPI_LIB nvidia-encode)

set(NV_CODEC_SOURCES
  ${NV_CODEC_DIR}/NvEncoder/NvEncoder.cpp
  ${NV_CODEC_DIR}/NvDecoder/NvDecoder.cpp
  ${NV_CODEC_DIR}/NvEncoder/NvEncoderCuda.cpp
  ${NVCODEC_UTILS_DIR}/BitDepth.cu
)

set(NV_CODEC_HDRS
  ${NV_CODEC_DIR}/NvEncoder/NvEncoder.h
  ${NV_CODEC_DIR}/NvDecoder/NvDecoder.h
  ${NV_CODEC_DIR}/NvEncoder/NvEncoderCuda.h
  ${NVCODEC_PUBLIC_INTERFACE_DIR}/nvEncodeAPI.h
  ${NVCODEC_UTILS_DIR}/NvCodecUtils.h
)

set(CUDA_HOST_COMPILER ${CMAKE_CXX_COMPILER})
set(CUDA_NVCC_FLAGS ${CUDA_NVCC_FLAGS};-gencode arch=compute_50,code=\"sm_50,compute_50\")
if (CMAKE_COMPILER_IS_GNUCC)
    if (NOT "${CUDA_NVCC_FLAGS}" MATCHES "-std=c\\+\\+11" )
        list(APPEND CUDA_NVCC_FLAGS -std=c++11)
    endif()
endif()

cuda_add_library(${PROJECT_NAME} ${NV_CODEC_SOURCES} ${NV_CODEC_HDRS})

set_target_properties(${PROJECT_NAME} PROPERTIES CUDA_SEPARABLE_COMPILATION ON)

target_include_directories(${PROJECT_NAME} PUBLIC
  ${CUDA_INCLUDE_DIRS}
  ${NVCODEC_PUBLIC_INTERFACE_DIR}
  ${NVCODEC_UTILS_DIR}
  ${NV_CODEC_DIR}
)

target_link_libraries(${PROJECT_NAME}
  ${CUDA_CUDA_LIBRARY}
  ${CMAKE_DL_LIBS}
  ${NVENCODEAPI_LIB}
  ${CUVID_LIB}
)

install(TARGETS ${PROJECT_NAME} ARCHIVE  DESTINATION lib)
