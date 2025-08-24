//
// Created by amitchaudhari on 8/21/25.
//

#ifndef HIDTEST3_LOADER_H
#define HIDTEST3_LOADER_H

#include "hid_modify.skel.h"

int run_bpf(int hid_id, const int *remap_array, int remap_count);

#endif //HIDTEST3_LOADER_H