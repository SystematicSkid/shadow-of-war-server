cmake_minimum_required(VERSION 3.26)

project( shadow-of-war-server )

include_directories( ./deps )
link_directories(
    ${CMAKE_SOURCE_DIR}/deps/minhook/lib
)

add_subdirectory( debugger )
