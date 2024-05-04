// DYNAMIC TYPE RESOURCES
// https://ziglang.org/documentation/master/#Function-Parameter-Type-Inference
// https://ikrima.dev/dev-notes/zig/zig-metaprogramming/
// https://ziglang.org/documentation/master/#Generic-Data-Structures

const std = @import("std");
const allocator = std.heap.page_allocator;

pub fn main() !void {
    std.log.info("hello from tempLang!", .{});

    var b = Bukkit{
        .values = std.ArrayList(BukkitValue).init(allocator),
    };

    try b.values.append(BukkitValue{ .yarn = std.ArrayList(u8).init(allocator) });
    try b.values.items[0].yarn.appendSlice("asd");
    std.log.info("b.values[0].yarn: {s}", .{b.values.items[0].yarn.items});
    defer b.deinit();
}

// To move later just testing
const BukkitValueTag = enum {
    yarn,
    number,
    numbar,
    troof,
    noob,
    bukkit,
};
const BukkitValue = union(BukkitValueTag) {
    yarn: std.ArrayList(u8),
    number: i64,
    numbar: f64,
    troof: bool,
    noob: void,
    bukkit: *Bukkit,
};
const Bukkit = struct {
    values: std.ArrayList(BukkitValue),
    fn deinit(self: *Bukkit) void {
        for (self.values.items) |value| {
            switch (value) {
                .yarn => value.yarn.deinit(),
                .bukkit => value.bukkit.deinit(),
                else => {},
            }
        }
        self.values.deinit();
    }
};
