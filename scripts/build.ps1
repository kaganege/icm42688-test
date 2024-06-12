# FIXME: Check directory

cmake.exe -E make_directory build
Set-Location .\build
cmake.exe -G Ninja ..
cmake.exe --build . -- -j12
# ninja.exe -j12

Set-Location ..
