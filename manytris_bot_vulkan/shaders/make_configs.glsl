#version 460

#include "shaders/common.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Sp {SearchParams sp;} search_params;

layout(set = 0, binding = 1) buffer ComputedDropConfigs {
    ComputedDropConfig configs[];
} drop_configs;

void main() {
    uint thread_idx = gl_GlobalInvocationID.x;
    uint8_t cur_search_depth = search_params.sp.cur_search_depth;

    uint drop_config_idx = config_index(thread_idx, cur_search_depth);
    if (drop_config_idx >= drop_configs.configs.length()) {
      return;
    }

    // if depth == 0, input fields are 0..1, output fields are 1..41
    // if depth == 1, input fields are 1..41, output fields are 41..(41+40*40)
    // if depth == 2, input fields are 41..(41+40*40), output fields are (41+40*40)..((41+40*40)+40*40*40)
    uint32_t src_field_start = 0;
    uint32_t dest_field_start = 1;
    for (uint8_t i = uint8_t(0); i < cur_search_depth; i++) {
        src_field_start += int_pow(OUTPUTS_PER_INPUT_FIELD, i);
        dest_field_start += int_pow(OUTPUTS_PER_INPUT_FIELD, i+1);
    }

    uint32_t src_field_idx = src_field_start + (thread_idx / OUTPUTS_PER_INPUT_FIELD);
    uint32_t dest_field_idx = dest_field_start + thread_idx;

    // Order of moves for each input is:
    // (rot 0, shift 0), (rot 0, shift 1)..(rot 3, shift 9)
    uint8_t shape_idx = search_params.sp.upcoming_shape_idxs[cur_search_depth];
    uint32_t start_position_idx = thread_idx % OUTPUTS_PER_INPUT_FIELD;
    uint8_t num_rotations = uint8_t(start_position_idx / SHIFTS_PER_ROTATION);
    int32_t shifts = int32_t(start_position_idx % SHIFTS_PER_ROTATION) - 4;
    uint8_t right_shifts = (shifts > 0) ? uint8_t(shifts) : uint8_t(0);
    uint8_t left_shifts = (shifts > 0) ? uint8_t(0) : uint8_t(-shifts);

    drop_configs.configs[drop_config_idx] = ComputedDropConfig(
        shape_idx,
        num_rotations,
        src_field_idx,
        dest_field_idx,
        left_shifts,
        right_shifts
    );
}
