
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
