// DYNAMIC TYPE RESOURCES
// https://ziglang.org/documentation/master/#Function-Parameter-Type-Inference
// https://ikrima.dev/dev-notes/zig/zig-metaprogramming/
// https://ziglang.org/documentation/master/#Generic-Data-Structures

const std = @import("std");

const types = @import("lib/compiler/types.zig");

const allocator = std.heap.page_allocator;

pub fn main() !void {
    std.log.info("hello from tempLang!", .{});

    var b = types.Bukkit{
        .values = std.ArrayList(types.Values).init(allocator),
    };

    try b.values.append(types.Values{ .yarn = std.ArrayList(u8).init(allocator) });
    try b.values.items[0].yarn.appendSlice("asd");
    std.log.info("b.values[0].yarn: {s}", .{b.values.items[0].yarn.items});
    defer b.deinit();
}
