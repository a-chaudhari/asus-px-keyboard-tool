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
    if (e->original == 0xec)
        return 0; // 0xec appears to be some sort of status report.  ignoring it
    if (e->remapped)
        printf("BPF: Remapped: 0x%02x -> 0x%02x\n", e->original, e->new);
    else
        printf("BPF: Detected unmapped scancode: 0x%02x\n", e->original);
    fflush(stdout);
    return 0;
}

void *poll_ringbuf( void *ptr )
{
    int err;
    struct ring_buffer *rb = ptr;
    printf("BPF: Starting ringbuf polling thread\n");
    fflush(stdout);
    while (1)
    {
        err = ring_buffer__poll(rb, 100 /* timeout, ms */);
        /* Ctrl-C will cause -EINTR */
        if (err == -EINTR) {
            err = 0;
            break;
        }
        if (err < 0) {
            printf("BPF: Error polling ring buffer: %d\n", err);
            fflush(stdout);
            break;
        }
    }
    printf("BPF: Exiting polling thread\n");
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
        fprintf(stderr, "BPF: Failed to open BPF skeleton\n");
        return -1;
    }

    skel->struct_ops.hid_modify_ops->hid_id = hid_id;

    err = hid_modify_bpf__load(skel);
    if (err) {
        fprintf(stderr, "BPF: Failed to load BPF skeleton\n");
        return -1;
   }

    // Attach to HID device
    err = hid_modify_bpf__attach(skel);
    if (err) {
        fprintf(stderr, "BPF: Failed to attach BPF program\n");
        hid_modify_bpf__destroy(skel);
        return -1;
    }

    map_fd = bpf_map__fd(skel->maps.remap_map);
    if (map_fd < 0) {
        fprintf(stderr, "BPF: Failed to get map fd\n");
        hid_modify_bpf__destroy(skel);
        return -1;
    }

    for (int i = 0; i < remap_count; i ++)
    {
        const int *from_code = remap_array + i * 2;
        const int *to_code = remap_array + i * 2 + 1;
        printf("BPF: Remapped: %x -> %x\n", *from_code, *to_code);
        fflush(stdout);
        bpf_map_update_elem(map_fd,
            from_code,
            to_code,
            BPF_ANY);
    }

    /* Set up ring buffer polling */
    rb = ring_buffer__new(bpf_map__fd(skel->maps.event_rb), handle_event, NULL, nullptr);
    if (!rb) {
        fprintf(stderr, "BPF: Failed to create ring buffer\n");
        return -1;
    }

    // need to poll to see output
    pthread_create( &ringbuf_polling_thread, nullptr, poll_ringbuf, rb);

    return 0;
}