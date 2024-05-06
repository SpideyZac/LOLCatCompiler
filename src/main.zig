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
    const source = "123 123.01 OBTW this\nis\na\ncomment TLDR\nTROOF";
    var l = lexer.Lexer.init(source);
    const t = try l.get_tokens();
    for (t) |token| {
        std.log.info("token: {s} {}", .{ token.to_name(), token });
    }
}
