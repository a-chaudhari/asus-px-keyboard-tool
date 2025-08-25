//
// Created by amitchaudhari on 8/21/25.
//

#include "loader.h"
#include <pthread.h>
#include <stdio.h>
#include <bpf/bpf.h>
#include <bpf/libbpf.h>
#include "common.h"

pthread_t ringbuf_polling_thread;

int handle_event(void *ctx, void *data, size_t data_sz)
{
    const struct event_log_entry *e = data;
    if (e->remapped)
        printf("Remapped: %x -> %x\n", e->original, e->new);
    else
        printf("Detected unmapped scancode: %x\n", e->original);

    return 0;
}

void *poll_ringbuf( void *ptr )
{
    int err;
    struct ring_buffer *rb = ptr;
    printf("Starting ringbuf polling thread\n");
    while (1)
    {
        err = ring_buffer__poll(rb, 100 /* timeout, ms */);
        /* Ctrl-C will cause -EINTR */
        if (err == -EINTR) {
            err = 0;
            break;
        }
        if (err < 0) {
            printf("Error polling ring buffer: %d\n", err);
            break;
        }
    }
    printf("Exiting polling thread\n");
    return nullptr;
}

/** * This function loads the BPF program, attaches it to the HID device,
 * and sets up a map for remapping scancodes.
 * @param skel: Pointer to the BPF skeleton structure
 * @param hid_id: The HID device ID to attach the BPF program to
 * @return 0 on success, -1 on error
 */
int run_bpf(int hid_id, const int *remap_array, int remap_count)
{
    struct hid_modify_bpf *skel = nullptr;
    int err, map_fd;
    struct ring_buffer *rb = nullptr;

    // Open and load the BPF program
    skel = hid_modify_bpf__open();
    if (!skel) {
        fprintf(stderr, "Failed to open BPF skeleton\n");
        return -1;
    }

    skel->struct_ops.hid_modify_ops->hid_id = hid_id;

    err = hid_modify_bpf__load(skel);
    if (err) {
        fprintf(stderr, "Failed to load BPF skeleton\n");
        return -1;
   }

    // Attach to HID device
    err = hid_modify_bpf__attach(skel);
    if (err) {
        fprintf(stderr, "Failed to attach BPF program\n");
        hid_modify_bpf__destroy(skel);
        return -1;
    }

    map_fd = bpf_map__fd(skel->maps.remap_map);
    if (map_fd < 0) {
        fprintf(stderr, "Failed to get map fd\n");
        hid_modify_bpf__destroy(skel);
        return -1;
    }

    for (int i = 0; i < remap_count; i ++)
    {
        const int *from_code = remap_array + i * 2;
        const int *to_code = remap_array + i * 2 + 1;
        printf("Remapped: %x -> %x\n", *from_code, *to_code);
        bpf_map_update_elem(map_fd,
            from_code,
            to_code,
            BPF_ANY);
    }

    /* Set up ring buffer polling */
    rb = ring_buffer__new(bpf_map__fd(skel->maps.event_rb), handle_event, NULL, nullptr);
    if (!rb) {
        fprintf(stderr, "Failed to create ring buffer\n");
        return -1;
    }

    // need to poll to see output
    pthread_create( &ringbuf_polling_thread, nullptr, poll_ringbuf, rb);

    return 0;
}