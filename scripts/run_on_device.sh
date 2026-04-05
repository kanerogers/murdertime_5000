#!/usr/bin/env bash
set -eux

# slangc -matrix-layout-column-major -g0 -O3 -fvk-use-scalar-layout -entry main -target spirv -profile glsl_460 -o shaders/fireflies.comp.spv shaders/fireflies.slang
slangc -matrix-layout-column-major -g0 -O3 -fvk-use-scalar-layout -entry vs_main -target spirv -profile glsl_460 -o shaders/lines.vert.spv shaders/lines.slang
slangc -matrix-layout-column-major -g0 -O3 -fvk-use-scalar-layout -entry fs_main -target spirv -profile glsl_460 -o shaders/lines.frag.spv shaders/lines.slang

adb shell am force-stop rust.murdertime_5000

scriptdir=$(dirname -- "$(realpath -- "$0")")
cd $scriptdir/..

cargo apk run
