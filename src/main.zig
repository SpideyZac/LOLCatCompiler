// DYNAMIC TYPE RESOURCES
// https://ziglang.org/documentation/master/#Function-Parameter-Type-Inference
// https://ikrima.dev/dev-notes/zig/zig-metaprogramming/
// https://ziglang.org/documentation/master/#Generic-Data-Structures

const std = @import("std");

const types = @import("lib/compiler/types.zig");
const tokens = @import("lib/lexer/tokens.zig");
const lexer = @import("lib/lexer/lexer.zig");

const allocator = std.heap.page_allocator;

pub fn main() !void {
    const source = "123 1.02";
    var l = lexer.Lexer.init(source);
    var t = l.next_token();
    while (!std.mem.eql(u8, t.to_name(), "eof")) {
        std.log.info("{s} {}", .{t.to_name(), t});
        t = l.next_token();
    }

    std.log.info("hello from tempLang!", .{});
}
