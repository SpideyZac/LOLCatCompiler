const std = @import("std");

const Lexer = @import("lib/lexer/lexer.zig").Lexer;
const Parser = @import("lib/parser/parser.zig").Parser;

pub fn main() !void {
    // Read File Passed as Argument
    // const args = try std.process.argsAlloc(std.heap.page_allocator);
    // defer std.process.argsFree(
    //     std.heap.page_allocator,
    //     args,
    // );

    // const contents = try std.fs.cwd().readFileAlloc(
    //     std.heap.page_allocator,
    //     args[1],
    //     std.math.maxInt(usize),
    // );
    // defer std.heap.page_allocator.free(contents);

    const contents = "HAI 1.2\nI HAS A x R SUM OF 1 AN 2\nKTHXBYE";

    // Initalize Lexer on Contents
    var lexer = Lexer.init(contents);
    const tokens = try lexer.get_tokens();

    std.debug.print("Tokens:\n", .{});
    for (tokens) |token| {
        std.debug.print("{s}: {}\n", .{token.to_name(), std.json.fmt(token, .{ .whitespace = .indent_2 })});
    }

    std.debug.print("\n\n\nLexer Errors:\n", .{});
    const hasErrors = Lexer.has_errors(tokens);
    if (hasErrors) {
        std.log.err(
            "{s}\n",
            .{Lexer.get_first_error(tokens).illegal.to_string()},
        );
    }

    if (hasErrors) {
        return;
    }
    // Initalize Parser on Tokens
    std.debug.print("\n\n\nParser:\n", .{});
    const parser = Parser.parse(tokens);
    std.debug.print("{}\n", .{std.json.fmt(parser.program.statements, .{ .whitespace = .indent_2 })});

    std.debug.print("\n\n\nParser Errors:\n", .{});
    for (parser.errors) |e| {
        std.log.err("{}", .{std.json.fmt(e, .{ .whitespace = .indent_2 })});
    }
}
