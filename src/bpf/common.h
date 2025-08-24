//
// Created by amitchaudhari on 8/21/25.
//

#ifndef HIDTEST3_COMMON_H
#define HIDTEST3_COMMON_H

#define MAX_PATH 512

struct event_log_entry {
    int original;
    int remapped;
    int new;
} ;

typedef struct {
    char input_device[MAX_PATH];
    char hidraw_device[MAX_PATH];
} hid_sub_paths_t;

typedef struct {
    char hid_path[MAX_PATH];
    int hid_id;
} hid_device_info_t;


#endif //HIDTEST3_COMMON_H