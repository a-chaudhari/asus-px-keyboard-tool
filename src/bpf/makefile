BPF_OBJ = hid_modify.bpf.o
SKEL_H = hid_modify.skel.h
LOADER_OBJ = libloader.so

all: $(LOADER_OBJ)

$(BPF_OBJ): hid_modify.bpf.c
	clang -target bpf -O2 -g -static -c $< -o $@

$(SKEL_H): $(BPF_OBJ)
	bpftool gen skeleton $< > $@

$(LOADER_OBJ): loader.c $(BPF_OBJ) $(SKEL_H)
	gcc -static -c loader.c -o $(LOADER_OBJ) -O2 -lbpf

vmlinux:
	bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h

clean:
	rm -f $(BPF_OBJ) $(SKEL_H) $(LOADER_OBJ)

.PHONY: all clean vmlinux
