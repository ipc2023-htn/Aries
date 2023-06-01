#!/usr/bin/python3

# Script that should be run from the root of the repository.
# It validates that the GG solver finds a solution on some simple problems.

import os
import subprocess

os.system("cargo build --profile ci --bin gg")
solver = "target/ci/gg"

solver_cmd = solver + " --expect-sat {instance}"

instances = [
    "planning/problems/pddl/ipc/1998-gripper-round-1-strips/instance.1.pb.pddl",
    "planning/problems/pddl/ipc/2000-blocks-strips-typed/instance.1.pb.pddl"
]

for instance in instances:
    cmd = solver_cmd.format(instance=instance).split(" ")
    print("Solving instance: " + instance)
    solver_run = subprocess.run(cmd, stdout=subprocess.PIPE, universal_newlines=True)
    if solver_run.returncode != 0:
        print("Solver did not return expected result")
        exit(1)


