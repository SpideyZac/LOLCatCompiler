// DYNAMIC TYPE RESOURCES
// https://ziglang.org/documentation/master/#Function-Parameter-Type-Inference
// https://ikrima.dev/dev-notes/zig/zig-metaprogramming/
// https://ziglang.org/documentation/master/#Generic-Data-Structures

const std = @import("std");

pub fn main() !void {
    std.log.info("hello from tempLang!", .{});
}

fn List(comptime T: type) type {
    return struct {
        items: []T,
        len: usize,
    };
}
