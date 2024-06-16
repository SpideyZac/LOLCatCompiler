#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include<string.h>

typedef struct machine {
    float* stack;
    char*  heap;
    bool*  allocated;
    int    hooks;
    int    stack_size;
    int    heap_size;
    int    stack_pointer;
    int    base_ptr;
    float  return_register;
} machine;

const int NO_FREE_MEMORY  = 1;
const int STACK_UNDERFLOW = 2;

void panic(int code) {
    printf("panic: ");
    switch (code) {
        case 1:
            printf("no free memory\n");
            break;
        case 2:
            printf("stack underflow\n");
            break;
        default:
            printf("unknown error\n");
    }
    printf("\n");
    exit(code);
}

void machine_push(machine *vm, float n) {
    if (vm->stack_pointer >= vm->stack_size) {
        panic(NO_FREE_MEMORY);
    }
    vm->stack[vm->stack_pointer++] = n;
}

float machine_pop(machine *vm) {
    if (vm->stack_pointer <= 0) {
        panic(STACK_UNDERFLOW);
    }
    float result = vm->stack[--vm->stack_pointer];
    vm->stack[vm->stack_pointer] = 0;
    return result;
}

machine *machine_new(int stack_size, int heap_size) {
    machine *result    = malloc(sizeof(machine));
    result->stack_size = stack_size;
    result->heap_size  = heap_size;
    result->stack      = malloc(sizeof(float) * stack_size);
    result->heap       = malloc(sizeof(char)  * heap_size);
    result->allocated  = malloc(sizeof(bool)  * heap_size);
    result->return_register = 0;
    result->stack_pointer = 0;
    result->hooks = 0;

    for (int i = 0; i < stack_size; i++) {
        result->stack[i] = 0;
    }

    for (int i = 0; i < heap_size; i++) {
        result->heap[i] = 0;
        result->allocated[i] = false;
    }

    result->base_ptr      = 0;

    return result;
}

void machine_drop(machine *vm) {
    free(vm->stack);
    free(vm->heap);
    free(vm->allocated);
    free(vm);
}

void machine_load_base_ptr(machine *vm) {
    machine_push(vm, vm->base_ptr);
}

void machine_establish_stack_frame(machine *vm) {
    machine_load_base_ptr(vm);
    vm->base_ptr = vm->stack_pointer - 1;
}

void machine_end_stack_frame(machine *vm, int arg_size, int local_scope_size) {
    for (int i = 0; i < local_scope_size; i++) {
        machine_pop(vm); // free local scope
    }

    vm->base_ptr = machine_pop(vm); // restore base pointer

    machine_pop(vm); // free return address (not used in a vm as c will handle this)

    for (int i = 0; i < arg_size; i++) {
        machine_pop(vm); // free arguments
    }
}

void machine_set_return_register(machine *vm) {
    vm->return_register = machine_pop(vm);
}

void machine_access_return_register(machine *vm) {
    machine_push(vm, vm->return_register);
}

int machine_allocate(machine *vm) {
    int size = machine_pop(vm) * 4, addr = -1, consecutive_free_calls = 0;

    for (int i = 0; i < vm->heap_size; i++) {
        if (!vm->allocated[i]) consecutive_free_calls++;
        else consecutive_free_calls = 0;

        if (consecutive_free_calls == size) {
            addr = i - size + 1;
            break;
        }
    }

    if (addr == -1) {
        panic(NO_FREE_MEMORY);
    }

    for (int i = 0; i < size; i++) {
        vm->allocated[addr + i] = true;
    }

    machine_push(vm, addr);
    return addr;
}

void machine_free(machine *vm) {
    int addr = machine_pop(vm), size = machine_pop(vm) * 4;

    for (int i = 0; i < size; i++) {
        vm->allocated[addr + i] = false;
        vm->heap[addr + i] = 0;
    }
}

void float2Bytes(unsigned char bytes_temp[4], float float_variable) {
    union {
        float a;
        unsigned char bytes[4];
    } thing;
    thing.a = float_variable;
    memcpy(bytes_temp, thing.bytes, 4);
}

float bytes2Float(unsigned char bytes_temp[4]) {
    union {
        float a;
        unsigned char bytes[4];
    } thing;
    memcpy(thing.bytes, bytes_temp, 4);
    return thing.a;
}

void machine_store(machine *vm, int floats) {
    int addr = machine_pop(vm);

    // store value in heap by breaking it into bytes
    for (int i = floats - 1; i >= 0; i--) {
        float value = machine_pop(vm);

        unsigned char bytes[4];
        float2Bytes(bytes, value);

        for (int j = 0; j < 4; j++) {
            vm->heap[addr + i * 4 + j] = bytes[j];
        }
    }
}

void machine_load(machine *vm, int floats) {
    int addr = machine_pop(vm);

    // load value from heap by combining bytes
    for (int i = 0; i < floats; i++) {
        unsigned char bytes[4];
        for (int j = 0; j < 4; j++) {
            bytes[j] = vm->heap[addr + i * 4 + j];
        }
        machine_push(vm, bytes2Float(bytes));
    }
}

void machine_copy(machine *vm) {
    int offset = machine_pop(vm);

    machine_push(vm, vm->stack[offset]);
}

void machine_mov(machine *vm) {
    int offset = machine_pop(vm);
    float value = machine_pop(vm);

    vm->stack[offset] = value;
    // print stack
    for (int i = 0; i < vm->stack_pointer; i++) {
        printf("%f\n", vm->stack[i]);
    }
    printf("\n");
}

void machine_hook(machine *vm, int hook) {
    vm->stack[hook] = vm->stack_pointer - 1;
}

void machine_ref_hook(machine *vm, int hook) {
    machine_push(vm, vm->stack[hook]);
}

void machine_add(machine *vm) {
    machine_push(vm, machine_pop(vm) + machine_pop(vm));
}

void machine_subtract(machine *vm) {
    float b = machine_pop(vm);
    float a = machine_pop(vm);
    machine_push(vm, a - b);
}

void machine_multiply(machine *vm) {
    machine_push(vm, machine_pop(vm) * machine_pop(vm));
}

void machine_divide(machine *vm) {
    float b = machine_pop(vm);
    float a = machine_pop(vm);
    machine_push(vm, a/b);
}

void machine_modulo(machine *vm) {
    float b = machine_pop(vm);
    float a = machine_pop(vm);
    machine_push(vm, (float)((int)a % (int)b));
}

void machine_sign(machine *vm) {
    float x = machine_pop(vm);
    if (x >= 0) {
        machine_push(vm, 1);
    } else {
        machine_push(vm, -1);
    }
}

void machine_halt(machine *vm) {
    machine_drop(vm);
    exit(0);
}