# FIXME: Check directory

cmake.exe -E make_directory build
Set-Location .\build
cmake.exe -G Ninja -DCMAKE_BUILD_TYPE=Release ..
cmake.exe --build . -- -j12
# ninja.exe -j12

Set-Location ..
