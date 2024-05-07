const std = @import("std");

const types = @import("lib/compiler/types.zig");
const tokens = @import("lib/lexer/tokens.zig");
const lexer = @import("lib/lexer/lexer.zig");

const allocator = std.heap.page_allocator;

pub fn main() !void {
    const source = "123 123.01 OBTW this\nis\na\ncomment \nTROOF";
    var l = lexer.Lexer.init(source);
    const t = try l.get_tokens();
    for (t) |token| {
        std.log.info("token: {s} {}", .{ token.to_name(), token });
    }
    const hasErrors = lexer.Lexer.has_errors(t);
    std.log.info("Has errors: {}", .{hasErrors});
    if (hasErrors) {
        std.log.info("Error: {s}", .{lexer.Lexer.get_first_error(t).illegal.to_string()});
    }
}
