cmake_minimum_required(VERSION 3.16)
project(ACInternalCheat)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Build as DLL
add_library(ACCheat SHARED
    dllmain.cpp
)

# Link Windows libraries
target_link_libraries(ACCheat PRIVATE)

# Output name
set_target_properties(ACCheat PROPERTIES
    OUTPUT_NAME "ACCheat"
    SUFFIX ".dll"
)
