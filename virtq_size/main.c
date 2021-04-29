#include <stdint.h>
#include <stdio.h>

struct virtq_desc { 
    /* Address (guest-physical). */ 
    uint64_t addr; 
    /* Length. */ 
    uint32_t len; 
    /* The flags as indicated above. */ 
    uint16_t flags; 
    /* Next field if flags & NEXT */ 
    uint16_t next; 
}; 

struct virtq_used_elem { 
    /* Index of start of used descriptor chain. */ 
    uint32_t id; 
    /* Total length of the descriptor chain which was used (written to) */ 
    uint32_t len; 
};

#define QUEUE_SIZE 1024
#define qalign 4095
#define ALIGN(x) (((x) + qalign) & ~qalign)

static inline unsigned virtq_size(unsigned int qsz) 
{ 
    return
        ALIGN(sizeof(struct virtq_desc)*qsz         // descriptor table
            + sizeof(uint16_t)*(3 + qsz))           // available ring
        + ALIGN(sizeof(uint16_t)*3                  // used ring
            + sizeof(struct virtq_used_elem)*qsz);  // used ring's elements
}

int main() {
    printf("sizeof(struct virtq_desc)=%#x\n", sizeof(struct virtq_desc));
    unsigned long descriptor_size = sizeof(struct virtq_desc)*QUEUE_SIZE;
    unsigned long available_size = sizeof(uint16_t)*(3 + QUEUE_SIZE);
    unsigned long used_size = sizeof(uint16_t)*3 + sizeof(struct virtq_used_elem)*QUEUE_SIZE;
    printf("total size=%#x\n", virtq_size(QUEUE_SIZE));
    printf("decriptor table + available ring size=%#lx\n", descriptor_size + available_size);
    printf("used ring begin=%#lx\n", ALIGN(descriptor_size + available_size));
    printf("used ring size=%#lx\n", used_size);
    printf("required padding=%#lx\n", ALIGN(descriptor_size + available_size) - (descriptor_size + available_size));
    return 0;
}
