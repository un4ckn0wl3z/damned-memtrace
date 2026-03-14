cmake_minimum_required(VERSION 3.16)
project(ACTrainer)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# For 32-bit build (AssaultCube is 32-bit)
# Use: cmake -A Win32 ..

add_executable(ACTrainer WIN32
    main.cpp
    imgui/imgui.cpp
    imgui/imgui_draw.cpp
    imgui/imgui_tables.cpp
    imgui/imgui_widgets.cpp
    imgui/imgui_demo.cpp
    imgui/imgui_impl_win32.cpp
    imgui/imgui_impl_dx11.cpp
)

target_include_directories(ACTrainer PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

target_link_libraries(ACTrainer
    d3d11
    dwmapi
)

# Copy executable to project root after build
add_custom_command(TARGET ACTrainer POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:ACTrainer> ${CMAKE_SOURCE_DIR}/ACTrainer.exe
)
