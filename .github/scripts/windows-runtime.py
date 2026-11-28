#!/usr/bin/env python3
"""Copies the Visual C++ runtime a Windows program needs to sit beside it.

    windows-runtime.py --into DIRECTORY PROGRAM...

`mash.exe` and `mash-server.exe` import the Visual C++ runtime
(VCRUNTIME140.dll, MSVCP140.dll and their `_1` companions): ONNX Runtime's
prebuilt library is built against it, so the programs cannot carry it inside
them. Windows does not include those files, and a program without them exits
before it starts (0xC0000135). Microsoft lets an application ship them in its
own folder, which Windows searches first.

This copies, from the Visual Studio that built the programs, each runtime
library a PROGRAM imports, for the program's own machine (x64 or ARM64), and
prints the names. It fails when a program imports a runtime library that
Visual Studio's redistributable folder does not hold.
"""

import argparse
import importlib.util
import os
import shutil
import struct
import subprocess
import sys

RUNTIME_PREFIXES = ("vcruntime", "msvcp", "concrt", "vccorlib", "vcomp")
MACHINES = {0x8664: "x64", 0xAA64: "arm64"}


def program_class():
    """`Program` from check-windows-delay-load.py, which reads import tables."""
    path = os.path.join(os.path.dirname(__file__), "check-windows-delay-load.py")
    spec = importlib.util.spec_from_file_location("delay_load", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.Program


def machine(path):
    with open(path, "rb") as file:
        data = file.read(0x400)
    (header,) = struct.unpack_from("<I", data, 0x3C)
    (code,) = struct.unpack_from("<H", data, header + 4)
    if code not in MACHINES:
        raise ValueError(
            f"{path} is built for machine {code:#x}, which has no runtime here"
        )
    return MACHINES[code]


def needed(programs):
    """The runtime libraries the programs import, lowercase."""
    program = program_class()
    return sorted(
        {
            name
            for path in programs
            for name in program(path).imports()
            if name.startswith(RUNTIME_PREFIXES)
        }
    )


def redistributable(architecture):
    """Each library in Visual Studio's redistributable folder, newest toolset last."""
    vswhere = os.path.join(
        os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"),
        r"Microsoft Visual Studio\Installer\vswhere.exe",
    )
    pattern = rf"VC\Redist\MSVC\*\{architecture}\Microsoft.VC*.CRT\*.dll"
    found = subprocess.run(
        [vswhere, "-products", "*", "-latest", "-find", pattern],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.split("\n")
    return {
        os.path.basename(path).lower(): path
        for path in sorted(filter(None, map(str.strip, found)))
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--into", required=True)
    parser.add_argument("programs", nargs="+")
    arguments = parser.parse_args()
    architectures = {machine(path) for path in arguments.programs}
    if len(architectures) != 1:
        sys.exit(
            f"the programs are built for different machines: {sorted(architectures)}"
        )
    names = needed(arguments.programs)
    libraries = redistributable(architectures.pop())
    missing = [name for name in names if name not in libraries]
    if missing:
        sys.exit(f"Visual Studio's redistributable folder has no {', '.join(missing)}")
    os.makedirs(arguments.into, exist_ok=True)
    for name in names:
        shutil.copyfile(libraries[name], os.path.join(arguments.into, name))
        print(name)


if __name__ == "__main__":
    main()
