cmake_minimum_required(VERSION 3.13)
project(bmark)
set(CMAKE_CXX_STANDARD 17)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(SOURCE_FILES bmark.cpp)

add_executable(bmark ${SOURCE_FILES})
