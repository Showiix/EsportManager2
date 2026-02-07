#!/usr/bin/env python3
"""Replace println! with log macros in Rust source files."""

import re
import os

BASE = "/Users/showiix/Documents/EsportManager2-Backend"

FILES = [
    "src-tauri/src/services/game_flow.rs",
    "src-tauri/src/commands/international_commands.rs",
    "src-tauri/src/services/honor_service.rs",
    "src-tauri/src/commands/stats_commands.rs",
    "src-tauri/src/commands/game_commands.rs",
    "src-tauri/src/commands/match_commands.rs",
    "src-tauri/src/commands/honor_commands.rs",
    "src-tauri/src/services/league_service.rs",
    "src-tauri/src/engines/meta_engine.rs",
    "src-tauri/src/db/repository.rs",
    "src-tauri/src/db/connection.rs",
    "src-tauri/src/commands/dev_commands.rs",
    "src-tauri/src/commands/team_commands.rs",
    "src-tauri/src/lib.rs",
    "src-tauri/src/commands/query_commands.rs",
    "src-tauri/src/commands/time_commands.rs",
    "src-tauri/src/services/logging_service.rs",
]

ERROR_KEYWORDS = ["错误", "失败", "Error", "ERROR", "fail", "Fail"]
WARN_KEYWORDS = ["警告", "Warning", "WARNING", "不足", "跳过"]
INFO_KEYWORDS = ["完成", "成功", "Created", "Updated", "创建", "保存",
                  "初始化", "生成", "已存在", "已有", "已创建",
                  "使用", "冠军", "亚军", "季军", "殿军", "MVP",
                  "Got", "records", "Updated"]

def determine_log_level(content):
    for kw in ERROR_KEYWORDS:
        if kw in content:
            return "error"
    for kw in WARN_KEYWORDS:
        if kw in content:
            return "warn"
    for kw in INFO_KEYWORDS:
        if kw in content:
            return "info"
    return "debug"

def remove_prefix_tag(s):
    """Remove [Prefix] tags from format string."""
    return re.sub(r'^\[[\w\s_\-:.]+\]\s*', '', s)

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Find test module start position (byte offset)
    test_pos = len(content)  # default: no test module
    for m in re.finditer(r'#\[cfg\(test\)\]', content):
        test_pos = min(test_pos, m.start())
    for m in re.finditer(r'\bmod\s+tests\s*\{', content):
        test_pos = min(test_pos, m.start())

    replacements = 0

    def replace_println(match):
        nonlocal replacements
        # Skip if inside test module
        if match.start() >= test_pos:
            return match.group(0)

        full = match.group(0)
        # Extract first string literal to determine level and remove prefix
        str_match = re.search(r'"((?:[^"\\]|\\.)*)"', full)
        if str_match:
            fmt_str = str_match.group(1)
            level = determine_log_level(fmt_str)
            cleaned = remove_prefix_tag(fmt_str)
            if cleaned != fmt_str:
                full = full.replace('"' + fmt_str + '"', '"' + cleaned + '"', 1)
        else:
            level = "debug"

        full = full.replace('println!', f'log::{level}!', 1)
        replacements += 1
        return full

    # Match println!(...) handling nested parens
    # Use a function-based approach to handle nested parens
    result = []
    i = 0
    while i < len(content):
        # Look for println! that's not in a comment
        m = re.search(r'println!\s*\(', content[i:])
        if m is None:
            result.append(content[i:])
            break

        start = i + m.start()

        # Check if it's in a comment (// ...)
        line_start = content.rfind('\n', 0, start) + 1
        line_prefix = content[line_start:start].strip()
        if line_prefix.startswith('//'):
            result.append(content[i:start + m.end() - m.start()])
            i = start + m.end() - m.start()
            continue

        # Find matching closing paren
        paren_start = i + m.start() + m.group(0).index('(')
        depth = 0
        j = paren_start
        in_string = False
        escape = False
        while j < len(content):
            ch = content[j]
            if escape:
                escape = False
                j += 1
                continue
            if ch == '\\' and in_string:
                escape = True
                j += 1
                continue
            if ch == '"' and not in_string:
                in_string = True
            elif ch == '"' and in_string:
                in_string = False
            elif not in_string:
                if ch == '(':
                    depth += 1
                elif ch == ')':
                    depth -= 1
                    if depth == 0:
                        break
            j += 1

        # j now points to the closing paren
        end = j + 1
        full_stmt = content[start:end]

        # Check if in test module
        if start >= test_pos:
            result.append(content[i:end])
            i = end
            continue

        # Process the println! statement
        str_match = re.search(r'"((?:[^"\\]|\\.)*)"', full_stmt)
        if str_match:
            fmt_str = str_match.group(1)
            level = determine_log_level(fmt_str)
            cleaned = remove_prefix_tag(fmt_str)
            if cleaned != fmt_str:
                full_stmt = full_stmt.replace('"' + fmt_str + '"', '"' + cleaned + '"', 1)
        else:
            level = "debug"

        full_stmt = full_stmt.replace('println!', f'log::{level}!', 1)
        replacements += 1

        result.append(content[i:start])
        result.append(full_stmt)
        i = end

    new_content = ''.join(result)

    with open(filepath, 'w') as f:
        f.write(new_content)

    return replacements


def main():
    total = 0
    for rel_path in FILES:
        filepath = os.path.join(BASE, rel_path)
        if not os.path.exists(filepath):
            print(f"SKIP (not found): {rel_path}")
            continue
        count = process_file(filepath)
        print(f"{rel_path}: {count} replacements")
        total += count
    print(f"\nTotal: {total} replacements")

if __name__ == '__main__':
    main()
