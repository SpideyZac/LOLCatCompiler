
void prn(machine *vm) {
    float n = machine_pop(vm);
    printf("%f", n);
}

void prs(machine *vm) {
    float addr = machine_pop(vm);
    int i;
    for (i=addr; vm->stack[i]; i++) {
        printf("%c", (char)vm->stack[i]);
    }
}

void prh(machine *vm) {
    float addr = machine_pop(vm);
    printf("%c", vm->heap[(int)addr]);
}

void prc(machine *vm) {
    float n = machine_pop(vm);
    printf("%c", (char)n);
}

void prend(machine *vm) {
    printf("\n");
}

void getch(machine *vm) {
    char ch = getchar();
    if (ch == '\r') {
        ch = getchar();
    }
    machine_push(vm, ch);
}

void float_to_int(machine *vm) {
    float n = machine_pop(vm);
    machine_push(vm, (int)n);
}

void string_to_int(machine *vm) {
    int size = machine_pop(vm);
    machine_load(vm, size);
    int number = 0;
    bool is_negative = false;

    for (int i = 0; i < size; i++) {
        int code = machine_pop(vm);

        if (code == 45) {
            if (is_negative) {
                printf("panic: ");
                printf("multiple negative signs in integer\n");
                printf("\n");
                exit(1);
            }
            is_negative = true;
            continue;
        }

        if (code < 48 || code > 57) {
            printf("panic: ");
            printf("cannot convert %d to char\n");
            printf("\n");
            exit(1);
        }

        int digit = code - 48;
        number = number * 10 + digit;
    }

    if (is_negative) {
        number = -number;
    }

    machine_push(vm, number);
}

void int_to_float(machine *vm) {
    int n = machine_pop(vm);
    machine_push(vm, (float)n);
}

void string_to_float(machine *vm) {
    int size = machine_pop(vm);
    machine_load(vm, size);
    int integer_part = 0;
    float fraction_part = 0.0;
    bool found_decimal_point = false;
    float divisor_for_fraction = 1.0;
    bool is_negative = false;

    for (int i = 0; i < size; i++) {
        int code = machine_pop(vm);

        if (code == 45) {
            if (is_negative) {
                printf("panic: ");
                printf("multiple negative signs in float\n");
                printf("\n");
                exit(1);
            }
            is_negative = true;
            continue;
        }
        if (code == 46) {
            if (found_decimal_point) {
                printf("panic: ");
                printf("multiple decimal points in float\n");
                printf("\n");
                exit(1);
            }
            found_decimal_point = true;
        } else if (code < 48 || code > 57) {
            printf("panic: ");
            printf("cannot convert %d to char\n");
            printf("\n");
            exit(1);
        } else {
            int digit = code - 48;
            if (!found_decimal_point) {
                integer_part = integer_part * 10 + digit;
            } else {
                divisor_for_fraction *= 10.0;
                fraction_part += digit / divisor_for_fraction;
            }
        }
    }

    float result = integer_part + fraction_part;
    if (is_negative) {
        result = -result;
    }
    machine_push(vm, result);
}

void int_to_string(machine *vm) {
    int n = machine_pop(vm);
    char buffer[32];
    for (int i = 0; i < 32; i++) {
        buffer[i] = 0;
    }
    sprintf(buffer, "%d", n);
    machine_push(vm, 32);
    int addr = machine_allocate(vm);
    for (int i = 0; i < 32; i++) {
        machine_push(vm, buffer[i]);
    }
    machine_push(vm, (float)addr);
    machine_store(vm, 32);
}

void float_to_string(machine *vm) {
    float n = machine_pop(vm);
    char buffer[32];
    for (int i = 0; i < 32; i++) {
        buffer[i] = 0;
    }
    sprintf(buffer, "%f", n);
    machine_push(vm, 32);
    int addr = machine_allocate(vm);
    for (int i = 0; i < 32; i++) {
        machine_push(vm, buffer[i]);
    }
    machine_push(vm, (float)addr);
    machine_store(vm, 32);
}

void print_string(machine *vm) {
    int size = machine_pop(vm);
    machine_load(vm, size);
    for (int i = 0; i < size; i++) {
        printf("%c", (char)vm->stack[(vm->stack_pointer - size) + i]);
    }
    // clear stack
    for (int i = 0; i < size; i++) {
        machine_pop(vm);
    }
}

void read_string(machine *vm) {
    char buffer[256];
    for (int i = 0; i < 256; i++) {
        buffer[i] = 0;
    }
    if (fgets(buffer, sizeof(buffer), stdin) != NULL) {
        machine_push(vm, 256);
        int addr = machine_allocate(vm);
        for (int i = 0; i < 256; i++) {
            machine_push(vm, (float)buffer[i]);
        }
        machine_push(vm, (float)addr);
        machine_store(vm, 256);
    } else {
        printf("panic: ");
        printf("cannot read string\n");
        printf("\n");
        exit(1);
    }
}
