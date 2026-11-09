/* Review-only variant of the paired inventory, with an NVIDIA preference hint. */
#include <windows.h>
__declspec(dllexport) DWORD NvOptimusEnablement = 1;
#include "cell_d3d9_inventory_review_v1.c"
