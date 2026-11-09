/* Separately built upload fixture with a high-performance GPU preference hint. */
#include <windows.h>
__declspec(dllexport) DWORD NvOptimusEnablement = 1;
#include "cell_skybox_d3d9_upload_host_review_v1.c"
