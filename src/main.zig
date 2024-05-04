// DYNAMIC TYPE RESOURCES
// https://ziglang.org/documentation/master/#Function-Parameter-Type-Inference
// https://ikrima.dev/dev-notes/zig/zig-metaprogramming/
// https://ziglang.org/documentation/master/#Generic-Data-Structures

const std = @import("std");

const lexer = @import("lib/lexer/lexer.zig");
const types = @import("lib/interpreter/types.zig");

const allocator = std.heap.page_allocator;

pub fn main() !void {
    std.log.info("hello from tempLang!", .{});
}
