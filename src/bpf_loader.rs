use crate::apkt_config::Remap;
use libbpf_rs::skel::OpenSkel;
use libbpf_rs::skel::SkelBuilder;
use libbpf_rs::{Link, MapCore, MapFlags};
use std::mem::MaybeUninit;
use std::thread;
use std::time::Duration;
extern crate plain;
use crate::bpf_loader::hid_modify::types::event_log_entry;
use plain::Plain;
use hid_modify::*;

mod hid_modify {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bpf/hid_modify.skel.rs"
    ));
}

unsafe impl Plain for event_log_entry {}
static mut LINK: Option<Link> = None;

pub fn start_bpf(hid_id: i32, remaps: &Vec<Remap>) {
    let skel_builder = HidModifySkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let open_skel = skel_builder
        .open(&mut open_object)
        .expect("Failed to open skel");

    // set hid_id in bpf program
    let hid_modify_ops = open_skel.struct_ops.hid_modify_ops;
    unsafe {
        (*hid_modify_ops).hid_id = hid_id;
    }
    let mut skel = open_skel.load()
        .expect("Failed to load skel.  Are you root?");
    let link = skel
        .maps
        .hid_modify_ops
        .attach_struct_ops()
        .expect("Failed to attach struct ops");

    // save the link to prevent it from being dropped
    unsafe {
        LINK = Some(link);
    }
    println!("BPF program loaded and attached");

    for remap in remaps {
        println!("Remapping {:#04x} to {:#04x}", remap.from, remap.to);
        skel.maps
            .remap_map
            .update(
                &remap.from.to_ne_bytes(),
                &remap.to.to_ne_bytes(),
                MapFlags::ANY,
            )
            .expect("Failed to map remap");
    }

    // set up the ring buffer
    let mut builder = libbpf_rs::RingBufferBuilder::new();
    builder
        .add(&skel.maps.event_rb, |data| process_log_entry(data))
        .expect("failed to add ringbuf");
    let ringbuf = builder.build().unwrap();
    let mutex = std::sync::Mutex::new(ringbuf);

    // spawn a thread to poll the ring buffer indefinitely without blocking
    thread::spawn(move || {
        loop {
            let lock = mutex.lock().expect("Failed to lock mutex");
            lock.poll(Duration::MAX).expect("ringbuf poll failed");
        }
    });
}

fn process_log_entry(data: &[u8]) -> i32 {
    let event = plain::from_bytes::<event_log_entry>(data).unwrap();
    if event.original == 0xec {
        return 0; // ignore status events
    }
    if event.remapped == 1{
        println!("Remapped scancode: {:#04x} -> {:#04x}", event.original, event.new);
    } else {
        println!("Unmapped scancode: {:#04x}", event.original);
    }
    0 // return value
}
