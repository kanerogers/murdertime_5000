#!/usr/bin/env bash
set -eux

slangc -matrix-layout-column-major -g0 -O3 -fvk-use-scalar-layout -entry main -target spirv -profile glsl_460 -o shaders/fireflies.comp.spv shaders/fireflies.slang
slangc -matrix-layout-column-major -g0 -O3 -fvk-use-scalar-layout -entry vs_main -target spirv -profile glsl_460 -o shaders/fireflies.vert.spv shaders/fireflies.slang
slangc -matrix-layout-column-major -g0 -O3 -fvk-use-scalar-layout -entry fs_main -target spirv -profile glsl_460 -o shaders/fireflies.frag.spv shaders/fireflies.slang

adb shell am force-stop rust.fireflies

scriptdir=$(dirname -- "$(realpath -- "$0")")
cd $scriptdir/..

cargo apk run
