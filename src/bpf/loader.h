//
// Created by amitchaudhari on 8/21/25.
//

#ifndef BPF_LOADER_H
#define BPF_LOADER_H

#include "hid_modify.skel.h"

int run_bpf(int hid_id, const int *remap_array, int remap_count);

#endif //BPF_LOADER_H