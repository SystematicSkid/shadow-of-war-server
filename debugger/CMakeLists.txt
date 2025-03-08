project( shadow-of-war-server-debugger )

# C++23
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_definitions(-DENABLE_LOGGING)

# Glob recursive ./src/**/*.cpp
file(GLOB_RECURSE SRC_FILES ./src/*.cpp)

# Add shared library
add_library( ${PROJECT_NAME} SHARED ${SRC_FILES})

# Add include directory
target_include_directories( ${PROJECT_NAME} PUBLIC ./include )

# Add minhook
target_link_libraries( ${PROJECT_NAME} libMinHook.x64 )