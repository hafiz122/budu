/*
 * Budu SteamWebHelper compatibility shim
 *
 * Copyright (c) 2026 Budu Contributors
 * SPDX-License-Identifier: MIT
 *
 * Modern Steam starts its CEF compositor in a separate GPU process. Wine's
 * macOS driver cannot currently present that process' surface in the browser
 * process, leaving Steam's window black. This transparent launcher starts the
 * original helper with software compositing enabled. Budu keeps the
 * original executable beside this shim and reapplies the shim after Steam
 * updates.
 */

#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif

#include <windows.h>
#include <shellapi.h>
#include <wchar.h>

__declspec(dllexport) const char gamerunner_steamwebhelper_shim_marker[] =
    "GameRunner SteamWebHelper Compatibility Shim v1";

static BOOL append_text(WCHAR **cursor, size_t *remaining, const WCHAR *text)
{
    size_t length = wcslen(text);
    if (length + 1 > *remaining) return FALSE;
    memcpy(*cursor, text, length * sizeof(WCHAR));
    *cursor += length;
    *remaining -= length;
    **cursor = L'\0';
    return TRUE;
}

static BOOL append_quoted_argument(WCHAR **cursor, size_t *remaining, const WCHAR *argument)
{
    const WCHAR *source = argument;
    size_t backslashes = 0;

    if (!append_text(cursor, remaining, L"\"")) return FALSE;
    while (*source)
    {
        if (*source == L'\\')
        {
            ++backslashes;
            ++source;
            continue;
        }

        if (*source == L'"')
        {
            while (backslashes-- > 0)
                if (!append_text(cursor, remaining, L"\\\\")) return FALSE;
            backslashes = 0;
            if (!append_text(cursor, remaining, L"\\\"")) return FALSE;
            ++source;
            continue;
        }

        while (backslashes-- > 0)
            if (!append_text(cursor, remaining, L"\\")) return FALSE;
        backslashes = 0;

        WCHAR character[2] = {*source++, L'\0'};
        if (!append_text(cursor, remaining, character)) return FALSE;
    }

    while (backslashes-- > 0)
        if (!append_text(cursor, remaining, L"\\\\")) return FALSE;
    return append_text(cursor, remaining, L"\"");
}

static BOOL sibling_path(WCHAR *path, size_t capacity, const WCHAR *name)
{
    DWORD length = GetModuleFileNameW(NULL, path, (DWORD)capacity);
    if (!length || length >= capacity) return FALSE;

    WCHAR *separator = wcsrchr(path, L'\\');
    if (!separator) return FALSE;
    separator[1] = L'\0';

    size_t used = wcslen(path);
    size_t name_length = wcslen(name);
    if (used + name_length + 1 > capacity) return FALSE;
    memcpy(path + used, name, (name_length + 1) * sizeof(WCHAR));
    return TRUE;
}

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE previous, WCHAR *ignored, int show)
{
    (void)instance;
    (void)previous;
    (void)ignored;
    (void)show;

    WCHAR original[MAX_PATH];
    if (!sibling_path(original, MAX_PATH, L"steamwebhelper.gamerunner-original.exe"))
        return ERROR_PATH_NOT_FOUND;

    int argc = 0;
    WCHAR **argv = CommandLineToArgvW(GetCommandLineW(), &argc);
    if (!argv) return ERROR_INVALID_PARAMETER;

    size_t capacity = 32768;
    WCHAR *command_line = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
                                    capacity * sizeof(WCHAR));
    if (!command_line)
    {
        LocalFree(argv);
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    WCHAR *cursor = command_line;
    size_t remaining = capacity;
    BOOL valid = append_quoted_argument(&cursor, &remaining, original);
    for (int index = 1; valid && index < argc; ++index)
    {
        valid = append_text(&cursor, &remaining, L" ") &&
                append_quoted_argument(&cursor, &remaining, argv[index]);
    }
    valid = valid && append_text(
        &cursor, &remaining,
        L" --no-sandbox --in-process-gpu --disable-gpu");
    LocalFree(argv);

    if (!valid)
    {
        HeapFree(GetProcessHeap(), 0, command_line);
        return ERROR_INSUFFICIENT_BUFFER;
    }

    STARTUPINFOW startup = {0};
    PROCESS_INFORMATION process = {0};
    startup.cb = sizeof(startup);
    BOOL created = CreateProcessW(
        original, command_line, NULL, NULL, TRUE, 0, NULL, NULL, &startup, &process);
    DWORD error = created ? ERROR_SUCCESS : GetLastError();
    HeapFree(GetProcessHeap(), 0, command_line);
    if (!created) return (int)error;

    CloseHandle(process.hThread);
    WaitForSingleObject(process.hProcess, INFINITE);
    DWORD exit_code = 1;
    GetExitCodeProcess(process.hProcess, &exit_code);
    CloseHandle(process.hProcess);
    return (int)exit_code;
}
