#!/usr/bin/env python3
"""Fails when a Windows program would need DirectML or Direct3D 12 to start.

    check-windows-delay-load.py PROGRAM...

`mash.exe` and `mash-server.exe` load `directml.dll` and `d3d12.dll` on first
use (build.rs), and `cli/delay_load.c` answers for the functions they take
from them when a library is missing. Three things are checked:

- neither library is in the import table Windows resolves at program start;
- both are delay-loaded, importing only functions the hook answers for, so a
  new import from ONNX Runtime is added to the hook before it ships;
- on Windows, a copy of the program beside files of those names that are not
  libraries still prints its version. A program that needed either at start
  would not run at all there.
"""

import os
import shutil
import struct
import subprocess
import sys
import tempfile

# The functions cli/delay_load.c answers for, by name and by the ordinal an
# import library may use instead. Keep the two lists the same.
ANSWERED = {
    "d3d12.dll": {"D3D12CreateDevice", 101, "D3D12SerializeVersionedRootSignature"},
    "directml.dll": {"DMLCreateDevice1", 2},
}
IMPORT_DIRECTORY = 1
DELAY_IMPORT_DIRECTORY = 13


class Program:
    """The import tables of a 64-bit PE file."""

    def __init__(self, path):
        with open(path, "rb") as file:
            self.data = file.read()
        (header,) = struct.unpack_from("<I", self.data, 0x3C)
        if self.data[:2] != b"MZ" or self.data[header : header + 4] != b"PE\0\0":
            raise ValueError("not a Windows program")
        (sections,) = struct.unpack_from("<H", self.data, header + 6)
        (optional_size,) = struct.unpack_from("<H", self.data, header + 20)
        optional = header + 24
        if struct.unpack_from("<H", self.data, optional)[0] != 0x20B:
            raise ValueError("not a 64-bit program")
        self.directories = optional + 112
        table = optional + optional_size
        # (virtual size, virtual address, size in the file, file offset) of each section.
        self.sections = [
            struct.unpack_from("<4I", self.data, table + 40 * index + 8)
            for index in range(sections)
        ]

    def offset(self, address):
        for size, start, _, position in self.sections:
            if start <= address < start + size:
                return position + address - start
        raise ValueError(f"address {address:#x} is outside every section")

    def text(self, address):
        start = self.offset(address)
        return self.data[start : self.data.index(b"\0", start)].decode("ascii")

    def directory(self, index):
        return struct.unpack_from("<I", self.data, self.directories + 8 * index)[0]

    def imports(self):
        """The libraries Windows loads before the program starts, lowercase."""
        names = []
        address = self.directory(IMPORT_DIRECTORY)
        while address:
            name = struct.unpack_from("<I", self.data, self.offset(address) + 12)[0]
            if not name:
                break
            names.append(self.text(name).lower())
            address += 20
        return names

    def delayed(self):
        """Each delay-loaded library, lowercase, with what is imported from it."""
        libraries = {}
        address = self.directory(DELAY_IMPORT_DIRECTORY)
        while address:
            attributes, name, _, _, lookup = struct.unpack_from(
                "<5I", self.data, self.offset(address)
            )
            if not name:
                break
            if not attributes & 1:
                raise ValueError(
                    "the delay-load table uses addresses this check does not read"
                )
            functions = set()
            entry = self.offset(lookup)
            while True:
                (value,) = struct.unpack_from("<Q", self.data, entry)
                if not value:
                    break
                functions.add(value & 0xFFFF if value >> 63 else self.text(value + 2))
                entry += 8
            libraries[self.text(name).lower()] = functions
            address += 32
        return libraries


def table_problems(path):
    program = Program(path)
    eager = program.imports()
    delayed = program.delayed()
    problems = []
    for library, answered in ANSWERED.items():
        if library in eager:
            problems.append(
                f"{library} is loaded at program start; it must be delay-loaded (build.rs)"
            )
        elif library not in delayed:
            problems.append(
                f"{library} is not imported at all; update this check and cli/delay_load.c"
            )
        for function in sorted(delayed.get(library, set()) - answered, key=str):
            problems.append(
                f"{library} function {function} is delay-loaded but cli/delay_load.c has no answer for it"
            )
    for library in sorted(set(delayed) - set(ANSWERED)):
        problems.append(
            f"{library} is delay-loaded but cli/delay_load.c does not answer for it"
        )
    return problems


def start_problems(path):
    """Run a copy that finds unusable files where it would look for the libraries."""
    with tempfile.TemporaryDirectory() as directory:
        copy = shutil.copy(path, directory)
        for library in ANSWERED:
            with open(os.path.join(directory, library), "w", encoding="ascii") as file:
                file.write("not a library\n")
        result = subprocess.run(
            [copy, "--version"],
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
    if result.returncode != 0 or not result.stdout.strip():
        return [
            f"did not start without {' and '.join(ANSWERED)}: "
            f"exit {result.returncode:#x}, {result.stderr.strip() or 'no message'}"
        ]
    return []


def main(paths):
    if not paths:
        print(__doc__.strip().splitlines()[2].strip(), file=sys.stderr)
        return 2
    status = 0
    for path in paths:
        problems = table_problems(path)
        if os.name == "nt" and not problems:
            problems = start_problems(path)
        for problem in problems:
            print(f"{path}: {problem}", file=sys.stderr)
        if problems:
            status = 1
        else:
            print(f"{path}: starts without {' and '.join(ANSWERED)}")
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
